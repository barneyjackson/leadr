use leadr_api::models::game::{Game, UpdateGame};

#[test]
fn test_generate_hex_id_format() {
    let hex_id = Game::generate_hex_id();
    assert_eq!(hex_id.len(), 6);
    assert!(hex_id.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_generate_hex_id_uniqueness() {
    let id1 = Game::generate_hex_id();
    let id2 = Game::generate_hex_id();
    // While not guaranteed, extremely unlikely to be equal
    assert_ne!(id1, id2);
}

#[test]
fn test_new_game_creation() {
    let name = "Test Game".to_string();
    let description = Some("A test game".to_string());

    let game = Game::new(name.clone(), description.clone());

    assert_eq!(game.id, 0); // Will be set by database
    assert_eq!(game.name, name);
    assert_eq!(game.description, description);
    assert_eq!(game.hex_id.len(), 6);
    assert!(game.hex_id.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(!game.is_deleted());
    assert_eq!(game.created_at, game.updated_at);
}

#[test]
fn test_new_game_without_description() {
    let name = "Test Game".to_string();

    let game = Game::new(name.clone(), None);

    assert_eq!(game.name, name);
    assert_eq!(game.description, None);
    assert!(!game.is_deleted());
}

#[test]
fn test_is_deleted_false_by_default() {
    let game = Game::new("Test".to_string(), None);
    assert!(!game.is_deleted());
}

#[test]
fn test_soft_delete() {
    let mut game = Game::new("Test".to_string(), None);
    let original_updated_at = game.updated_at;

    // Small delay to ensure timestamp difference
    std::thread::sleep(std::time::Duration::from_millis(1));

    game.soft_delete();

    assert!(game.is_deleted());
    assert!(game.deleted_at.is_some());
    assert!(game.updated_at > original_updated_at);
}

#[test]
fn test_restore_from_soft_delete() {
    let mut game = Game::new("Test".to_string(), None);

    game.soft_delete();
    assert!(game.is_deleted());

    let deleted_updated_at = game.updated_at;
    std::thread::sleep(std::time::Duration::from_millis(1));

    game.restore();

    assert!(!game.is_deleted());
    assert!(game.deleted_at.is_none());
    assert!(game.updated_at > deleted_updated_at);
}

#[test]
fn test_update_name_only() {
    let mut game = Game::new("Original".to_string(), Some("Desc".to_string()));
    let original_updated_at = game.updated_at;
    let original_description = game.description.clone();

    std::thread::sleep(std::time::Duration::from_millis(1));

    let update = UpdateGame {
        name: Some("Updated Name".to_string()),
        description: None,
    };

    game.update(update);

    assert_eq!(game.name, "Updated Name");
    assert_eq!(game.description, original_description);
    assert!(game.updated_at > original_updated_at);
}

#[test]
fn test_update_description_only() {
    let mut game = Game::new("Name".to_string(), Some("Original Desc".to_string()));
    let original_name = game.name.clone();

    let update = UpdateGame {
        name: None,
        description: Some("Updated Description".to_string()),
    };

    game.update(update);

    assert_eq!(game.name, original_name);
    assert_eq!(game.description, Some("Updated Description".to_string()));
}

#[test]
fn test_update_both_fields() {
    let mut game = Game::new("Original".to_string(), Some("Original Desc".to_string()));

    let update = UpdateGame {
        name: Some("New Name".to_string()),
        description: Some("New Description".to_string()),
    };

    game.update(update);

    assert_eq!(game.name, "New Name");
    assert_eq!(game.description, Some("New Description".to_string()));
}

#[test]
fn test_update_with_empty_changes() {
    let mut game = Game::new("Original".to_string(), Some("Desc".to_string()));
    let original_name = game.name.clone();
    let original_description = game.description.clone();
    let original_updated_at = game.updated_at;

    std::thread::sleep(std::time::Duration::from_millis(1));

    let update = UpdateGame {
        name: None,
        description: None,
    };

    game.update(update);

    assert_eq!(game.name, original_name);
    assert_eq!(game.description, original_description);
    // updated_at should still change even with no field updates
    assert!(game.updated_at > original_updated_at);
}

#[test]
fn test_validate_hex_id_valid() {
    assert!(Game::validate_hex_id("abc123").is_ok());
    assert!(Game::validate_hex_id("000000").is_ok());
    assert!(Game::validate_hex_id("ffffff").is_ok());
    assert!(Game::validate_hex_id("a1b2c3").is_ok());
}

#[test]
fn test_validate_hex_id_invalid_length() {
    assert!(Game::validate_hex_id("abc12").is_err());
    assert!(Game::validate_hex_id("abc1234").is_err());
    assert!(Game::validate_hex_id("").is_err());
}

#[test]
fn test_validate_hex_id_invalid_characters() {
    assert!(Game::validate_hex_id("abc@yz").is_err()); // special characters not allowed
    assert!(Game::validate_hex_id("ABC123").is_err()); // uppercase not allowed
    assert!(Game::validate_hex_id("123-45").is_err()); // hyphen not allowed
    assert!(Game::validate_hex_id("12 345").is_err()); // space not allowed
}

#[test]
fn test_validate_name_valid() {
    assert!(Game::validate_name("Valid Game").is_ok());
    assert!(Game::validate_name("A").is_ok());
    assert!(Game::validate_name(&"a".repeat(255)).is_ok());
}

#[test]
fn test_validate_name_empty() {
    assert!(Game::validate_name("").is_err());
    assert!(Game::validate_name("   ").is_err());
}

#[test]
fn test_validate_name_too_long() {
    assert!(Game::validate_name(&"a".repeat(256)).is_err());
}
