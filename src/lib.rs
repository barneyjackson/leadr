pub mod auth;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod utils;

use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::{auth::api_key_middleware, db::DbPool};

pub fn create_app(pool: DbPool) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new().route("/health", get(handlers::health::health_check));

    // Protected routes (require API key)
    let protected_routes = Router::new()
        .route("/games", get(handlers::game::list_games))
        .route("/games", post(handlers::game::create_game))
        .route("/games/:hex_id", get(handlers::game::get_game))
        .route("/games/:hex_id", put(handlers::game::update_game))
        .route(
            "/games/:game_hex_id/scores",
            get(handlers::score::list_scores),
        )
        .route(
            "/games/:game_hex_id/scores",
            post(handlers::score::create_score),
        )
        .route("/scores/:id", get(handlers::score::get_score))
        .route("/scores/:id", put(handlers::score::update_score))
        .layer(middleware::from_fn(api_key_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(pool)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
