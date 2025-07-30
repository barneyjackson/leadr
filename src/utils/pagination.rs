use serde::{Deserialize, Serialize};

pub const DEFAULT_PAGE_SIZE: u32 = 25;
pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub cursor: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub current_cursor: Option<String>,
    pub total_returned: usize,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    #[default]
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ScoreSortField {
    #[serde(rename = "score")]
    #[default]
    Score,
    #[serde(rename = "date")]
    Date,
    #[serde(rename = "user_name")]
    UserName,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreSortParams {
    pub sort_by: Option<ScoreSortField>,
    pub order: Option<SortOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreQueryParams {
    pub game_hex_id: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<u32>,
    pub sort_by: Option<ScoreSortField>,
    pub order: Option<SortOrder>,
}

impl ScoreQueryParams {
    #[must_use]
    pub fn to_pagination_params(&self) -> PaginationParams {
        PaginationParams {
            cursor: self.cursor.clone(),
            limit: self.limit,
        }
    }

    #[must_use]
    pub fn to_sort_params(&self) -> ScoreSortParams {
        ScoreSortParams {
            sort_by: self.sort_by.clone(),
            order: self.order.clone(),
        }
    }
}

impl PaginationParams {
    #[must_use]
    pub fn new(cursor: Option<String>, limit: Option<u32>) -> Self {
        Self { cursor, limit }
    }

    #[must_use]
    pub fn get_limit(&self) -> u32 {
        match self.limit {
            Some(limit) if limit > 0 && limit <= MAX_PAGE_SIZE => limit,
            Some(_) => MAX_PAGE_SIZE, // Cap at maximum
            None => DEFAULT_PAGE_SIZE,
        }
    }

    #[must_use]
    pub fn get_page_size_from_env() -> u32 {
        std::env::var("LEADR_PAGE_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE)
    }
}

impl<T> PaginatedResponse<T> {
    #[must_use]
    pub fn new(
        data: Vec<T>,
        has_more: bool,
        next_cursor: Option<String>,
        current_cursor: Option<String>,
        page_size: u32,
    ) -> Self {
        let total_returned = data.len();
        Self {
            data,
            has_more,
            next_cursor,
            current_cursor,
            total_returned,
            page_size,
        }
    }

    /// Create a paginated response from query results
    pub fn from_query_results(
        mut data: Vec<T>,
        requested_limit: u32,
        current_cursor: Option<String>,
        next_cursor_fn: impl FnOnce(&T) -> Option<String>,
    ) -> Self {
        let has_more = data.len() > requested_limit as usize;

        // Remove the extra item if we fetched limit + 1 to check for more pages
        if has_more {
            data.pop();
        }

        let next_cursor = if has_more && !data.is_empty() {
            next_cursor_fn(data.last().unwrap())
        } else {
            None
        };

        Self::new(data, has_more, next_cursor, current_cursor, requested_limit)
    }

    /// Get pagination metadata for client use
    #[must_use]
    pub fn get_pagination_info(&self) -> PaginationInfo {
        PaginationInfo {
            has_more: self.has_more,
            next_cursor: self.next_cursor.clone(),
            current_cursor: self.current_cursor.clone(),
            total_returned: self.total_returned,
            page_size: self.page_size,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub has_more: bool,
    pub next_cursor: Option<String>,
    pub current_cursor: Option<String>,
    pub total_returned: usize,
    pub page_size: u32,
}

impl ScoreSortParams {
    #[must_use]
    pub fn new(sort_by: Option<ScoreSortField>, order: Option<SortOrder>) -> Self {
        Self { sort_by, order }
    }

    #[must_use]
    pub fn get_sort_field(&self) -> ScoreSortField {
        self.sort_by.clone().unwrap_or_default()
    }

    #[must_use]
    pub fn get_sort_order(&self) -> SortOrder {
        self.order.clone().unwrap_or_default()
    }

    #[must_use]
    pub fn to_sql_order_clause(&self) -> String {
        let field = match self.get_sort_field() {
            ScoreSortField::Score => "score_val",
            ScoreSortField::Date => "submitted_at",
            ScoreSortField::UserName => "user_name",
        };

        let order = match self.get_sort_order() {
            SortOrder::Ascending => "ASC",
            SortOrder::Descending => "DESC",
        };

        format!("{field} {order}")
    }

    #[must_use]
    pub fn get_cursor_field(&self) -> &'static str {
        match self.get_sort_field() {
            ScoreSortField::Score => "score_val",
            ScoreSortField::Date => "submitted_at",
            ScoreSortField::UserName => "user_name",
        }
    }
}

// Cursor encoding/decoding utilities
pub mod cursor {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct GameCursor {
        pub hex_id: String,
        pub created_at: String, // ISO 8601 format
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ScoreCursor {
        pub id: i64,
        pub sort_value: String, // The value of the field we're sorting by
    }

    pub fn encode_game_cursor(cursor: &GameCursor) -> Result<String, String> {
        let json = serde_json::to_string(cursor)
            .map_err(|e| format!("Failed to serialize cursor: {e}"))?;
        Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
    }

    pub fn decode_game_cursor(cursor: &str) -> Result<GameCursor, String> {
        let bytes = URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|e| format!("Failed to decode cursor: {e}"))?;
        let json =
            String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in cursor: {e}"))?;
        serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize cursor: {e}"))
    }

    pub fn encode_score_cursor(cursor: &ScoreCursor) -> Result<String, String> {
        let json = serde_json::to_string(cursor)
            .map_err(|e| format!("Failed to serialize cursor: {e}"))?;
        Ok(URL_SAFE_NO_PAD.encode(json.as_bytes()))
    }

    pub fn decode_score_cursor(cursor: &str) -> Result<ScoreCursor, String> {
        let bytes = URL_SAFE_NO_PAD
            .decode(cursor)
            .map_err(|e| format!("Failed to decode cursor: {e}"))?;
        let json =
            String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in cursor: {e}"))?;
        serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize cursor: {e}"))
    }

    // Helper functions to create cursors from model data
    use crate::models::{Game, Score};

    impl GameCursor {
        pub fn from_game(game: &Game) -> Self {
            Self {
                hex_id: game.hex_id.clone(),
                created_at: game.created_at.to_rfc3339(),
            }
        }
    }

    impl ScoreCursor {
        pub fn from_score(score: &Score, sort_field: &str) -> Self {
            let sort_value = match sort_field {
                "score_val" => score.score_val.to_string(),
                "submitted_at" => score.submitted_at.to_rfc3339(),
                "user_name" => score.user_name.clone(),
                _ => score.score_val.to_string(), // fallback to score
            };

            Self {
                id: score.id,
                sort_value,
            }
        }
    }
}
