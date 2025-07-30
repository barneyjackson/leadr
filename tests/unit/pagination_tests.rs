use chrono::Utc;
use leadr_api::models::{Game, Score};
use leadr_api::utils::pagination::cursor::*;
use leadr_api::utils::pagination::*;
use serde_json::json;

#[test]
fn test_pagination_params_default_limit() {
    let params = PaginationParams::new(None, None);
    assert_eq!(params.get_limit(), DEFAULT_PAGE_SIZE);
}

#[test]
fn test_pagination_params_custom_limit() {
    let params = PaginationParams::new(None, Some(10));
    assert_eq!(params.get_limit(), 10);
}

#[test]
fn test_pagination_params_max_limit() {
    let params = PaginationParams::new(None, Some(200));
    assert_eq!(params.get_limit(), MAX_PAGE_SIZE);
}

#[test]
fn test_pagination_params_zero_limit() {
    let params = PaginationParams::new(None, Some(0));
    assert_eq!(params.get_limit(), MAX_PAGE_SIZE);
}

#[test]
fn test_score_sort_params_defaults() {
    let params = ScoreSortParams::new(None, None);
    assert!(matches!(params.get_sort_field(), ScoreSortField::Score));
    assert!(matches!(params.get_sort_order(), SortOrder::Descending));
}

#[test]
fn test_score_sort_params_custom() {
    let params = ScoreSortParams::new(Some(ScoreSortField::Date), Some(SortOrder::Ascending));
    assert!(matches!(params.get_sort_field(), ScoreSortField::Date));
    assert!(matches!(params.get_sort_order(), SortOrder::Ascending));
}

#[test]
fn test_sql_order_clause_score_desc() {
    let params = ScoreSortParams::new(Some(ScoreSortField::Score), Some(SortOrder::Descending));
    assert_eq!(params.to_sql_order_clause(), "score_val DESC");
}

#[test]
fn test_sql_order_clause_date_asc() {
    let params = ScoreSortParams::new(Some(ScoreSortField::Date), Some(SortOrder::Ascending));
    assert_eq!(params.to_sql_order_clause(), "submitted_at ASC");
}

#[test]
fn test_sql_order_clause_user_name_desc() {
    let params = ScoreSortParams::new(Some(ScoreSortField::UserName), Some(SortOrder::Descending));
    assert_eq!(params.to_sql_order_clause(), "user_name DESC");
}

#[test]
fn test_cursor_field_mapping() {
    let score_params = ScoreSortParams::new(Some(ScoreSortField::Score), None);
    assert_eq!(score_params.get_cursor_field(), "score_val");

    let date_params = ScoreSortParams::new(Some(ScoreSortField::Date), None);
    assert_eq!(date_params.get_cursor_field(), "submitted_at");

    let name_params = ScoreSortParams::new(Some(ScoreSortField::UserName), None);
    assert_eq!(name_params.get_cursor_field(), "user_name");
}

#[test]
fn test_paginated_response_creation() {
    let data = vec![1, 2, 3];
    let response = PaginatedResponse::new(
        data.clone(),
        true,
        Some("cursor123".to_string()),
        Some("current123".to_string()),
        25,
    );

    assert_eq!(response.data, data);
    assert!(response.has_more);
    assert_eq!(response.next_cursor, Some("cursor123".to_string()));
    assert_eq!(response.current_cursor, Some("current123".to_string()));
    assert_eq!(response.total_returned, 3);
    assert_eq!(response.page_size, 25);
}

#[test]
fn test_from_query_results_with_more_data() {
    let data = vec![1, 2, 3, 4]; // 4 items, limit 3
    let response =
        PaginatedResponse::from_query_results(data, 3, Some("current".to_string()), |item| {
            Some(format!("cursor_{}", item))
        });

    assert_eq!(response.data, vec![1, 2, 3]); // Should remove extra item
    assert!(response.has_more);
    assert_eq!(response.next_cursor, Some("cursor_3".to_string()));
    assert_eq!(response.current_cursor, Some("current".to_string()));
    assert_eq!(response.total_returned, 3);
    assert_eq!(response.page_size, 3);
}

#[test]
fn test_from_query_results_no_more_data() {
    let data = vec![1, 2];
    let response = PaginatedResponse::from_query_results(data, 3, None, |_| {
        Some("should_not_be_called".to_string())
    });

    assert_eq!(response.data, vec![1, 2]);
    assert!(!response.has_more);
    assert_eq!(response.next_cursor, None);
    assert_eq!(response.current_cursor, None);
    assert_eq!(response.total_returned, 2);
    assert_eq!(response.page_size, 3);
}

#[test]
fn test_get_pagination_info() {
    let response = PaginatedResponse::new(
        vec![1, 2, 3],
        true,
        Some("next".to_string()),
        Some("current".to_string()),
        10,
    );

    let info = response.get_pagination_info();
    assert!(info.has_more);
    assert_eq!(info.next_cursor, Some("next".to_string()));
    assert_eq!(info.current_cursor, Some("current".to_string()));
    assert_eq!(info.total_returned, 3);
    assert_eq!(info.page_size, 10);
}

