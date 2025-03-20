use thiserror::Error;

#[derive(Error, Debug)]
pub enum OddbotError {
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Environment variable error")]
    EnvVar(#[from] std::env::VarError),
    #[error("Invalid configuration")]
    InvalidConfig,
}
