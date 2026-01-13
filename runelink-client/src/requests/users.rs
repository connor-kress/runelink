use log::info;
use reqwest::Client;
use runelink_types::{NewUser, User};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json};

pub async fn create(
    client: &Client,
    api_url: &str,
    new_user: &NewUser,
) -> Result<User> {
    let url = format!("{api_url}/users");
    info!("creating user: {url}");
    post_json::<NewUser, User>(client, &url, new_user).await
}

pub async fn fetch_all(client: &Client, api_url: &str) -> Result<Vec<User>> {
    let url = format!("{api_url}/users");
    info!("fetching all users: {url}");
    fetch_json::<Vec<User>>(client, &url).await
}

pub async fn fetch_by_id(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<User> {
    let url = format!("{api_url}/users/{user_id}");
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
