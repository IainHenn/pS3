use axum::body::Bytes;
use axum::extract::{Multipart};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::config::Config;
use crate::lib::file_actions;
use crate::models::file::{CreateFile, FileUpdateModel, ViewFile};
use crate::repositories::files_repository as file;

pub async fn get_file_by_id(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let res: Result<ViewFile, sqlx::Error> = file::get_file(&pg_pool, bucket_id, file_id).await;

    match res {
        Ok(file) => (StatusCode::OK, Json(file)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    let res: Result<Vec<ViewFile>, sqlx::Error> =
        file::get_files(&pg_pool, bucket_id, file_ids).await;

    match res {
        Ok(files) => (StatusCode::OK, Json(files)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn create_file(
    pg_pool: PgPool,
    bucket_id: Uuid,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let config = Config::from_env();
    let mut file_name = String::new();
    let mut content_type = String::new();
    let size = 0;
    let mut bytes = Bytes::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        
        if let Some(name) = field.file_name(){
            file_name = name.to_string();
        }

        if let Some(mime) = field.content_type(){
            content_type = mime.to_string();
        }

        let Ok(file_size) = field.bytes().await else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        };

        bytes = file_size;
    };

    let create: CreateFile = CreateFile {
        mime_type: content_type,
        path: format!("{}/{}/{}", config.buckets_home_path, bucket_id, file_name),
        name: file_name,
        size: size,
    };

    let res: Result<ViewFile, sqlx::Error> = file::create_file(&pg_pool, bucket_id, create).await;

    match res {
        Ok(file) => {
            let map: HashMap<String, Bytes> = HashMap::from([
                (format!("{}/{}", bucket_id, file.id.to_string()), bytes)
            ]);
            
            let (_, failed_files): (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) = file_actions::create_or_update_files(&map).await;
            
            if failed_files.len() > 0 {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
                // Need to call a rollback here for the database!
            } else {
                return (StatusCode::OK, Json(file)).into_response();
            }
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn update_file(
    pg_pool: PgPool,
    bucket_id: Uuid,
    file_id: Uuid,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let config = Config::from_env();
    let mut file_name = String::new();
    let mut content_type = String::new();
    let mut bytes = Bytes::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        
        if let Some(name) = field.file_name(){
            file_name = name.to_string();
        }

        if let Some(mime) = field.content_type(){
            content_type = mime.to_string();
        }

        let Ok(file_size) = field.bytes().await else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
        };

        bytes = file_size;
    };

    let file_size = bytes.len() as i64;

    let file_update: FileUpdateModel = FileUpdateModel {
        bucket_id: Some(bucket_id.to_string()),
        name: Some(file_name),
        mime_type: Some(content_type),
        path: Some(format!("{}/{}/{}", config.buckets_home_path, bucket_id, file_name)),
        size: Some(file_size),
    };

    let new_size = file_update.size;
    let new_mime_type = file_update.mime_type.clone();

    let res: Result<ViewFile, sqlx::Error> =
        file::update_file(&pg_pool, bucket_id, file_id, &file_update).await;

    match res {
        Ok(file) => {

            // This just means the file itself changed 
            if new_mime_type != Some(file.mime_type.clone()) || 
            new_size != Some(file.size.clone())
            {
               let map: HashMap<String, Bytes> = HashMap::from([
                    (format!("{}/{}", bucket_id, file.id.to_string()), bytes)
                ]);

                let (_, failed_files): (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) = file_actions::create_or_update_files(&map).await;

                if failed_files.len() > 0 {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    // Need to call a rollback here for the database!
                } else {
                    return (StatusCode::OK, Json(file)).into_response();
                }
            }

            // This means the bucket changed, and in this case we move the file
            else if bucket_id != file.bucket_id {
                file_actions::move_file(&format!("{}/{}", file.bucket_id, file.id.to_string()), &format!("{}/{}", bucket_id, file.id.to_string()));
            }
            return (StatusCode::OK, Json(file)).into_response();
        },
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
