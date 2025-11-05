use axum::Router;
use dotenvy::{EnvLoader, EnvSequence};
use leadr_api::{create_app, db};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

/// Application environment for loading configuration
enum AppEnv {
    Dev,
    Prod,
    Test,
}

impl FromStr for AppEnv {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            "test" => Ok(Self::Test),
            s => Err(format!("Invalid AppEnv: {s}")),
        }
    }
}

impl From<AppEnv> for EnvSequence {
    fn from(v: AppEnv) -> Self {
        match v {
            AppEnv::Dev => Self::InputThenEnv,
            AppEnv::Prod => Self::EnvOnly,
            AppEnv::Test => Self::EnvThenInput, // File overrides existing env vars
        }
    }
}

// Load .env.test before any tests run
fn load_test_env() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let app_env = AppEnv::Test;

        EnvLoader::with_path(".env.test")
            .sequence(app_env.into())
            .load()
            .expect("Failed to load .env.test");
    });
}

// Global test database instance - created once per test run
static TEST_DB: OnceCell<Arc<TestDatabase>> = OnceCell::const_new();

/// Test database that persists for the entire test run
pub struct TestDatabase {
    pub name: String,
    pub pool: db::DbPool,
}

impl TestDatabase {
    async fn setup() -> Self {
        // Ensure .env.test is loaded
        load_test_env();

        let db_name = format!("leadr_test_{}", Uuid::new_v4().simple());
        let base_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set (base postgres URL without database name)");

        tracing::info!("Setting up test database: {}", db_name);

        // Connect to postgres database to create test DB
        let admin_url = format!("{}/postgres", base_url);
        let admin_pool = db::create_pool(&admin_url)
            .await
            .expect("Failed to connect to postgres database");

        // Create test database
        sqlx::query(&format!("CREATE DATABASE \"{}\"", db_name))
            .execute(&admin_pool)
            .await
            .expect("Failed to create test database");

        // Connect to test database and run migrations
        let test_url = format!("{}/{}", base_url, db_name);
        let pool = db::create_pool(&test_url).await.unwrap();
        db::run_migrations(&pool).await.unwrap();

        tracing::info!("Test database {} ready with migrations applied", db_name);

        Self { name: db_name, pool }
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Database cleanup happens when test process exits
        tracing::info!(
            "Test run complete, database {} can be cleaned up manually if needed",
            self.name
        );
    }
}

/// Get or initialize the shared test database
/// This is called automatically by test helpers
pub async fn get_test_db() -> Arc<TestDatabase> {
    TEST_DB
        .get_or_init(|| async { Arc::new(TestDatabase::setup().await) })
        .await
        .clone()
}

/// Test application fixture with automatic database cleanup
///
/// Each integration test should create a new `TestApp` instance which will:
/// 1. Clean the database before the test runs (truncate all tables)
/// 2. Provide an isolated Router for making HTTP requests
/// 3. Ensure test isolation without manual cleanup
///
/// # Example
/// ```
/// #[tokio::test]
/// async fn test_create_game() {
///     let test_app = TestApp::new().await;
///     let response = test_app.router()
///         .oneshot(request_with_api_key("POST", "/games", Some(&body)))
///         .await
///         .unwrap();
/// }
/// ```
pub struct TestApp {
    router: Router,
    _pool: db::DbPool,
}

impl TestApp {
    /// Create a new test app with a clean database
    pub async fn new() -> Self {
        load_test_env();
        std::env::set_var("LEADR_API_KEY", "test_api_key_123");

        let test_db = get_test_db().await;
        let pool = test_db.pool.clone();

        // Clean database before test for isolation
        cleanup_database(&pool).await;

        let router = create_app(pool.clone());

        Self {
            router,
            _pool: pool,
        }
    }

    /// Get the router for making test requests
    pub fn router(&self) -> Router {
        self.router.clone()
    }
}

/// Clean all data from the database for test isolation
/// Truncates tables in the correct order to respect foreign key constraints
async fn cleanup_database(pool: &db::DbPool) {
    // Truncate in reverse dependency order (scores depend on games)
    sqlx::query("TRUNCATE TABLE score CASCADE")
        .execute(pool)
        .await
        .expect("Failed to truncate score table");

    sqlx::query("TRUNCATE TABLE game CASCADE")
        .execute(pool)
        .await
        .expect("Failed to truncate game table");
}

/// Create test app (DEPRECATED - use TestApp::new() instead)
///
/// This function is kept for backward compatibility during migration.
/// New tests should use TestApp::new() for automatic cleanup.
#[deprecated(since = "0.1.0", note = "Use TestApp::new() instead for automatic cleanup")]
pub async fn create_test_app() -> Router {
    load_test_env();
    std::env::set_var("LEADR_API_KEY", "test_api_key_123");

    let test_db = get_test_db().await;
    create_app(test_db.pool.clone())
}

/// Get the test database pool directly for unit tests
/// Unit tests should start their own transactions for isolation:
///
/// ```
/// let pool = common::create_test_pool().await;
/// let mut tx = pool.begin().await.unwrap();
/// // Use &mut tx for all operations
/// // Automatic rollback when tx drops
/// ```
pub async fn create_test_pool() -> db::DbPool {
    load_test_env();
    let test_db = get_test_db().await;
    test_db.pool.clone()
}

/// Helper to get a test transaction for explicit transaction testing
/// The transaction automatically rolls back when dropped, ensuring test isolation
pub async fn begin_test_transaction() -> sqlx::Transaction<'static, sqlx::Postgres> {
    let pool = create_test_pool().await;
    pool.begin()
        .await
        .expect("Failed to start test transaction")
}
