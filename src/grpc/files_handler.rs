use axum::response::IntoResponse;
use sqlx::PgPool;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;

use crate::grpc::helpers::{json_str, parse_uuid, parse_uuid_list, service_json_response};
use crate::grpc::ps3::files_server::Files;
use crate::grpc::ps3::{
    FileCreateReply, FileCreateRequest, FileDeleteReply, FileDeleteRequest, FileMovementReply,
    FileMovementRequest, FileReply, FileRequest, FilesDeleteMetadata, FilesDeleteReply,
    FilesDeleteRequest, FilesMetadata, FilesReply, FilesRequest, FilesUpdateRequest,
};
use crate::lib::multipart::FileUpload;
use crate::services::files_service;

pub struct GrpcFilesService {
    pub pool: PgPool,
}

#[tonic::async_trait]
impl Files for GrpcFilesService {
    type GetFileByIdStream = ReceiverStream<Result<FileReply, Status>>;

    async fn create_file(
        &self,
        request: Request<FileCreateRequest>,
    ) -> Result<Response<FileCreateReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;
        let upload = FileUpload::new(req.name, req.mime_type, req.content.into());

        let (status, body) = service_json_response(files_service::create_file(
            self.pool.clone(),
            bucket_id,
            upload,
        ))
        .await?;

        if !status.is_success() {
            return Ok(Response::new(FileCreateReply {
                new_id: String::new(),
                message: json_str(&body, "message"),
                error: json_str(&body, "error"),
            }));
        }

        Ok(Response::new(FileCreateReply {
            new_id: json_str(&body, "new_id"),
            message: json_str(&body, "message"),
            error: String::new(),
        }))
    }

    async fn get_file_by_id(
        &self,
        request: Request<FileRequest>,
    ) -> Result<Response<Self::GetFileByIdStream>, Status> {
        let req = request.into_inner();
        let file_id = parse_uuid(&req.file_id, "file_id")?;
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;

        let response = files_service::get_file_by_id(self.pool.clone(), bucket_id, file_id)
            .await
            .into_response();
        let (parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|_| Status::internal("failed to read response body"))?;

        if !parts.status.is_success() {
            let body_json: serde_json::Value = serde_json::from_slice(&body_bytes)
                .map_err(|_| Status::internal("failed to parse response body"))?;

            return Err(Status::not_found(
                body_json["error"]
                    .as_str()
                    .unwrap_or("file not found"),
            ));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        tx.send(Ok(FileReply {
            content: body_bytes.to_vec(),
        }))
        .await
        .ok();

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn get_files(
        &self,
        request: Request<FilesRequest>,
    ) -> Result<Response<FilesReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;
        let file_ids = parse_uuid_list(&req.file_ids, "file_id")?;

        let response = files_service::fetch_files(self.pool.clone(), bucket_id, file_ids)
            .await
            .map_err(|_| Status::internal("failed to fetch files"))?;

        Ok(Response::new(FilesReply {
            files: response
                .result
                .into_iter()
                .map(|file| FilesMetadata {
                    file_id: file.file_id,
                    message: file.message,
                    error: file.error.unwrap_or_default(),
                    bucket_id: file.bucket_id.unwrap_or_default(),
                    name: file.name.unwrap_or_default(),
                    mime_type: file.mime_type.unwrap_or_default(),
                    size: file.size.unwrap_or_default(),
                    path: file.path.unwrap_or_default(),
                    created_at: file.created_at.unwrap_or_default(),
                    updated_at: file.updated_at.unwrap_or_default(),
                })
                .collect(),
            message: response.message,
            error: response.error.unwrap_or_default(),
        }))
    }

    async fn delete_file(
        &self,
        request: Request<FileDeleteRequest>,
    ) -> Result<Response<FileDeleteReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;
        let file_id = parse_uuid(&req.file_id, "file_id")?;

        let (status, body) = service_json_response(files_service::delete_file(
            self.pool.clone(),
            bucket_id,
            file_id,
        ))
        .await?;

        if status.is_success() {
            return Ok(Response::new(FileDeleteReply {
                file_id: json_str(&body, "result"),
                message: json_str(&body, "message"),
                error: String::new(),
            }));
        }

        Ok(Response::new(FileDeleteReply {
            file_id: req.file_id,
            message: json_str(&body, "message"),
            error: json_str(&body, "error"),
        }))
    }

    async fn delete_files(
        &self,
        request: Request<FilesDeleteRequest>,
    ) -> Result<Response<FilesDeleteReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;
        let file_ids = parse_uuid_list(&req.file_ids, "file_id")?;

        let (status, body) = service_json_response(files_service::delete_files(
            self.pool.clone(),
            bucket_id,
            file_ids,
        ))
        .await?;

        if !status.is_success() {
            return Err(Status::internal("failed to delete files"));
        }

        let files = body
            .get("result")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|entry| {
                let error = entry
                    .get("error")
                    .and_then(|value| value.as_str())
                    .map(str::to_string);
                let message = if error.is_some() {
                    "failed".to_string()
                } else {
                    "success".to_string()
                };

                FilesDeleteMetadata {
                    file_id: json_str(&entry, "id"),
                    message,
                    error: error.unwrap_or_default(),
                }
            })
            .collect();

        Ok(Response::new(FilesDeleteReply { files }))
    }

    async fn update_file(
        &self,
        request: Request<FilesUpdateRequest>,
    ) -> Result<Response<FileReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;
        let file_id = parse_uuid(&req.file_id, "file_id")?;
        let content = req.content;
        let upload = FileUpload::new(req.name, req.mime_type, content.into());
        let saved_content = upload.bytes.clone();

        let (status, body) = service_json_response(files_service::update_file(
            self.pool.clone(),
            bucket_id,
            file_id,
            upload,
        ))
        .await?;

        if !status.is_success() {
            return Err(Status::internal(json_str(&body, "error")));
        }

        Ok(Response::new(FileReply {
            content: saved_content.to_vec(),
        }))
    }

    async fn move_file(
        &self,
        request: Request<FileMovementRequest>,
    ) -> Result<Response<FileMovementReply>, Status> {
        let req = request.into_inner();
        let old_bucket_id = parse_uuid(&req.old_bucket_id, "old_bucket_id")?;
        let new_bucket_id = parse_uuid(&req.new_bucket_id, "new_bucket_id")?;
        let file_id = parse_uuid(&req.file_id, "file_id")?;

        let (_status, body) = service_json_response(files_service::move_file(
            self.pool.clone(),
            old_bucket_id,
            new_bucket_id,
            file_id,
        ))
        .await?;

        Ok(Response::new(FileMovementReply {
            result: json_str(&body, "result"),
            message: json_str(&body, "message"),
            error: json_str(&body, "error"),
        }))
    }
}
