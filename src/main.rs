mod config;
mod controllers;
mod lib;
mod models;
mod repositories;
mod router;
mod services;

use axum::Router;
use sqlx::PgPool;
use tokio::net::TcpListener;

use crate::config::Config;


#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let db_pool = PgPool::connect(&config.database_url)
        .await
        .expect("Failed to connect to the database!");

    let app: Router = router::create_router(db_pool);

    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    println!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}