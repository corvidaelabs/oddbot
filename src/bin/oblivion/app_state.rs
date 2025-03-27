use oddbot::{nats::create_nats_client, prelude::*, skeever::squeak::Squeak};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct AppState {
    pub event_sender: broadcast::Sender<Squeak>,
}

impl AppState {
    pub async fn init() -> Result<Self, OddbotError> {
        let (event_sender, _) = broadcast::channel(100); // Adjust buffer size as needed

        Ok(Self { event_sender })
    }

    pub async fn get_event_stream(&self) -> Result<Arc<EventStream>, OddbotError> {
        let stream_name = std::env::var("EVENT_STREAM_NAME").map_err(OddbotError::EnvVar)?;
        let nats_client = create_nats_client().await?;
        let event_stream = EventStream::connect(stream_name, nats_client).await?;

        Ok(Arc::new(event_stream))
    }
}
