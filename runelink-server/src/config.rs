use std::env;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub local_domain: String,
    pub database_url: String,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let local_domain = env::var("LOCAL_DOMAIN").map_err(|_| {
            ConfigError::MissingEnvVar("LOCAL_DOMAIN".to_string())
        })?;
        let database_url = env::var("DATABASE_URL").map_err(|_| {
            ConfigError::MissingEnvVar("DATABASE_URL".to_string())
        })?;

        Ok(ServerConfig {
            local_domain,
            database_url,
        })
    }
}
