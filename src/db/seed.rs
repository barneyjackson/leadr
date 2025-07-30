use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    db::{repository::{GameRepository, ScoreRepository}, DbPool},
    models::{
        game::CreateGame,
        score::CreateScore,
    },
};

#[derive(Debug, Deserialize)]
struct CsvRow {
    // Game fields
    game_hex_id: String,
    game_name: String,
    game_description: Option<String>, 
    game_created_at: String,
    game_updated_at: String,
    game_deleted_at: Option<String>,
    
    // Score fields - all optional for games without scores
    score_id: Option<i64>,
    score_value: Option<String>,
    score_val: Option<f64>,
    user_name: Option<String>,
    user_id: Option<String>,
    extra: Option<String>,
    score_submitted_at: Option<String>,
    score_updated_at: Option<String>,
    score_deleted_at: Option<String>,
}

/// Seeds the database from a CSV file if the database is empty.
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `csv_path` - Path to the CSV file to import
/// 
/// # Errors
/// Returns error if file cannot be read, CSV is malformed, or database operations fail.
pub async fn seed_from_csv(pool: &DbPool, csv_path: &str) -> Result<(), Box<dyn std::error::Error>> {

    // Check if database is empty (no games exist)
    let game_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM game WHERE deleted_at IS NULL"
    )
    .fetch_one(pool)
    .await?;

    if game_count > 0 {
        tracing::info!("Database already contains {} games, skipping seed", game_count);
        return Ok(());
    }

    if !Path::new(csv_path).exists() {
        tracing::info!("Seed file {} does not exist, skipping seed", csv_path);
        return Ok(());
    }

    tracing::info!("Starting database seed from {}", csv_path);

    // Read and parse CSV
    let mut reader = csv::Reader::from_path(csv_path)?;
    let mut rows: Vec<CsvRow> = Vec::new();
    
    for result in reader.deserialize() {
        let row: CsvRow = result?;
        rows.push(row);
    }

    tracing::info!("Read {} rows from CSV", rows.len());

    // Group rows by game to avoid duplicates
    let mut games_map: HashMap<String, (CreateGame, DateTime<Utc>)> = HashMap::new();
    let mut scores: Vec<(CreateScore, String, DateTime<Utc>)> = Vec::new(); // (score, game_hex_id, submitted_at)

    for (row_num, row) in rows.iter().enumerate() {
        // Normalize hex_id to lowercase
        let normalized_hex_id = row.game_hex_id.to_lowercase();
        
        // Parse timestamps - skip row if timestamp is invalid
        let game_created_at = match DateTime::parse_from_rfc3339(&row.game_created_at) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(e) => {
                tracing::warn!("Row {}: Invalid game timestamp '{}': {}, skipping", row_num + 1, row.game_created_at, e);
                continue;
            }
        };

        // Add game if not already added (with normalized hex_id)
        if !games_map.contains_key(&normalized_hex_id) {
            let create_game = CreateGame {
                name: row.game_name.clone(),
                description: row.game_description.clone(),
            };
            games_map.insert(
                normalized_hex_id.clone(),
                (create_game, game_created_at),
            );
        }

        // Add score if it has valid data (score_id > 0 and required fields are present)
        if let (Some(score_id), Some(score_value), Some(score_val), Some(user_name), Some(user_id)) = 
            (row.score_id, &row.score_value, row.score_val, &row.user_name, &row.user_id) {
            
            if score_id > 0 && !user_name.is_empty() && !user_id.is_empty() {
                let score_submitted_at = if let Some(ref submitted_str) = row.score_submitted_at {
                    if !submitted_str.is_empty() {
                        match DateTime::parse_from_rfc3339(submitted_str) {
                            Ok(dt) => dt.with_timezone(&Utc),
                            Err(e) => {
                                tracing::warn!("Row {}: Invalid score timestamp '{}': {}, using current time", row_num + 1, submitted_str, e);
                                Utc::now()
                            }
                        }
                    } else {
                        Utc::now()
                    }
                } else {
                    Utc::now()
                };

                let create_score = CreateScore {
                    game_hex_id: normalized_hex_id.clone(),  // Use normalized hex_id
                    score: score_value.clone(),
                    score_val: Some(score_val),
                    user_name: user_name.clone(),
                    user_id: user_id.clone(),
                    extra: row.extra.as_ref()
                        .filter(|s| !s.is_empty())
                        .and_then(|s| serde_json::from_str(s).ok()),
                };
                scores.push((create_score, normalized_hex_id.clone(), score_submitted_at));
            }
        }
    }

    tracing::info!("Importing {} games and {} scores", games_map.len(), scores.len());

    // Create games first
    let mut created_games = 0;
    let mut failed_games = 0;
    for (hex_id, (create_game, created_at)) in games_map {
        match GameRepository::create_with_hex_id(pool, create_game.clone(), hex_id.clone(), created_at).await {
            Ok(game) => {
                created_games += 1;
                tracing::debug!("Created game: {} ({})", game.name, hex_id);
            }
            Err(e) => {
                tracing::warn!("Failed to create game '{}' ({}): {}, continuing with remaining games", create_game.name, hex_id, e);
                failed_games += 1;
            }
        }
    }

    // Create scores
    let mut created_scores = 0;
    let mut failed_scores = 0;
    for (create_score, game_hex_id, submitted_at) in scores {
        match ScoreRepository::create_with_timestamp(pool, create_score.clone(), submitted_at).await {
            Ok(score) => {
                created_scores += 1;
                tracing::debug!("Created score: {} for game {}", score.id, game_hex_id);
            }
            Err(e) => {
                tracing::warn!("Failed to create score '{}' for game {}: {}, continuing with remaining scores", 
                    create_score.user_name, game_hex_id, e);
                failed_scores += 1;
            }
        }
    }

    if failed_games > 0 || failed_scores > 0 {
        tracing::warn!(
            "Database seed completed with some failures: {}/{} games and {}/{} scores imported from {} ({} games failed, {} scores failed)",
            created_games, created_games + failed_games, created_scores, created_scores + failed_scores, 
            csv_path, failed_games, failed_scores
        );
    } else {
        tracing::info!(
            "Database seed completed successfully: {} games and {} scores imported from {}",
            created_games, created_scores, csv_path
        );
    }

    Ok(())
}

/// Checks for seed file and imports if present.
/// Uses LEADR_SEED_FILE environment variable or defaults to "/data/seed.csv"
pub async fn check_and_seed(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let seed_file = std::env::var("LEADR_SEED_FILE")
        .unwrap_or_else(|_| "/data/seed.csv".to_string());
    
    tracing::info!("Checking for seed file at: {}", seed_file);
    
    seed_from_csv(pool, &seed_file).await
}