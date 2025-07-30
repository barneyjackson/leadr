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
    utils::pagination::{PaginationParams, ScoreSortParams},
};

pub async fn create_score(
    State(pool): State<DbPool>,
    Path(game_hex_id): Path<String>,
    Json(mut create_data): Json<CreateScore>,
) -> Result<impl IntoResponse, ApiError> {
    // Set the game_hex_id from the path first
    create_data.game_hex_id.clone_from(&game_hex_id);

    // Validate the input data first (this will return 422 if invalid)
    Score::validate_user_name(&create_data.user_name)?;
    Score::validate_user_id(&create_data.user_id)?;

    // Then check if the game exists (this will return 404 if not found)
    if GameRepository::get_by_hex_id(&pool, &game_hex_id)
        .await
        .is_err()
    {
        return Err(ApiError::NotFound);
    }

    let score = ScoreRepository::create(&pool, create_data).await?;
    Ok((StatusCode::CREATED, Json(score)))
}

pub async fn list_scores(
    State(pool): State<DbPool>,
    Path(game_hex_id): Path<String>,
    RawQuery(query_string): RawQuery,
) -> Result<impl IntoResponse, ApiError> {
    // Parse query parameters manually to provide better error messages
    let query_str = query_string.unwrap_or_default();

    // Parse pagination parameters
    let pagination = serde_urlencoded::from_str::<PaginationParams>(&query_str)
        .map_err(|e| ApiError::ValidationError(format!("Invalid pagination parameters: {e}")))?;

    // Parse sort parameters
    let sort_params = serde_urlencoded::from_str::<ScoreSortParams>(&query_str)
        .map_err(|e| ApiError::ValidationError(format!("Invalid sort parameters: {e}")))?;

    let result =
        ScoreRepository::list_by_game(&pool, &game_hex_id, pagination, sort_params).await?;
    Ok(Json(result))
}

pub async fn get_score(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    let score = ScoreRepository::get_by_id(&pool, id).await?;
    Ok(Json(score))
}

pub async fn update_score(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Json(update_data): Json<UpdateScore>,
) -> Result<impl IntoResponse, ApiError> {
    let score = ScoreRepository::update(&pool, id, update_data).await?;
    Ok(Json(score))
}

pub async fn delete_score(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, ApiError> {
    ScoreRepository::soft_delete(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
