use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CreateFile {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ViewFile {
    pub id: Uuid,
    pub bucket_id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileUpdateModel {
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub path: Option<String>,
}