use async_nats::jetstream;
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt, stream::SplitSink};
use oddbot::{error::OddbotError, prelude::EventStream, skeever::squeak::Squeak};
use std::{sync::Arc, time::Duration};
use tokio::sync::broadcast;

pub async fn forward_events_to_websockets(
    listener: Arc<EventStream>,
    event_sender: broadcast::Sender<Squeak>,
) {
    let skeever_subject = Squeak::get_subject();

    let consumer = match listener
        .create_consumer(
            Some("oblivion_websocket_main_consumer".to_string()),
            skeever_subject.clone(),
            Some(jetstream::consumer::DeliverPolicy::New),
        )
        .await
    {
        Ok(consumer) => consumer,
        Err(e) => {
            tracing::error!("Failed to create consumer: {:?}", e);
            return;
        }
    };

    loop {
        let Ok(mut messages) = consumer.fetch().max_messages(20).messages().await else {
            tracing::error!("Failed to fetch messages");
            continue;
        };

        while let Some(message) = messages.next().await {
            let Ok(message) = message else {
                tracing::error!("Failed to receive message");
                continue;
            };

            let Ok(squeak) = serde_json::from_slice::<Squeak>(&message.payload) else {
                tracing::error!("Failed to deserialize event");
                continue;
            };

            tracing::debug!("Successfully deserialized squeak {:?}", squeak);
            match event_sender.receiver_count() {
                0 => {
                    tracing::trace!("No active websocket connections, ignore squeak");
                }
                n => match event_sender.send(squeak) {
                    Ok(_) => {
                        tracing::debug!("Successfully broadcast event to {} receivers", n);
                    }
                    Err(e) => {
                        tracing::error!("Failed to broadcast event: {}", e);
                    }
                },
            };

            if let Err(e) = message.ack().await {
                tracing::error!("Failed to ack message: {}", e);
            }
        }

        // Add a small delay between fetches
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    pub historical_batch_size: usize,
    pub historical_max_age: Option<Duration>,
    pub historical_batch_delay: Duration,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            historical_batch_size: 100,
            historical_max_age: Some(Duration::from_secs(60 * 60 * 24)), // 24 hours
            historical_batch_delay: Duration::from_millis(50),
        }
    }
}

pub async fn send_historical_messages(
    listener: Arc<EventStream>,
    mut ws_sender: SplitSink<WebSocket, Message>,
) -> Result<SplitSink<WebSocket, Message>, OddbotError> {
    let skeever_subject = Squeak::get_subject();
    let batch_size = 100;

    let temp_consumer = listener
        .create_consumer(
            None,
            skeever_subject,
            Some(jetstream::consumer::DeliverPolicy::All),
        )
        .await?;

    loop {
        let mut messages = temp_consumer
            .fetch()
            .max_messages(batch_size)
            .messages()
            .await?;

        let mut batch_count = 0;
        while let Some(message) = messages.next().await {
            let Ok(message) = message else {
                tracing::error!("Failed to fetch message");
                continue;
            };
            let squeak = serde_json::from_slice::<Squeak>(&message.payload)?;

            ws_sender
                .send(Message::Text(serde_json::to_string(&squeak)?.into()))
                .await
                .map_err(|e| OddbotError::WebsocketSend(e.to_string()))?;

            message
                .ack()
                .await
                .map_err(|e| OddbotError::WebsocketSend(e.to_string()))?; // Convert ack error

            batch_count += 1;
        }

        if batch_count < batch_size {
            break; // No more messages
        }

        // Optional: Add a small delay between batches to prevent overwhelming the client
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    Ok(ws_sender)
}
