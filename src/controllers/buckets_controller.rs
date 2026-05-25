use axum::{extract::Query, extract::Json, response::IntoResponse};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{models::bucket::BucketUpdateModel, services::buckets_service as bucket, models::bucket::Bucket};
use axum::extract::{Path, State};

pub async fn get_bucket_by_id(State(pool): State<PgPool>, Path(bucket_id): Path<Uuid>) -> impl IntoResponse {
    return bucket::get_bucket_by_id(pool, bucket_id).await;
}

pub async fn get_buckets(State(pool): State<PgPool>, Query(bucket_ids): Query<String>) -> impl IntoResponse {
    let bucket_ids_parsed: Vec<&str> = bucket_ids.split(',').collect();
    let mut bucket_ids_uuid: Vec<Uuid> = Vec::new();

    for id in bucket_ids_parsed {
        bucket_ids_uuid.push(Uuid::parse_str(id).unwrap());
    }
    return bucket::get_buckets(pool, bucket_ids_uuid).await;
}

pub async fn create_bucket(State(pool): State<PgPool>, Json(bucket): Json<Bucket>) -> impl IntoResponse {
    return bucket::create_bucket(pool, bucket).await;
}

pub async fn update_bucket(State(pool): State<PgPool>, Path(bucket_id): Path<Uuid>, Json(bucket_update): Json<BucketUpdateModel>) -> impl IntoResponse {
    return bucket::update_bucket(pool, bucket_id, bucket_update).await;
}

pub async fn delete_bucket(State(pool): State<PgPool>, Path(bucket_id): Path<Uuid>) -> impl IntoResponse {
    return bucket::delete_bucket(pool, bucket_id).await;
}

pub async fn delete_buckets(State(pool): State<PgPool>, Query(bucket_ids): Query<String>) -> impl IntoResponse {
    let bucket_ids_parsed: Vec<&str> = bucket_ids.split(',').collect();
    let mut bucket_ids_uuid: Vec<Uuid> = Vec::new();

    for id in bucket_ids_parsed {
        bucket_ids_uuid.push(Uuid::parse_str(id).unwrap());
    }
    
    return bucket::delete_buckets(pool, bucket_ids_uuid).await;
}

