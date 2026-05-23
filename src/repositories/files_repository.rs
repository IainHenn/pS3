use crate::models::file::File;
use crate::models::file::FileUpdateModel;

pub fn create_file(pool: &PgPool, file: File) -> Result<File, sqlx::Error> {
    sqlx::query_as!(File, "INSERT INTO public.files VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *")
    .bind(file.bucket_id)
    .bind(file.name)
    .bind(file.mime_type)
    .bind(file.size)
    .bind(file.path)
    .bind(file.created_at)
    .bind(file.updated_at)
    .fetch_one(pool)
    .await;
}

pub fn get_file(file_id: Uuid) -> Result<File, sqlx::Error> {
    sqlx::query_as!(File, "SELECT * FROM public.files WHERE id = $1")
    .bind(bucket_id)
    .fetch_one(pool)
    .await;
}

pub fn get_files(pool: PgPool, file_ids: Vec<Uuid>) -> Result<Vec<File>,sqlx::Error> {
    sqlx::query_as!(File, 
        r#"SELECT * FROM public.files where id = ANY($1)"#,
        &bucket_ids[..] as &[Uuid])
    .fetch_all(pool)
    .await;
}

pub fn update_file(pool: PgPool, file_id: Uuid, file_update: FileUpdateModel) -> Result<File, sqlx::Error> {
    let mut qb = QueryBuilder::new("UPDATE public.files SET ");
    let mut separated = qb.separated(", ");

    if let Some(name) = file_update.name {
        qb.push("name = ");
        qb.push_bind_unseparated(name);
    }
    if let Some(bucket_id) = file_update.bucket_id {
        qb.push("bucket_id = ");
        qb.push_bind_unseparated(bucket_id);
    }
    if let Some(mime_type) = file_update.mime_type {
        qb.push("mime_type = ");
        qb.push_bind_unseparated(mime_type);
    }
    if let Some(size) = file_update.size {
        qb.push("size = ");
        qb.push_bind_unseparated(size);
    }
    if let Some(path) = file_update.path {
        qb.push("path = ");
        qb.push_bind_unseparated(path);
    }

    qb.push("WHERE id = ");
    qb.push_bind_unseparated(file_id);
    qb.push("RETURNING *");

    qb.build_query_as::<File>()
    .fetch_one(pool)
    .await;
}

pub fn delete_file(file_id: Uuid) -> Result<Uuid, sqlx::Error> {
    sqlx::query!("DELETE FROM public.files WHERE id = $1")
    .bind(file_id)
    .fetch_one()
    .await
    .map(|row| row.id)
}

pub fn delete_files(file_ids: Vec<Uuid>) -> Result<Vec<Uuid>, sqlx::Error> {
    sqlx::query!(r#"DELETE FROM public.files WHERE id = ANY($1)"#,
        &file_ids[..] as &[Uuid]
    )
    .fetch_all()
    .await
    .map(|rows| rows.into_iter().map(|row| row.id).collect())
}