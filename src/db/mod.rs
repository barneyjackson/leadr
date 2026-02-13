use sqlx::{Pool, Postgres};

pub mod repository;
pub mod seed;

pub type DbPool = Pool<Postgres>;

/// Initializes the database with proper lifecycle management.
///
/// This function handles the complete database setup sequence:
/// 1. Establishes connection pool
/// 2. Runs migrations
/// 3. Performs seeding if configured
///
/// # Errors
/// Returns `sqlx::Error` if any step fails.
pub async fn initialize_database() -> Result<DbPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost:5432/leadr".to_string());

    tracing::info!("Initializing database: {}", database_url);

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
    sqlx::PgPool::connect(database_url).await
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
