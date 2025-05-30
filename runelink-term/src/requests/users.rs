use crate::error::CliError;
use reqwest::Client;
use runelink_types::User;
use uuid::Uuid;

use super::fetch_json;

pub async fn fetch_users(
    client: &Client,
    api_base: &str,
) -> Result<Vec<User>, CliError> {
    let url = format!("{}/users", api_base);
    fetch_json::<Vec<User>>(client, &url).await
}

pub async fn fetch_user_by_id(
    client: &Client,
    api_base: &str,
    user_id: Uuid,
) -> Result<User, CliError> {
    let url = format!("{}/users/{}", api_base, user_id);
    fetch_json::<User>(client, &url).await
}

pub async fn fetch_user_by_name_and_domain(
    client: &Client,
    api_base: &str,
    name: String,
    domain: String,
) -> Result<User, CliError> {
    let url = format!(
        "{}/users/find?name={}&domain={}",
        api_base, name, domain
    );
    fetch_json::<User>(client, &url).await
}
