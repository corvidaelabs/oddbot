use super::{character::CharacterStore, handler::Handler};
use crate::prelude::*;
use serenity::{Client, all::GatewayIntents};
use sqlx::PgPool;
use std::{env, sync::Arc};

#[allow(dead_code)] // TODO: Remove after we use db_pool
pub struct DiscordBot {
    pub client: Client,
    db_pool: Arc<PgPool>,
    event_stream: Option<Arc<EventStream>>,
    character_store: Arc<CharacterStore>,
}

impl DiscordBot {
    pub async fn init(
        db_pool: Arc<PgPool>,
        event_stream: Option<Arc<EventStream>>,
        character_store: Arc<CharacterStore>,
    ) -> Result<Self, OddbotError> {
        // Get our discord token
        let discord_token = env::var("DISCORD_TOKEN").map_err(OddbotError::EnvVar)?;

        // Create a handler for handling Discord events
        let handler = Handler::new(
            db_pool.clone(),
            event_stream.clone(),
            character_store.clone(),
        );

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

        Ok(Self {
            client,
            db_pool,
            event_stream,
            character_store,
        })
    }
}
