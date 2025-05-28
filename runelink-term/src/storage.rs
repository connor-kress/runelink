use crate::error::CliError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub default_host: Option<String>,
    pub default_server: Option<Uuid>,
    pub default_channel: Option<Uuid>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            default_host: None,
            default_server: None,
            default_channel: None,
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
