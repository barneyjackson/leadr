use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Score {
    pub id: i64,
    pub game_hex_id: String,
    pub score: String,  // Changed to String per schema
    pub score_val: f64, // Renamed from score_num
    pub user_name: String,
    pub user_id: String,
    pub extra: Option<JsonValue>,
    pub submitted_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Database representation with proper SQLite types
#[derive(Debug, sqlx::FromRow)]
pub struct ScoreRow {
    pub id: i64,
    pub game_hex_id: String,
    pub score: String,
    pub score_val: f64,
    pub user_name: String,
    pub user_id: String,
    pub extra: Option<String>, // JSON stored as TEXT
    pub submitted_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl From<ScoreRow> for Score {
    fn from(row: ScoreRow) -> Self {
        Self {
            id: row.id,
            game_hex_id: row.game_hex_id,
            score: row.score,
            score_val: row.score_val,
            user_name: row.user_name,
            user_id: row.user_id,
            extra: row.extra.and_then(|s| serde_json::from_str(&s).ok()),
            submitted_at: DateTime::from_naive_utc_and_offset(row.submitted_at, Utc),
            deleted_at: row
                .deleted_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateScore {
    pub game_hex_id: String,
    pub score: String,          // Changed to String
    pub score_val: Option<f64>, // Renamed from score_num
    pub user_name: String,
    pub user_id: String,
    pub extra: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScore {
    pub score: Option<String>,  // Changed to String
    pub score_val: Option<f64>, // Renamed from score_num
    pub user_name: Option<String>,
    pub user_id: Option<String>,
    pub extra: Option<JsonValue>,
}

impl Score {
    #[must_use]
    pub fn new(create_data: CreateScore) -> Self {
        // Try to parse score as f64 for score_val, fallback to 0.0
        let score_val = create_data
            .score_val
            .unwrap_or_else(|| create_data.score.parse::<f64>().unwrap_or(0.0));

        Self {
            id: 0, // Will be set by database
            game_hex_id: create_data.game_hex_id,
            score: create_data.score,
            score_val,
            user_name: create_data.user_name,
            user_id: create_data.user_id,
            extra: create_data.extra,
            submitted_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
    }

    pub fn update(&mut self, update_data: UpdateScore) {
        if let Some(score) = update_data.score {
            self.score.clone_from(&score);
            // Update score_val to match if not explicitly provided
            if update_data.score_val.is_none() {
                self.score_val = score.parse::<f64>().unwrap_or(0.0);
            }
        }
        if let Some(score_val) = update_data.score_val {
            self.score_val = score_val;
        }
        if let Some(user_name) = update_data.user_name {
            self.user_name = user_name;
        }
        if let Some(user_id) = update_data.user_id {
            self.user_id = user_id;
        }
        if let Some(extra) = update_data.extra {
            self.extra = Some(extra);
        }
    }

    /// Validates that a user name meets the requirements.
    /// 
    /// # Errors
    /// Returns an error string if the name is empty or exceeds 100 characters.
    pub fn validate_user_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("User name cannot be empty".to_string());
        }
        if name.len() > 100 {
            return Err("User name cannot exceed 100 characters".to_string());
        }
        Ok(())
    }

    /// Validates that a user ID meets the requirements.
    /// 
    /// # Errors
    /// Returns an error string if the ID is empty or exceeds 255 characters.
    pub fn validate_user_id(id: &str) -> Result<(), String> {
        if id.trim().is_empty() {
            return Err("User ID cannot be empty".to_string());
        }
        if id.len() > 255 {
            return Err("User ID cannot exceed 255 characters".to_string());
        }
        Ok(())
    }
}
