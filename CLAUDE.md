# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

LEADR is a lightweight, high-performance game leaderboard API built with Rust. It's a single-tenant service designed for indie game developers to deploy their own instance and manage game leaderboards with full data ownership.

## Key Architecture Components

### Core Technology Stack
- **Web Framework**: Axum with Tokio async runtime
- **Database**: SQLite with SQLx for async queries
- **API Documentation**: OpenAPI via utoipa, served with Swagger UI
- **Authentication**: Simple API key middleware (header: `leadr-api-key`)

### Project Structure
- `src/handlers/` - HTTP request handlers for each resource (game, score, export, health)
- `src/models/` - Data models with serde serialization
- `src/db/` - Database pool management, repository pattern, and seeding logic
- `src/auth.rs` - API key authentication middleware
- `src/error/` - Centralized error handling
- `src/utils/` - Pagination utilities
- `migrations/` - SQLx database migrations
- `tests/` - Unit and integration tests

### Database Schema
The application uses two main tables:
- **games**: Stores leaderboard definitions with hex_id as unique identifier
- **scores**: Stores player scores linked to games via game_hex_id

Both tables support soft deletion via `is_deleted` flag.

## Essential Commands

### Development
```bash
# Start development server with hot reload
cargo watch -x dev

# Or run directly with required environment
LEADR_API_KEY=dev-key cargo run

# Run with custom settings
RUST_LOG=debug DATABASE_URL=sqlite:./test.db cargo dev
```

### Testing
```bash
# Run all tests (unit + integration)
cargo test

# Run tests sequentially (important for integration tests)
cargo test-sequential

# Run only integration tests
cargo test-integration

# Run a specific test
cargo test test_name
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Both should pass before committing
cargo fmt && cargo clippy
```

### Database Management
```bash
# Reset database (delete and re-migrate)
cargo db-reset

# Generate SQLx offline cache (required after schema changes)
cargo db-prepare

# Seed database from CSV file
LEADR_SEED_FILE=test_seed.csv cargo db-seed
```

### Documentation Generation
```bash
# Generate OpenAPI spec only
cargo run --bin generate_openapi

# Generate complete static docs site
./scripts/generate-docs.sh
```

## API Authentication

All API endpoints (except `/health`) require the `leadr-api-key` header matching the `LEADR_API_KEY` environment variable.

## Testing Approach

- **Unit Tests**: Located in `tests/unit/` - test individual components
- **Integration Tests**: In `tests/integration_tests.rs` - test full API flows
- Tests use temporary databases to ensure isolation
- Integration tests should run sequentially (`--test-threads=1`)

## Key Implementation Patterns

### Pagination
- All list endpoints use cursor-based pagination
- Response format includes `data`, `has_more`, `next_cursor`, `total_returned`
- Cursor encodes the last item's ID and sort value

### Error Handling
- Centralized error types in `src/error/mod.rs`
- All handlers return `Result<T, AppError>`
- Errors automatically converted to appropriate HTTP responses

### Repository Pattern
- Database operations abstracted in `src/db/repository.rs`
- Handlers call repository methods, not raw SQL
- Maintains separation between HTTP and database layers

## Environment Variables

Required:
- `LEADR_API_KEY` - API authentication key

Optional:
- `DATABASE_URL` - SQLite connection string (default: `sqlite:./leadr.db`)
- `RUST_LOG` - Logging level (default: `info`)
- `LEADR_SEED_FILE` - CSV file path for initial data import

## Docker Development

```bash
# Build Docker image
docker build -t leadr-api .

# Run container
docker run -p 3000:3000 -e LEADR_API_KEY=test-key leadr-api
```

## CI/CD Workflows

- `.github/workflows/release.yml` - Automated versioning and Docker image publishing
- `.github/workflows/docs.yml` - Automatic documentation deployment to Vercel
- Always run `cargo check` after completing your changes to check if code is valid and compiles