use crate::controllers::health_controller;
use crate::controllers::buckets_controller;
use axum::routing::get;
use axum::routing::put;
use axum::routing::post;
use axum::routing::delete;
use axum::Router;
use sqlx::PgPool;

pub fn create_router(db: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_controller::get_health))
    // Bucket routes (uncomment when buckets_controller handlers exist)
        .route("/buckets/:id", get(buckets_controller::get_bucket_by_id))
        .route("/buckets/:id", put(buckets_controller::update_bucket))
        .route("/buckets/:id", delete(buckets_controller::delete_bucket))
        .route("/buckets", get(buckets_controller::get_buckets))
        .route("/buckets", post(buckets_controller::create_bucket))
        .route("/buckets", delete(buckets_controller::delete_buckets))
        .with_state(db)
   
}
