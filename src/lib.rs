pub mod auth;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod utils;

use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;

use crate::{auth::api_key_middleware, db::DbPool};

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::health::health_check,
        handlers::game::create_game,
        handlers::game::list_games,
        handlers::game::get_game,
        handlers::game::update_game,
        handlers::game::delete_game,
        handlers::score::create_score,
        handlers::score::list_scores,
        handlers::score::get_score,
        handlers::score::update_score,
        handlers::score::delete_score,
        handlers::export::export_data
    ),
    components(
        schemas(
            models::Game,
            models::CreateGame,
            models::UpdateGame,
            models::Score,
            models::CreateScore,
            models::UpdateScore,
            models::PaginatedResponse<models::Game>,
            models::PaginatedResponse<models::Score>,
            utils::pagination::PaginationParams,
            utils::pagination::ScoreQueryParams,
            utils::pagination::ScoreSortField,
            utils::pagination::SortOrder
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "Games", description = "Game/Leaderboard management"),
        (name = "Scores", description = "Score management"),
        (name = "Export", description = "Data export operations")
    ),
    info(
        title = "LEADR API",
        version = "1.0.0",
        description = "A blazingly fast, single-tenant leaderboard API built for indie game developers",
        contact(
            name = "LEADR Support",
            url = "https://github.com/barneyjackson/leadr"
        )
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("leadr-api-key"))),
            );
        }
    }
}

pub fn create_app(pool: DbPool) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new().route("/health", get(handlers::health::health_check));

    // Protected routes (require API key)
    let protected_routes = Router::new()
        .route("/games", get(handlers::game::list_games))
        .route("/games", post(handlers::game::create_game))
        .route("/games/:hex_id", get(handlers::game::get_game))
        .route("/games/:hex_id", put(handlers::game::update_game))
        .route("/games/:hex_id", delete(handlers::game::delete_game))
        .route("/scores", get(handlers::score::list_scores))
        .route("/scores", post(handlers::score::create_score))
        .route("/scores/:id", get(handlers::score::get_score))
        .route("/scores/:id", put(handlers::score::update_score))
        .route("/scores/:id", delete(handlers::score::delete_score))
        .route("/export", get(handlers::export::export_data))
        .layer(middleware::from_fn(api_key_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .with_state(pool)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
}
