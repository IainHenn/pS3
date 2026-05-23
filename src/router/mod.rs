mod health;
use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

pub fn create_router(_db: PgPool) -> Router {
    Router::new()
    .route("/health", get(health::get_health))
}