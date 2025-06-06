use crate::{discord, skeever::squeak::SqueakError};
use async_nats::{
    ConnectErrorKind,
    jetstream::{
        consumer::pull::BatchErrorKind,
        context::{CreateStreamError, DeleteStreamError, PublishError},
        stream::ConsumerError,
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OddbotError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Serialization error")]
    SerdeError(#[from] serde_json::Error),
    #[error("Error creating stream")]
    StreamCreate(#[from] CreateStreamError),
    #[error("Error deleting stream")]
    StreamDelete(#[from] DeleteStreamError),
    #[error("Error publishing event")]
    StreamPublish(#[from] PublishError),
    #[error("NATS connection error")]
    NatsConnect(#[from] async_nats::error::Error<ConnectErrorKind>),
    #[error("Error batching nats messages")]
    NatsBatch(#[from] async_nats::error::Error<BatchErrorKind>),
    #[error("Environment variable error")]
    EnvVar(#[from] std::env::VarError),
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("Squeak publish error")]
    SqueakPublish(SqueakError),
    #[error("Error creating consumer")]
    StreamConsumerCreate(#[from] ConsumerError),
    #[error("Error with Oblivion functionality")]
    OblivionError(#[from] discord::character::OblivionError),
    #[error("Error with serenity functionality")]
    Serenity(#[from] serenity::Error),
    #[error("Error sending websockets message to client")]
    WebsocketSend(String),
}
