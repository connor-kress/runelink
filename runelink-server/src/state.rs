use std::sync::Arc;

use crate::{config::ServerConfig, db::DbPool};

#[derive(Clone, Debug)]
pub struct AppState {
    pub db_pool: Arc<DbPool>,
    pub config: Arc<ServerConfig>,
}
