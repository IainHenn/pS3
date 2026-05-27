use crate::controllers::health_controller;
use crate::controllers::files_controller;
use axum::routing::get;
use axum::routing::put;
use axum::routing::post;
use axum::routing::delete;
use axum::Router;
use sqlx::PgPool;

pub fn create_router(db: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_controller::get_health))
    // Bucket routes
        .route("/buckets/:id", get(buckets_controller::get_bucket_by_id))
        .route("/buckets/:id", put(buckets_controller::update_bucket))
        .route("/buckets/:id", delete(buckets_controller::delete_bucket))
        .route("/buckets", get(buckets_controller::get_buckets))
        .route("/buckets", post(buckets_controller::create_bucket))
        .route("/buckets", delete(buckets_controller::delete_buckets))
    // File routes
       .route("/files/:id", get(files_controller::get_file_by_id))
       .route("/files/:id", put(files_controller::update_file))
       .route("/files/:id", delete(files_controller::delete_file))
       .route("/files", get(files_controller::get_files))
       .route("/files", post(files_controller::create_file))
       .route("/files", delete(files_controller::delete_files))
       .with_state(db)
}
