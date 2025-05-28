use crate::error::CliError;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Host {
    pub domain: String,
}

pub fn get_data_path() -> Result<PathBuf, CliError> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "RuneLink", "RuneLink") {
        let data_dir = proj_dirs.data_dir();
        if !data_dir.exists() {
            fs::create_dir_all(data_dir)?;
        }
        Ok(data_dir.join("hosts.json"))
    } else {
        Err(CliError::ConfigError(
            "Could not determine home directory or project directories.".into(),
        ))
    }
}

pub fn load_hosts() -> Result<Vec<Host>, CliError> {
    let data_path = get_data_path()?;
    if data_path.exists() {
        let data_str = fs::read_to_string(&data_path)?;
        if data_str.trim().is_empty() {
            return Ok(Vec::new());
        }
        let hosts: Vec<Host> = serde_json::from_str(&data_str)?;
        Ok(hosts)
    } else {
        Ok(Vec::new())
    }
}

pub fn save_hosts(hosts: &Vec<Host>) -> Result<(), CliError> {
    let data_path = get_data_path()?;
    let data_str = serde_json::to_string_pretty(hosts)?;
    fs::write(&data_path, data_str)?;
    Ok(())
}
