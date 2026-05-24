use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use crate::models::bucket::Bucket;
use crate::repositories::buckets_repository as bucket;
use sqlx::PgPool;

pub async fn get_bucket_by_id(pg_pool: PgPool, bucket_id: Uuid) -> impl IntoResponse {
    
    let res: Result<Bucket, sqlx::Error> = bucket::get_bucket(&pg_pool, bucket_id).await;
    
    match res {
        Ok(bucket) => (StatusCode::OK, Json(bucket)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }

}