use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::PgConnection;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use crate::error::ApiError;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    return Pool::builder()
        .connection_timeout(Duration::from_secs(2))
        .build(manager)
        .expect("Failed to create pool.");
}

pub fn get_conn(
    pool: &Arc<DbPool>,
) -> Result<PooledConnection<ConnectionManager<PgConnection>>, ApiError> {
    pool.get()
        .map_err(|e| ApiError::DbConnectionError(e.to_string()))
}
