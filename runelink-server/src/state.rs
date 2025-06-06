use std::sync::Arc;

use crate::{config::ServerConfig, db::DbPool};

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<ServerConfig>,
    pub db_pool: Arc<DbPool>,
    pub http_client: reqwest::Client,
}
