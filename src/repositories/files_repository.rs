use sqlx::{PgPool};
use uuid::Uuid;

use crate::models::file::{CreateFile, FileUpdateModel, ViewFile};

pub async fn create_file(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bucket_id: Uuid,
    create: CreateFile,
) -> Result<ViewFile, sqlx::Error> {
    sqlx::query_as!(
        ViewFile,
        r#"
        INSERT INTO files (id, bucket_id, name, mime_type, size, path)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, bucket_id, name, mime_type, size, path, created_at, updated_at
        "#,
        create.id,
        bucket_id,
        create.name,
        create.mime_type,
        create.size,
        create.path,
    )
    .fetch_one(&mut **tx)
    .await
}

pub async fn get_file(
    pool: &PgPool,
    bucket_id: Uuid,
    file_id: Uuid,
) -> Result<ViewFile, sqlx::Error> {
    sqlx::query_as!(
        ViewFile,
        r#"
        SELECT id, bucket_id, name, mime_type, size, path, created_at, updated_at
        FROM files
        WHERE id = $1 AND bucket_id = $2
        "#,
        file_id,
        bucket_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn get_files(
    pool: &PgPool,
    bucket_id: Uuid,
    file_ids: &Vec<Uuid>,
) -> Result<Vec<ViewFile>, sqlx::Error> {
    sqlx::query_as!(
        ViewFile,
        r#"
        SELECT id, bucket_id, name, mime_type, size, path, created_at, updated_at
        FROM files
        WHERE id = ANY($1) AND bucket_id = $2
        "#,
        &file_ids[..] as &[Uuid],
        bucket_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_file(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    bucket_id: Uuid,
    file_id: Uuid,
    file_update: &FileUpdateModel,
) -> Result<ViewFile, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::new("UPDATE files SET ");
    let mut separated = qb.separated(", ");

    if let Some(name) = &file_update.name {
        separated.push("name = ");
        separated.push_bind_unseparated(name);
    }
    if let Some(mime_type) = &file_update.mime_type {
        separated.push("mime_type = ");
        separated.push_bind_unseparated(mime_type);
    }
    if let Some(size) = file_update.size {
        separated.push("size = ");
        separated.push_bind_unseparated(size);
    }
    if let Some(path) = &file_update.path {
        separated.push("path = ");
        separated.push_bind_unseparated(path);
    }

    qb.push(" WHERE id = ");
    qb.push_bind(file_id);
    qb.push(" AND bucket_id = ");
    qb.push_bind(bucket_id);
    qb.push(" RETURNING id, bucket_id, name, mime_type, size, path, created_at, updated_at");

    qb.build_query_as::<ViewFile>().fetch_one(&mut **tx).await
}

pub async fn move_file(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    old_bucket_id: Uuid,
    new_bucket_id: Uuid,
    file_id: Uuid
) -> Result<ViewFile, sqlx::Error> {
    let mut qb = sqlx::QueryBuilder::new("UPDATE files SET ");
    let mut separated = qb.separated(", ");

    separated.push("bucket_id = ");
    separated.push_bind_unseparated(new_bucket_id);

    qb.push(" WHERE id = ");
    qb.push_bind(file_id);
    qb.push(" AND bucket_id = ");
    qb.push_bind(old_bucket_id);
    qb.push(" RETURNING id, bucket_id, name, mime_type, size, path, created_at, updated_at");

    qb.build_query_as::<ViewFile>().fetch_one(&mut **tx).await
}


pub async fn delete_file(
    pool: &PgPool,
    bucket_id: Uuid,
    file_id: Uuid,
) -> Result<Uuid, sqlx::Error> {
    sqlx::query!(
        "DELETE FROM files WHERE id = $1 AND bucket_id = $2 RETURNING id",
        file_id,
        bucket_id,
    )
    .fetch_one(pool)
    .await
    .map(|row| row.id)
}

pub async fn delete_files(
    pool: &PgPool,
    bucket_id: Uuid,
    file_ids: Vec<Uuid>,
) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query!(
        r#"DELETE FROM files WHERE id = ANY($1) AND bucket_id = $2 RETURNING id"#,
        &file_ids[..] as &[Uuid],
        bucket_id,
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|row| row.id).collect())
}
