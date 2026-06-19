use axum::body::Body;
use axum::extract::{Path, Query, Request, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use crate::lib::multipart;
use crate::services::files_service as file;


#[derive(Deserialize)]
pub struct BucketFilePath {
    pub bucket_id: Uuid,
    pub file_id: Uuid,
}

#[derive(Deserialize)]
pub struct BucketTransfer {
    pub old_bucket_id: Uuid,
    pub new_bucket_id: Uuid,
    pub file_id: Uuid,
}

#[derive(Deserialize)]
pub struct IdsQuery {
    file_ids: String,
}


pub async fn get_file_by_id(
    State(pool): State<PgPool>,
    Path(path): Path<BucketFilePath>,
) -> impl IntoResponse {
    file::get_file_by_id(pool, path.bucket_id, path.file_id).await
}

pub async fn get_files(
    State(pool): State<PgPool>,
    Path(bucket_id): Path<Uuid>,
    Query(query): Query<IdsQuery>,
) -> impl IntoResponse {
    let file_ids_uuid: Vec<Uuid> = query.file_ids
        .split(',')
        .map(|id| Uuid::parse_str(id).unwrap())
        .collect();

    file::get_files(pool, bucket_id, file_ids_uuid).await
}

pub async fn create_file(
    State(pool): State<PgPool>,
    Path(bucket_id): Path<Uuid>,
    request: Request<Body>,
) -> impl IntoResponse {
    let upload = match multipart::parse_file_upload_from_request(request).await {
        Ok(upload) => upload,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    file::create_file(pool, bucket_id, upload).await.into_response()
}

// This is for updating a file that you have in a bucket --> PUT
pub async fn update_file(
    State(pool): State<PgPool>,
    Path(path): Path<BucketFilePath>,
    request: Request<Body>,
) -> impl IntoResponse {
    let upload = match multipart::parse_file_upload_from_request(request).await {
        Ok(upload) => upload,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };

    file::update_file(pool, path.bucket_id, path.file_id, upload).await.into_response()
}

pub async fn move_file(
    State(pool): State<PgPool>,
    Path(path): Path<BucketTransfer>
) -> impl IntoResponse {

    file::move_file(pool, path.old_bucket_id, path.new_bucket_id, path.file_id).await
}
pub async fn delete_file(
    State(pool): State<PgPool>,
    Path(path): Path<BucketFilePath>,
) -> impl IntoResponse {
    file::delete_file(pool, path.bucket_id, path.file_id).await
}

pub async fn delete_files(
    State(pool): State<PgPool>,
    Path(bucket_id): Path<Uuid>,
    Query(query): Query<IdsQuery>,
) -> impl IntoResponse {
    let file_ids_uuid: Vec<Uuid> = query.file_ids
        .split(',')
        .map(|id| Uuid::parse_str(id).unwrap())
        .collect();

    file::delete_files(pool, bucket_id, file_ids_uuid).await
}
