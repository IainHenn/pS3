use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub buckets_home_path: String,
    pub grpc_port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "ps3".to_string());
                let password =
                    env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "ps3".to_string());
                let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
                let port = env::var("POSTGRES_PORT").unwrap_or_else(|_| "5432".to_string());
                let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "ps3".to_string());
                format!("postgres://{user}:{password}@{host}:{port}/{db}")
            }
        };

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("API_PORT")
            .or_else(|_| env::var("PORT"))
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("API_PORT must be a valid number");

        let buckets_home_path = env::var("BUCKETS_HOME_PATH").unwrap_or_else(|_| "/c/Home".to_string());

        let grpc_port = env::var("GRPC_PORT")
            .unwrap_or_else(|_| "50051".to_string())
            .parse()
            .expect("GRPC_PORT must be a valid number");

        Self {
            database_url,
            host,
            port,
            buckets_home_path,
            grpc_port,
        }
    }
}
