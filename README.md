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

## Project layout

- `config.yaml` — Docker Compose (Postgres + `ps3` API)
- `Dockerfile` — multi-stage Rust build for the API image
- `src/` — Axum application
