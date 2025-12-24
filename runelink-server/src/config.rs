use std::path::PathBuf;

use runelink_client::util::get_api_url;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid environment variable `{0}`: {1}")]
    InvalidEnvVar(String, #[source] std::num::ParseIntError),
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub local_domain_raw: String,
    pub database_url: String,
    pub port: u16,
    pub key_dir: PathBuf,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let local_domain = std::env::var("LOCAL_DOMAIN").map_err(|_| {
            ConfigError::MissingEnvVar("LOCAL_DOMAIN".to_string())
        })?;
        let database_url = std::env::var("DATABASE_URL").map_err(|_| {
            ConfigError::MissingEnvVar("DATABASE_URL".to_string())
        })?;
        let port_str = std::env::var("PORT").unwrap_or_else(|_| "7000".into());
        let port = port_str
            .parse::<u16>()
            .map_err(|e| ConfigError::InvalidEnvVar("PORT".into(), e))?;
        let key_dir = std::env::var("KEY_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut path = dirs_next::home_dir()
                    .expect("failed to get home directory");
                path.extend([".local", "share", "runelink", "keys"]);
                path
            });

        Ok(ServerConfig {
            local_domain_raw: local_domain,
            database_url,
            port,
            key_dir,
        })
    }

    /// Includes port if it's not the default port (7000)
    pub fn local_domain(&self) -> String {
        if self.port == 7000 {
            self.local_domain_raw.clone()
        } else {
            format!("{}:{}", &self.local_domain_raw, self.port)
        }
    }

    /// Always includes port for machine-to-machine communication
    pub fn local_domain_with_explicit_port(&self) -> String {
        format!("{}:{}", &self.local_domain_raw, self.port)
    }

    pub fn api_url(&self) -> String {
        get_api_url(self.local_domain_with_explicit_port().as_str())
    }
}
