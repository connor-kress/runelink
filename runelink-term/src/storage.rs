use crate::error::CliError;
use directories::ProjectDirs;
use runelink_types::User;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppConfig {
    pub default_host: Option<String>,
    pub default_server: Option<Uuid>,
    pub accounts: Vec<AccountConfig>,
    pub servers: Vec<ServerConfig>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AccountConfig {
    pub user_id: Uuid,
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ServerConfig {
    pub server_id: Uuid,
    pub default_channel: Option<Uuid>,
}

impl AppConfig {
    #[allow(dead_code)]
    pub fn get_account_config(&self, user_id: Uuid) -> Option<&AccountConfig> {
        self.accounts.iter().find(|ac| ac.user_id == user_id)
    }

    pub fn get_server_config(&self, server_id: Uuid) -> Option<&ServerConfig> {
        self.servers.iter().find(|sc| sc.server_id == server_id)
    }

    #[allow(dead_code)]
    pub fn get_account_config_mut(
        &mut self,
        user_id: Uuid,
    ) -> Option<&mut AccountConfig> {
        self.accounts.iter_mut().find(|ac| ac.user_id == user_id)
    }

    #[allow(dead_code)]
    pub fn get_server_config_mut(
        &mut self,
        server_id: Uuid,
    ) -> Option<&mut ServerConfig> {
        self.servers.iter_mut().find(|sc| sc.server_id == server_id)
    }

    pub fn get_or_create_account_config(
        &mut self,
        user: &User,
    ) -> &mut AccountConfig {
        if let Some(idx) =
            self.accounts.iter().position(|ac| ac.user_id == user.id)
        {
            self.accounts[idx].domain = user.domain.clone();
            &mut self.accounts[idx]
        } else {
            self.accounts.push(AccountConfig {
                user_id: user.id,
                domain: user.domain.clone(),
            });
            self.accounts.last_mut().unwrap()
        }
    }

    pub fn get_or_create_server_config(
        &mut self,
        server_id: Uuid,
    ) -> &mut ServerConfig {
        if let Some(idx) =
            self.servers.iter().position(|sc| sc.server_id == server_id)
        {
            &mut self.servers[idx]
        } else {
            self.servers.push(ServerConfig {
                server_id,
                default_channel: None,
            });
            self.servers.last_mut().unwrap()
        }
    }
}

const CONFIG_FILENAME: &str = "config.json";
// const CACHE_FILENAME: &str = "cache.json";

pub fn get_data_dir() -> Result<PathBuf, CliError> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "RuneLink", "RuneLink") {
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }
        Ok(data_dir.to_path_buf())
    } else {
        Err(CliError::ConfigError(
            "Could not determine home directory or project directories.".into(),
        ))
    }
}

pub fn get_data_file_path(filename: &str) -> Result<PathBuf, CliError> {
    Ok(get_data_dir()?.join(filename))
}

pub fn load_data<T>(filename: &str) -> Result<T, CliError>
where
    T: for<'de> Deserialize<'de> + Default,
{
    let file_path = get_data_file_path(filename)?;
    if file_path.exists() {
        let data_str = fs::read_to_string(&file_path)?;
        if data_str.trim().is_empty() {
            Ok(T::default())
        } else {
            serde_json::from_str(&data_str).map_err(CliError::from)
        }
    } else {
        Ok(T::default())
    }
}

pub fn save_data<T>(data: &T, filename: &str) -> Result<(), CliError>
where
    T: Serialize,
{
    let file_path = get_data_file_path(filename)?;
    let data_str = serde_json::to_string_pretty(data)?;
    fs::write(&file_path, data_str)?;
    Ok(())
}

#[allow(dead_code)]
pub fn load_config() -> Result<AppConfig, CliError> {
    load_data(CONFIG_FILENAME)
}

#[allow(dead_code)]
pub fn save_config(config: &AppConfig) -> Result<(), CliError> {
    save_data(config, CONFIG_FILENAME)
}
