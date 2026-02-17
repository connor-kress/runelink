use log::info;
use reqwest::Client;
use runelink_types::UserRef;

use crate::error::Result;

use super::fetch_json;

pub async fn fetch_user_associated_domains(
    client: &Client,
    api_url: &str,
    user_ref: UserRef,
    target_domain: Option<&str>,
) -> Result<Vec<String>> {
    let mut url = format!(
        "{api_url}/users/{domain}/{name}/domains",
        domain = user_ref.domain,
        name = user_ref.name
    );
    if let Some(d) = target_domain {
        url = format!("{url}?target_domain={d}");
    }
    info!("fetching user associated domains: {url}");
    fetch_json::<Vec<String>>(client, &url).await
}
