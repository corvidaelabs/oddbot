use std::sync::Arc;

use oddbot::{db, prelude::*};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), OddbotError> {
    // Start the tracer
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "oddbot=info".into()),
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
        tracing::error!("Discord Client error: {why:?}");
    }

    Ok(())
}
