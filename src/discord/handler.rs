use crate::discord::character::OblivionError;
use crate::prelude::*;
use crate::{
    config::OddbotConfig, error::OddbotError, prelude::EventStream, skeever::squeak::Squeak,
};
use serenity::all::{Context, GuildId, Member, Message};
use sqlx::{PgPool, types::time::OffsetDateTime};
use std::sync::Arc;

use super::character::CharacterStore;
use super::commands::oblivion;

/// Default handler for the Discord bot
pub struct Handler {
    pub guild_id: Option<GuildId>,
    pub db_pool: Arc<PgPool>,
    pub event_stream: Option<Arc<EventStream>>,
    pub character_store: Arc<CharacterStore>,
}

impl Handler {
    /// Create a new handler instance
    pub fn new(
        db_pool: Arc<PgPool>,
        event_stream: Option<Arc<EventStream>>,
        character_store: Arc<CharacterStore>,
    ) -> Self {
        let guild_id = {
            // Check if we're configured to run against a specific guild
            let guild_id = OddbotConfig::get_guild_id();
            match guild_id {
                Some(id) => Some(GuildId::new(id)),
                None => None,
            }
        };

        Self {
            guild_id,
            db_pool,
            event_stream,
            character_store,
        }
    }

    /// Start-up initialization when the bot is ready
    pub async fn init(&self, ctx: Context) -> Result<(), OddbotError> {
        // We only initialize if we're configured to run against a specific guild
        let Some(guild_id) = self.guild_id else {
            return Ok(());
        };

        // Register the slash commands
        let commands = vec![
            oblivion::commands::register_character(),
            oblivion::commands::get_character(),
            oblivion::commands::delete_character(),
        ];
        let commands = guild_id.set_commands(&ctx.http, commands).await;
        tracing::debug!("Registered guild slash commands: {commands:?}");

        // Save published members with configured member role id
        if let Some(published_role) = OddbotConfig::get_published_member_role_id() {
            let published_members = self.get_members(ctx, published_role, guild_id).await;
            tracing::debug!("Updating {} published members", published_members.len());
            // Save all published members to the database
            futures::future::join_all(published_members.into_iter().map(async |member| {
                let member_id = member.user.id.to_string();
                let member_name = member.user.name;
                self.save_member(member_id, member_name).await
            }))
            .await;
        }

        Ok(())
    }

    /// Validates a screenshot message and saves it to the database
    pub async fn handle_ss(&self, msg: &Message, role_id: u64, channel_id: u64) {
        let will_process_message = self.should_process_message(msg, role_id, channel_id);
        if !will_process_message {
            return;
        }

        // Map each screenshot into a separate promise so we can save them concurrently
        let promises = msg.attachments.iter().map(|attachment| {
            self.save_ss(
                msg.author.id.to_string(),
                msg.author.name.clone(),
                attachment.url.clone(),
            )
        });

        // Await all promises concurrently
        futures::future::join_all(promises).await;
    }

    /// Saves a screenshot to the database
    async fn save_ss(
        &self,
        discord_id: String,
        username: String,
        url: String,
    ) -> Result<(), sqlx::Error> {
        tracing::debug!("Saving screenshot from user: {}", username);
        let now = OffsetDateTime::now_utc();
        sqlx::query(
            "INSERT INTO member_screenshots (
                    url,
                    member_id,
                    created_at,
                    updated_at
                )
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (id) DO UPDATE
                SET url = EXCLUDED.url,
                    updated_at = EXCLUDED.updated_at",
        )
        .bind(url)
        .bind(discord_id)
        .bind(now)
        .bind(now)
        .execute(self.db_pool.as_ref())
        .await?;

        Ok(())
    }

    /// Get members with a specific role ID
    pub async fn get_members(
        &self,
        ctx: Context,
        with_role_id: u64,
        guild_id: GuildId,
    ) -> Vec<Member> {
        tracing::debug!("Attempting to fetch guild members for guild {}", guild_id);

        // Get the members of the guild
        let Ok(members) = guild_id.members(&ctx.http, None, None).await else {
            tracing::error!("Failed to fetch members for guild {}", guild_id);
            return Vec::new();
        };

        tracing::debug!("Fetched {} members", members.len());
        // Only grab members that have our target role
        members
            .into_iter()
            .filter(|member| member.roles.iter().any(|role| role.get() == with_role_id))
            .collect()
    }

    /// Utility function for checking the right role and channel
    pub fn should_process_message(&self, msg: &Message, role_id: u64, channel_id: u64) -> bool {
        // Make sure we're in the right channel
        let correct_channel = msg.channel_id.get() == channel_id;
        let correct_role = {
            let Some(member) = &msg.member else {
                return false;
            };

            if !member.roles.iter().any(|role| role.get() == role_id) {
                return false;
            }

            true
        };

        correct_channel && correct_role
    }

    /// Saves a member into the database
    pub async fn save_member(&self, discord_id: String, name: String) -> Result<(), sqlx::Error> {
        tracing::debug!("Upserting published member with name: {}", name);
        let now = OffsetDateTime::now_utc();

        sqlx::query(
            "INSERT INTO published_members (discord_id, name, created_at, updated_at)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (discord_id) DO UPDATE
             SET name = EXCLUDED.name,
                 updated_at = EXCLUDED.updated_at",
        )
        .bind(discord_id)
        .bind(name)
        .bind(now)
        .bind(now)
        .execute(self.db_pool.as_ref())
        .await?;

        Ok(())
    }

    /// Sends an oblivion message as a "squeak" (post) to the event stream
    pub async fn handle_oblivion_message(&self, msg: &Message, role_id: u64, channel_id: u64) {
        let will_process_message = self.should_process_message(msg, role_id, channel_id);
        if !will_process_message {
            return;
        }

        if let Err(err) = self.publish_message(msg).await {
            tracing::error!("Failed to publish squeak: {}", err);
        }
    }

    /// Publishes a message as a "squeak" (post) to the event stream
    pub async fn publish_message(&self, msg: &Message) -> Result<(), OddbotError> {
        // We need an event stream to be able to publish messages
        let Some(event_stream) = self.event_stream.as_ref() else {
            return Err(OddbotError::InvalidConfig(
                "Event stream not initialized".to_string(),
            ));
        };

        // Check if the user has a character
        let discord_id = msg.author.id.to_string();
        let character = self.character_store.get_character(&discord_id).await?;
        let message = msg.content.clone();

        let Some(character) = character else {
            return Err(OblivionError::NoCharacterFound(discord_id).into());
        };

        // Build the squeak out of the message
        let squeak = Squeak::builder()
            .content(message)
            .user(character.name)
            .await
            .map_err(OddbotError::SqueakPublish)?;

        // Convert the squeak into an Event Stream message
        let message = EventMessage::from(squeak);

        tracing::debug!("Publishing squeak {} to event stream", message.payload.id);
        // Publish the message to the event stream
        Ok(event_stream.publish(message).await?)
    }
}
