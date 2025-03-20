use std::{env, sync::Arc};

use serenity::{Client, all::GatewayIntents};
use sqlx::PgPool;

use crate::OddbotError;

use super::handler::Handler;

#[allow(dead_code)] // TODO: Remove after we use db_pool
pub struct DiscordBot {
    pub client: Client,
    db_pool: Arc<PgPool>,
}

impl DiscordBot {
    pub async fn init(db_pool: Arc<PgPool>) -> Result<Self, OddbotError> {
        // Get our discord token
        let discord_token = env::var("DISCORD_TOKEN").map_err(OddbotError::EnvVar)?;

        // Create a handler for handling Discord events
        let handler = Handler::new(db_pool.clone());

        // Declare our intents for events we're going to listen to
        let intents = GatewayIntents::GUILDS
            | GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT
            | GatewayIntents::GUILD_MEMBERS
            | GatewayIntents::GUILD_MESSAGE_REACTIONS;

        // Build our client.
        let client = Client::builder(discord_token, intents)
            .event_handler(handler)
            .await
            .expect("Error creating client");

        Ok(Self { client, db_pool })
    }
}
