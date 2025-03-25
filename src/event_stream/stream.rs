use crate::prelude::*;
use async_nats::{
    Client as NatsClient,
    jetstream::{
        self, Context as Jetstream,
        consumer::{Consumer, pull},
        stream::RetentionPolicy,
    },
};
use serde::Serialize;
use std::time::Duration;

pub struct EventStream {
    stream_name: String,
    jetstream: Jetstream,
}

impl EventStream {
    /// Connect to an existing event stream
    pub async fn connect(
        stream_name: String,
        nats_client: NatsClient,
    ) -> Result<Self, OddbotError> {
        let jetstream = jetstream::new(nats_client);

        Ok(Self {
            stream_name,
            jetstream,
        })
    }

    /// Creates a new queue
    pub async fn new_stream(
        stream_name: String,
        nats_client: NatsClient,
        subjects: Vec<String>,
        description: Option<String>,
    ) -> Result<Self, OddbotError> {
        let jetstream = jetstream::new(nats_client);
        jetstream
            .create_stream(jetstream::stream::Config {
                name: stream_name.clone(),
                subjects,
                description,
                // max_bytes: 1024 * 1024 * 1024, // 1GB
                max_age: Duration::from_secs(60 * 60 * 24 * 30), // 1 month
                ..Default::default()
            })
            .await
            .map_err(OddbotError::StreamCreate)?;

        Ok(Self {
            stream_name,
            jetstream,
        })
    }

    /// Deletes the stream
    pub async fn delete(stream_name: String, nats_client: NatsClient) -> Result<(), OddbotError> {
        let jetstream = jetstream::new(nats_client);

        jetstream
            .delete_stream(stream_name)
            .await
            .map_err(OddbotError::StreamDelete)?;

        Ok(())
    }

    /// Publish an event to the stream
    pub async fn publish<T>(&self, message: EventMessage<T>) -> Result<(), OddbotError>
    where
        T: Serialize,
    {
        let data = serde_json::to_vec(&message.payload).map_err(OddbotError::SerdeError)?;
        self.jetstream
            .publish(message.subject, data.into())
            .await
            .map_err(OddbotError::StreamPublish)?;

        Ok(())
    }

    /// Creates a new consumer
    pub async fn create_consumer(
        &self,
        name: Option<String>,
        filter: String,
    ) -> Result<Consumer<pull::Config>, OddbotError> {
        let config = jetstream::consumer::pull::Config {
            durable_name: name,
            filter_subject: filter,
            max_deliver: 3,
            ..Default::default()
        };

        self.jetstream
            .create_consumer_on_stream(config, self.stream_name.to_string())
            .await
            .map_err(OddbotError::StreamConsumerCreate)
    }
}
