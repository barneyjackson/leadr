use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error")]
    InternalServerError,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            ApiError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::ValidationError(ref msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.as_str()),
            ApiError::InvalidParameter(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}

impl From<String> for ApiError {
    fn from(msg: String) -> Self {
        ApiError::ValidationError(msg)
    }
}

pub type Result<T> = std::result::Result<T, ApiError>;
