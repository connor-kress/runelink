use directories::ProjectDirs;
use runelink_types::{Server, User};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{error::CliError, util::get_api_url};

const CONFIG_FILENAME: &str = "config.json";
// const CACHE_FILENAME: &str = "cache.json";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AppConfig {
    pub default_account: Option<Uuid>,
    pub default_server: Option<Uuid>,
    pub accounts: Vec<AccountConfig>,
    pub servers: Vec<ServerConfig>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountConfig {
    pub user_id: Uuid,
    pub name: String,
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub server_id: Uuid,
    pub title: String,
    pub domain: String,
    pub default_channel: Option<Uuid>,
}

#[allow(dead_code)]
impl AppConfig {
    pub fn load() -> Result<Self, CliError> {
        load_data(CONFIG_FILENAME)
    }

    pub fn save(&self) -> Result<(), CliError> {
        save_data(self, CONFIG_FILENAME)
    }

    pub fn get_default_account(&self) -> Option<&AccountConfig> {
        self.default_account.and_then(|user_id| {
            self.accounts.iter().find(|ac| ac.user_id == user_id)
        })
    }

    pub fn get_default_account_mut(&mut self) -> Option<&mut AccountConfig> {
        self.default_account.and_then(|user_id| {
            self.accounts.iter_mut().find(|ac| ac.user_id == user_id)
        })
    }

    pub fn get_account_config(&self, user_id: Uuid) -> Option<&AccountConfig> {
        self.accounts.iter().find(|ac| ac.user_id == user_id)
    }

    pub fn get_account_config_mut(
        &mut self,
        user_id: Uuid,
    ) -> Option<&mut AccountConfig> {
        self.accounts.iter_mut().find(|ac| ac.user_id == user_id)
    }

    pub fn get_account_config_by_name(
        &self,
        name: &str,
        domain: &str,
    ) -> Option<&AccountConfig> {
        self.accounts
            .iter()
            .find(|ac| ac.name == name && ac.domain == domain)
    }

    pub fn get_account_config_by_name_mut(
        &mut self,
        name: String,
        domain: String,
    ) -> Option<&mut AccountConfig> {
        self.accounts
            .iter_mut()
            .find(|ac| ac.name == name && ac.domain == domain)
    }

    pub fn get_server_config(&self, server_id: Uuid) -> Option<&ServerConfig> {
        self.servers.iter().find(|sc| sc.server_id == server_id)
    }

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
                name: user.name.clone(),
                domain: user.domain.clone(),
            });
            self.accounts.last_mut().unwrap()
        }
    }

    pub fn get_or_create_server_config(
        &mut self,
        server: &Server,
        domain: &str,
    ) -> &mut ServerConfig {
        if let Some(idx) =
            self.servers.iter().position(|sc| sc.server_id == server.id)
        {
            &mut self.servers[idx]
        } else {
            self.servers.push(ServerConfig {
                server_id: server.id,
                title: server.title.clone(),
                domain: domain.to_string(),
                default_channel: None,
            });
            self.servers.last_mut().unwrap()
        }
    }
}

pub trait TryGetDomainName {
    fn try_get_domain_name(&self) -> Result<&str, CliError>;
    fn try_get_api_url(&self) -> Result<String, CliError>;
}

impl TryGetDomainName for Option<&AccountConfig> {
    fn try_get_domain_name(&self) -> Result<&str, CliError> {
        self.map(|ac| ac.domain.as_str())
            .ok_or(CliError::MissingAccount)
    }

    fn try_get_api_url(&self) -> Result<String, CliError> {
        let domain = self.try_get_domain_name()?;
        Ok(get_api_url(domain))
    }
}

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
