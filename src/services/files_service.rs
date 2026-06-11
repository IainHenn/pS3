use axum::body::Bytes;
use axum::extract::{Multipart};
use axum::response::IntoResponse;
use axum::Json;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;
use serde::Serialize;

use axum::response::Response;
use axum::http::{header, StatusCode};
use axum::body::Body;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use serde_json::json;

use crate::config::Config;
use crate::lib::file_actions;
use crate::models::file::{CreateFile, FileUpdateModel, ViewFile};
use crate::repositories::files_repository as file;

#[derive(Serialize)]
struct FileResult {
    id: String,
    status: u16,
    error: Option<String>,
}

pub async fn get_file_by_id(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let config = Config::from_env();
    let res: Result<ViewFile, sqlx::Error> = file::get_file(&pg_pool, bucket_id, file_id).await;
    match res {
        Ok(file) => {
            let disk_path = file_actions::file_path(&config.buckets_home_path, bucket_id, file.id);
            let map: HashMap<Uuid, String> = HashMap::from([
                (file_id, disk_path.clone())
            ]);

            let (_, not_found_files) = file_actions::read_files(&map).await;

            if not_found_files.len() > 0 {
                return (StatusCode::NOT_FOUND, Json(json!({
                    "message": "failed",
                    "error": "File not found",
                }))).into_response();
            }

            let physical_file = match File::open(&disk_path).await {
                Ok(f) => f,
                Err(_) => {
                    return (StatusCode::NOT_FOUND, Json(json!({
                        "message": "failed",
                        "error": "Failed to open file for download",
                    }))).into_response();
                }
            };

            let stream = ReaderStream::new(physical_file);
            let body = Body::from_stream(stream);

            return Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, &file.mime_type)
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", file.name),
            )
            .body(body)
            .unwrap()
            .into_response();

        },
        Err(sqlx::Error::RowNotFound) => {
            return (StatusCode::NOT_FOUND, Json(json!({
                "message": "failed",
                "error": "File not found",
            }))).into_response();
        }

        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn get_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    let config = Config::from_env();
    let res: Result<Vec<ViewFile>, sqlx::Error> =
        file::get_files(&pg_pool, bucket_id, &file_ids).await;
    let mut response_body: Vec<FileResult> = Vec::new(); 

    match res {
        Ok(files) => {
            let mut map: HashMap<Uuid, String> = HashMap::new();

            for file in &files {
                map.insert(
                    file.id,
                    file_actions::file_path(&config.buckets_home_path, bucket_id, file.id),
                );
            }

            let (found_files, mut not_found_files) = file_actions::read_files(&map).await;
            
            // For IDs not found in SQL query
            for original_file_id in &file_ids {
                let mut found = false;
                for file in &files {
                    if original_file_id.to_string() == file.id.to_string() {
                        found = true;
                    }
                }

                if found == false {
                    not_found_files.push(original_file_id.clone());
                }
            }
            
            for (file_id, _) in found_files {
                response_body.push(FileResult { id: file_id.to_string(), status: 200, error: None});
            }

            for file_id in not_found_files {
                response_body.push(FileResult { id: file_id.to_string(), status: 404, error: Some("File not found".to_string())});
            }

            return (StatusCode::MULTI_STATUS, Json(json!({
                "result": response_body,
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

    let file_id = Uuid::new_v4();
    let create: CreateFile = CreateFile {
        id: file_id,
        mime_type: content_type,
        path: file_actions::file_path(&config.buckets_home_path, bucket_id, file_id),
        name: file_name,
        size: size,
    };

    let mut tx  = match pg_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "failure",
            "error": "Failed to create file"
        }))).into_response();
        }
    };

    let res: Result<ViewFile, sqlx::Error> =
        file::create_file(&mut tx, bucket_id, create).await;

    match res {
        Ok(file) => {
            let map: HashMap<String, Bytes> = HashMap::from([(
                file_actions::file_path(&config.buckets_home_path, bucket_id, file.id),
                bytes,
            )]);

            let (_, failed_files): (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) =
                file_actions::create_or_update_files(&map).await;

            if failed_files.len() > 0 {
                let _ = tx.rollback().await;
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "message": "failure",
                    "error": "Failed to create file"
                }))).into_response();
            }

            if tx.commit().await.is_err() {
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "message": "failure",
                    "error": "Failed to create file"
                }))).into_response();
            }

            return (StatusCode::OK, Json(json!({
                "new_id": file.id.to_string(),
                "message": "success"
            }))).into_response();
        }
        Err(_) => {
            let _ = tx.rollback().await;
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "message": "failure",
                "error": "Failed to create file"
            }))).into_response();
        },
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
        name: Some(file_name),
        mime_type: Some(content_type),
        path: Some(file_actions::file_path(&config.buckets_home_path, bucket_id, file_id)),
        size: Some(file_size),
    };
    
    let res: Result<ViewFile, sqlx::Error> =
        file::update_file(&pg_pool, bucket_id, file_id, &file_update).await;

    match res {
        Ok(file) => {
            let map: HashMap<String, Bytes> = HashMap::from([(
                file_actions::file_path(&config.buckets_home_path, bucket_id, file.id),
                bytes,
            )]);

            let (_, failed_files): (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) = file_actions::create_or_update_files(&map).await;

            if failed_files.len() > 0 {
                return (StatusCode::OK, Json(json!({
                    "result": file.id.to_string(),
                    "message": "success"
                }))).into_response();
                // Need to call a rollback here for the database!
            } else {
                return (StatusCode::OK, Json(json!({
                    "result": file.id.to_string(),
                    "message": "success"
                }))).into_response();
            }
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn move_file(pg_pool: PgPool, old_bucket_id: Uuid, new_bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let config = Config::from_env();

    let res: Result<ViewFile, sqlx::Error> =
        file::move_file(&pg_pool, old_bucket_id, new_bucket_id, file_id).await;

    match res {
        Ok(file) => {
            let old_path = file_actions::file_path(&config.buckets_home_path, old_bucket_id, file.id);
            let new_path = file_actions::file_path(&config.buckets_home_path, new_bucket_id, file.id);
            let success = file_actions::move_file(&old_path, &new_path).await;
        
            if success {
                return (StatusCode::OK, Json(json!({
                    "result": file.id.to_string(),
                    "message": "success"
                }))).into_response();
            } else {
                return (StatusCode::OK, Json(json!({
                    "result": file.id.to_string(),
                    "message": "success",
                    "error": "Failed to move file to new bucket",
                }))).into_response();
                // Need to call a rollback here for the database!
            }
        },

        Err(_) => {
            return (StatusCode::OK, Json(json!({
                "error": "Failed to move file to new bucket! Database issue.",
                "message": "failed"
            }))).into_response();
        }
    }
}

pub async fn delete_file(pg_pool: PgPool, bucket_id: Uuid, file_id: Uuid) -> impl IntoResponse {
    let config = Config::from_env();
    let files_to_delete = HashMap::from([(
        file_id.to_string(),
        file_actions::file_path(&config.buckets_home_path, bucket_id, file_id),
    )]);

    let res: Result<Uuid, sqlx::Error> = file::delete_file(&pg_pool, bucket_id, file_id).await;

    match res {
        Ok(_) => {
            // Even if the deletion failed physically, we can untag in the db and it won't be searcheable
            // Wagering if it's worth returning, but leaving alone for now
            let (_, _) = file_actions::delete_files(files_to_delete).await;

            return (StatusCode::OK, Json(json!({
                "result": file_id,
                "message": "success"
            }))).into_response();
        },
        Err(sqlx::Error::RowNotFound) => {
            return (StatusCode::NOT_FOUND, Json(json!({
                "message": "failed",
                "error": "File not found!"
            }))).into_response();
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn delete_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    let config = Config::from_env();
    let mut files_to_delete = HashMap::new();
    let mut files_not_found: Vec<Uuid> = Vec::new();
    let mut response_body: Vec<FileResult> = Vec::new();

    let res: Result<Vec<Uuid>, sqlx::Error> =
        file::delete_files(&pg_pool, bucket_id, file_ids.clone()).await;

    match res {
        Ok(deleteable_files) => {
            for file_id in file_ids {
                if deleteable_files.contains(&file_id) == false {
                    files_not_found.push(file_id);
                }
            }

            for id in &deleteable_files {
                files_to_delete.insert(
                    id.to_string(),
                    file_actions::file_path(&config.buckets_home_path, bucket_id, *id),
                );
            }

            let (deleted_files, failed_deletes) = file_actions::delete_files(files_to_delete).await;
           
            for deleted_file in deleted_files {
                response_body.push(FileResult {id: deleted_file, status: 200, error: None});
            }

            for failed_file in failed_deletes {
                response_body.push(FileResult {id: failed_file, status: 200, error: Some("Failed to physically delete file, but file untagged in database metadata".to_string())})
            }

            for not_found_file in files_not_found {
                response_body.push(FileResult {id: not_found_file.to_string(), status: 404, error: Some("File not found".to_string())})
            }

            return (StatusCode::MULTI_STATUS, Json(json!({
                "result": response_body
            }))).into_response();
            
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
