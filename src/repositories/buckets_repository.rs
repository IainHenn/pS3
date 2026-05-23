use crate::models::bucket::Bucket;

pub async fn create_bucket(pool: &PgPool, bucketModel: Bucket) -> Result<Bucket, sqlx::Error>{
    sqlx::query_as!(Bucket, "INSERT INTO public.buckets VALUES ($1,$2) RETURNING *")
    .bind(bucketModel.name)
    .bind(bucketModel.created_at)
    .fetch_one(pool)
    .await;
}

pub async fn get_bucket(pool: &PgPool, bucket_id: String) -> Result<Bucket, sqlx::Error>{
    sqlx::query_as!(Bucket, "SELECT * FROM public.buckets WHERE id = $1")
    .bind(bucket_id)
    .fetch_one(pool)
    .await;
}

pub async fn get_buckets(pool: &PgPool, bucket_ids: Vec<Uuid>) -> Result<Vec<Bucket>, sqlx::Error>{
    sqlx::query_as!(Bucket, 
        r#"SELECT * FROM public.buckets WHERE id = ANY($1)"#
        &buckets_ids[..] as &[Uuid]
    )
    .bind(bucket_ids_str)
    .fetch_all(pool)
    .await;
}

pub async fn update_bucket(pool: &PgPool, bucket_id: Uuid) Result<Bucket, sqlx::Error>{
    sqlx::query_as!(Bucket, "UPDATE public.bucket WHERE id = $1 RETURNING *")
    .bind(bucket_id)
    .fetch_one(pool)
    .await;
}

pub async fn delete_bucket(pool: &PgPool, bucket_id: Uuid) -> Result<Uuid, sqlx::Error>{
    sqlx::query!("DELETE public.bucket WHERE id = $1 RETURNING id", bucket_id)
    .fetch_one(pool)
    .await
    .map(|row| row.id)
}

pub async fn delete_buckets(pool: &PgPool, bucket_ids: Vec<Uuid>) -> Result<Vec<String>, sqlx::Error>{
    sqlx::query!(r#"DELETE FROM public.buckets WHERE id = ANY($1) RETURNING id"#,
        &buckets_ids[..] as &[Uuid]
    )
    .fetch_all(pool)
    .await
    .map(|rows| rows.into_iter().map(|row| row.id).collect())
}