use async_nats::{
    ConnectErrorKind,
    jetstream::context::{CreateStreamError, DeleteStreamError, PublishError},
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
    #[error("Environment variable error")]
    EnvVar(#[from] std::env::VarError),
    #[error("Invalid configuration")]
    InvalidConfig,
}
