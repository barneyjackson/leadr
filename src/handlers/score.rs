use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    db::{
        repository::{GameRepository, ScoreRepository},
        DbPool,
    },
    error::ApiError,
    models::score::{CreateScore, Score, UpdateScore},
    utils::pagination::ScoreQueryParams,
};

/// Creates a new score for a specific game.
/// 
/// # Errors
/// Returns `ApiError::ValidationError` if user name, user ID, or JSON data is invalid.
/// Returns `ApiError::NotFound` if the game does not exist.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn create_score(
    State(pool): State<DbPool>,
    Json(create_data): Json<CreateScore>,
) -> Result<impl IntoResponse, ApiError> {
    // Validate the input data first (this will return 422 if invalid)
    Score::validate_user_name(&create_data.user_name)?;
    Score::validate_user_id(&create_data.user_id)?;

    // Then check if the game exists (this will return 404 if not found)
    if GameRepository::get_by_hex_id(&pool, &create_data.game_hex_id)
        .await
        .is_err()
    {
        return Err(ApiError::NotFound);
    }

    let score = ScoreRepository::create(&pool, create_data).await?;
    Ok((StatusCode::CREATED, Json(score)))
}

/// Lists scores with optional game filtering, pagination and sorting support.
/// 
/// # Errors
/// Returns `ApiError::ValidationError` if pagination or sort parameters are invalid.
/// Returns `ApiError::InvalidParameter` if the game hex_id format is invalid.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn list_scores(
    State(pool): State<DbPool>,
    RawQuery(query_string): RawQuery,
) -> Result<impl IntoResponse, ApiError> {
    // Parse query parameters manually to provide better error messages
    let query_str = query_string.unwrap_or_default();

    // Parse all query parameters together
    let query_params = serde_urlencoded::from_str::<ScoreQueryParams>(&query_str)
        .map_err(|e| ApiError::ValidationError(format!("Invalid query parameters: {e}")))?;

    // If game_hex_id is provided, list scores for that game, otherwise list all scores
    let result = if let Some(ref game_hex_id) = query_params.game_hex_id {
        ScoreRepository::list_by_game(&pool, game_hex_id, query_params.to_pagination_params(), query_params.to_sort_params()).await?
    } else {
        ScoreRepository::list_all(&pool, query_params.to_pagination_params(), query_params.to_sort_params()).await?
    };
    Ok(Json(result))
}

/// Retrieves a specific score by its ID.
/// 
/// # Errors
/// Returns `ApiError::NotFound` if no score exists with the given ID.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn get_score(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let score = ScoreRepository::get_by_id(&pool, id).await?;
    Ok(Json(score))
}

/// Updates an existing score.
/// 
/// # Errors
/// Returns `ApiError::ValidationError` if user name, user ID, or JSON data is invalid.
/// Returns `ApiError::NotFound` if no score exists with the given ID.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn update_score(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(update_data): Json<UpdateScore>,
) -> Result<impl IntoResponse, ApiError> {
    let score = ScoreRepository::update(&pool, id, update_data).await?;
    Ok(Json(score))
}

/// Soft deletes a score (marks as deleted without removing from database).
/// 
/// # Errors
/// Returns `ApiError::NotFound` if no score exists with the given ID.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn delete_score(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    ScoreRepository::soft_delete(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
