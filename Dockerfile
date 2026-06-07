FROM rust:1-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
COPY .sqlx ./.sqlx
COPY src ./src
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pS3 /usr/local/bin/ps3

EXPOSE 3000
CMD ["ps3"]
