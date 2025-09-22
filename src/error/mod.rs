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

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

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

    #[error("Unsupported database: {database_type}")]
    UnsupportedDatabase { database_type: String },

    #[error("Database provider mismatch: expected {expected}, got {actual}")]
    ProviderMismatch { expected: String, actual: String },

    #[error("Connection pool error: {0}")]
    ConnectionPool(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::Migration(err) => {
                tracing::error!("Migration error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database migration error",
                )
            }
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found"),
            ApiError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            ApiError::ValidationError(ref msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg.as_str()),
            ApiError::InvalidParameter(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            ApiError::UnsupportedDatabase { .. } => {
                tracing::error!("Unsupported database error: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database configuration error")
            }
            ApiError::ProviderMismatch { .. } => {
                tracing::error!("Database provider mismatch: {:?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database configuration error")
            }
            ApiError::ConnectionPool(ref msg) => {
                tracing::error!("Connection pool error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database connection error")
            }
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
