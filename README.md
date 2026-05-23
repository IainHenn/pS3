# pS3

Personal S3 — Rust API backed by PostgreSQL.

## Prerequisites

- [Rust](https://rustup.rs/) (for local builds)
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose

## Configuration

Copy the example env file and adjust if needed:

```bash
cp .env.example .env
```

For local `cargo run`, use `POSTGRES_HOST=localhost`. Docker Compose sets `DATABASE_URL` for the API container automatically.

## Docker

Compose is defined in `config.yaml` (Postgres + API).

### Start everything (build API image + pull Postgres)

```bash
docker compose -f config.yaml up --build
```

Run in the background:

```bash
docker compose -f config.yaml up --build -d
```

### Postgres only (for local development)

```bash
docker compose -f config.yaml up postgres -d
```

Then run the API on the host with `cargo run`.

### Stop and remove containers

```bash
docker compose -f config.yaml down
```

Remove the database volume as well:

```bash
docker compose -f config.yaml down -v
```

### Health check

With the stack running:

```bash
curl http://localhost:3000/health
```

## Local build and run

```bash
cargo build          # debug
cargo build --release
cargo run            # requires Postgres (e.g. via Docker above)
```

## Database migrations

Migrations live in `src/migrations/` as timestamped `.sql` files (for example `20260523055746_create_buckets_table.sql`). They are applied with [sqlx-cli](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli), not automatically when the API starts.

### One-time setup

Install the CLI (Postgres only):

```bash
cargo install sqlx-cli --features postgres
```

Start Postgres if it is not already running:

```bash
docker compose -f config.yaml up postgres -d
```

Set `DATABASE_URL` so sqlx can connect. It must match your `.env` (local dev uses `localhost`):

```bash
# Git Bash / Linux / macOS
export DATABASE_URL="postgres://ps3:ps3@localhost:5432/ps3"
```

```powershell
# PowerShell
$env:DATABASE_URL = "postgres://ps3:ps3@localhost:5432/ps3"
```

### Apply migrations

Run all pending migrations against the database:

```bash
sqlx migrate run --source src/migrations
```

Check status:

```bash
sqlx migrate info --source src/migrations
```

### Create a new table (new migration)

Add a new migration file with a descriptive name:

```bash
sqlx migrate add create_objects_table --source src/migrations
```

This creates `src/migrations/<timestamp>_create_objects_table.sql`. Edit that file with your SQL, for example:

```sql
CREATE TABLE objects (
    id   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL
);
```

Then apply it:

```bash
sqlx migrate run --source src/migrations
```

**Tips**

- Run migrations after starting Postgres and before (or after) `cargo run`; the API does not run them for you yet.
- File names are ordered by timestamp — if one migration references another table, its timestamp must be later (e.g. `files` after `buckets`).
- To undo the most recent migration: `sqlx migrate revert --source src/migrations`

## Project layout

- `config.yaml` — Docker Compose (Postgres + `ps3` API)
- `Dockerfile` — multi-stage Rust build for the API image
- `src/migrations/` — SQL schema migrations (sqlx)
- `src/` — Axum application
