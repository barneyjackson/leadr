use chrono::Utc;
use sqlx::{Row, SqlitePool};

use crate::error::{ApiError, Result};
use crate::models::{
    CreateGame, CreateScore, Game, GameRow, Score, ScoreRow, UpdateGame, UpdateScore,
};
use crate::utils::pagination::{
    cursor::{
        decode_game_cursor, decode_score_cursor, encode_game_cursor, encode_score_cursor,
        GameCursor, ScoreCursor,
    },
    PaginatedResponse, PaginationParams, ScoreSortParams, SortOrder,
};

pub struct GameRepository;
pub struct ScoreRepository;

impl GameRepository {
    /// Create a new game
    ///
    /// # Errors
    /// Returns `ApiError::ValidationError` if the game name is invalid.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Does not panic under normal operation.
    pub async fn create(pool: &SqlitePool, create_data: CreateGame) -> Result<Game> {
        // Validate inputs
        Game::validate_name(&create_data.name)?;

        let hex_id = Game::generate_hex_id();
        let now = Utc::now();
        let now_naive = now.naive_utc();

        let row = sqlx::query!(
            r#"
            INSERT INTO game (hex_id, name, description, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            RETURNING id, hex_id, name, description, created_at, updated_at, deleted_at
            "#,
            hex_id,
            create_data.name,
            create_data.description,
            now_naive,
            now_naive
        )
        .fetch_one(pool)
        .await?;

        let game_row = GameRow {
            id: row.id,
            hex_id: row.hex_id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };

        let game = Game::from(game_row);
        Ok(game)
    }

    /// Get a game by `hex_id`
    ///
    /// # Errors
    /// Returns `ApiError::InvalidParameter` if the `hex_id` is invalid.
    /// Returns `ApiError::NotFound` if no game exists with the given `hex_id`.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Panics if the database returns a NULL id, which should never happen.
    pub async fn get_by_hex_id(pool: &SqlitePool, hex_id: &str) -> Result<Game> {
        Game::validate_hex_id(hex_id).map_err(ApiError::InvalidParameter)?;

        let row = sqlx::query!(
            r#"
            SELECT id, hex_id, name, description, created_at, updated_at, deleted_at
            FROM game 
            WHERE hex_id = ?1 AND deleted_at IS NULL
            "#,
            hex_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;

        let game_row = GameRow {
            id: row.id.unwrap(),
            hex_id: row.hex_id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };

        let game = Game::from(game_row);
        Ok(game)
    }

    /// Get a game by numeric id
    ///
    /// # Errors
    /// Returns `ApiError::NotFound` if no game exists with the given id.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Does not panic under normal operation.
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Game> {
        let row = sqlx::query!(
            r#"
            SELECT id, hex_id, name, description, created_at, updated_at, deleted_at
            FROM game 
            WHERE id = ?1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;

        let game_row = GameRow {
            id: row.id,
            hex_id: row.hex_id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };

        let game = Game::from(game_row);
        Ok(game)
    }

    /// List games with pagination
    ///
    /// # Errors
    /// Returns `ApiError::ValidationError` if the cursor is invalid.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Panics if the database returns a NULL id, which should never happen.
    pub async fn list(
        pool: &SqlitePool,
        pagination: PaginationParams,
    ) -> Result<PaginatedResponse<Game>> {
        let limit = pagination.get_limit();
        let fetch_limit = i64::from(limit + 1); // Fetch one extra to check for more pages

        let games = if let Some(cursor_str) = &pagination.cursor {
            let cursor = decode_game_cursor(cursor_str)
                .map_err(|e| ApiError::ValidationError(format!("Invalid cursor: {e}")))?;

            let cursor_datetime = chrono::DateTime::parse_from_rfc3339(&cursor.created_at)
                .map_err(|e| ApiError::ValidationError(format!("Invalid cursor date: {e}")))?
                .with_timezone(&chrono::Utc);
            let cursor_created_at = cursor_datetime.naive_utc();
            let game_rows = sqlx::query!(
                r#"
                SELECT id, hex_id, name, description, created_at, updated_at, deleted_at
                FROM game 
                WHERE deleted_at IS NULL 
                AND (created_at, hex_id) < (?1, ?2)
                ORDER BY created_at DESC, hex_id DESC
                LIMIT ?3
                "#,
                cursor_created_at,
                cursor.hex_id,
                fetch_limit
            )
            .fetch_all(pool)
            .await?;

            game_rows
                .into_iter()
                .map(|row| {
                    Game::from(GameRow {
                        id: row.id.unwrap(),
                        hex_id: row.hex_id,
                        name: row.name,
                        description: row.description,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        deleted_at: row.deleted_at,
                    })
                })
                .collect()
        } else {
            let game_rows = sqlx::query!(
                r#"
                SELECT id, hex_id, name, description, created_at, updated_at, deleted_at
                FROM game 
                WHERE deleted_at IS NULL
                ORDER BY created_at DESC, hex_id DESC
                LIMIT ?1
                "#,
                fetch_limit
            )
            .fetch_all(pool)
            .await?;

            game_rows
                .into_iter()
                .map(|row| {
                    Game::from(GameRow {
                        id: row.id.unwrap(),
                        hex_id: row.hex_id,
                        name: row.name,
                        description: row.description,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        deleted_at: row.deleted_at,
                    })
                })
                .collect()
        };

        let response =
            PaginatedResponse::from_query_results(games, limit, pagination.cursor, |game| {
                let cursor = GameCursor::from_game(game);
                encode_game_cursor(&cursor).ok()
            });

        Ok(response)
    }

    /// Update a game
    ///
    /// # Errors
    /// Returns `ApiError::InvalidParameter` if the `hex_id` or name is invalid.
    /// Returns `ApiError::NotFound` if no game exists with the given `hex_id`.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Panics if the database returns a NULL id, which should never happen.
    pub async fn update(pool: &SqlitePool, hex_id: &str, update_data: UpdateGame) -> Result<Game> {
        Game::validate_hex_id(hex_id).map_err(ApiError::InvalidParameter)?;

        if let Some(ref name) = update_data.name {
            Game::validate_name(name)?;
        }

        let now = Utc::now();
        let now_naive = now.naive_utc();

        let row = sqlx::query!(
            r#"
            UPDATE game 
            SET name = COALESCE(?1, name),
                description = COALESCE(?2, description),
                updated_at = ?3
            WHERE hex_id = ?4 AND deleted_at IS NULL
            RETURNING id, hex_id, name, description, created_at, updated_at, deleted_at
            "#,
            update_data.name,
            update_data.description,
            now_naive,
            hex_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;

        let game_row = GameRow {
            id: row.id.unwrap(),
            hex_id: row.hex_id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };

        let game = Game::from(game_row);
        Ok(game)
    }

    /// Soft delete a game (this will cascade to scores via trigger)
    ///
    /// # Errors
    /// Returns `ApiError::InvalidParameter` if the `hex_id` is invalid.
    /// Returns `ApiError::NotFound` if no game exists with the given `hex_id`.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Does not panic under normal operation.
    pub async fn soft_delete(pool: &SqlitePool, hex_id: &str) -> Result<()> {
        Game::validate_hex_id(hex_id).map_err(ApiError::InvalidParameter)?;

        let now = Utc::now();
        let now_naive = now.naive_utc();
        let rows_affected = sqlx::query!(
            "UPDATE game SET deleted_at = ?1, updated_at = ?1 WHERE hex_id = ?2 AND deleted_at IS NULL",
            now_naive,
            hex_id
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ApiError::NotFound);
        }

        Ok(())
    }

    /// Restore a soft-deleted game (this will cascade to scores via trigger)
    ///
    /// # Errors
    /// Returns `ApiError::InvalidParameter` if the `hex_id` is invalid.
    /// Returns `ApiError::NotFound` if no game exists with the given `hex_id` or it's not deleted.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Panics if the database returns a NULL id, which should never happen.
    pub async fn restore(pool: &SqlitePool, hex_id: &str) -> Result<Game> {
        Game::validate_hex_id(hex_id).map_err(ApiError::InvalidParameter)?;

        let now = Utc::now();
        let now_naive = now.naive_utc();
        let row = sqlx::query!(
            r#"
            UPDATE game 
            SET deleted_at = NULL, updated_at = ?1
            WHERE hex_id = ?2 AND deleted_at IS NOT NULL
            RETURNING id, hex_id, name, description, created_at, updated_at, deleted_at
            "#,
            now_naive,
            hex_id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;

        let game_row = GameRow {
            id: row.id.unwrap(),
            hex_id: row.hex_id,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
            deleted_at: row.deleted_at,
        };

        let game = Game::from(game_row);
        Ok(game)
    }
}

impl ScoreRepository {
    /// Create a new score
    ///
    /// # Errors
    /// Returns `ApiError::ValidationError` if user name, user ID, or JSON data is invalid.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Panics if `serde_json::to_string` fails on valid JSON data, which should never happen.
    pub async fn create(pool: &SqlitePool, create_data: CreateScore) -> Result<Score> {
        // Validate inputs
        Score::validate_user_name(&create_data.user_name)?;
        Score::validate_user_id(&create_data.user_id)?;

        // Validate JSON if provided
        if let Some(ref extra) = create_data.extra {
            serde_json::to_string(extra).map_err(|e| {
                ApiError::ValidationError(format!("Invalid JSON in extra field: {e}"))
            })?;
        }

        // Parse score_val from score if not provided
        let score_val = create_data
            .score_val
            .unwrap_or_else(|| create_data.score.parse::<f64>().unwrap_or(0.0));

        let now = Utc::now();
        let now_naive = now.naive_utc();
        let extra_json = create_data
            .extra
            .map(|v| serde_json::to_string(&v).unwrap());

        let row = sqlx::query!(
            r#"
            INSERT INTO score (game_hex_id, score, score_val, user_name, user_id, extra, submitted_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            RETURNING id, game_hex_id, score, score_val, user_name, user_id, extra, submitted_at, deleted_at
            "#,
            create_data.game_hex_id,
            create_data.score,
            score_val,
            create_data.user_name,
            create_data.user_id,
            extra_json,
            now_naive
        )
        .fetch_one(pool)
        .await?;

        let score_row = ScoreRow {
            id: row.id,
            game_hex_id: row.game_hex_id,
            score: row.score,
            score_val: row.score_val,
            user_name: row.user_name,
            user_id: row.user_id,
            extra: row.extra,
            submitted_at: row.submitted_at,
            deleted_at: row.deleted_at,
        };

        let score = Score::from(score_row);
        Ok(score)
    }

    /// Get a score by id
    ///
    /// # Errors
    /// Returns `ApiError::NotFound` if no score exists with the given id.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Does not panic under normal operation.
    pub async fn get_by_id(pool: &SqlitePool, id: i64) -> Result<Score> {
        let row = sqlx::query!(
            r#"
            SELECT id, game_hex_id, score, score_val, user_name, user_id, extra, submitted_at, deleted_at
            FROM score 
            WHERE id = ?1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;

        let score_row = ScoreRow {
            id: row.id,
            game_hex_id: row.game_hex_id,
            score: row.score,
            score_val: row.score_val,
            user_name: row.user_name,
            user_id: row.user_id,
            extra: row.extra,
            submitted_at: row.submitted_at,
            deleted_at: row.deleted_at,
        };

        let score = Score::from(score_row);
        Ok(score)
    }

    /// List scores for a game with pagination and sorting
    ///
    /// # Errors
    /// Returns `ApiError::ValidationError` if the game `hex_id` or cursor is invalid.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Does not panic under normal operation.
    pub async fn list_by_game(
        pool: &SqlitePool,
        game_hex_id: &str,
        pagination: PaginationParams,
        sort_params: ScoreSortParams,
    ) -> Result<PaginatedResponse<Score>> {
        Game::validate_hex_id(game_hex_id)?;

        let limit = pagination.get_limit();
        let fetch_limit = i64::from(limit + 1);
        let sort_field = sort_params.get_cursor_field();

        let scores = if let Some(cursor_str) = &pagination.cursor {
            let cursor = decode_score_cursor(cursor_str)
                .map_err(|e| ApiError::ValidationError(format!("Invalid cursor: {e}")))?;

            // Build dynamic query based on sort parameters
            let order_clause = sort_params.to_sql_order_clause();
            let comparison_op = match sort_params.get_sort_order() {
                SortOrder::Ascending => ">",
                SortOrder::Descending => "<",
            };

            let query = format!(
                r"
                SELECT id, game_hex_id, score, score_val, user_name, user_id, extra, submitted_at, deleted_at
                FROM score 
                WHERE deleted_at IS NULL 
                AND game_hex_id = ?1
                AND ({sort_field} {comparison_op} ?2 OR ({sort_field} = ?2 AND id > ?3))
                ORDER BY {order_clause}, id
                LIMIT ?4
                "
            );

            let score_rows = sqlx::query(&query)
                .bind(game_hex_id)
                .bind(&cursor.sort_value)
                .bind(cursor.id)
                .bind(fetch_limit)
                .fetch_all(pool)
                .await?;

            score_rows
                .into_iter()
                .map(|row| {
                    Score::from(ScoreRow {
                        id: row.get("id"),
                        game_hex_id: row.get("game_hex_id"),
                        score: row.get("score"),
                        score_val: row.get("score_val"),
                        user_name: row.get("user_name"),
                        user_id: row.get("user_id"),
                        extra: row.get("extra"),
                        submitted_at: row.get("submitted_at"),
                        deleted_at: row.get("deleted_at"),
                    })
                })
                .collect()
        } else {
            let order_clause = sort_params.to_sql_order_clause();
            let query = format!(
                r"
                SELECT id, game_hex_id, score, score_val, user_name, user_id, extra, submitted_at, deleted_at
                FROM score 
                WHERE deleted_at IS NULL AND game_hex_id = ?1
                ORDER BY {order_clause}, id
                LIMIT ?2
                "
            );

            let score_rows = sqlx::query(&query)
                .bind(game_hex_id)
                .bind(fetch_limit)
                .fetch_all(pool)
                .await?;

            score_rows
                .into_iter()
                .map(|row| {
                    Score::from(ScoreRow {
                        id: row.get("id"),
                        game_hex_id: row.get("game_hex_id"),
                        score: row.get("score"),
                        score_val: row.get("score_val"),
                        user_name: row.get("user_name"),
                        user_id: row.get("user_id"),
                        extra: row.get("extra"),
                        submitted_at: row.get("submitted_at"),
                        deleted_at: row.get("deleted_at"),
                    })
                })
                .collect()
        };

        // Scores are already parsed from ScoreRow conversion
        let response =
            PaginatedResponse::from_query_results(scores, limit, pagination.cursor, |score| {
                let cursor = ScoreCursor::from_score(score, sort_field);
                encode_score_cursor(&cursor).ok()
            });

        Ok(response)
    }

    /// Update a score
    ///
    /// # Errors
    /// Returns `ApiError::ValidationError` if user name, user ID, or JSON data is invalid.
    /// Returns `ApiError::NotFound` if no score exists with the given id.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Panics if the database returns a NULL id, which should never happen.
    pub async fn update(pool: &SqlitePool, id: i64, update_data: UpdateScore) -> Result<Score> {
        if let Some(ref user_name) = update_data.user_name {
            Score::validate_user_name(user_name)?;
        }
        if let Some(ref user_id) = update_data.user_id {
            Score::validate_user_id(user_id)?;
        }

        // Validate JSON if provided
        let extra_json = if let Some(ref extra) = update_data.extra {
            Some(serde_json::to_string(extra).map_err(|e| {
                ApiError::ValidationError(format!("Invalid JSON in extra field: {e}"))
            })?)
        } else {
            None
        };

        // Calculate score_val from score if needed
        let score_val = if let Some(ref score) = update_data.score {
            Some(
                update_data
                    .score_val
                    .unwrap_or_else(|| score.parse::<f64>().unwrap_or(0.0)),
            )
        } else {
            update_data.score_val
        };

        let row = sqlx::query!(
            r#"
            UPDATE score 
            SET score = COALESCE(?1, score),
                score_val = COALESCE(?2, score_val),
                user_name = COALESCE(?3, user_name),
                user_id = COALESCE(?4, user_id),
                extra = COALESCE(?5, extra)
            WHERE id = ?6 AND deleted_at IS NULL
            RETURNING id, game_hex_id, score, score_val, user_name, user_id, extra, submitted_at, deleted_at
            "#,
            update_data.score,
            score_val,
            update_data.user_name,
            update_data.user_id,
            extra_json,
            id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;

        let score_row = ScoreRow {
            id: row.id.unwrap(),
            game_hex_id: row.game_hex_id,
            score: row.score,
            score_val: row.score_val,
            user_name: row.user_name,
            user_id: row.user_id,
            extra: row.extra,
            submitted_at: row.submitted_at,
            deleted_at: None, // Record is not deleted since WHERE clause ensures deleted_at IS NULL
        };

        let score = Score::from(score_row);
        Ok(score)
    }

    /// Soft delete a score
    ///
    /// # Errors
    /// Returns `ApiError::NotFound` if no score exists with the given id.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Does not panic under normal operation.
    pub async fn soft_delete(pool: &SqlitePool, id: i64) -> Result<()> {
        let now = Utc::now();
        let now_naive = now.naive_utc();
        let rows_affected = sqlx::query!(
            "UPDATE score SET deleted_at = ?1 WHERE id = ?2 AND deleted_at IS NULL",
            now_naive,
            id
        )
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(ApiError::NotFound);
        }

        Ok(())
    }

    /// Restore a soft-deleted score
    ///
    /// # Errors
    /// Returns `ApiError::NotFound` if no score exists with the given id or it's not deleted.
    /// Returns `ApiError::DatabaseError` if the database operation fails.
    ///
    /// # Panics
    /// Panics if the database returns a NULL id, which should never happen.
    pub async fn restore(pool: &SqlitePool, id: i64) -> Result<Score> {
        let row = sqlx::query!(
            r#"
            UPDATE score 
            SET deleted_at = NULL
            WHERE id = ?1 AND deleted_at IS NOT NULL
            RETURNING id, game_hex_id, score, score_val, user_name, user_id, extra, submitted_at, deleted_at
            "#,
            id
        )
        .fetch_optional(pool)
        .await?
        .ok_or(ApiError::NotFound)?;

        let score_row = ScoreRow {
            id: row.id.unwrap(),
            game_hex_id: row.game_hex_id,
            score: row.score,
            score_val: row.score_val,
            user_name: row.user_name,
            user_id: row.user_id,
            extra: row.extra,
            submitted_at: row.submitted_at,
            deleted_at: row.deleted_at,
        };

        let score = Score::from(score_row);
        Ok(score)
    }
}
