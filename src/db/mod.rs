use sqlx::{sqlite::SqlitePool, Pool, Sqlite};

pub mod repository;
pub mod seed;

pub type DbPool = Pool<Sqlite>;

/// Initializes the database with proper lifecycle management.
/// 
/// This function handles the complete database setup sequence:
/// 1. Creates database file if it doesn't exist
/// 2. Establishes connection pool
/// 3. Runs migrations
/// 4. Performs seeding if configured
/// 
/// # Errors
/// Returns `sqlx::Error` if any step fails.
pub async fn initialize_database() -> Result<DbPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./leadr.db".to_string());
    
    tracing::info!("Initializing database: {}", database_url);
    
    // Create database file if it doesn't exist (SQLite-specific)
    if database_url.starts_with("sqlite:") {
        let db_path = database_url.strip_prefix("sqlite:").unwrap_or(&database_url);
        if !std::path::Path::new(db_path).exists() {
            tracing::info!("Creating new database file: {}", db_path);
            // Create empty file - SQLite will initialize it
            std::fs::File::create(db_path).map_err(|e| {
                sqlx::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to create database file: {}", e)
                ))
            })?;
        }
    }
    
    // Create connection pool
    let pool = create_pool(&database_url).await?;
    
    // Run migrations
    tracing::info!("Running database migrations...");
    run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");
    
    // Handle seeding
    if let Err(e) = seed::check_and_seed(&pool).await {
        tracing::warn!("Seeding failed but continuing startup: {}", e);
    }
    
    Ok(pool)
}

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
