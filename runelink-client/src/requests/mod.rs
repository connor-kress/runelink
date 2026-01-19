use reqwest::Client;

use crate::error::Result;

pub mod auth;
pub mod channels;
pub mod generic;
pub mod hosts;
pub mod memberships;
pub mod messages;
pub mod servers;
pub mod users;

pub use generic::*;

pub async fn ping(client: &Client, api_url: &str) -> Result<String> {
    let url = format!("{api_url}/ping");
    generic::fetch_text(client, &url).await
}
