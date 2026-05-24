use uuid::Uuid;

pub struct File {
    pub id: Uuid,
    pub bucket_id: String,
    pub name: String,
    pub mime_type: String,
    pub size: i32,
    pub path: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>
}

pub struct FileUpdateModel {
    pub bucket_id: Option<String>,
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i32>,
    pub path: Option<String>,
}