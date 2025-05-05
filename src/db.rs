use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::env;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn get_pool() -> DbPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL missing");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    return Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
}
