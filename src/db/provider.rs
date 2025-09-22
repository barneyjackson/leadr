use async_trait::async_trait;
use sqlx::{Database, Row};
use std::fmt::Debug;

use crate::error::Result;

/// Database provider abstraction trait supporting multiple database backends.
///
/// This trait provides a common interface for database operations that can be
/// implemented by different database providers (SQLite, PostgreSQL, etc.).
///
/// # Example
///
/// ```rust
/// use leadr_api::db::provider::{DatabaseProvider, SqliteProvider};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let provider = SqliteProvider::new();
///     let pool = SqliteProvider::create_pool("sqlite:./test.db").await?;
///     SqliteProvider::run_migrations(&pool).await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait DatabaseProvider: Send + Sync + Clone + Debug + 'static {
    /// The underlying database type from sqlx
    type DB: Database;
    /// The connection pool type for this provider
    type Pool: Send + Sync + Clone;
    /// The row type returned by queries
    type Row: Row;

    /// Create a new database connection pool from a connection string.
    ///
    /// # Arguments
    ///
    /// * `database_url` - Connection string for the database
    ///
    /// # Errors
    ///
    /// Returns error if connection fails or URL is invalid.
    async fn create_pool(database_url: &str) -> Result<Self::Pool>;

    /// Run database migrations for this provider.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    ///
    /// # Errors
    ///
    /// Returns error if migrations fail to apply.
    async fn run_migrations(pool: &Self::Pool) -> Result<()>;

    /// Execute a query and return all result rows.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `query` - SQL query string
    /// * `params` - Query parameters
    ///
    /// # Errors
    ///
    /// Returns error if query execution fails.
    async fn execute_query(
        &self,
        pool: &Self::Pool,
        query: &str,
        params: &[&(dyn sqlx::Encode<Self::DB> + Send + Sync)],
    ) -> Result<Vec<Self::Row>>;

    /// Execute a query and return a scalar result.
    ///
    /// # Arguments
    ///
    /// * `pool` - Database connection pool
    /// * `query` - SQL query string
    /// * `params` - Query parameters
    ///
    /// # Errors
    ///
    /// Returns error if query execution fails or no result found.
    async fn execute_scalar<T>(
        &self,
        pool: &Self::Pool,
        query: &str,
        params: &[&(dyn sqlx::Encode<Self::DB> + Send + Sync)],
    ) -> Result<T>
    where
        T: for<'r> sqlx::Decode<'r, Self::DB> + sqlx::Type<Self::DB> + Send + Unpin;

    /// Get the name of this database provider.
    fn database_name(&self) -> &'static str;

    /// Check if this provider supports row-level security.
    fn supports_row_level_security(&self) -> bool;
}

/// SQLite database provider implementation.
///
/// Provides SQLite-specific optimizations including WAL mode and
/// connection pooling tuned for single-file database access.
#[derive(Debug, Clone)]
pub struct SqliteProvider;

impl SqliteProvider {
    /// Create a new SQLite provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqliteProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseProvider for SqliteProvider {
    type DB = sqlx::Sqlite;
    type Pool = sqlx::Pool<sqlx::Sqlite>;
    type Row = sqlx::sqlite::SqliteRow;

    async fn create_pool(database_url: &str) -> Result<Self::Pool> {
        use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
        use std::str::FromStr;
        use std::time::Duration;

        tracing::info!("Creating SQLite connection pool for: {}", database_url);

        // Parse connection string and set optimized options
        let options = SqliteConnectOptions::from_str(database_url)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(30))
            .pragma("cache_size", "-64000") // 64MB cache
            .pragma("temp_store", "memory")
            .pragma("mmap_size", "268435456"); // 256MB memory map

        let pool = sqlx::SqlitePool::connect_with(options).await?;

        tracing::info!("SQLite connection pool created successfully");
        Ok(pool)
    }

    async fn run_migrations(pool: &Self::Pool) -> Result<()> {
        tracing::info!("Running SQLite migrations...");

        sqlx::migrate!("./migrations").run(pool).await?;

        tracing::info!("SQLite migrations completed successfully");
        Ok(())
    }

    async fn execute_query(
        &self,
        pool: &Self::Pool,
        query: &str,
        _params: &[&(dyn sqlx::Encode<Self::DB> + Send + Sync)],
    ) -> Result<Vec<Self::Row>> {
        // For now, implement basic query execution
        // Full parameter binding will be implemented in repository layer
        let rows = sqlx::query(query).fetch_all(pool).await?;

        Ok(rows)
    }

    async fn execute_scalar<T>(
        &self,
        pool: &Self::Pool,
        query: &str,
        _params: &[&(dyn sqlx::Encode<Self::DB> + Send + Sync)],
    ) -> Result<T>
    where
        T: for<'r> sqlx::Decode<'r, Self::DB> + sqlx::Type<Self::DB> + Send + Unpin,
    {
        // For now, implement basic scalar query execution
        // Full parameter binding will be implemented in repository layer
        let result: T = sqlx::query_scalar(query).fetch_one(pool).await?;

        Ok(result)
    }

    fn database_name(&self) -> &'static str {
        "SQLite"
    }

    fn supports_row_level_security(&self) -> bool {
        false
    }
}

