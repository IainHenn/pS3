use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;
use crate::lib::file_actions;
use crate::config::Config;
use serde::Serialize;


use crate::models::bucket::{BucketUpdateModel, CreateBucket, ViewBucket};
use crate::repositories::buckets_repository as bucket;

pub async fn get_bucket_by_id(pg_pool: PgPool, bucket_id: Uuid) -> impl IntoResponse {
    let res: Result<ViewBucket, sqlx::Error> = bucket::get_bucket(&pg_pool, bucket_id).await;

    match res {
        Ok(bucket) => (StatusCode::OK, Json(bucket)).into_response(),
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, Json(json!({
            "message": "failed",
            "error": "Bucket not found",
        }))).into_response(),
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
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, Json(json!({
            "message": "failed",
            "error": "Bucket not found",
        }))).into_response(),
        Err(e) => (StatusCode::NOT_FOUND, Json(json!({
            "message": "failed",
            "error": e.to_string(),
        }))).into_response(),
    }
}

pub async fn delete_bucket(pg_pool: PgPool, bucket_id: Uuid) -> impl IntoResponse {
    let config = Config::from_env();
    let res: Result<Uuid, sqlx::Error> = bucket::delete_bucket(&pg_pool, bucket_id).await;

    match res {
        Ok(bucket_id) => {
            
            let deleted_files = file_actions::delete_files_in_bucket(&config.buckets_home_path, bucket_id).await;
            return (StatusCode::OK, Json(json!({
                "deleted_bucket": bucket_id,
                "deleted_files": deleted_files,
                "message": "success",
            }))).into_response();
        },
        Err(sqlx::Error::RowNotFound) => (StatusCode::NOT_FOUND, Json(json!({
            "message": "failed",
            "error": "Bucket not found",
        }))).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[derive(Serialize)]
struct BucketDeleteResult {
    id: String,
    deleted_files: Vec<String>,
    status: u16,
    error: Option<String>,
}

pub async fn delete_buckets(pg_pool: PgPool, bucket_ids: Vec<Uuid>) -> impl IntoResponse {
    let config = Config::from_env();
    let res: Result<Vec<Uuid>, sqlx::Error> = bucket::delete_buckets(&pg_pool, &bucket_ids).await;
    let mut response_body: Vec<BucketDeleteResult> = Vec::new();
    match res {
        Ok(queried_bucket_ids) => {

            // For IDs not found in SQL query
            for original_bucket_id in &bucket_ids {
                let mut found = false;
                for bucket_id in &queried_bucket_ids {
                    if original_bucket_id.to_string() == bucket_id.to_string() {
                        found = true;
                    }
                }

                if found == false {
                    response_body.push(BucketDeleteResult { id: (original_bucket_id.to_string()), deleted_files: Vec::new(), status: 404, error: Some("Bucket not found".to_string())});
                }
            }

            for queried_bucket_id in &queried_bucket_ids {
                response_body.push(BucketDeleteResult { id: (queried_bucket_id.to_string()), 
                    deleted_files: file_actions::delete_files_in_bucket(&config.buckets_home_path, *queried_bucket_id).await, 
                    status: 200, 
                    error: None});
            }

            return (StatusCode::MULTI_STATUS, Json(json!({
                "result": response_body,
            }))).into_response();
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
