use log::info;
use reqwest::Client;
use runelink_types::{NewUser, User, UserRef};

use crate::error::Result;

use super::{delete_authed, delete_federated, fetch_json, post_json_authed};

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

pub async fn fetch_by_ref(
    client: &Client,
    api_url: &str,
    user: UserRef,
) -> Result<User> {
    let url = format!(
        "{api_url}/users/{domain}/{name}",
        domain = user.domain,
        name = user.name
    );
    info!("fetching user: {url}");
    fetch_json::<User>(client, &url).await
}

pub async fn delete(
    client: &Client,
    api_url: &str,
    access_token: &str,
    user: UserRef,
) -> Result<()> {
    let url = format!(
        "{api_url}/users/{domain}/{name}",
        domain = user.domain,
        name = user.name
    );
    info!("deleting user: {url}");
    delete_authed(client, &url, access_token).await
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// DELETE /federation/users/{domain}/{name}
    pub async fn delete(
        client: &Client,
        api_url: &str,
        token: &str,
        user: UserRef,
    ) -> Result<()> {
        let url = format!(
            "{api_url}/federation/users/{domain}/{name}",
            domain = user.domain,
            name = user.name
        );
        info!("deleting user (federation): {url}");
        delete_federated(client, &url, token).await
    }
}
