use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::bucket::{BucketUpdateModel, CreateBucket, ViewBucket};
use crate::repositories::buckets_repository as bucket;

pub async fn get_bucket_by_id(pg_pool: PgPool, bucket_id: Uuid) -> impl IntoResponse {
    let res: Result<ViewBucket, sqlx::Error> = bucket::get_bucket(&pg_pool, bucket_id).await;

    match res {
        Ok(bucket) => (StatusCode::OK, Json(bucket)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_buckets(pg_pool: PgPool, bucket_ids: Vec<Uuid>) -> impl IntoResponse {
    let res: Result<Vec<ViewBucket>, sqlx::Error> = bucket::get_buckets(&pg_pool, bucket_ids).await;

    match res {
        Ok(buckets) => (StatusCode::OK, Json(buckets)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn create_bucket(pg_pool: PgPool, create: CreateBucket) -> impl IntoResponse {
    let res: Result<ViewBucket, sqlx::Error> = bucket::create_bucket(&pg_pool, create).await;

    match res {
        Ok(bucket) => (StatusCode::OK, Json(bucket)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn update_bucket(
    pg_pool: PgPool,
    bucket_id: Uuid,
    bucket_update: BucketUpdateModel,
) -> impl IntoResponse {
    let res: Result<ViewBucket, sqlx::Error> =
        bucket::update_bucket(&pg_pool, bucket_id, bucket_update).await;

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
