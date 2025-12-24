use crate::{error::ApiError, queries, state::AppState};
use runelink_types::Host;

/// List all hosts (public).
pub async fn list_hosts(state: &AppState) -> Result<Vec<Host>, ApiError> {
    let hosts = queries::get_all_hosts(&state.db_pool).await?;
    Ok(hosts)
}

/// Get a host by domain (public).
pub async fn get_host(
    state: &AppState,
    domain: &str,
) -> Result<Host, ApiError> {
    let host = queries::get_host_by_domain(&state.db_pool, domain).await?;
    Ok(host)
}
