# 🎮 LEADR - Lightweight Game Leaderboard API

> **A blazingly fast, single-tenant leaderboard API built for indie game developers**

Whether you're building a retro arcade game, puzzle platformer, or competitive multiplayer experience, LEADR handles your leaderboard needs without the bloat. Deploy your own instance, own your data, and integrate with any game engine in minutes.

## Features

- **⚡ Lightning Fast** - Built with Rust for maximum performance
- **🔒 Secure by Default** - Simple API key authentication
- **📦 Docker Ready** - Deploy to any cloud platform in minutes
- **💾 Zero Config** - SQLite database that just works
- **🎯 Multiple Leaderboards** - Different modes, levels, or difficulties
- **📊 Flexible Scoring** - Store custom metadata with every score

## Quick Start

Deploy our prebuilt & production-ready image to your preferred cloud host:

```plaintext
ghcr.io/barneyjackson/leadr:latest
```

Or try it out locally:

```bash
# Pull and run with Docker
docker run -d \
  -p 3000:3000 \
  -v leadr_data:/app/data \
  -e LEADR_API_KEY=your-secure-api-key \
  ghcr.io/barneyjackson/leadr:latest

# Test it's working
curl http://localhost:3000/health
```

**Required Environment Variables:**
- `LEADR_API_KEY` - Your API authentication key (required)

**Optional Configuration:**
- `DATABASE_URL` - Database location (default: `sqlite:/app/data/leadr.db`)
- `RUST_LOG` - Logging level (default: `info`)

## API Overview

All requests require the `leadr-api-key` header with your configured API key.

### API Documentation

- **Online Documentation**: [https://leadr-docs.vercel.app/](https://leadr-docs.vercel.app/) (Vercel)

The documentation is automatically generated from the code and updated on every push to main.

### Create a Game/Leaderboard

```bash
curl -X POST http://localhost:3000/games \
  -H "leadr-api-key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"name": "Space Invaders", "description": "Classic mode high scores"}'
```

**Game Fields:**
- `name` (required) - Game/leaderboard name (max 255 chars)
- `description` (optional) - Game description

**Response includes:**
- `hex_id` - 6-character unique identifier for the game
- `created_at`, `updated_at` - Timestamps

### Submit a Score

```bash
curl -X POST http://localhost:3000/scores \
  -H "leadr-api-key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "game_hex_id": "abc123",
    "score": "1000",
    "score_val": 1000,
    "user_name": "PlayerOne",
    "user_id": "player-123",
    "extra": {"level": 5, "time": 120.5}
  }'
```

**Score Fields:**
- `game_hex_id` (required) - The game's 6-character hex ID
- `score` (required) - Display score as string (e.g., "1,000 pts")
- `score_val` (optional) - Numeric value for sorting (defaults to parsing `score`)
- `user_name` (required) - Player display name (max 100 chars)
- `user_id` (required) - Unique player identifier (max 255 chars)
- `extra` (optional) - JSON object for custom metadata

### Get Leaderboard

```bash
# Top 10 scores for a specific game
curl "http://localhost:3000/scores?game_hex_id=abc123&limit=10" \
  -H "leadr-api-key: your-api-key"

# Global leaderboard across all games
curl "http://localhost:3000/scores?sort_by=score&order=desc" \
  -H "leadr-api-key: your-api-key"
```

## API Reference

### Game Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/games` | Create a new leaderboard |
| GET | `/games` | List all leaderboards (paginated) |
| GET | `/games/{hex_id}` | Get specific leaderboard |
| PUT | `/games/{hex_id}` | Update leaderboard |
| DELETE | `/games/{hex_id}` | Soft delete leaderboard |

### Score Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/scores` | Submit a new score |
| GET | `/scores` | Get scores (filterable, sortable, paginated) |
| GET | `/scores/{id}` | Get a specific score |
| PUT | `/scores/{id}` | Update a score |
| DELETE | `/scores/{id}` | Soft delete a score |

### Query Parameters for `/scores`

- `game_hex_id` - Filter by game (omit for global leaderboard)
- `sort_by` - Sort field: `score` (default), `date`, `user_name`
- `order` - Sort order: `desc` (default), `asc`
- `limit` - Results per page (default: 25, max: 100)
- `cursor` - Pagination cursor from previous response

### Pagination

All list endpoints return paginated responses:

```json
{
  "data": [...],
  "has_more": true,
  "next_cursor": "eyJpZCI6NDU2LCJzb3J0X3ZhbHVlIjoiMjAwMC4wIn0",
  "total_returned": 25,
  "page_size": 25
}
```

Use `next_cursor` as the `cursor` parameter for the next page.

## Backup & Restore

### Export Data

```bash
# Download complete backup as CSV
curl "http://localhost:3000/export" \
  -H "leadr-api-key: your-api-key" \
  -o leadr_backup_$(date +%Y%m%d).csv
```

### Import Data

Mount your backup CSV when starting the container:

```bash
docker run -d \
  -p 3000:3000 \
  -v /path/to/backup.csv:/data/seed.csv \
  -e LEADR_SEED_FILE=/data/seed.csv \
  -e LEADR_API_KEY=your-api-key \
  ghcr.io/barneyjackson/leadr:latest
```

Import only happens if the database is empty.

## Cloud Deployment

LEADR works with any cloud platform that supports Docker.

Remember to:
1. Set a strong `LEADR_API_KEY` environment variable
2. Set up regular backups using the `/export` endpoint

More docs coming soon.

---

## Developer Documentation

### Local Development

1. **Clone and setup**:
   ```bash
   git clone <repository>
   cd leadr
   cargo build
   ```

2. **Start development server**:
   ```bash
   # Set required environment variable
   export LEADR_API_KEY=dev-key-123
   
   # Run with hot reload
   cargo watch -x dev
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

### Docker Build

```bash
# Build image
docker buildx build -t leadr-api --load .

# Run locally
docker run -p 3000:3000 \
  -e LEADR_API_KEY=your_secret_key \
  leadr-api
```

### Database Management

```bash
# Reset database
cargo db-reset

# Generate SQLx offline data
cargo db-prepare

# Seed from CSV
LEADR_SEED_FILE=data.csv cargo db-seed
```

### Documentation Generation

```bash
# Generate OpenAPI spec only
cargo run --bin generate_openapi

# Generate complete static documentation site
./scripts/generate-docs.sh

# View locally (after generating docs)
cd docs && python3 -m http.server 8000
# Then open: http://localhost:8000
```

### Contributing

We follow test-driven development:

1. Write tests first
2. Implement features
3. Ensure all tests pass
4. Run `./scripts/ci.sh`
5. Make a PR

---

## Future Work

The following features are coming soon to leadr:

- **Hosted leadr service** - The quickest way to get started, just sign up and integrate
- **Game engine SDKs** - Simple integration tools for Unity, Unreal, Godot, and more
- **Customisable hosted leaderboard pages** - Easy online visibility for your game's scores
- **Automated cheat and spam detection** - One less thing for you to worry about
- **SSO login** - Easy login for you and your players with Google, Steam, Discord, and more
- Let us know what else you want!

---

*Built with ❤️ for the indie game development community*
