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
- [x] ~~Set up Rust project with Axum web framework~~
- [x] ~~Configure SQLite database with migrations~~
- [x] ~~Implement error handling and logging~~
- [x] ~~Add hot reload development setup~~
- [x] ~~**Design and implement Game model with hex ID and soft delete**~~
- [x] ~~**Design and implement Score model with relationships and JSON extras**~~
- [x] ~~**Create database migrations for games and scores tables**~~
- [x] ~~**Add API key authentication middleware**~~
- [ ] Create Docker configuration files (Dockerfile, docker-compose, .dockerignore)
- [ ] No rate limiting - vulnerable to abuse
- [ ] Add a "restore"/seed mechanism, that can take an optional CSV mounted and image build time and populate the SQLite DB. Unless you think there's a better way to achieve the same thing

### API Endpoints
- [x] ~~**Implement CRUD endpoints for games (Create, Read, Update)**~~
- [x] ~~**Implement CRUD endpoints for scores (Create, Read, Update)**~~
- [x] ~~**Restructure endpoints to use top-level /scores with query parameters**~~
- [x] ~~**Add global leaderboard functionality (scores across all games)**~~
- [x] Expose DELETE endpoints in router (handlers exist but routes are missing)
- [ ] Add /download endpoint that exports the full SQLite DB as a CSV as a means of taking a backup of data, as the DB will on'y exist in the Docker container and won't be truly persisted

### Testing & Quality
- [x] ~~Set up comprehensive test suite with TDD approach~~
- [x] ~~Add integration tests for all endpoints~~
- [ ] **Code Quality & Documentation Improvements**
  - [x] ~~Add comprehensive documentation with `# Errors` and `# Panics` sections for all public functions~~
  - [x] ~~Implement `#[must_use]` attributes on constructor and validation methods for better API safety~~
  - [x] ~~Replace redundant closures with direct function references for better performance~~
  - [x] ~~Modernize string formatting to use inline variable syntax (`format!("text: {var}")`)~~
  - [x] ~~Fix lossless type casting to use `From::from()` instead of `as` for type safety~~
  - [x] ~~Resolve code organization issues (import ordering, item placement, raw string usage)~~
  - [x] ~~**Add missing `# Errors` documentation to 20+ handler functions (TOP PRIORITY - 20 warnings)**~~
  - [x] ~~**Add proper backticks around code terms in documentation (13 warnings)**~~
  - [x] ~~**Add `#[must_use]` attributes to utility methods (12 warnings)**~~
  - [x] ~~**Complete string formatting modernization (9 remaining instances)**~~
  - [x] ~~Fix double `#[must_use]` attributes on Result-returning functions (4 warnings)~~
  - [ ] Merge duplicate error handling match arms to reduce code duplication
  - [ ] Add missing `# Panics` documentation for functions that may panic
  - [x] ~~Replace manual clamp logic with `.clamp()` method~~
  - [ ] Optimize string building in hex ID generation
- [ ] Add performance benchmarks
- [ ] Set up CI/CD pipeline with automated quality checks

### Deployment
- [ ] **Create multi-stage Dockerfile with security best practices**
- [ ] Add Docker Compose for local development
- [ ] Create deployment guides for major cloud providers
- [ ] Add environment configuration documentation

### Documentation
- [ ] Add API documentation with examples
- [ ] Create integration guides for popular game engines
- [ ] Add performance tuning recommendations

## Quick Start

### Deploy LEADR

*Coming soon - Docker deployment guide*

### Development

1. **Clone and setup**:
   ```bash
   git clone <repository>
   cd leadr
   cargo build
   ```

2. **Run with hot reload**:
   ```bash
   cargo watch -x run
   ```

3. **Run tests**:
   ```bash
   # Run all tests
   cargo test
   
   # Run tests sequentially (recommended for integration tests)
   cargo test-sequential
   
   # Run just integration tests
   cargo test-integration
   ```

4. **Code quality checks**:
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

5. **Test the health endpoint**:
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
LEADR_API_KEY=your_secret_key     # API authentication
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
