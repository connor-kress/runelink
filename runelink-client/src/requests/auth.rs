use reqwest::Client;
use runelink_types::{SignupRequest, User};

use crate::error::Result;

use super::post_json;

pub async fn signup(
    client: &Client,
    api_url: &str,
    signup_req: &SignupRequest,
) -> Result<User> {
    let url = format!("{}/auth/signup", api_url);
    post_json::<SignupRequest, User>(client, &url, signup_req).await
}
