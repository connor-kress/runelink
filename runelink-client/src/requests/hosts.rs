use log::info;
use reqwest::Client;
use uuid::Uuid;

use crate::error::Result;

use super::fetch_json;

pub async fn fetch_user_associated_domains(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
    target_domain: Option<&str>,
) -> Result<Vec<String>> {
    let mut url = format!("{api_url}/users/{user_id}/domains");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("fetching user associated domains: {url}");
    fetch_json::<Vec<String>>(client, &url).await
}
