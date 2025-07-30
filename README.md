# 🎮 LEADR - Lightweight Game Leaderboard API

> **A blazingly fast, single-tenant leaderboard API built for indie game developers**

```
    ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
    █                                                           █
    █  ┌─┐  ┌─┐  ┌─┐  ┌─┐  ┌─┐    LEADR API                   █
    █  │ │  │ │  │ │  │ │  │ │    Fast • Simple • Secure     █
    █  └─┘  └─┘  └─┘  └─┘  └─┘                                █
    █                                                           █
    ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀
```

## Features

- **⚡ Lightning Fast**: Built with Rust and Axum for maximum performance
- **🪶 Ultra Lightweight**: Minimal resource usage, perfect for indie budgets
- **🔒 Secure by Default**: API key authentication with no user management overhead
- **🎯 Single-Tenant**: Deploy your own instance, own your data
- **📦 Docker Ready**: One-command deployment to any cloud platform
- **🧪 Test-Driven**: Comprehensive test suite for reliability
- **💾 SQLite Backend**: Zero-config database that just works

## Perfect for Game Developers

Whether you're building a retro arcade game, puzzle platformer, or competitive multiplayer experience, LEADR handles your leaderboard needs without the bloat:

- **Multiple Leaderboards**: Support different game modes, levels, or difficulties
- **Flexible Scoring**: Store scores with custom metadata using JSON extras
- **Soft Deletes**: Never lose data, just hide what you don't need
- **Simple Integration**: RESTful API that works with any game engine

## TODO List

### Core Infrastructure
- [x] Create Docker configuration files (Dockerfile, docker-compose, .dockerignore)

### Testing & Quality
- [ ] **Code Quality & Documentation Improvements**
  - [ ] Merge duplicate error handling match arms to reduce code duplication
  - [ ] Add missing `# Panics` documentation for functions that may panic
  - [ ] Optimize string building in hex ID generation
- [ ] Add performance benchmarks
- [ ] Set up CI/CD pipeline with automated quality checks

### Deployment
- [x] **Create multi-stage Dockerfile with security best practices**
- [ ] Add Docker Compose for local development
- [ ] Create deployment guides for major cloud providers
- [ ] Add environment configuration documentation

### Documentation
- [ ] Add API documentation with examples
- [ ] Create integration guides for popular game engines
- [ ] Add performance tuning recommendations

## Quick Start

### Deploy LEADR

#### Using Docker (Recommended)

Pull the latest image from GitHub Container Registry:

```bash
docker pull ghcr.io/[your-github-username]/leadr:latest

# Run with required environment variables
docker run -d \
  -p 3000:3000 \
  -v leadr_data:/app/data \
  -e LEADR_API_KEY=your-secure-api-key \
  -e DATABASE_URL=sqlite:/app/data/leadr.db \
  ghcr.io/[your-github-username]/leadr:latest
```

#### Release Process

This project uses automated semantic versioning based on commit messages:

- **Breaking changes**: Commits starting with `BREAKING CHANGE:`, `breaking:`, or `major:` trigger a major version bump (1.0.0 → 2.0.0)
- **New features**: Commits starting with `feat:` or `feature:` trigger a minor version bump (1.0.0 → 1.1.0)
- **Bug fixes & other**: All other commits trigger a patch version bump (1.0.0 → 1.0.1)

To create a new release:
1. Go to Actions → Release and Publish
2. Click "Run workflow"
3. Select version bump type (or leave as "auto" for commit-based versioning)
4. The workflow will:
   - Determine the next version
   - Create a GitHub release with changelog
   - Build and push Docker images to GitHub Container Registry
   - Update the version in Cargo.toml

### Development

1. **Clone and setup**:
   ```bash
   git clone <repository>
   cd leadr
   cargo build
   ```

2. **Start development server**:
   ```bash
   # Start server with optimal development settings
   cargo dev
   
   # Or with hot reload for rapid development
   cargo watch -x dev
   ```

3. **Database management**:
   ```bash
   # Reset database (clears + runs all migrations)
   cargo db-reset
   
   # Seed database with CSV data (set LEADR_SEED_FILE env var)
   LEADR_SEED_FILE=your_data.csv cargo db-seed
   
   # Update SQLx offline query cache for compilation
   cargo db-prepare
   ```

4. **Run tests**:
   ```bash
   # Run all tests
   cargo test
   
   # Run tests sequentially (recommended for integration tests)
   cargo test-sequential
   
   # Run just integration tests
   cargo test-integration
   ```

5. **Code quality checks**:
   ```bash
   # Format code
   cargo fmt
   
   # Check for linting issues
   cargo clippy
   
   # Run pedantic linting (development)
   cargo clippy -- -W clippy::pedantic
   
   # Check formatting without making changes
   cargo fmt --check
   ```

