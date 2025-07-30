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

pub async fn create_game(
    State(pool): State<DbPool>,
    Json(create_data): Json<CreateGame>,
) -> Result<impl IntoResponse, ApiError> {
    let game = GameRepository::create(&pool, create_data).await?;
    Ok((StatusCode::CREATED, Json(game)))
}

pub async fn list_games(
    State(pool): State<DbPool>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApiError> {
    let result = GameRepository::list(&pool, params).await?;
    Ok(Json(result))
}

pub async fn get_game(
    State(pool): State<DbPool>,
    Path(hex_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let game = GameRepository::get_by_hex_id(&pool, &hex_id).await?;
    Ok(Json(game))
}

pub async fn update_game(
    State(pool): State<DbPool>,
    Path(hex_id): Path<String>,
    Json(update_data): Json<UpdateGame>,
) -> Result<impl IntoResponse, ApiError> {
    let game = GameRepository::update(&pool, &hex_id, update_data).await?;
    Ok(Json(game))
}

pub async fn delete_game(
    State(pool): State<DbPool>,
    Path(hex_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    GameRepository::soft_delete(&pool, &hex_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
