use oddbot::{db, discord::character::CharacterStore, nats::create_nats_client, prelude::*};
use std::sync::Arc;
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
    let event_stream_name = OddbotConfig::get_event_stream_name();

    let event_stream = match event_stream_name {
        Some(name) => {
            let event_nats = create_nats_client().await?;
            let event_stream = EventStream::connect(name, event_nats).await?;
            Some(Arc::new(event_stream))
        }
        None => {
            tracing::warn!("Starting without event stream");
            None
        }
    };

    // Connect to our Oblivion character store
    let character_nats = create_nats_client().await?;
    let character_store = Arc::new(CharacterStore::new(character_nats).await?);

    // Initialize our bot
    let mut oddbot = DiscordBot::init(pool, event_stream, character_store).await?;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = oddbot.client.start().await {
        tracing::error!("Discord Client error: {why:?}");
    }

    Ok(())
}
