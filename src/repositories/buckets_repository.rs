use crate::models::bucket::{BucketUpdateModel, CreateBucket, ViewBucket};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_bucket(pool: &PgPool, create: CreateBucket) -> Result<ViewBucket, sqlx::Error> {
    sqlx::query_as!(
        ViewBucket,
        "INSERT INTO buckets (name) VALUES ($1) RETURNING id, name, created_at",
        create.name
    )
    .fetch_one(pool)
    .await
}

pub async fn get_bucket(pool: &PgPool, bucket_id: Uuid) -> Result<ViewBucket, sqlx::Error> {
    sqlx::query_as!(
        ViewBucket,
        "SELECT id, name, created_at FROM buckets WHERE id = $1",
        bucket_id
    )
    .fetch_one(pool)
    .await
}

pub async fn get_buckets(
    pool: &PgPool,
    bucket_ids: Vec<Uuid>,
) -> Result<Vec<ViewBucket>, sqlx::Error> {
    sqlx::query_as!(
        ViewBucket,
        "SELECT id, name, created_at FROM buckets WHERE id = ANY($1)",
        &bucket_ids[..] as &[Uuid]
    )
    .fetch_all(pool)
    .await
}

pub async fn update_bucket(
    pool: &PgPool,
    bucket_id: Uuid,
    bucket_update: BucketUpdateModel,
) -> Result<ViewBucket, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::new("UPDATE buckets SET ");
    let mut separated = qb.separated(", ");

    if let Some(name) = bucket_update.name {
        separated.push("name = ");
        separated.push_bind_unseparated(name);
    }

    qb.push(" WHERE id = ");
    qb.push_bind(bucket_id);
    qb.push(" RETURNING id, name, created_at");

    qb.build_query_as::<ViewBucket>().fetch_one(pool).await
}

pub async fn delete_bucket(pool: &PgPool, bucket_id: Uuid) -> Result<Uuid, sqlx::Error> {
    sqlx::query!("DELETE FROM buckets WHERE id = $1 RETURNING id", bucket_id)
        .fetch_one(pool)
        .await
        .map(|row| row.id)
}

pub async fn delete_buckets(
    pool: &PgPool,
    bucket_ids: &Vec<Uuid>,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM buckets WHERE id = ANY($1) RETURNING id",
        &bucket_ids[..] as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|row| row.id).collect())
}
