use axum::extract::{Json, Multipart, Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::file::{FileUpdateModel};
use crate::services::files_service as file;


#[derive(Deserialize)]
pub struct BucketFilePath {
    pub bucket_id: Uuid,
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
    multipart: Multipart,
) -> impl IntoResponse {
    file::create_file(pool, bucket_id, multipart).await
}


pub async fn update_file(
    State(pool): State<PgPool>,
    Path(path): Path<BucketFilePath>,
    Json(file_update): Json<FileUpdateModel>,
) -> impl IntoResponse {
    file::update_file(pool, path.bucket_id, path.file_id, file_update).await
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