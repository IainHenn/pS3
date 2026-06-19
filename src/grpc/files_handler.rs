use axum::response::IntoResponse;
use sqlx::PgPool;
use tonic::{Request, Response, Status};
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::grpc::ps3::files_server::Files;
use crate::grpc::ps3::{FileCreateReply, FileCreateRequest, FileReply, FilesMetadata, FilesReply};
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
        let bucket_id = Uuid::parse_str(&req.bucket_id)
            .map_err(|_| Status::invalid_argument("invalid bucket_id"))?;

        let upload = FileUpload::new(
            req.name,
            req.mime_type,
            req.content.into(),
        );

        let response = files_service::create_file(self.pool.clone(), bucket_id, upload)
            .await
            .into_response();
        let (parts, body) = response.into_parts();
        let body_bytes = axum::body::to_bytes(body, usize::MAX)
            .await
            .map_err(|_| Status::internal("failed to read response body"))?;
        let body_json: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|_| Status::internal("failed to parse response body"))?;

        if !parts.status.is_success() {
            return Ok(Response::new(FileCreateReply {
                new_id: String::new(),
                message: body_json["message"].as_str().unwrap_or("failure").to_string(),
                error: body_json["error"].as_str().unwrap_or("failed to create file").to_string(),
            }));
        }

        Ok(Response::new(FileCreateReply {
            new_id: body_json["new_id"].as_str().unwrap_or_default().to_string(),
            message: body_json["message"].as_str().unwrap_or("success").to_string(),
            error: String::new(),
        }))
    }

    async fn get_file_by_id(
        &self,
        request: Request<crate::grpc::ps3::FileRequest>,
    ) -> Result<Response<Self::GetFileByIdStream>, Status> {
        let req = request.into_inner();
        let file_id: Uuid = Uuid::parse_str(&req.file_id).map_err(|_| Status::invalid_argument("Failed to parse provided file id"))?;
        let bucket_id: Uuid = Uuid::parse_str(&req.bucket_id).map_err(|_| Status::invalid_argument("Failed to parse provided bucket id"))?;

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
        } else {
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            tx.send(Ok(FileReply { content: body_bytes.to_vec() })).await.ok();
            Ok(Response::new(ReceiverStream::new(rx)))
        }
    }

    async fn get_files(
        &self,
        request: Request<crate::grpc::ps3::FilesRequest>,
    ) -> Result<Response<crate::grpc::ps3::FilesReply>, Status> {
        let req = request.into_inner();
        let bucket_id = Uuid::parse_str(&req.bucket_id)
            .map_err(|_| Status::invalid_argument("Failed to parse provided bucket id"))?;

        let file_ids: Vec<Uuid> = if req.file_ids.trim().is_empty() {
            Vec::new()
        } else {
            req.file_ids
                .split(',')
                .map(|id| {
                    Uuid::parse_str(id.trim())
                        .map_err(|_| Status::invalid_argument("Failed to parse provided file id"))
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let response = files_service::fetch_files(self.pool.clone(), bucket_id, file_ids)
            .await
            .map_err(|_| Status::internal("failed to fetch files"))?;

        Ok(Response::new(FilesReply {
            files: response
                .result
                .into_iter()
                .map(|file: files_service::FileFetchResult| FilesMetadata {
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
        _request: Request<crate::grpc::ps3::FileDeleteRequest>,
    ) -> Result<Response<crate::grpc::ps3::FileDeleteReply>, Status> {
        Err(Status::unimplemented("delete_file not yet implemented"))
    }

    async fn delete_files(
        &self,
        _request: Request<crate::grpc::ps3::FilesDeleteRequest>,
    ) -> Result<Response<crate::grpc::ps3::FilesDeleteReply>, Status> {
        Err(Status::unimplemented("delete_files not yet implemented"))
    }

    async fn update_file(
        &self,
        _request: Request<crate::grpc::ps3::FilesUpdateRequest>,
    ) -> Result<Response<crate::grpc::ps3::FileReply>, Status> {
        Err(Status::unimplemented("update_file not yet implemented"))
    }

    async fn move_file(
        &self,
        _request: Request<crate::grpc::ps3::FileMovementRequest>,
    ) -> Result<Response<crate::grpc::ps3::FileMovementReply>, Status> {
        Err(Status::unimplemented("move_file not yet implemented"))
    }
}
