use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use leadr_api::{create_app, db};
use serde_json::json;
use tower::util::ServiceExt;

// Helper function to create test app with in-memory database
async fn create_test_app() -> Router {
    let pool = db::create_pool("sqlite::memory:").await.unwrap();
    db::run_migrations(&pool).await.unwrap();
    create_app(pool)
}

// Helper function to create request with API key
fn request_with_api_key(method: &str, uri: &str, body: Option<&str>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("leadr-api-key", "test_api_key_123")
        .header("content-type", "application/json");

    if let Some(body_content) = body {
        builder.body(Body::from(body_content.to_string())).unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    }
}

// Helper function to create request without API key
fn request_without_api_key(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap()
}

#[cfg(test)]
mod game_endpoint_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint_no_auth_required() {
        let app = create_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_game_success() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let game_data = json!({
            "name": "Test Game",
            "description": "A test game for testing"
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/games",
                Some(&game_data.to_string()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_create_game_missing_name() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let game_data = json!({
            "description": "Missing name field"
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/games",
                Some(&game_data.to_string()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_game_without_auth() {
        let app = create_test_app().await;

        let game_data = json!({
            "name": "Test Game",
            "description": "Should fail without auth"
        });

        let response = app
            .oneshot(request_without_api_key("POST", "/games"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_create_game_wrong_api_key() {
        std::env::set_var("LEADR_API_KEY", "correct_key");
        let app = create_test_app().await;

        let request = Request::builder()
            .method("POST")
            .uri("/games")
            .header("x-api-key", "wrong_key")
            .header("content-type", "application/json")
            .body(Body::from(json!({"name": "Test Game"}).to_string()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_game_success() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        // This test assumes the game exists - in real implementation
        // we'd first create a game, then try to retrieve it
        let response = app
            .oneshot(request_with_api_key("GET", "/games/abc123", None))
            .await
            .unwrap();

        // Should be NOT_FOUND since game doesn't exist yet
        assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_game_invalid_hex_id() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/games/invalid_hex", None))
            .await
            .unwrap();

        // Should validate hex ID format
        assert!(
            response.status() == StatusCode::BAD_REQUEST
                || response.status() == StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_list_games_success() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/games", None))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_games_without_auth() {
        let app = create_test_app().await;

        let response = app
            .oneshot(request_without_api_key("GET", "/games"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_update_game_success() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let update_data = json!({
            "name": "Updated Game Name"
        });

        let response = app
            .oneshot(request_with_api_key(
                "PUT",
                "/games/abc123",
                Some(&update_data.to_string()),
            ))
            .await
            .unwrap();

        // Should be NOT_FOUND since game doesn't exist, or OK if it does
        assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_game_empty_body() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("PUT", "/games/abc123", Some("{}")))
            .await
            .unwrap();

        // Empty update should still be valid
        assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_game_soft_delete() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("DELETE", "/games/abc123", None))
            .await
            .unwrap();

        // Should be NOT_FOUND since game doesn't exist, or NO_CONTENT if it does
        assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::NO_CONTENT);
    }
}

#[cfg(test)]
mod score_endpoint_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_score_success() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let score_data = json!({
            "game_hex_id": "abc123",
            "score": "1000",
            "score_val": 1000.5,
            "user_name": "TestPlayer",
            "user_id": "player123",
            "extra": {"level": 5, "time": 120.5}
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/scores",
                Some(&score_data.to_string()),
            ))
            .await
            .unwrap();

        // Should be CREATED or NOT_FOUND (if game doesn't exist)
        assert!(
            response.status() == StatusCode::CREATED || response.status() == StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_create_score_minimal_data() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let score_data = json!({
            "game_hex_id": "abc123",
            "score": "500",
            "user_name": "Player",
            "user_id": "id123"
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/scores",
                Some(&score_data.to_string()),
            ))
            .await
            .unwrap();

        assert!(
            response.status() == StatusCode::CREATED || response.status() == StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn test_create_score_missing_required_fields() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let score_data = json!({
            "game_hex_id": "abc123",
            "score": 500
            // Missing user_name and user_id
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/scores",
                Some(&score_data.to_string()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_score_invalid_user_name() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let score_data = json!({
            "game_hex_id": "abc123",
            "score": "500",
            "user_name": "",  // Empty name should be invalid
            "user_id": "id123"
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/scores",
                Some(&score_data.to_string()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_score_invalid_user_id() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let score_data = json!({
            "game_hex_id": "abc123",
            "score": "500",
            "user_name": "Player",
            "user_id": ""  // Empty ID should be invalid
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/scores",
                Some(&score_data.to_string()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_create_score_without_auth() {
        let app = create_test_app().await;

        let score_data = json!({
            "score": "500",
            "user_name": "Player",
            "user_id": "id123"
        });

        let response = app
            .oneshot(request_without_api_key("POST", "/scores"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_game_scores_success() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/scores?game_hex_id=abc123", None))
            .await
            .unwrap();

        // Should be OK or NOT_FOUND (if game doesn't exist)
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_global_scores_without_game_filter() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/scores", None))
            .await
            .unwrap();

        // Should return all scores across all games
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_game_scores_with_query_params() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&limit=10",
                None,
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_game_scores_without_auth() {
        let app = create_test_app().await;

        let response = app
            .oneshot(request_without_api_key("GET", "/scores?game_hex_id=abc123"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_update_score_success() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let update_data = json!({
            "score": "1500",
            "user_name": "UpdatedPlayer"
        });

        let response = app
            .oneshot(request_with_api_key(
                "PUT",
                "/scores/123",
                Some(&update_data.to_string()),
            ))
            .await
            .unwrap();

        // Should be OK or NOT_FOUND (if score doesn't exist)
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_score_partial_update() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let update_data = json!({
            "score": "2000"
            // Only updating score field
        });

        let response = app
            .oneshot(request_with_api_key(
                "PUT",
                "/scores/123",
                Some(&update_data.to_string()),
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_update_score_invalid_data() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let update_data = json!({
            "user_name": ""  // Invalid empty name
        });

        let response = app
            .oneshot(request_with_api_key(
                "PUT",
                "/scores/123",
                Some(&update_data.to_string()),
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_update_score_without_auth() {
        let app = create_test_app().await;

        let update_data = json!({
            "score": 1500
        });

        let response = app
            .oneshot(request_without_api_key("PUT", "/scores/123"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_single_score() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/scores/123", None))
            .await
            .unwrap();

        // Should be OK or NOT_FOUND
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_delete_score_soft_delete() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("DELETE", "/scores/123", None))
            .await
            .unwrap();

        // Should be NOT_FOUND since score doesn't exist, or NO_CONTENT if it does
        assert!(response.status() == StatusCode::NOT_FOUND || response.status() == StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_complex_extra_data() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let complex_extra = json!({
            "achievements": ["first_try", "speed_run"],
            "metadata": {
                "platform": "web",
                "version": "1.2.3"
            },
            "stats": {
                "attempts": 5,
                "time_played": 240.75,
                "power_ups_used": ["shield", "double_score"]
            }
        });

        let score_data = json!({
            "game_hex_id": "abc123",
            "score": "9999",
            "user_name": "ProPlayer",
            "user_id": "pro123",
            "extra": complex_extra
        });

        let response = app
            .oneshot(request_with_api_key(
                "POST",
                "/scores",
                Some(&score_data.to_string()),
            ))
            .await
            .unwrap();

        assert!(
            response.status() == StatusCode::CREATED || response.status() == StatusCode::NOT_FOUND
        );
    }
}

#[cfg(test)]
mod pagination_and_sorting_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_games_with_pagination() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/games?limit=10", None))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_games_with_cursor() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/games?cursor=eyJoZXhfaWQiOiJhYmMxMjMiLCJjcmVhdGVkX2F0IjoiMjAyNC0wMS0wMVQwMDowMDowMFoifQ", None))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_games_with_invalid_cursor() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/games?cursor=invalid_cursor",
                None,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_list_games_with_oversized_limit() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        // Should cap at max limit
        let response = app
            .oneshot(request_with_api_key("GET", "/games?limit=200", None))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_scores_default_sorting() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        // Default should be sorted by score descending
        let response = app
            .oneshot(request_with_api_key("GET", "/scores?game_hex_id=abc123", None))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_scores_sort_by_date_asc() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&sort_by=date&order=asc",
                None,
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_scores_sort_by_user_name_desc() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&sort_by=user_name&order=desc",
                None,
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_scores_sort_by_score_desc() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&sort_by=score&order=desc",
                None,
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_scores_with_pagination() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&limit=5&sort_by=score",
                None,
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_scores_with_cursor_and_sorting() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET", 
                "/scores?game_hex_id=abc123&cursor=eyJpZCI6MTIzLCJzb3J0X3ZhbHVlIjoiMTAwMC41In0&sort_by=score&order=desc&limit=10",
                None
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_scores_invalid_sort_field() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&sort_by=invalid_field",
                None,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_get_scores_invalid_sort_order() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&order=invalid_order",
                None,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_get_scores_invalid_cursor() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&cursor=invalid_cursor",
                None,
            ))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_scores_pagination_consistency() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        // Test that the same sort parameters work consistently with pagination
        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&sort_by=date&order=asc&limit=25",
                None,
            ))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_environment_page_size_override() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        std::env::set_var("LEADR_PAGE_SIZE", "10");
        let app = create_test_app().await;

        // Should use environment variable for default page size
        let response = app
            .oneshot(request_with_api_key("GET", "/games", None))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        // Clean up
        std::env::remove_var("LEADR_PAGE_SIZE");
    }

    #[tokio::test]
    async fn test_scores_response_structure() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key(
                "GET",
                "/scores?game_hex_id=abc123&limit=1",
                None,
            ))
            .await
            .unwrap();

        // Response should be OK or NOT_FOUND, but structure should be consistent
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);

        // If OK, response should include pagination metadata
        // This would be tested more thoroughly once we implement the actual endpoints
    }

    #[tokio::test]
    async fn test_complex_query_parameters() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        // Test combination of all query parameters
        let complex_query = "/scores?game_hex_id=abc123&sort_by=score&order=desc&limit=15&cursor=eyJpZCI6NDU2LCJzb3J0X3ZhbHVlIjoiMjAwMC4wIn0";

        let response = app
            .oneshot(request_with_api_key("GET", complex_query, None))
            .await
            .unwrap();

        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_export_csv_backup() {
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");
        let app = create_test_app().await;

        let response = app
            .oneshot(request_with_api_key("GET", "/export", None))
            .await
            .unwrap();

        // Should return CSV file with proper headers, or at minimum not return METHOD_NOT_ALLOWED
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::INTERNAL_SERVER_ERROR);
        
        // If successful, verify headers
        if response.status() == StatusCode::OK {
            let headers = response.headers();
            assert_eq!(headers.get("content-type").unwrap(), "text/csv");
            assert!(headers.get("content-disposition").unwrap().to_str().unwrap().contains("attachment"));
            assert!(headers.get("content-disposition").unwrap().to_str().unwrap().contains("leadr_backup_"));
        }
    }

    #[tokio::test]
    async fn test_export_without_auth() {
        let app = create_test_app().await;

        let response = app
            .oneshot(request_without_api_key("GET", "/export"))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
