use reqwest::Client;
use runelink_types::{NewUser, User};
use uuid::Uuid;

use crate::error::Result;

use super::{fetch_json, post_json};

pub async fn create_user(
    client: &Client,
    api_url: &str,
    new_user: &NewUser,
) -> Result<User> {
    let url = format!("{}/users", api_url);
    post_json::<NewUser, User>(client, &url, new_user).await
}

pub async fn fetch_users(
    client: &Client,
    api_url: &str,
) -> Result<Vec<User>> {
    let url = format!("{}/users", api_url);
    fetch_json::<Vec<User>>(client, &url).await
}

pub async fn fetch_user_by_id(
    client: &Client,
    api_url: &str,
    user_id: Uuid,
) -> Result<User> {
    let url = format!("{}/users/{}", api_url, user_id);
    fetch_json::<User>(client, &url).await
}

pub async fn fetch_user_by_name_and_domain(
    client: &Client,
    api_url: &str,
    name: String,
    domain: String,
) -> Result<User> {
    let url = format!(
        "{}/users/find?name={}&domain={}",
        api_url, name, domain
    );
    fetch_json::<User>(client, &url).await
}
