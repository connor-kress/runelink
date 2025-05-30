pub mod channels;
pub mod generic;
pub mod messages;
pub mod servers;
pub mod users;
pub use channels::*;
pub use generic::*;
pub use messages::*;
pub use servers::*;
pub use users::*;

use crate::error::CliError;
use reqwest::Client;

pub async fn do_ping(
    client: &Client,
    api_base: &str,
) -> Result<String, CliError> {
    let url = format!("{}/ping", api_base);
    generic::fetch_text(client, &url).await
}
