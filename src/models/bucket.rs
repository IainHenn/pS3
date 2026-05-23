use uuid::Uuid;

pub struct Bucket {
    pub id: Uuid;
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
