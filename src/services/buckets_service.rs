use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse};
use uuid::Uuid;
use crate::models::bucket::{Bucket, BucketUpdateModel};
use crate::repositories::buckets_repository as bucket;
use sqlx::PgPool;

pub async fn get_bucket_by_id(pg_pool: PgPool, bucket_id: Uuid) -> impl IntoResponse {
    
    let res: Result<Bucket, sqlx::Error> = bucket::get_bucket(&pg_pool, bucket_id).await;
    
    match res {
        Ok(bucket) => (StatusCode::OK, Json(bucket)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_buckets(pg_pool: PgPool, bucket_ids: Vec<Uuid>) -> impl IntoResponse {
    let res: Result<Vec<Bucket>, sqlx::Error> = bucket::get_buckets(&pg_pool, bucket_ids).await;

    match res {
        Ok(buckets) => (StatusCode::OK, Json(buckets)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn create_bucket(pg_pool: PgPool, bucket: Bucket) -> impl IntoResponse {
    let res: Result<Bucket, sqlx::Error> = bucket::create_bucket(&pg_pool, bucket).await;

    match res {
        Ok(bucket) => (StatusCode::OK, Json(bucket)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn update_bucket(pg_pool: PgPool, bucket_id: Uuid, bucket_update: BucketUpdateModel) -> impl IntoResponse {
    let res: Result<Bucket, sqlx::Error> = bucket::update_bucket(&pg_pool, bucket_id, bucket_update).await;

    match res {
        Ok(bucket) => (StatusCode::OK, Json(bucket)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_bucket(pg_pool: PgPool, bucket_id: Uuid) -> impl IntoResponse {
    let res: Result<Uuid, sqlx::Error> = bucket::delete_bucket(&pg_pool, bucket_id).await;

    match res {
        Ok(bucket_id) => (StatusCode::OK, Json(bucket_id)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_buckets(pg_pool: PgPool, bucket_ids: Vec<Uuid>) -> impl IntoResponse {
    let res: Result<Vec<Uuid>, sqlx::Error> = bucket::delete_buckets(&pg_pool, bucket_ids).await;

    match res {
        Ok(bucket_ids) => (StatusCode::OK, Json(bucket_ids)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}