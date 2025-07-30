use axum::{
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::IntoResponse,
};
use serde::Serialize;
use sqlx::Row;

use crate::{db::DbPool, error::ApiError};

#[derive(Debug, Serialize)]
struct ExportRow {
    // Game fields
    game_hex_id: String,
    game_name: String,
    game_description: Option<String>,
    game_created_at: String,
    game_updated_at: String,
    game_deleted_at: Option<String>,
    
    // Score fields  
    score_id: i64,
    score_value: String,
    score_val: f64,
    user_name: String,
    user_id: String,
    extra: String, // JSON as string
    score_submitted_at: String,
    score_updated_at: String,
    score_deleted_at: Option<String>,
}

/// Exports all game and score data as a CSV file for backup purposes.
/// Returns denormalized data with one row per score, including all game information.
/// 
/// # Errors
/// Returns `ApiError::Database` if the database query fails.
/// Returns `ApiError::ValidationError` if CSV serialization fails.
pub async fn export_data(State(pool): State<DbPool>) -> Result<impl IntoResponse, ApiError> {
    // Query to get denormalized game-score data (including soft-deleted records for complete backup)
    let rows = sqlx::query(
        r#"
        SELECT 
            g.hex_id as game_hex_id,
            g.name as game_name,
            g.description as game_description,
            g.created_at as game_created_at,
            g.updated_at as game_updated_at,
            g.deleted_at as game_deleted_at,
            s.id as score_id,
            s.score as score_value,
            s.score_val,
            s.user_name,
            s.user_id,
            s.extra,
            s.submitted_at as score_submitted_at,
            s.updated_at as score_updated_at,
            s.deleted_at as score_deleted_at
        FROM games g
        LEFT JOIN scores s ON g.hex_id = s.game_hex_id
        ORDER BY g.created_at, s.submitted_at
        "#
    )
    .fetch_all(&pool)
    .await?;

    // Convert to CSV
    let mut csv_output = Vec::new();
    let mut writer = csv::Writer::from_writer(&mut csv_output);

    // Write all rows
    for row in rows {
        let export_row = ExportRow {
            game_hex_id: row.get("game_hex_id"),
            game_name: row.get("game_name"),
            game_description: row.get("game_description"),
            game_created_at: row.get::<chrono::DateTime<chrono::Utc>, _>("game_created_at").to_rfc3339(),
            game_updated_at: row.get::<chrono::DateTime<chrono::Utc>, _>("game_updated_at").to_rfc3339(),
            game_deleted_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("game_deleted_at").map(|dt| dt.to_rfc3339()),
            score_id: row.get::<Option<i64>, _>("score_id").unwrap_or(0),
            score_value: row.get::<Option<String>, _>("score_value").unwrap_or_default(),
            score_val: row.get::<Option<f64>, _>("score_val").unwrap_or(0.0),
            user_name: row.get::<Option<String>, _>("user_name").unwrap_or_default(),
            user_id: row.get::<Option<String>, _>("user_id").unwrap_or_default(),
            extra: row.get::<Option<String>, _>("extra").unwrap_or_default(),
            score_submitted_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("score_submitted_at").map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            score_updated_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("score_updated_at").map(|dt| dt.to_rfc3339()).unwrap_or_default(),
            score_deleted_at: row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("score_deleted_at").map(|dt| dt.to_rfc3339()),
        };
        
        writer.serialize(&export_row).map_err(|e| {
            ApiError::ValidationError(format!("Failed to serialize CSV row: {e}"))
        })?;
    }

    writer.flush().map_err(|e| {
        ApiError::ValidationError(format!("Failed to flush CSV writer: {e}"))
    })?;

    // Drop the writer to release the borrow on csv_output
    drop(writer);

    // Generate filename with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("leadr_backup_{timestamp}.csv");

    // Create headers
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
            .map_err(|e| ApiError::ValidationError(format!("Invalid header value: {e}")))?,
    );

    // Return CSV response with appropriate headers
    let response = (StatusCode::OK, headers, csv_output);

    Ok(response)
}