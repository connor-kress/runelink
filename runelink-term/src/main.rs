use crate::error::CliError;
use reqwest::Client;
use runelink_types::User;

mod error;

const API_BASE_URL: &str = "http://localhost:3000/api";

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let client = Client::new();

    // Example: List users
    let users_url = format!("{}/users", API_BASE_URL);
    println!("Fetching users from: {}", users_url);

    let response = client
        .get(&users_url)
        .send()
        .await?;

    // Check the HTTP status code
    if !response.status().is_success() {
        let status = response.status();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to get error message body".to_string());

        return Err(CliError::ApiStatusError { status, message });
    }

    let users: Vec<User> = response
        .json()
        .await?;

    println!("Successfully fetched {} users:", users.len());
    for user in users {
        println!("{:?}", user);
    }

    Ok(())
}
