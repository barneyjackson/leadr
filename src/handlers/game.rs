use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    db::{repository::GameRepository, DbPool},
    error::ApiError,
    models::game::{CreateGame, UpdateGame},
    utils::pagination::PaginationParams,
};

/// Creates a new game.
/// 
/// # Errors
/// Returns `ApiError::ValidationError` if the game name is invalid.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn create_game(
    State(pool): State<DbPool>,
    Json(create_data): Json<CreateGame>,
) -> Result<impl IntoResponse, ApiError> {
    let game = GameRepository::create(&pool, create_data).await?;
    Ok((StatusCode::CREATED, Json(game)))
}

/// Lists games with pagination support.
/// 
/// # Errors
/// Returns `ApiError::ValidationError` if pagination parameters are invalid.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn list_games(
    State(pool): State<DbPool>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApiError> {
    let result = GameRepository::list(&pool, params).await?;
    Ok(Json(result))
}

/// Retrieves a specific game by its hex ID.
/// 
/// # Errors
/// Returns `ApiError::InvalidParameter` if the hex_id format is invalid.
/// Returns `ApiError::NotFound` if no game exists with the given hex_id.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn get_game(
    State(pool): State<DbPool>,
    Path(hex_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let game = GameRepository::get_by_hex_id(&pool, &hex_id).await?;
    Ok(Json(game))
}

/// Updates an existing game.
/// 
/// # Errors
/// Returns `ApiError::InvalidParameter` if the hex_id format or name is invalid.
/// Returns `ApiError::NotFound` if no game exists with the given hex_id.
/// Returns `ApiError::ValidationError` if the update data is invalid.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn update_game(
    State(pool): State<DbPool>,
    Path(hex_id): Path<String>,
    Json(update_data): Json<UpdateGame>,
) -> Result<impl IntoResponse, ApiError> {
    let game = GameRepository::update(&pool, &hex_id, update_data).await?;
    Ok(Json(game))
}

/// Soft deletes a game (marks as deleted without removing from database).
/// 
/// # Errors
/// Returns `ApiError::InvalidParameter` if the hex_id format is invalid.
/// Returns `ApiError::NotFound` if no game exists with the given hex_id.
/// Returns `ApiError::DatabaseError` if the database operation fails.
pub async fn delete_game(
    State(pool): State<DbPool>,
    Path(hex_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    GameRepository::soft_delete(&pool, &hex_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
