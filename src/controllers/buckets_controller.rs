use axum::extract::{Json, Path, Query, State};
use axum::response::IntoResponse;
use sqlx::{PgPool};
use uuid::Uuid;
use serde::Deserialize;


use crate::models::bucket::{BucketUpdateModel, CreateBucket};
use crate::services::buckets_service as bucket;

#[derive(Deserialize)]
pub struct IdsQuery {
    bucket_ids: String,
}


pub async fn get_bucket_by_id(
    State(pool): State<PgPool>,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    bucket::get_bucket_by_id(pool, bucket_id).await
}

pub async fn get_buckets(
    State(pool): State<PgPool>,
    Query(query): Query<IdsQuery>,
) -> impl IntoResponse {
    let bucket_ids_uuid: Vec<Uuid> = query.bucket_ids
        .split(',')
        .map(|id| Uuid::parse_str(id).unwrap())
        .collect();

    bucket::get_buckets(pool, bucket_ids_uuid).await
}

pub async fn create_bucket(
    State(pool): State<PgPool>,
    Json(create): Json<CreateBucket>,
) -> impl IntoResponse {
    bucket::create_bucket(pool, create).await
}

pub async fn update_bucket(
    State(pool): State<PgPool>,
    Path(bucket_id): Path<Uuid>,
    Json(bucket_update): Json<BucketUpdateModel>,
) -> impl IntoResponse {
    bucket::update_bucket(pool, bucket_id, bucket_update).await
}

pub async fn delete_bucket(
    State(pool): State<PgPool>,
    Path(bucket_id): Path<Uuid>,
) -> impl IntoResponse {
    bucket::delete_bucket(pool, bucket_id).await
}

pub async fn delete_buckets(
    State(pool): State<PgPool>,
    Query(query): Query<IdsQuery>,
) -> impl IntoResponse {
    let bucket_ids_uuid: Vec<Uuid> = query.bucket_ids
        .split(',')
        .map(|id| Uuid::parse_str(id).unwrap())
        .collect();

    bucket::delete_buckets(pool, bucket_ids_uuid).await
}
