#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid environment variable `{0}`: {1}")]
    InvalidEnvVar(String, #[source] std::num::ParseIntError),
}

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub local_domain: String,
    pub database_url: String,
    pub port: u16,
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

        Ok(ServerConfig {
            local_domain,
            database_url,
            port,
        })
    }

    pub fn local_domain_with_port(&self) -> String {
        if self.port == 7000 {
            self.local_domain.clone()
        } else {
            format!("{}:{}", &self.local_domain, self.port)
        }
    }
}
