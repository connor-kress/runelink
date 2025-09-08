use reqwest::Client;

use crate::error::Result;

pub mod auth;
pub mod channels;
pub mod generic;
pub mod messages;
pub mod servers;
pub mod users;

pub use channels::*;
pub use generic::*;
pub use messages::*;
pub use servers::*;
pub use users::*;

pub async fn do_ping(client: &Client, api_url: &str) -> Result<String> {
    let url = format!("{}/ping", api_url);
    generic::fetch_text(client, &url).await
}
