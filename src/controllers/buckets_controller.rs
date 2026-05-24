use axum::response::IntoResponse;
use sqlx::PgPool;
use uuid::Uuid;
use crate::services::buckets_service as bucket;
use axum::extract::{Path, State};

pub async fn get_bucket_by_id(State(pool): State<PgPool>, Path(bucket_id): Path<Uuid>) -> impl IntoResponse {
    return bucket::get_bucket_by_id(pool, bucket_id).await;
}