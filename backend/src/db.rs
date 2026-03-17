// backend/src/db.rs
use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn create_pool(url: &str) -> anyhow::Result<PgPool> {
    Ok(PgPoolOptions::new().max_connections(10).connect(url).await?)
}
