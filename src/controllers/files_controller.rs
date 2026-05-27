use axum::{extract::Query, extract::Json, response::IntoResponse};
use sqlx::PgPool;
use uuid::Uuid;
use crate::{models::file::FileUpdateModel, services::files_service as file, models::file::File};
use axum::extract::{Path, State};

pub async fn get_file_by_id(State(pool): State<PgPool>, Path(file_id): Path<Uuid>) -> impl IntoResponse {
    return file::get_file_by_id(pool, file_id).await;
}

pub async fn get_files(State(pool): State<PgPool>, Query(file_ids): Query<String>) -> impl IntoResponse {
    let file_ids_parsed: Vec<&str> = file_ids.split(',').collect();
    let mut file_ids_uuid: Vec<Uuid> = Vec::new();

    for id in file_ids_parsed {
        file_ids_uuid.push(Uuid::parse_str(id).unwrap());
    }
    return file::get_files(pool, file_ids_uuid).await;
}

pub async fn create_file(State(pool): State<PgPool>, Json(file): Json<File>) -> impl IntoResponse {
    return file::create_file(pool, file).await;
}

pub async fn update_file(State(pool): State<PgPool>, Path(file_id): Path<Uuid>, Json(file_update): Json<FileUpdateModel>) -> impl IntoResponse {
    return file::update_file(pool, file_id, file_update).await;
}

pub async fn delete_file(State(pool): State<PgPool>, Path(file_id): Path<Uuid>) -> impl IntoResponse {
    return file::delete_file(pool, file_id).await;
}

pub async fn delete_files(State(pool): State<PgPool>, Query(file_ids): Query<String>) -> impl IntoResponse {
    let file_ids_parsed: Vec<&str> = file_ids.split(',').collect();
    let mut file_ids_uuid: Vec<Uuid> = Vec::new();

    for id in file_ids_parsed {
        file_ids_uuid.push(Uuid::parse_str(id).unwrap());
    }
    
    return file::delete_files(pool, file_ids_uuid).await;
}

