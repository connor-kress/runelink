use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, time::Duration};

pub type DbPool = Pool<Postgres>;

pub async fn get_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(50)
        .idle_timeout(Duration::from_secs(2))
        .connect(&database_url)
        .await
}
