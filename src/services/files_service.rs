use axum::body::Bytes;
use axum::extract::{Multipart};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use serde_json::json;

use crate::config::Config;
use crate::lib::file_actions;
use crate::models::file::{CreateFile, FileUpdateModel, ViewFile};
use crate::repositories::files_repository as file;

pub async fn get_file_by_id(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let res: Result<ViewFile, sqlx::Error> = file::get_file(&pg_pool, bucket_id, file_id).await;

    match res {
        Ok(file) => {
            let map: HashMap<Uuid, String> = HashMap::from([
                (file_id, format!("{}/{}", bucket_id, file.id.to_string()))
            ]);

            let (found_files, not_found_files) = file_actions::read_files(&map).await;

            return (StatusCode::OK, Json(json!({
                "found_files": found_files.keys().map(|k| (*k).to_string()).collect::<Vec<String>>(),
                "not_found_files": not_found_files,
                "message": "success"
            }))).into_response();

        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    let res: Result<Vec<ViewFile>, sqlx::Error> =
        file::get_files(&pg_pool, bucket_id, file_ids).await;

    match res {
        Ok(files) => {
            let mut map: HashMap<Uuid, String> = HashMap::new();

            for file in files {
                map.insert(file.id, format!("{}/{}", bucket_id, file.id.to_string()));
            }

            let (found_files, not_found_files) = file_actions::read_files(&map).await;

            return (StatusCode::OK, Json(json!({
                "found_files": found_files.keys().map(|k| (*k).to_string()).collect::<Vec<String>>(),
                "not_found_files": not_found_files,
                "message": "success"
            }))).into_response();
        },
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
            
            let (new_files, failed_files): (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) = file_actions::create_or_update_files(&map).await;
            
            if failed_files.len() > 0 {
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
                // Need to call a rollback here for the database!
            } else {
                return (StatusCode::OK, Json(json!({
                    "new_files": new_files.keys().map(|k| (*k).clone()).collect::<Vec<String>>(),
                    "failed_files": failed_files.keys().map(|k| (*k).clone()).collect::<Vec<String>>(),
                    "message": "success"
                }))).into_response();
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
    let file_name_unborrowed = file_name.clone();

    let file_update: FileUpdateModel = FileUpdateModel {
        bucket_id: Some(bucket_id.to_string()),
        name: Some(file_name),
        mime_type: Some(content_type),
        path: Some(format!("{}/{}/{}", config.buckets_home_path, bucket_id, file_name_unborrowed)),
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

                let (updated_files, failed_files): (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) = file_actions::create_or_update_files(&map).await;

                if failed_files.len() > 0 {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                    // Need to call a rollback here for the database!
                } else {
                    return (StatusCode::OK, Json(json!({
                        "updated_files": updated_files.keys().map(|k| (*k).clone()).collect::<Vec<String>>(),
                        "not_found_files": failed_files.keys().map(|k| (*k).clone()).collect::<Vec<String>>(),
                        "message": "success"
                    }))).into_response();
                }
            } else if bucket_id != file.bucket_id {
                let success = file_actions::move_file(&format!("{}/{}", file.bucket_id, file.id.to_string()), &format!("{}/{}", bucket_id, file.id.to_string())).await;

                if success {
                    return (StatusCode::OK, Json(json!({
                        "updated_files": vec![file.id.to_string()],
                        "not_found_files": Vec::<String>::new(),
                        "message": "success"
                    }))).into_response();
                } else {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }

            (StatusCode::OK, Json(file)).into_response()
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_file(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let config = Config::from_env();
    let files_to_delete = HashMap::from([(
        file_id.to_string(),
        format!("{}/{}/{}", config.buckets_home_path, bucket_id, file_id),
    )]);

    let res: Result<Uuid, sqlx::Error> = file::delete_file(&pg_pool, bucket_id, file_id).await;

    match res {
        Ok(_) => {
            let (deleted_files, failed_deletes) = file_actions::delete_files(files_to_delete).await;

            return (StatusCode::OK, Json(json!({
                "deleted_files": deleted_files,
                "not_deleted_files": failed_deletes,
                "message": "success"
            }))).into_response();
        }

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    let config = Config::from_env();
    let mut files_to_delete = HashMap::new();

    for id in &file_ids {
        files_to_delete.insert(
            id.to_string(),
            format!("{}/{}/{}", config.buckets_home_path, bucket_id, id),
        );
    }

    let res: Result<Vec<Uuid>, sqlx::Error> =
        file::delete_files(&pg_pool, bucket_id, file_ids).await;

    match res {
        Ok(_) => {
            let (deleted_files, failed_deletes) = file_actions::delete_files(files_to_delete).await;
           
            return (StatusCode::OK, Json(json!({
                "deleted_files": deleted_files,
                "not_deleted_files": failed_deletes,
                "message": "success"
            }))).into_response();
            
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
