use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OblivionServerError {
    #[error("Failed to save event")]
    FailedToSaveEvent,
    #[error("Failed to publish event")]
    FailedToPublishEvent,
}

impl IntoResponse for OblivionServerError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            OblivionServerError::FailedToPublishEvent => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to publish event")
            }
            OblivionServerError::FailedToSaveEvent => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save event")
            }
        };
        (status, error_message).into_response()
    }
}
