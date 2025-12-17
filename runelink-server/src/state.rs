use std::sync::Arc;

use crate::{config::ServerConfig, db::DbPool, key_manager::KeyManager};

pub type JwksCache =
    std::collections::HashMap<String, crate::jwks_resolver::CachedJwks>;

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<ServerConfig>,
    pub db_pool: Arc<DbPool>,
    pub http_client: reqwest::Client,
    pub key_manager: KeyManager,
    #[allow(dead_code)]
    pub jwks_cache: Arc<tokio::sync::RwLock<JwksCache>>,
}
