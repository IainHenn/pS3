use sqlx::PgPool;
use tonic::{Request, Response, Status};

use crate::grpc::helpers::{json_str, parse_uuid, parse_uuid_list, service_json_response};
use crate::grpc::ps3::buckets_server::Buckets;
use crate::grpc::ps3::{
    BucketCreateRequest, BucketDeleteReply, BucketDeleteRequest, BucketDeletesRequest,
    BucketReply, BucketRequest, BucketUpdateRequest, BucketsDeleteMetadata, BucketsDeleteReply,
    BucketsReply, BucketsRequest, FileDeleteRequest,
};
use crate::models::bucket::{BucketUpdateModel, CreateBucket};
use crate::services::buckets_service;

pub struct GrpcBucketsService {
    pub pool: PgPool,
}

fn bucket_reply_from_json(value: &serde_json::Value, message: &str, error: &str) -> BucketReply {
    BucketReply {
        bucket_id: json_str(value, "id"),
        bucket_name: json_str(value, "name"),
        created_at: json_str(value, "created_at"),
        message: message.to_string(),
        error: error.to_string(),
    }
}

#[tonic::async_trait]
impl Buckets for GrpcBucketsService {
    async fn get_bucket(
        &self,
        request: Request<BucketRequest>,
    ) -> Result<Response<BucketReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;

        let (status, body) =
            service_json_response(buckets_service::get_bucket_by_id(self.pool.clone(), bucket_id))
                .await?;

        if status.is_success() {
            return Ok(Response::new(bucket_reply_from_json(
                &body,
                "success",
                "",
            )));
        }

        Ok(Response::new(BucketReply {
            bucket_id: req.bucket_id,
            bucket_name: String::new(),
            created_at: String::new(),
            message: json_str(&body, "message"),
            error: json_str(&body, "error"),
        }))
    }

    async fn get_buckets(
        &self,
        request: Request<BucketsRequest>,
    ) -> Result<Response<BucketsReply>, Status> {
        let req = request.into_inner();
        let bucket_ids = parse_uuid_list(&req.bucket_ids, "bucket_id")?;

        let (status, body) =
            service_json_response(buckets_service::get_buckets(self.pool.clone(), bucket_ids))
                .await?;

        if !status.is_success() {
            return Err(Status::internal("failed to fetch buckets"));
        }

        let buckets = body
            .as_array()
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|entry| bucket_reply_from_json(&entry, "success", ""))
            .collect();

        Ok(Response::new(BucketsReply {
            buckets,
            message: "success".to_string(),
            error: String::new(),
        }))
    }

    async fn create_bucket(
        &self,
        request: Request<BucketCreateRequest>,
    ) -> Result<Response<BucketReply>, Status> {
        let req = request.into_inner();
        let create = CreateBucket {
            name: req.bucket_name,
        };

        let (status, body) =
            service_json_response(buckets_service::create_bucket(self.pool.clone(), create)).await?;

        if status.is_success() {
            return Ok(Response::new(bucket_reply_from_json(
                &body,
                "success",
                "",
            )));
        }

        Err(Status::internal("failed to create bucket"))
    }

    async fn update_bucket(
        &self,
        request: Request<BucketUpdateRequest>,
    ) -> Result<Response<BucketReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;
        let update = BucketUpdateModel {
            name: Some(req.bucket_name),
        };

        let (status, body) = service_json_response(buckets_service::update_bucket(
            self.pool.clone(),
            bucket_id,
            update,
        ))
        .await?;

        if status.is_success() {
            return Ok(Response::new(bucket_reply_from_json(
                &body,
                "success",
                "",
            )));
        }

        Ok(Response::new(BucketReply {
            bucket_id: req.bucket_id,
            bucket_name: String::new(),
            created_at: String::new(),
            message: json_str(&body, "message"),
            error: json_str(&body, "error"),
        }))
    }

    async fn delete_bucket(
        &self,
        request: Request<BucketDeleteRequest>,
    ) -> Result<Response<BucketDeleteReply>, Status> {
        let req = request.into_inner();
        let bucket_id = parse_uuid(&req.bucket_id, "bucket_id")?;

        let (status, body) =
            service_json_response(buckets_service::delete_bucket(self.pool.clone(), bucket_id))
                .await?;

        if status.is_success() {
            let deleted_files = body
                .get("deleted_files")
                .and_then(|value| value.as_array())
                .map(|entries| {
                    entries
                        .iter()
                        .filter_map(|entry| entry.as_str())
                        .map(|file_id| FileDeleteRequest {
                            bucket_id: String::new(),
                            file_id: file_id.to_string(),
                        })
                        .collect()
                })
                .unwrap_or_default();

            return Ok(Response::new(BucketDeleteReply {
                deleted_bucket: json_str(&body, "deleted_bucket"),
                files_deleted: deleted_files,
                error: String::new(),
            }));
        }

        Ok(Response::new(BucketDeleteReply {
            deleted_bucket: String::new(),
            files_deleted: Vec::new(),
            error: json_str(&body, "error"),
        }))
    }

    async fn delete_buckets(
        &self,
        request: Request<BucketDeletesRequest>,
    ) -> Result<Response<BucketsDeleteReply>, Status> {
        let req = request.into_inner();
        let bucket_ids = parse_uuid_list(&req.bucket_ids, "bucket_id")?;

        let (status, body) =
            service_json_response(buckets_service::delete_buckets(self.pool.clone(), bucket_ids))
                .await?;

        if !status.is_success() {
            return Err(Status::internal("failed to delete buckets"));
        }

        let buckets = body
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

                BucketsDeleteMetadata {
                    bucket_id: json_str(&entry, "id"),
                    deleted_files: entry
                        .get("deleted_files")
                        .and_then(|value| value.as_array())
                        .map(|files| {
                            files
                                .iter()
                                .filter_map(|file| file.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default(),
                    message,
                    error: error.unwrap_or_default(),
                }
            })
            .collect();

        Ok(Response::new(BucketsDeleteReply {
            buckets,
            message: "success".to_string(),
            error: String::new(),
        }))
    }
}
