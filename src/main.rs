use std::sync::Arc;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod discord;
mod error;
mod event_stream;

pub use config::Config;
pub use discord::bot::DiscordBot;
pub use error::OddbotError;

#[tokio::main]
async fn main() -> Result<(), OddbotError> {
    // Start the tracer
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "discord_bot=debug,serenity=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize our database pool
    let pool = Arc::new(db::create_db_pool().await?);

    // Initialize our bot
    let mut oddbot = DiscordBot::init(pool).await?;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = oddbot.client.start().await {
        println!("Discord Client error: {why:?}");
    }

    Ok(())
}
