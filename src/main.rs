use config::Config;
use serenity::all::{ChannelId, GuildId, GuildMemberUpdateEvent, MessageId, Reaction};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::guild::{Member, Role};
use serenity::model::id::RoleId;
use serenity::prelude::*;
use sqlx::{PgPool, types::time::OffsetDateTime};
use std::collections::HashSet;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;

/// Default handler for the Discord bot
struct Handler {
    pub enabled_guilds: HashSet<GuildId>,
    pub db_pool: PgPool,
}

impl Handler {
    /// Create a new handler instance
    fn new(db_pool: PgPool) -> Self {
        let enabled_guilds = Config::get_allowed_guild_ids()
            .into_iter()
            .map(|id| GuildId::new(id))
            .collect();

        Self {
            enabled_guilds,
            db_pool,
        }
    }

    /// Handle a screenshot message
    async fn handle_screenshot(
        &self,
        msg: &Message,
        target_role_id: u64,
        screenshot_channel_id: u64,
    ) {
        // Check if message is in the screenshots channel
        if msg.channel_id.get() != screenshot_channel_id {
            return;
        }

        // Check if user has target role
        if let Some(member) = &msg.member {
            if !member.roles.iter().any(|role| role.get() == target_role_id) {
                return;
            }

            // Get attachments/media from message
            for attachment in &msg.attachments {
                if let Err(e) = self
                    .save_screenshot(
                        msg.author.id.to_string(),
                        msg.author.name.clone(),
                        attachment.url.clone(),
                    )
                    .await
                {
                    tracing::error!("Failed to save screenshot: {:?}", e);
                }
            }
        }
    }

    /// Saves a screenshot to the database
    async fn save_screenshot(
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
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get members with a specific role ID
    async fn get_members(&self, ctx: Context, with_role_id: u64) -> Vec<Member> {
        let mut to_publish_members = Vec::new();

        for guild_id in &self.enabled_guilds {
            tracing::debug!("Attempting to fetch guild {}", guild_id);

            match guild_id.members(&ctx.http, None, None).await {
                Ok(members) => {
                    // Filter out only users with target role
                    let members_with_roles: Vec<_> = members
                        .into_iter()
                        .filter(|member| member.roles.iter().any(|role| role.get() == with_role_id))
                        .collect();
                    if members_with_roles.len() == 0 {
                        tracing::warn!("No members with target role found in guild {}", guild_id);
                        return to_publish_members;
                    }
                    to_publish_members.extend(members_with_roles.into_iter());
                }
                Err(e) => {
                    tracing::error!("Failed to fetch members for guild {}: {:?}", guild_id, e)
                }
            }
        }

        tracing::debug!(
            "Fetched {} members with target role",
            to_publish_members.len()
        );
        to_publish_members
    }

    /// Upserts a published member into the database
    async fn upsert_published_member(
        &self,
        discord_id: String,
        name: String,
    ) -> Result<(), sqlx::Error> {
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
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    /// This event will be dispatched on any message
    async fn message(&self, _ctx: Context, msg: Message) {
        tracing::debug!("Received message: {:?}", msg);
        // Having a screenshot channel ID presumes that we want to handle screenshots
        if let Some(screenshot_channel_id) = Config::get_screenshot_channel_id() {
            // The default behavior is to only allow screenshots from users with a certain role
            // Later, we want probably add more complex logic to handle screenshots, like reaction-based permissions
            // Currently, we don't allow screenshots from users without a certain role
            if let Some(role_id) = Config::get_screenshot_role_id() {
                self.handle_screenshot(&msg, role_id, screenshot_channel_id)
                    .await;
            }
        }
    }

    /// This event will be dispatched when the bot is ready
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Connected as {}", ready.user.name);
        tracing::info!("Enabled guilds: {:?}", self.enabled_guilds);

        // Save published members with configured member role id
        if let Some(role_id) = Config::get_published_member_role_id() {
            tracing::debug!("Grabbing members");
            let members = self.get_members(ctx, role_id).await;
            tracing::debug!("Members found {:?}", members);

            // Upsert members
            for member in members {
                self.upsert_published_member(member.user.id.to_string(), member.user.name)
                    .await
                    .expect("Failed to upsert member");
            }
        }
    }

    /// This event will be dispatched when a member's roles are updated
    async fn guild_member_update(
        &self,
        _ctx: Context,
        old_if_available: Option<Member>,
        new: Option<Member>,
        _update: GuildMemberUpdateEvent,
    ) {
        if let Some(old) = old_if_available {
            if let Some(new) = new {
                tracing::debug!(
                    "Member updated: {} ({:?} -> {:?})",
                    new.user.name,
                    old.roles,
                    new.roles
                );
            }
        }
    }

    /// This event will be dispatched when a role is created
    async fn guild_role_create(&self, _ctx: Context, new: Role) {
        tracing::debug!("New role created: {} in guild", new.name);
    }

    /// This event will be dispatched when a role is deleted
    async fn guild_role_delete(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        _removed_role_id: RoleId,
        removed_role_data: Option<Role>,
    ) {
        tracing::debug!("Role deleted: {:?} in guild", removed_role_data);
    }

    /// This event will be dispatched when a role is updated
    async fn guild_role_update(&self, _ctx: Context, _old_if_available: Option<Role>, new: Role) {
        tracing::debug!("Role updated: {} in guild", new.name);
    }

    /// This event will be dispatched when a reaction is added to a message
    async fn reaction_add(&self, _ctx: Context, reaction: Reaction) {
        tracing::debug!("Reaction added: {:?}", reaction);
    }

    /// This event will be dispatched when a reaction is removed from a message
    async fn reaction_remove(&self, _ctx: Context, reaction: Reaction) {
        tracing::debug!("Reaction removed: {:?}", reaction);
    }

    /// This event will be dispatched when all reactions are removed from a message
    async fn reaction_remove_all(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        message_id: MessageId,
    ) {
        tracing::debug!(
            "All reactions removed from message {} in channel {}",
            message_id,
            channel_id
        );
    }

    /// This event will be dispatched when all reactions of a specific emoji are removed from a message
    async fn reaction_remove_emoji(&self, _ctx: Context, reaction: Reaction) {
        tracing::debug!("All {:?} reactions removed from message", reaction.emoji);
    }
}

#[tokio::main]
async fn main() {
    // Start the tracer
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "discord_bot=debug,serenity=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Intents are a bitflag, bitwise operations can be used to dictate which intents to use
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

    // Initialize our handler
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url)
        .await
        .expect("Could not connect to database");
    let handler = Handler::new(pool);
    // Build our client.
    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
