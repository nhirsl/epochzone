# Epoch Zone

REST API providing timezone information and conversion, built with Rust. Powers [epoch.zone](https://epoch.zone).

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `ADMIN_API_KEY` | Yes | - | Admin key for managing API keys (min 32 chars) |
| `DATABASE_URL` | No | `epochzone.db` | SQLite database path |
| `CORS_ALLOWED_ORIGINS` | No | `http://localhost:5173,...` | Comma-separated allowed origins |

## Build & Run

```bash
# Build
cargo build --release

# Run tests
cargo test

# Run the server (listens on port 3000)
ADMIN_API_KEY="your-admin-key-at-least-32-characters" cargo run
```

Or with a `.env` file:

```bash
cp .env.example .env  # then edit with your values
cargo run
```

## API Endpoints

All `/api/*` endpoints require an `X-API-Key` header.

| Method | Endpoint | Description |
|---|---|---|
| `GET` | `/health` | Health check |
| `GET` | `/api/timezones` | List all timezones |
| `GET` | `/api/time/{timezone}` | Get current time info for a timezone |
| `POST` | `/api/convert` | Convert time between timezones |

### Convert Examples

**By timestamp:**
```json
{ "timestamp": 1707580800, "to": "America/New_York" }
```

**By datetime + source timezone:**
```json
{ "datetime": "2026-02-10T15:30:00", "from": "Europe/Belgrade", "to": "America/New_York" }
```

### Admin Endpoints

Require `X-API-Key` header matching `ADMIN_API_KEY`.

| Method | Endpoint | Description |
|---|---|---|
| `POST` | `/admin/api-keys` | Create an API key |
| `GET` | `/admin/api-keys` | List API keys |
| `DELETE` | `/admin/api-keys/{id}` | Revoke an API key |

## Deploy

Deployed on [Railway](https://railway.app). Set the environment variables in your Railway service settings, and Railway will build and run the binary automatically.
