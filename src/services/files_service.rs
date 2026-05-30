use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse};
use uuid::Uuid;
use crate::models::file::{File, FileUpdateModel};
use crate::repositories::files_repository as file;
use sqlx::PgPool;

pub async fn get_file_by_id(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let res: Result<File, sqlx::Error> = file::get_file(&pg_pool, bucket_id, file_id).await;

    match res {
        Ok(file) => (StatusCode::OK, Json(file)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    let res: Result<Vec<File>, sqlx::Error> = file::get_files(&pg_pool, bucket_id, file_ids).await;

    match res {
        Ok(files) => (StatusCode::OK, Json(files)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn create_file(pg_pool: PgPool, bucket_id: Uuid, file: File) -> impl IntoResponse {
    let res: Result<File, sqlx::Error> = file::create_file(&pg_pool, bucket_id, file).await;

    match res {
        Ok(file) => (StatusCode::OK, Json(file)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn update_file(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid, file_update: FileUpdateModel) -> impl IntoResponse {
    let res: Result<File, sqlx::Error> = file::update_file(&pg_pool, bucket_id, file_id, file_update).await;

    match res {
        Ok(file) => (StatusCode::OK, Json(file)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_file(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let res: Result<Uuid, sqlx::Error> = file::delete_file(&pg_pool, bucket_id, file_id).await;

    match res {
        Ok(file_id) => (StatusCode::OK, Json(file_id)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    let res: Result<Vec<Uuid>, sqlx::Error> =
        file::delete_files(&pg_pool, bucket_id, file_ids).await;

    match res {
        Ok(file_ids) => (StatusCode::OK, Json(file_ids)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}