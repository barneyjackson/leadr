use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Game {
    pub id: i64,
    pub hex_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Database representation with proper SQLite types
#[derive(Debug, sqlx::FromRow)]
pub struct GameRow {
    pub id: i64,
    pub hex_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub deleted_at: Option<chrono::NaiveDateTime>,
}

impl From<GameRow> for Game {
    fn from(row: GameRow) -> Self {
        Self {
            id: row.id,
            hex_id: row.hex_id,
            name: row.name,
            description: row.description,
            created_at: DateTime::from_naive_utc_and_offset(row.created_at, Utc),
            updated_at: DateTime::from_naive_utc_and_offset(row.updated_at, Utc),
            deleted_at: row
                .deleted_at
                .map(|dt| DateTime::from_naive_utc_and_offset(dt, Utc)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateGame {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGame {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Game {
    #[must_use]
    pub fn generate_hex_id() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| format!("{:x}", rng.gen_range(0..16)))
            .collect()
    }

    #[must_use]
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: 0, // Will be set by database
            hex_id: Self::generate_hex_id(),
            name,
            description,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }

    /// Normalizes and validates a hex ID. Converts to lowercase and validates format.
    /// 
    /// # Errors
    /// Returns an error string if the hex ID is not exactly 6 characters or contains invalid characters.
    pub fn normalize_and_validate_hex_id(hex_id: &str) -> Result<String, String> {
        if hex_id.len() != 6 {
            return Err("Hex ID must be exactly 6 characters".to_string());
        }
        
        let normalized = hex_id.to_lowercase();
        if !normalized
            .chars()
            .all(|c| c.is_ascii_alphanumeric())
        {
            return Err(
                "Hex ID must contain only alphanumeric characters (0-9, a-z, A-Z)".to_string(),
            );
        }
        Ok(normalized)
    }

    /// Validates that a hex ID has the correct format (6 lowercase alphanumeric characters).
    /// 
    /// # Errors
    /// Returns an error string if the hex ID is not exactly 6 characters or contains invalid characters.
    pub fn validate_hex_id(hex_id: &str) -> Result<(), String> {
        if hex_id.len() != 6 {
            return Err("Hex ID must be exactly 6 characters".to_string());
        }
        if !hex_id
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase())
        {
            return Err(
                "Hex ID must contain only lowercase alphanumeric characters (0-9, a-z)".to_string(),
            );
        }
        Ok(())
    }

    /// Validates that a game name meets the requirements.
    /// 
    /// # Errors
    /// Returns an error string if the name is empty or exceeds 255 characters.
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Game name cannot be empty".to_string());
        }
        if name.len() > 255 {
            return Err("Game name cannot exceed 255 characters".to_string());
        }
        Ok(())
    }

    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn soft_delete(&mut self) {
        self.deleted_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn restore(&mut self) {
        self.deleted_at = None;
        self.updated_at = Utc::now();
    }

    pub fn update(&mut self, update_data: UpdateGame) {
        if let Some(name) = update_data.name {
            self.name = name;
        }
        if let Some(description) = update_data.description {
            self.description = Some(description);
        }
        self.updated_at = Utc::now();
    }
}
