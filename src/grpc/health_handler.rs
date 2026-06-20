use tonic::{Request, Response, Status};

use crate::grpc::ps3::health_server::Health;
use crate::grpc::ps3::{HealthReply, HealthyRequest};

pub struct GrpcHealthService;

#[tonic::async_trait]
impl Health for GrpcHealthService {
    async fn get_health(
        &self,
        _request: Request<HealthyRequest>,
    ) -> Result<Response<HealthReply>, Status> {
        Ok(Response::new(HealthReply {
            message: "Success!".to_string(),
            error: String::new(),
        }))
    }
}
