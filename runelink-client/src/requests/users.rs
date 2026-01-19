use log::info;
use reqwest::Client;
use runelink_types::{NewUser, User};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json_authed};

pub async fn create(
    client: &Client,
    api_url: &str,
    access_token: &str,
    new_user: &NewUser,
) -> Result<User> {
    let url = format!("{api_url}/users");
    info!("creating user: {url}");
    post_json_authed::<NewUser, User>(client, &url, access_token, new_user)
        .await
}

pub async fn fetch_all(
    client: &Client,
    api_url: &str,
    target_domain: Option<&str>,
) -> Result<Vec<User>> {
    let mut url = format!("{api_url}/users");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("fetching all users: {url}");
    fetch_json::<Vec<User>>(client, &url).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
    target_domain: Option<&str>,
) -> Result<User> {
    let mut url = format!("{api_url}/users/{user_id}");
    if let Some(domain) = target_domain {
        url = format!("{url}?target_domain={domain}");
    }
    info!("fetching user: {url}");
    fetch_json::<User>(client, &url).await
}

pub async fn fetch_by_name_and_domain(
    client: &Client,
    api_url: &str,
    name: String,
    domain: String,
) -> Result<User> {
    let url = format!("{api_url}/users/find?name={name}&domain={domain}");
    info!("fetching user: {url}");
    fetch_json::<User>(client, &url).await
}