6. **Docker development**:
   ```bash
   # Build production Docker image
   docker buildx build -t leadr-api --load .
   
   # Run with Docker (requires API key)
   docker run -p 3000:3000 \
     -e LEADR_API_KEY=your_secret_key \
     -e RUST_LOG=info \
     leadr-api
   
   # Run with persistent database volume
   docker run -p 3000:3000 \
     -v $(pwd)/data:/app/data \
     -e DATABASE_URL=sqlite:/app/data/leadr.db \
     -e LEADR_API_KEY=your_secret_key \
     leadr-api
   
   # Run with CSV seeding
   docker run -p 3000:3000 \
     -v /path/to/backup.csv:/data/seed.csv \
     -e LEADR_SEED_FILE=/data/seed.csv \
     -e LEADR_API_KEY=your_secret_key \
     leadr-api
   ```

7. **Test the health endpoint**:
   ```bash
   curl http://localhost:3000/health
   ```

## API Overview

### Game Management
```http
POST   /games              # Create a new leaderboard
GET    /games              # List all leaderboards (paginated)
GET    /games/{hex_id}     # Get specific leaderboard
PUT    /games/{hex_id}     # Update leaderboard
DELETE /games/{hex_id}     # Soft delete leaderboard
```

**Pagination Parameters for `/games`:**
- `limit` - Number of items per page (default: 25, max: 100)
- `cursor` - Cursor for pagination (base64 encoded)

### Score Submission & Leaderboards
```http
POST   /scores                   # Submit a new score (game_hex_id in JSON body)
GET    /scores                   # Get scores with optional game filtering (paginated & sortable)
GET    /scores/{id}              # Get a specific score
PUT    /scores/{id}              # Update a score
DELETE /scores/{id}              # Soft delete a score
```

### Data Export & Backup
```http
GET    /export                   # Download CSV backup of all game and score data
```

The `/export` endpoint generates a timestamped CSV file (e.g., `leadr_backup_20250730_105929.csv`) containing denormalized game-score data. Each row includes all game information plus score details, making it easy to restore or analyze your leaderboard data.

### Data Import & Seeding

LEADR automatically checks for a seed file on startup and imports data if the database is empty:

**Environment Variables:**
- `LEADR_SEED_FILE` - Path to CSV file for seeding (default: `/data/seed.csv`)

**Docker Usage:**
```bash
# Mount your backup CSV and set the seed file path
docker run -v /path/to/backup.csv:/data/seed.csv \
           -e LEADR_SEED_FILE=/data/seed.csv \
           -e LEADR_API_KEY=your_secret_key \
           your-leadr-image
```

**Seeding Behavior:**
- Only seeds if database is empty (no games exist)
- Preserves original timestamps from CSV
- Skips invalid/incomplete rows
- Logs import progress and results
- Non-blocking: server starts even if seeding fails

**Query Parameters for `/scores`:**
- `game_hex_id` - Filter scores for a specific game (optional - omit for global leaderboard)
- `limit` - Number of scores per page (default: 25, max: 100)
- `cursor` - Cursor for pagination (base64 encoded)
- `sort_by` - Sort field: `score` (default), `date`, `user_name`
- `order` - Sort order: `desc` (default), `asc`

**Examples:**
```http
# Get scores for a specific game
GET /scores?game_hex_id=abc123&sort_by=score&order=desc&limit=10
GET /scores?game_hex_id=abc123&sort_by=date&order=asc&cursor=eyJpZCI6MTIzfQ
GET /scores?game_hex_id=abc123&sort_by=user_name&order=asc&limit=50

# Get global leaderboard (all games)
GET /scores?sort_by=score&order=desc&limit=25
GET /scores?sort_by=date&order=asc
```

**Score Creation Request Body:**
```json
{
  "game_hex_id": "abc123",
  "score": "1000",
  "score_val": 1000.5,
  "user_name": "PlayerOne",
  "user_id": "player123",
  "extra": {"level": 5, "time": 120.5}
}
```

**Paginated Response Format:**
```json
{
  "data": [...],
  "has_more": true,
  "next_cursor": "eyJpZCI6NDU2LCJzb3J0X3ZhbHVlIjoiMjAwMC4wIn0",
  "current_cursor": "eyJpZCI6MTIzLCJzb3J0X3ZhbHVlIjoiMTAwMC41In0",
  "total_returned": 10,
  "page_size": 10
}
```

**Using Cursors for Navigation:**
- Use `next_cursor` from the response as the `cursor` parameter for the next page
- `current_cursor` represents the cursor that was used for the current page
- `has_more` indicates if there are additional pages available

## Configuration

Set these environment variables:

```bash
DATABASE_URL=sqlite:./leadr.db    # Database location
LEADR_API_KEY=your_secret_key     # API authentication (required)
LEADR_SEED_FILE=/data/seed.csv    # CSV file for database seeding (optional)
LEADR_PAGE_SIZE=25                # Default pagination page size (max: 100)
RUST_LOG=info                     # Logging level
```

## 🤝 Contributing

We welcome contributions! This project follows test-driven development:

1. Write tests first
2. Implement features to make tests pass
3. Refactor and optimize
4. Ensure all tests pass

## License

Open source and ready for your game development needs.

---

*Built with ❤️ for the indie game development community*
