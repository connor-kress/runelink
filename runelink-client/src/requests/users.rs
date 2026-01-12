use reqwest::Client;
use runelink_types::{NewUser, User};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, fetch_json_federated, post_json};

pub async fn create_user(
    client: &Client,
    api_url: &str,
    new_user: &NewUser,
) -> Result<User> {
    let url = format!("{api_url}/users");
    post_json::<NewUser, User>(client, &url, new_user).await
}

pub async fn fetch_users(client: &Client, api_url: &str) -> Result<Vec<User>> {
    let url = format!("{api_url}/users");
    fetch_json::<Vec<User>>(client, &url).await
}

pub async fn fetch_user_by_id(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<User> {
    let url = format!("{api_url}/users/{user_id}");
    fetch_json::<User>(client, &url).await
}

pub async fn fetch_user_by_name_and_domain(
    client: &Client,
    api_url: &str,
    name: String,
    domain: String,
) -> Result<User> {
    let url = format!("{api_url}/users/find?name={name}&domain={domain}");
    fetch_json::<User>(client, &url).await
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// Fetch user info via federation (requires federation JWT).
    ///
    /// GET /federation/users/{user_id}
    pub async fn fetch_user_by_id(
        client: &Client,
        api_url: &str,
        token: &str,
        user_id: Uuid,
    ) -> Result<User> {
        let url = format!("{api_url}/federation/users/{user_id}");
        fetch_json_federated::<User>(client, &url, token).await
    }
}
