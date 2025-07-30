use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use utoipa::OpenApi;

use crate::{
    db::{repository::GameRepository, DbPool},
    error::ApiError,
    models::{game::{CreateGame, UpdateGame, Game}, PaginatedResponse},
    utils::pagination::PaginationParams,
};

/// Creates a new game.
/// 
/// # Errors
/// Returns `ApiError::ValidationError` if the game name is invalid.
/// Returns `ApiError::DatabaseError` if the database operation fails.
#[utoipa::path(
    post,
    path = "/games",
    request_body = CreateGame,
    responses(
        (status = 201, description = "Game created successfully", body = Game),
        (status = 400, description = "Invalid game data"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Games"
)]
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
#[utoipa::path(
    get,
    path = "/games",
    params(
        PaginationParams
    ),
    responses(
        (status = 200, description = "List of games", body = PaginatedResponse<Game>),
        (status = 400, description = "Invalid pagination parameters"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Games"
)]
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
#[utoipa::path(
    get,
    path = "/games/{hex_id}",
    params(
        ("hex_id" = String, Path, description = "6-character game identifier")
    ),
    responses(
        (status = 200, description = "Game found", body = Game),
        (status = 400, description = "Invalid hex_id format"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 404, description = "Game not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Games"
)]
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
#[utoipa::path(
    put,
    path = "/games/{hex_id}",
    params(
        ("hex_id" = String, Path, description = "6-character game identifier")
    ),
    request_body = UpdateGame,
    responses(
        (status = 200, description = "Game updated successfully", body = Game),
        (status = 400, description = "Invalid data"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 404, description = "Game not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Games"
)]
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
#[utoipa::path(
    delete,
    path = "/games/{hex_id}",
    params(
        ("hex_id" = String, Path, description = "6-character game identifier")
    ),
    responses(
        (status = 204, description = "Game deleted successfully"),
        (status = 400, description = "Invalid hex_id format"),
        (status = 401, description = "Missing or invalid API key"),
        (status = 404, description = "Game not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = [])
    ),
    tag = "Games"
)]
pub async fn delete_game(
    State(pool): State<DbPool>,
    Path(hex_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    GameRepository::soft_delete(&pool, &hex_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
