use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

use crate::db::DbPool;

/// Health check endpoint that verifies both application and database status.
/// 
/// # Errors
/// Returns 503 Service Unavailable if the database connection fails.
pub async fn health_check(State(pool): State<DbPool>) -> impl IntoResponse {
    let timestamp = chrono::Utc::now();
    
    // Test database connectivity with a simple query
    match sqlx::query("SELECT 1").fetch_one(&pool).await {
        Ok(_) => {
            let response = Json(json!({
                "status": "healthy",
                "database": "connected",
                "timestamp": timestamp
            }));
            (StatusCode::OK, response)
        }
        Err(e) => {
            let response = Json(json!({
                "status": "unhealthy",
                "database": "disconnected",
                "error": e.to_string(),
                "timestamp": timestamp
            }));
            (StatusCode::SERVICE_UNAVAILABLE, response)
        }
    }
}
