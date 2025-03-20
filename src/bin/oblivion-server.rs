use oddbot::event_stream::create_nats_client;
use oddbot::prelude::*;

#[tokio::main]
async fn main() -> Result<(), OddbotError> {
    // Create a connection to the Oblivion event stream
    let stream_name = std::env::var("OBLIVION_STREAM_NAME").map_err(OddbotError::EnvVar)?;
    let nats_client = create_nats_client().await?;
    let _event_stream = EventStream::connect(stream_name, nats_client).await?;

    // TODO: Initialize an axum server
    // TODO: Create a websockets endpoint
    // TODO: Subscribe to events with a consumer
    // TODO: Pass those events to the websockets server

    Ok(())
}
