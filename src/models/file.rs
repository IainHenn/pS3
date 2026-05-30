use uuid::Uuid;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: Uuid,
    pub bucket_id: String,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileUpdateModel {
    pub bucket_id: Option<String>,
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub path: Option<String>,
}