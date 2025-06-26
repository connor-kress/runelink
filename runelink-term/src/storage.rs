use directories::ProjectDirs;
use runelink_client::util::get_api_url;
use runelink_types::{Server, User};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fmt, fs};
use uuid::Uuid;

use crate::error::CliError;

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

impl fmt::Display for AccountConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.domain)
    }
}

impl AccountConfig {
    pub fn verbose(&self) -> String {
        format!("{} ({})", self, self.user_id)
    }
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

    pub fn get_default_channel(&self, server_id: Uuid) -> Option<Uuid> {
        self.get_server_config(server_id)
            .and_then(|sc| sc.default_channel)
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
            if self.accounts.is_empty() {
                self.default_account = Some(user.id);
            }
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
            if self.servers.is_empty() {
                self.default_server = Some(server.id);
            }
            self.servers.push(ServerConfig {
                server_id: server.id,
                title: server.title.clone(),
                domain: domain.to_string(),
                default_channel: None,
            });
            self.servers.last_mut().unwrap()
        }
    }

    pub fn try_get_server_domain(
        &self,
        server_id: Uuid,
    ) -> Result<String, CliError> {
        self.get_server_config(server_id)
            .map(|sc| sc.domain.clone())
            .ok_or_else(|| {
                CliError::MissingContext(
                    "Server domain could not be determined.".into(),
                )
            })
    }

    pub fn try_get_server_api_url(
        &self,
        server_id: Uuid,
    ) -> Result<String, CliError> {
        // TODO: use user membership endpoint of account home server
        self.try_get_server_domain(server_id)
            .map(|ref domain| get_api_url(domain))
    }
}

pub trait TryGetDomain {
    fn try_get_domain(&self) -> Result<&str, CliError>;
    fn try_get_api_url(&self) -> Result<String, CliError>;
}

impl TryGetDomain for Option<&AccountConfig> {
    fn try_get_domain(&self) -> Result<&str, CliError> {
        self.map(|ac| ac.domain.as_str())
            .ok_or(CliError::MissingAccount)
    }

    fn try_get_api_url(&self) -> Result<String, CliError> {
        self.try_get_domain().map(get_api_url)
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