#[test]
fn test_get_page_size_from_env() {
    // Test default when env var not set
    std::env::remove_var("LEADR_PAGE_SIZE");
    assert_eq!(
        PaginationParams::get_page_size_from_env(),
        DEFAULT_PAGE_SIZE
    );

    // Test custom value
    std::env::set_var("LEADR_PAGE_SIZE", "50");
    assert_eq!(PaginationParams::get_page_size_from_env(), 50);

    // Test value too high gets capped
    std::env::set_var("LEADR_PAGE_SIZE", "200");
    assert_eq!(PaginationParams::get_page_size_from_env(), MAX_PAGE_SIZE);

    // Test value too low gets set to minimum
    std::env::set_var("LEADR_PAGE_SIZE", "0");
    assert_eq!(PaginationParams::get_page_size_from_env(), 1);

    // Test invalid value falls back to default
    std::env::set_var("LEADR_PAGE_SIZE", "invalid");
    assert_eq!(
        PaginationParams::get_page_size_from_env(),
        DEFAULT_PAGE_SIZE
    );

    // Clean up
    std::env::remove_var("LEADR_PAGE_SIZE");
}

// Cursor encoding/decoding tests
#[test]
fn test_game_cursor_encode_decode() {
    let original = GameCursor {
        hex_id: "abc123".to_string(),
        created_at: Utc::now().to_rfc3339(),
    };

    let encoded = encode_game_cursor(&original).unwrap();
    let decoded = decode_game_cursor(&encoded).unwrap();

    assert_eq!(decoded.hex_id, original.hex_id);
    assert_eq!(decoded.created_at, original.created_at);
}

#[test]
fn test_score_cursor_encode_decode() {
    let original = ScoreCursor {
        id: 123,
        sort_value: "1000.5".to_string(),
    };

    let encoded = encode_score_cursor(&original).unwrap();
    let decoded = decode_score_cursor(&encoded).unwrap();

    assert_eq!(decoded.id, original.id);
    assert_eq!(decoded.sort_value, original.sort_value);
}

#[test]
fn test_decode_invalid_cursor() {
    assert!(decode_game_cursor("invalid_base64!").is_err());
    assert!(decode_score_cursor("invalid_base64!").is_err());
}

#[test]
fn test_decode_invalid_json() {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let invalid_json = URL_SAFE_NO_PAD.encode(b"not valid json");
    assert!(decode_game_cursor(&invalid_json).is_err());
    assert!(decode_score_cursor(&invalid_json).is_err());
}

#[test]
fn test_game_cursor_from_game() {
    let game = Game {
        id: 1,
        hex_id: "abc123".to_string(),
        name: "Test Game".to_string(),
        description: Some("Test".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let cursor = GameCursor::from_game(&game);
    assert_eq!(cursor.hex_id, "abc123");
    assert_eq!(cursor.created_at, game.created_at.to_rfc3339());
}

#[test]
fn test_score_cursor_from_score_different_fields() {
    let score = Score {
        id: 123,
        game_hex_id: "abc123".to_string(),
        score: "1000".to_string(),
        score_val: 1000.5,
        user_name: "TestPlayer".to_string(),
        user_id: "player123".to_string(),
        extra: Some(json!({"level": 5})),
        submitted_at: Utc::now(),
        deleted_at: None,
    };

    // Test score_val field
    let score_cursor = ScoreCursor::from_score(&score, "score_val");
    assert_eq!(score_cursor.id, 123);
    assert_eq!(score_cursor.sort_value, "1000.5");

    // Test submitted_at field
    let date_cursor = ScoreCursor::from_score(&score, "submitted_at");
    assert_eq!(date_cursor.id, 123);
    assert_eq!(date_cursor.sort_value, score.submitted_at.to_rfc3339());

    // Test user_name field
    let name_cursor = ScoreCursor::from_score(&score, "user_name");
    assert_eq!(name_cursor.id, 123);
    assert_eq!(name_cursor.sort_value, "TestPlayer");

    // Test fallback for invalid field
    let fallback_cursor = ScoreCursor::from_score(&score, "invalid_field");
    assert_eq!(fallback_cursor.id, 123);
    assert_eq!(fallback_cursor.sort_value, "1000.5");
}

#[test]
fn test_round_trip_with_helpers() {
    // Test Game round trip
    let game = Game {
        id: 2,
        hex_id: "def456".to_string(),
        name: "Round Trip Game".to_string(),
        description: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        deleted_at: None,
    };

    let game_cursor = GameCursor::from_game(&game);
    let encoded = encode_game_cursor(&game_cursor).unwrap();
    let decoded = decode_game_cursor(&encoded).unwrap();
    assert_eq!(decoded.hex_id, game.hex_id);
    assert_eq!(decoded.created_at, game.created_at.to_rfc3339());

    // Test Score round trip
    let score = Score {
        id: 456,
        game_hex_id: "def456".to_string(),
        score: "2000".to_string(),
        score_val: 2000.75,
        user_name: "RoundTripPlayer".to_string(),
        user_id: "roundtrip789".to_string(),
        extra: Some(json!({"test": true})),
        submitted_at: Utc::now(),
        deleted_at: None,
    };

    let score_cursor = ScoreCursor::from_score(&score, "score_val");
    let encoded = encode_score_cursor(&score_cursor).unwrap();
    let decoded = decode_score_cursor(&encoded).unwrap();
    assert_eq!(decoded.id, score.id);
    assert_eq!(decoded.sort_value, "2000.75");
}
