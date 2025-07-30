use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

pub const API_KEY_HEADER: &str = "leadr-api-key";

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    pub api_key: String,
}

impl ApiKeyAuth {
    /// Creates a new ApiKeyAuth instance with the provided API key.
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    /// Validates the provided key against the stored API key using constant-time comparison.
    ///
    /// # Errors
    /// Returns `false` if either key is empty or if the keys don't match.
    #[must_use]
    pub fn validate_key(&self, provided_key: &str) -> bool {
        if provided_key.trim().is_empty() || self.api_key.trim().is_empty() {
            return false;
        }

        // Constant-time comparison to prevent timing attacks
        provided_key.len() == self.api_key.len()
            && provided_key
                .bytes()
                .zip(self.api_key.bytes())
                .fold(0, |acc, (a, b)| acc | (a ^ b))
                == 0
    }

    /// Extracts the API key from HTTP headers.
    ///
    /// # Errors
    /// Returns `None` if the header is missing or contains invalid UTF-8.
    #[must_use]
    pub fn extract_api_key_from_headers(headers: &HeaderMap) -> Option<String> {
        headers
            .get(API_KEY_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(std::string::ToString::to_string)
    }
}

/// Middleware for API key authentication.
///
/// # Errors
/// Returns `StatusCode::INTERNAL_SERVER_ERROR` if the LEADR_API_KEY environment variable is not set.
/// Returns `StatusCode::UNAUTHORIZED` if no API key is provided or if the key is invalid.
///
/// # Panics
/// Does not panic under normal operation.
pub async fn api_key_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = std::env::var("LEADR_API_KEY").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let auth = ApiKeyAuth::new(api_key);

    let provided_key =
        ApiKeyAuth::extract_api_key_from_headers(&headers).ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth.validate_key(&provided_key) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
