use std::path::PathBuf;

use runelink_client::util::{get_api_url, pad_host};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid environment variable `{0}`: {1}")]
    InvalidEnvVar(String, #[source] std::num::ParseIntError),
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub local_host_raw: String,
    pub database_url: String,
    pub port: u16,
    pub key_dir: PathBuf,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let local_host = std::env::var("LOCAL_HOST").map_err(|_| {
            ConfigError::MissingEnvVar("LOCAL_HOST".to_string())
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
            local_host_raw: local_host,
            database_url,
            port,
            key_dir,
        })
    }

    /// Includes port if it's not the default port (7000)
    pub fn local_host(&self) -> String {
        if self.port == 7000 {
            self.local_host_raw.clone()
        } else {
            format!("{}:{}", &self.local_host_raw, self.port)
        }
    }

    /// Always includes port for machine-to-machine communication
    pub fn local_host_with_explicit_port(&self) -> String {
        format!("{}:{}", &self.local_host_raw, self.port)
    }

    pub fn api_url(&self) -> String {
        get_api_url(self.local_host_with_explicit_port().as_str())
    }

    pub fn is_remote_host(&self, host: Option<&str>) -> bool {
        let Some(host) = host else {
            return false;
        };
        pad_host(host) != pad_host(self.local_host().as_str())
    }
}
