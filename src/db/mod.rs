use sqlx::{sqlite::SqlitePool, Pool, Sqlite};

pub mod repository;

pub type DbPool = Pool<Sqlite>;

/// Creates a new database connection pool.
/// 
/// # Errors
/// Returns `sqlx::Error` if the database connection fails.
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    SqlitePool::connect(database_url).await
}

/// Runs database migrations.
/// 
/// # Errors
/// Returns `sqlx::Error` if the migration fails.
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(sqlx::Error::from)
}
