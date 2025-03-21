use crate::config::Config;
use serenity::all::{Context, GuildId, Member, Message};
use sqlx::{PgPool, types::time::OffsetDateTime};
use std::sync::Arc;

/// Default handler for the Discord bot
pub struct Handler {
    pub guild_id: Option<GuildId>,
    pub db_pool: Arc<PgPool>,
}

impl Handler {
    /// Create a new handler instance
    pub fn new(db_pool: Arc<PgPool>) -> Self {
        let guild_id = {
            // Check if we're configured to run against a specific guild
            let guild_id = Config::get_guild_id();
            match guild_id {
                Some(id) => Some(GuildId::new(id)),
                None => None,
            }
        };

        Self { guild_id, db_pool }
    }

    /// Validates a screenshot message and saves it to the database
    pub async fn handle_ss(&self, msg: &Message, target_role_id: u64, screenshot_channel_id: u64) {
        // Check if message is in the screenshots channel
        if msg.channel_id.get() != screenshot_channel_id {
            return;
        }

        // Make sure the message has a member associated with it
        let Some(member) = &msg.member else {
            return;
        };

        // Make sure the member has the target role
        if !member.roles.iter().any(|role| role.get() == target_role_id) {
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
        match guild_id.members(&ctx.http, None, None).await {
            Ok(members) => {
                tracing::debug!("Fetched {} members", members.len());
                // Only grab members that have our target role
                members
                    .into_iter()
                    .filter(|member| member.roles.iter().any(|role| role.get() == with_role_id))
                    .collect()
            }
            Err(e) => {
                tracing::error!("Failed to fetch members for guild {}: {:?}", guild_id, e);
                Vec::new()
            }
        }
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
}
