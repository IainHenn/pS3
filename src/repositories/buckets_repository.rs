use crate::models::bucket::Bucket;
use crate::models::bucket::BucketUpdateModel;

pub async fn create_bucket(pool: &PgPool, bucket: Bucket) -> Result<Bucket, sqlx::Error>{
    sqlx::query_as!(Bucket, "INSERT INTO public.buckets VALUES ($1,$2) RETURNING *")
    .bind(bucket.name)
    .bind(bucket.created_at)
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
        r#"SELECT * FROM public.buckets WHERE id = ANY($1)"#,
        &buckets_ids[..] as &[Uuid]
    )
    .fetch_all(pool)
    .await;
}

pub async fn update_bucket(pool: &PgPool, bucket_id: Uuid, bucket_update: BucketUpdateModel) -> Result<Bucket, sqlx::Error>{
    let mut qb = sqlx::QueryBuilder::new("UPDATE public.buckets SET");
    let mut separated = qb.separated(", ");

    if let Some(name) = update.name {
        separated.push("name = ");
        separated.push_bind_unseparated(name);
    }
    
    qb.push("WHERE id = ");
    qb.push_bind_unseparated(bucket_id);
    qb.push("RETURNING *");

    qb.build_query_as::<Bucket>()
    .fetch_one()
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