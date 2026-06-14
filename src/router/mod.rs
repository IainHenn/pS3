use crate::controllers::health_controller;
use crate::controllers::files_controller;
use axum::routing::get;
use axum::routing::put;
use axum::routing::post;
use axum::routing::delete;
use axum::routing::patch;
use axum::Router;
use sqlx::PgPool;
use crate::controllers::buckets_controller;

/*
things to do:
    - update the GET /buckets route to have an optional query, providing no query shows all bucketss
        - same idea with GET /buckets/{bucket_id}/files
*/

pub fn create_router(db: PgPool) -> Router {
    Router::new()
        .route("/health", get(health_controller::get_health))
    // Bucket routes
        .route("/buckets/{id}", get(buckets_controller::get_bucket_by_id))
        .route("/buckets/{id}", put(buckets_controller::update_bucket))
        .route("/buckets/{id}", delete(buckets_controller::delete_bucket))
        .route("/buckets", get(buckets_controller::get_buckets))
        .route("/buckets", post(buckets_controller::create_bucket))
        .route("/buckets", delete(buckets_controller::delete_buckets))
    // File routes
       .route("/buckets/{bucket_id}/files/{file_id}", get(files_controller::get_file_by_id))
       .route("/buckets/{bucket_id}/files/{file_id}", patch(files_controller::update_file))
       .route("/buckets/{bucket_id}/files/{file_id}", delete(files_controller::delete_file))
       .route("/buckets/{bucket_id}/files", get(files_controller::get_files))
       .route("/buckets/{bucket_id}/files", post(files_controller::create_file))
       .route("/buckets/{bucket_id}/files", delete(files_controller::delete_files))      
       .route("/buckets/{old_bucket_id}/files/{file_id}/to/{new_bucket_id}",patch(files_controller::move_file))       
       .with_state(db)
}
