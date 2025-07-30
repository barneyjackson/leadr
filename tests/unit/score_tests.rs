use leadr_api::models::score::{CreateScore, Score, UpdateScore};
use serde_json::json;

fn create_test_score_data() -> CreateScore {
    CreateScore {
        game_hex_id: "abc123".to_string(),
        score: "1000".to_string(),
        score_val: Some(1000.5),
        user_name: "TestPlayer".to_string(),
        user_id: "player123".to_string(),
        extra: Some(json!({"level": 5, "time": 120.5})),
    }
}

#[test]
fn test_new_score_creation() {
    let game_hex_id = "abc123".to_string();
    let create_data = create_test_score_data();

    let score = Score::new(create_data);

    assert_eq!(score.game_hex_id, game_hex_id);
    assert_eq!(score.score, "1000");
    assert_eq!(score.score_val, 1000.5);
    assert_eq!(score.user_name, "TestPlayer");
    assert_eq!(score.user_id, "player123");
    assert_eq!(score.extra, Some(json!({"level": 5, "time": 120.5})));
    assert!(!score.is_deleted());
    assert_eq!(score.id, 0); // Will be set by database
}

#[test]
fn test_new_score_without_score_val() {
    let create_data = CreateScore {
        game_hex_id: "game1".to_string(),
        score: "500".to_string(),
        score_val: None,
        user_name: "Player".to_string(),
        user_id: "id123".to_string(),
        extra: None,
    };

    let score = Score::new(create_data);

    assert_eq!(score.score, "500");
    assert_eq!(score.score_val, 500.0); // Should default to parsed score
}

#[test]
fn test_new_score_without_extra() {
    let create_data = CreateScore {
        game_hex_id: "game2".to_string(),
        score: "250".to_string(),
        score_val: Some(250.7),
        user_name: "Player".to_string(),
        user_id: "id456".to_string(),
        extra: None,
    };

    let score = Score::new(create_data);

    assert_eq!(score.extra, None);
}

#[test]
fn test_is_deleted_false_by_default() {
    let score = Score::new(create_test_score_data());
    assert!(!score.is_deleted());
}

#[test]
fn test_soft_delete() {
    let mut score = Score::new(create_test_score_data());

    score.soft_delete();

    assert!(score.is_deleted());
    assert!(score.deleted_at.is_some());
}

#[test]
fn test_restore_from_soft_delete() {
    let mut score = Score::new(create_test_score_data());

    score.soft_delete();
    assert!(score.is_deleted());

    score.restore();

    assert!(!score.is_deleted());
    assert!(score.deleted_at.is_none());
}

#[test]
fn test_update_score_only() {
    let mut score = Score::new(create_test_score_data());
    let original_user_name = score.user_name.clone();

    let update = UpdateScore {
        score: Some("2000".to_string()),
        score_val: None,
        user_name: None,
        user_id: None,
        extra: None,
    };

    score.update(update);

    assert_eq!(score.score, "2000");
    assert_eq!(score.score_val, 2000.0); // Should auto-update when score changes
    assert_eq!(score.user_name, original_user_name);
}

#[test]
fn test_update_score_and_score_num() {
    let mut score = Score::new(create_test_score_data());

    let update = UpdateScore {
        score: Some("1500".to_string()),
        score_val: Some(1500.75),
        user_name: None,
        user_id: None,
        extra: None,
    };

    score.update(update);

    assert_eq!(score.score, "1500");
    assert_eq!(score.score_val, 1500.75); // Should use explicit value
}

#[test]
fn test_update_user_info() {
    let mut score = Score::new(create_test_score_data());
    let original_score = score.score.clone();

    let update = UpdateScore {
        score: None,
        score_val: None,
        user_name: Some("NewPlayer".to_string()),
        user_id: Some("newid456".to_string()),
        extra: None,
    };

    score.update(update);

    assert_eq!(score.score, original_score);
    assert_eq!(score.user_name, "NewPlayer");
    assert_eq!(score.user_id, "newid456");
}

#[test]
fn test_update_extra_data() {
    let mut score = Score::new(create_test_score_data());

    let new_extra = json!({"achievements": ["speed_run", "perfect_score"]});
    let update = UpdateScore {
        score: None,
        score_val: None,
        user_name: None,
        user_id: None,
        extra: Some(new_extra.clone()),
    };

    score.update(update);

    assert_eq!(score.extra, Some(new_extra));
}

#[test]
fn test_validate_user_name_valid() {
    assert!(Score::validate_user_name("ValidName").is_ok());
    assert!(Score::validate_user_name("Player123").is_ok());
    assert!(Score::validate_user_name("A").is_ok());
}

#[test]
fn test_validate_user_name_empty() {
    assert!(Score::validate_user_name("").is_err());
    assert!(Score::validate_user_name("   ").is_err());
}

#[test]
fn test_validate_user_name_too_long() {
    let long_name = "a".repeat(101);
    assert!(Score::validate_user_name(&long_name).is_err());
}

#[test]
fn test_validate_user_name_max_length() {
    let max_name = "a".repeat(100);
    assert!(Score::validate_user_name(&max_name).is_ok());
}

#[test]
fn test_validate_user_id_valid() {
    assert!(Score::validate_user_id("valid_id").is_ok());
    assert!(Score::validate_user_id("user123").is_ok());
    assert!(Score::validate_user_id("x").is_ok());
}

#[test]
fn test_validate_user_id_empty() {
    assert!(Score::validate_user_id("").is_err());
    assert!(Score::validate_user_id("   ").is_err());
}

#[test]
fn test_validate_user_id_too_long() {
    let long_id = "a".repeat(256);
    assert!(Score::validate_user_id(&long_id).is_err());
}

#[test]
fn test_validate_user_id_max_length() {
    let max_id = "a".repeat(255);
    assert!(Score::validate_user_id(&max_id).is_ok());
}
