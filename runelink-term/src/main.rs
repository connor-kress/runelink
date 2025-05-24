use crate::{
    error::CliError,
    requests::{do_ping, fetch_users},
};
use requests::fetch_user_by_id;
use reqwest::Client;

mod error;
mod requests;

fn get_api_url(domain: &str) -> String {
    format!("http://{}/api", domain)
}

async fn test_connectivities(client: &Client, domains: Vec<&str>) {
    println!("Hosts:");
    for domain in domains {
        let api_url = get_api_url(domain);
        match do_ping(client, &api_url).await {
            Ok(_) => println!("{} (ready)", domain),
            Err(_) => println!("{} (down)", domain),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let client = Client::new();

    let domain = "localhost:3000";
    let bad_domain = "localhost:9999";

    test_connectivities(&client, vec![domain, bad_domain]).await;
    println!();

    // Example: List users
    let api_url = get_api_url(domain);
    let users = fetch_users(&client, &api_url).await?;
    println!("Successfully fetched {} users:", users.len());
    for user in users {
        println!("{:?}", user);
        let new_copy = fetch_user_by_id(&client, &api_url, user.id).await?;
        println!("{:?}", new_copy);
    }

    Ok(())
}
