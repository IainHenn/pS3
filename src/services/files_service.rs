use axum::body::Bytes;
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
use crate::lib::multipart::FileUpload;
use crate::models::file::{CreateFile, FileUpdateModel, ViewFile};
use crate::repositories::files_repository as file;

#[derive(Serialize, Clone)]
pub struct FileFetchResult {
    pub file_id: String,
    pub bucket_id: Option<String>,
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub path: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub message: String,
    pub error: Option<String>,
    pub status: u16,
}

#[derive(Serialize)]
pub struct GetFilesResponse {
    pub result: Vec<FileFetchResult>,
    pub message: String,
    pub error: Option<String>,
}

impl FileFetchResult {
    fn found(file: ViewFile) -> Self {
        Self {
            file_id: file.id.to_string(),
            bucket_id: Some(file.bucket_id.to_string()),
            name: Some(file.name),
            mime_type: Some(file.mime_type),
            size: Some(file.size),
            path: Some(file.path),
            created_at: Some(file.created_at.to_rfc3339()),
            updated_at: Some(file.updated_at.to_rfc3339()),
            message: "success".to_string(),
            error: None,
            status: 200,
        }
    }

    fn not_found(file_id: Uuid, error: &str) -> Self {
        Self {
            file_id: file_id.to_string(),
            bucket_id: None,
            name: None,
            mime_type: None,
            size: None,
            path: None,
            created_at: None,
            updated_at: None,
            message: "failed".to_string(),
            error: Some(error.to_string()),
            status: 404,
        }
    }
}

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

pub async fn fetch_files(
    pg_pool: PgPool,
    bucket_id: Uuid,
    file_ids: Vec<Uuid>,
) -> Result<GetFilesResponse, sqlx::Error> {
    let config = Config::from_env();
    let files = file::get_files(&pg_pool, bucket_id, &file_ids).await?;

    let mut files_by_id: HashMap<Uuid, ViewFile> = HashMap::new();
    let mut map: HashMap<Uuid, String> = HashMap::new();

    for file in files {
        map.insert(
            file.id,
            file_actions::file_path(&config.buckets_home_path, bucket_id, file.id),
        );
        files_by_id.insert(file.id, file);
    }

    let (found_on_disk, _) = file_actions::read_files(&map).await;
    let mut results: Vec<FileFetchResult> = Vec::with_capacity(file_ids.len());

    for file_id in file_ids {
        match files_by_id.get(&file_id) {
            Some(file) if found_on_disk.contains_key(&file_id) => {
                results.push(FileFetchResult::found(file.clone()));
            }
            _ => {
                results.push(FileFetchResult::not_found(file_id, "File not found"));
            }
        }
    }

    Ok(GetFilesResponse {
        result: results,
        message: "success".to_string(),
        error: None,
    })
}

pub async fn get_files(pg_pool: PgPool, bucket_id: Uuid, file_ids: Vec<Uuid>) -> impl IntoResponse {
    match fetch_files(pg_pool, bucket_id, file_ids).await {
        Ok(response) => (StatusCode::MULTI_STATUS, Json(response)).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn create_file(
    pg_pool: PgPool,
    bucket_id: Uuid,
    upload: FileUpload,
) -> impl IntoResponse {
    let config = Config::from_env();

    let file_id = Uuid::new_v4();
    let file_size = upload.size();
    let create: CreateFile = CreateFile {
        id: file_id,
        mime_type: upload.mime_type,
        path: file_actions::file_path(&config.buckets_home_path, bucket_id, file_id),
        name: upload.name,
        size: file_size,
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
                upload.bytes,
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
    upload: FileUpload,
) -> impl IntoResponse {
    let config = Config::from_env();

    let file_size = upload.size();
    let file_update: FileUpdateModel = FileUpdateModel {
        name: Some(upload.name),
        mime_type: Some(upload.mime_type),
        path: Some(file_actions::file_path(&config.buckets_home_path, bucket_id, file_id)),
        size: Some(file_size),
    };

    let mut tx  = match pg_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "failure",
            "error": "Failed to update file"
        }))).into_response();
        }
    };
    
    let res: Result<ViewFile, sqlx::Error> =
        file::update_file(&mut tx, bucket_id, file_id, &file_update).await;

    match res {
        Ok(file) => {
            let map: HashMap<String, Bytes> = HashMap::from([(
                file_actions::file_path(&config.buckets_home_path, bucket_id, file.id),
                upload.bytes,
            )]);

            let (_, failed_files): (HashMap<&String, &Bytes>, HashMap<&String, &Bytes>) = file_actions::create_or_update_files(&map).await;

            if failed_files.len() > 0 {

                let _ = tx.rollback().await;

                return (StatusCode::OK, Json(json!({
                    "result": file.id.to_string(),
                    "message": "success"
                }))).into_response();                
            } else {
                if tx.commit().await.is_err() {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "message": "failure",
                        "error": "Failed to update file"
                    }))).into_response();
                }

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

    let mut tx  = match pg_pool.begin().await {
        Ok(tx) => tx,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
            "message": "failure",
            "error": "Failed to move file"
        }))).into_response();
        }
    };

    let res: Result<ViewFile, sqlx::Error> =
        file::move_file(&mut tx, old_bucket_id, new_bucket_id, file_id).await;

    match res {
        Ok(file) => {
            let old_path = file_actions::file_path(&config.buckets_home_path, old_bucket_id, file.id);
            let new_path = file_actions::file_path(&config.buckets_home_path, new_bucket_id, file.id);
            let success = file_actions::move_file(&old_path, &new_path).await;
        
            if success {

                if tx.commit().await.is_err() {
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                        "message": "failure",
                        "error": "Failed to move file"
                    }))).into_response();
                }

                return (StatusCode::OK, Json(json!({
                    "result": file.id.to_string(),
                    "message": "success"
                }))).into_response();
            } else {

                let _ = tx.rollback().await;

                return (StatusCode::OK, Json(json!({
                    "result": file.id.to_string(),
                    "message": "success",
                    "error": "Failed to move file to new bucket",
                }))).into_response();
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
