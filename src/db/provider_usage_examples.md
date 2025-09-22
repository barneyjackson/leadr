# Database Provider Usage Examples

This document provides examples of how to use the new `DatabaseProvider` abstraction layer in LEADR.

## Basic Usage

### Creating a SQLite Provider

```rust
use leadr_api::db::provider::{DatabaseProvider, SqliteProvider};

// Create a new SQLite provider instance
let provider = SqliteProvider::new();

// Create a connection pool
let pool = SqliteProvider::create_pool("sqlite:./leadr.db").await?;

// Run migrations
SqliteProvider::run_migrations(&pool).await?;
```

### Using the Provider in Repositories

```rust
use leadr_api::db::provider::{DatabaseProvider, SqliteProvider};

// Example: Generic repository function that works with any provider
async fn count_games<P: DatabaseProvider>(
    provider: &P,
    pool: &P::Pool
) -> Result<i64, ApiError> {
    let count: i64 = provider.execute_scalar(
        pool,
        "SELECT COUNT(*) FROM game WHERE deleted_at IS NULL",
        &[]
    ).await?;

    Ok(count)
}

// Usage with SQLite
let provider = SqliteProvider::new();
let pool = SqliteProvider::create_pool("sqlite:./leadr.db").await?;
let game_count = count_games(&provider, &pool).await?;
```

## Migration from Direct SQLx Usage

### Before (Direct SQLite)

```rust
use sqlx::SqlitePool;

let pool = SqlitePool::connect("sqlite:./leadr.db").await?;
let games = sqlx::query_as!(
    Game,
    "SELECT * FROM game WHERE deleted_at IS NULL"
)
.fetch_all(&pool)
.await?;
```

### After (Provider Abstraction)

```rust
use leadr_api::db::provider::{DatabaseProvider, SqliteProvider};

let provider = SqliteProvider::new();
let pool = SqliteProvider::create_pool("sqlite:./leadr.db").await?;

// Use provider methods for database-agnostic operations
let rows = provider.execute_query(
    &pool,
    "SELECT * FROM game WHERE deleted_at IS NULL",
    &[]
).await?;
```

## Error Handling

The provider abstraction includes enhanced error handling:

```rust
use leadr_api::error::ApiError;

match SqliteProvider::create_pool("invalid://url").await {
    Ok(pool) => {
        // Success - continue with operations
    },
    Err(ApiError::Database(err)) => {
        // Handle database connection errors
        eprintln!("Database connection failed: {}", err);
    },
    Err(ApiError::ConnectionPool(msg)) => {
        // Handle connection pool errors
        eprintln!("Pool configuration error: {}", msg);
    },
    Err(other) => {
        // Handle other error types
        eprintln!("Unexpected error: {}", other);
    }
}
```

## Future PostgreSQL Usage

When PostgreSQL support is added (LDR-33), the same interface will work:

```rust
use leadr_api::db::provider::{DatabaseProvider, PostgresProvider};

// Same interface, different implementation
let provider = PostgresProvider::new();
let pool = PostgresProvider::create_pool("postgresql://...").await?;
PostgresProvider::run_migrations(&pool).await?;

// All repository functions work unchanged
let game_count = count_games(&provider, &pool).await?;
```

## Key Benefits

1. **Database Agnostic**: Same code works with SQLite and PostgreSQL
2. **Type Safety**: Compile-time guarantees for database operations
3. **Error Consistency**: Unified error handling across database types
4. **Performance**: Optimized connection settings per database type
5. **Backward Compatibility**: Existing code continues to work unchanged