mod config;
mod controllers;
mod grpc;
mod lib;
mod models;
mod repositories;
mod router;
mod services;

use std::net::SocketAddr;

use sqlx::PgPool;
use tokio::net::TcpListener;
use tonic::transport::Server;

use crate::config::Config;
use crate::grpc::buckets_handler::GrpcBucketsService;
use crate::grpc::files_handler::GrpcFilesService;
use crate::grpc::health_handler::GrpcHealthService;
use crate::grpc::ps3::buckets_server::BucketsServer;
use crate::grpc::ps3::files_server::FilesServer;
use crate::grpc::ps3::health_server::HealthServer;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let db_pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to the database!");

    let grpc_addr: SocketAddr = format!("{}:{}", config.host, config.grpc_port)
        .parse()
        .expect("GRPC_PORT must be a valid address");

    let grpc_server = Server::builder()
        .add_service(FilesServer::new(GrpcFilesService {
            pool: db_pool.clone(),
        }))
        .add_service(BucketsServer::new(GrpcBucketsService {
            pool: db_pool.clone(),
        }))
        .add_service(HealthServer::new(GrpcHealthService))
        .serve(grpc_addr);

    let app = router::create_router(db_pool);

    let http_addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&http_addr).await.unwrap();
    println!("listening on http {http_addr}");
    println!("listening on grpc {}:{}", config.host, config.grpc_port);

    tokio::select! {
        result = axum::serve(listener, app) => result.unwrap(),
        result = grpc_server => result.unwrap(),
    }
}
