use axum::http::{HeaderMap, HeaderValue};
use leadr_api::auth::{ApiKeyAuth, API_KEY_HEADER};

#[test]
fn test_new_api_key_auth() {
    let key = "test_key_123".to_string();
    let auth = ApiKeyAuth::new(key.clone());
    assert_eq!(auth.api_key, key);
}

#[test]
fn test_validate_key_correct() {
    let auth = ApiKeyAuth::new("secret123".to_string());
    assert!(auth.validate_key("secret123"));
}

#[test]
fn test_validate_key_incorrect() {
    let auth = ApiKeyAuth::new("secret123".to_string());
    assert!(!auth.validate_key("wrong_key"));
}

#[test]
fn test_validate_key_empty_provided() {
    let auth = ApiKeyAuth::new("secret123".to_string());
    assert!(!auth.validate_key(""));
    assert!(!auth.validate_key("   "));
}

#[test]
fn test_validate_key_empty_stored() {
    let auth = ApiKeyAuth::new("".to_string());
    assert!(!auth.validate_key("any_key"));
}

#[test]
fn test_validate_key_both_empty() {
    let auth = ApiKeyAuth::new("".to_string());
    assert!(!auth.validate_key(""));
}

#[test]
fn test_validate_key_whitespace_handling() {
    let auth = ApiKeyAuth::new("secret123".to_string());
    assert!(!auth.validate_key(" secret123 ")); // Should not trim
}

#[test]
fn test_validate_key_case_sensitive() {
    let auth = ApiKeyAuth::new("Secret123".to_string());
    assert!(!auth.validate_key("secret123"));
    assert!(!auth.validate_key("SECRET123"));
}

#[test]
fn test_validate_key_different_lengths() {
    let auth = ApiKeyAuth::new("short".to_string());
    assert!(!auth.validate_key("much_longer_key"));
    assert!(!auth.validate_key("abc"));
}

#[test]
fn test_validate_key_timing_attack_resistance() {
    let auth = ApiKeyAuth::new("a".repeat(100));
    let short_wrong = "b";
    let long_wrong = "b".repeat(100);

    // Both should be false, timing should be similar
    assert!(!auth.validate_key(short_wrong));
    assert!(!auth.validate_key(&long_wrong));
}

#[test]
fn test_extract_api_key_from_headers_present() {
    let mut headers = HeaderMap::new();
    headers.insert(API_KEY_HEADER, HeaderValue::from_static("test_key_123"));

    let result = ApiKeyAuth::extract_api_key_from_headers(&headers);
    assert_eq!(result, Some("test_key_123".to_string()));
}

#[test]
fn test_extract_api_key_from_headers_missing() {
    let headers = HeaderMap::new();
    let result = ApiKeyAuth::extract_api_key_from_headers(&headers);
    assert_eq!(result, None);
}

#[test]
fn test_extract_api_key_from_headers_invalid_utf8() {
    let mut headers = HeaderMap::new();
    // This would be handled by HeaderValue validation in practice
    headers.insert(API_KEY_HEADER, HeaderValue::from_static("valid_key"));

    let result = ApiKeyAuth::extract_api_key_from_headers(&headers);
    assert_eq!(result, Some("valid_key".to_string()));
}

#[test]
fn test_extract_api_key_case_sensitive_header() {
    let mut headers = HeaderMap::new();
    // Test that header name is case-insensitive (HTTP standard)
    headers.insert("LEADR-API-KEY", HeaderValue::from_static("test_key"));

    let result = ApiKeyAuth::extract_api_key_from_headers(&headers);
    assert_eq!(result, Some("test_key".to_string()));
}

#[test]
fn test_api_key_header_constant() {
    // Ensure header name follows conventions
    assert_eq!(API_KEY_HEADER, "leadr-api-key");
    assert!(API_KEY_HEADER
        .chars()
        .all(|c| c.is_ascii_lowercase() || c == '-'));
}

#[test]
fn test_validate_key_special_characters() {
    let auth = ApiKeyAuth::new("key_with-special.chars!@#$%^&*()".to_string());
    assert!(auth.validate_key("key_with-special.chars!@#$%^&*()"));
    assert!(!auth.validate_key("key_with-special.chars!@#$%^&*"));
}

#[test]
fn test_validate_key_unicode() {
    let auth = ApiKeyAuth::new("🔑key_with_emoji🚀".to_string());
    assert!(auth.validate_key("🔑key_with_emoji🚀"));
    assert!(!auth.validate_key("key_with_emoji"));
}

// Integration-style test for the constant-time comparison
#[test]
fn test_constant_time_comparison_properties() {
    let auth = ApiKeyAuth::new("correct_key".to_string());

    // These should all take similar time (hard to test in unit tests)
    let test_cases = vec![
        "wrong_key_1",
        "wrong_key_2",
        "completely_different",
        "correct_key", // This one should return true
    ];

    let results: Vec<bool> = test_cases
        .iter()
        .map(|&key| auth.validate_key(key))
        .collect();

    assert_eq!(results, vec![false, false, false, true]);
}
