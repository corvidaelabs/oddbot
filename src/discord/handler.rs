use crate::config::Config;
use serenity::all::{Context, GuildId, Member, Message};
use sqlx::{PgPool, types::time::OffsetDateTime};
use std::{collections::HashSet, sync::Arc};

/// Default handler for the Discord bot
pub struct Handler {
    pub enabled_guilds: HashSet<GuildId>,
    pub db_pool: Arc<PgPool>,
}

impl Handler {
    /// Create a new handler instance
    pub fn new(db_pool: Arc<PgPool>) -> Self {
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
    pub async fn handle_screenshot(
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
        .execute(self.db_pool.as_ref())
        .await?;

        Ok(())
    }

    /// Get members with a specific role ID
    pub async fn get_members(&self, ctx: Context, with_role_id: u64) -> Vec<Member> {
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
    pub async fn upsert_published_member(
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
        .execute(self.db_pool.as_ref())
        .await?;

        Ok(())
    }
}
