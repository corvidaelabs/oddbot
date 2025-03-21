use std::sync::Arc;

use oddbot::{
    db,
    event_stream::{create_nats_client, nats::get_nats_url},
    prelude::*,
};
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

    // Connect to our database
    let pool = Arc::new(db::create_db_pool().await?);

    // Connect to our event stream
    let event_stream_name = Config::get_event_stream_name();

    let event_stream = match event_stream_name {
        Some(name) => {
            let nats_client = create_nats_client().await?;
            let event_stream = EventStream::connect(name, nats_client).await?;
            Some(Arc::new(event_stream))
        }
        None => None,
    };

    // Initialize our bot
    let mut oddbot = DiscordBot::init(pool, event_stream).await?;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = oddbot.client.start().await {
        tracing::error!("Discord Client error: {why:?}");
    }

    Ok(())
}
