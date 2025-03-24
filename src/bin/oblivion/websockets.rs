use futures::StreamExt;
use oddbot::{prelude::EventStream, skeever::squeak::Squeak};
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn forward_events_to_websockets(
    listener: Arc<EventStream>,
    event_sender: broadcast::Sender<Squeak>,
) {
    let skeever_subject = Squeak::get_subject();
    let consumer_name = format!("oblivion_websocket_consumer_{}", ulid::Ulid::new());

    let Ok(consumer) = listener
        .create_consumer(Some(consumer_name), skeever_subject)
        .await
    else {
        tracing::error!("Failed to create consumer");
        return;
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
            }
        }

        // Add a small delay between fetches
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