/// PostgreSQL database provider implementation.
///
/// Provides PostgreSQL-specific optimizations including connection pooling
/// tuned for high-performance multi-tenant scenarios and row-level security support.
#[derive(Debug, Clone)]
pub struct PostgresProvider;

impl PostgresProvider {
    /// Create a new PostgreSQL provider instance.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PostgresProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DatabaseProvider for PostgresProvider {
    type DB = sqlx::Postgres;
    type Pool = sqlx::Pool<sqlx::Postgres>;
    type Row = sqlx::postgres::PgRow;

    async fn create_pool(database_url: &str) -> Result<Self::Pool> {
        use sqlx::postgres::PgPoolOptions;
        use std::time::Duration;

        tracing::info!(
            "Creating PostgreSQL connection pool for: {}",
            // Mask sensitive parts of the URL for logging
            database_url.split('@').last().unwrap_or("***")
        );

        // Configure PostgreSQL connection pool with optimizations
        let pool = PgPoolOptions::new()
            .max_connections(20) // Higher concurrency for PostgreSQL
            .min_connections(5) // Keep minimum connections warm
            .acquire_timeout(Duration::from_secs(30))
            .idle_timeout(Duration::from_secs(300)) // 5 minutes
            .max_lifetime(Duration::from_secs(1800)) // 30 minutes
            .test_before_acquire(true) // Validate connections
            .connect(database_url)
            .await
            .map_err(|e| crate::error::ApiError::Database(e))?;

        tracing::info!("PostgreSQL connection pool created successfully");
        Ok(pool)
    }

    async fn run_migrations(pool: &Self::Pool) -> Result<()> {
        tracing::info!("Running PostgreSQL migrations...");

        sqlx::migrate!("./migrations")
            .run(pool)
            .await
            .map_err(|e| crate::error::ApiError::Migration(e))?;

        tracing::info!("PostgreSQL migrations completed successfully");
        Ok(())
    }

    async fn execute_query(
        &self,
        pool: &Self::Pool,
        query: &str,
        _params: &[&(dyn sqlx::Encode<Self::DB> + Send + Sync)],
    ) -> Result<Vec<Self::Row>> {
        // For now, implement basic query execution
        // Full parameter binding will be implemented in repository layer
        let rows = sqlx::query(query)
            .fetch_all(pool)
            .await
            .map_err(|e| crate::error::ApiError::Database(e))?;

        Ok(rows)
    }

    async fn execute_scalar<T>(
        &self,
        pool: &Self::Pool,
        query: &str,
        _params: &[&(dyn sqlx::Encode<Self::DB> + Send + Sync)],
    ) -> Result<T>
    where
        T: for<'r> sqlx::Decode<'r, Self::DB> + sqlx::Type<Self::DB> + Send + Unpin,
    {
        // For now, implement basic scalar query execution
        // Full parameter binding will be implemented in repository layer
        let result: T = sqlx::query_scalar(query)
            .fetch_one(pool)
            .await
            .map_err(|e| crate::error::ApiError::Database(e))?;

        Ok(result)
    }

    fn database_name(&self) -> &'static str {
        "PostgreSQL"
    }

    fn supports_row_level_security(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_sqlite_provider_creation() {
        let provider = SqliteProvider::new();
        assert_eq!(provider.database_name(), "SQLite");
        assert!(!provider.supports_row_level_security());
    }

    #[tokio::test]
    async fn test_sqlite_pool_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_string_lossy());

        let pool = SqliteProvider::create_pool(&db_url).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_sqlite_migrations() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_string_lossy());

        let pool = SqliteProvider::create_pool(&db_url).await.unwrap();
        let result = SqliteProvider::run_migrations(&pool).await;

        // Should succeed even with no migrations or existing migrations
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sqlite_basic_query() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_string_lossy());

        let provider = SqliteProvider::new();
        let pool = SqliteProvider::create_pool(&db_url).await.unwrap();

        // Test basic query execution
        let rows = provider
            .execute_query(&pool, "SELECT 1 as test_col", &[])
            .await;
        assert!(rows.is_ok());
    }

    #[tokio::test]
    async fn test_sqlite_scalar_query() {
        let temp_file = NamedTempFile::new().unwrap();
        let db_url = format!("sqlite:{}", temp_file.path().to_string_lossy());

        let provider = SqliteProvider::new();
        let pool = SqliteProvider::create_pool(&db_url).await.unwrap();

        // Test scalar query execution
        let result: i32 = provider
            .execute_scalar(&pool, "SELECT 42", &[])
            .await
            .unwrap();
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_postgres_provider_creation() {
        let provider = PostgresProvider::new();
        assert_eq!(provider.database_name(), "PostgreSQL");
        assert!(provider.supports_row_level_security());
    }

    #[tokio::test]
    async fn test_postgres_provider_default() {
        let provider = PostgresProvider::default();
        assert_eq!(provider.database_name(), "PostgreSQL");
    }

    // Note: Actual PostgreSQL connection tests would require a running PostgreSQL instance
    // These tests focus on the provider structure and basic functionality
    #[tokio::test]
    async fn test_postgres_invalid_connection_string() {
        // Test that invalid connection strings are properly handled
        let result = PostgresProvider::create_pool("invalid://connection/string").await;
        assert!(result.is_err());

        // Verify it returns the expected error type
        match result {
            Err(crate::error::ApiError::Database(_)) => {}
            _ => panic!("Expected Database error for invalid connection string"),
        }
    }
}
