use reqwest::Client;

use crate::storage::{AccountConfig, AppConfig};

pub struct CliContext<'a> {
    pub client: &'a Client,
    pub config: &'a mut AppConfig,
    pub account: Option<&'a AccountConfig>,
}
