use crate::{error::ApiError, queries, state::AppState};

/// List all hosts.
pub async fn list_hosts(
    state: &AppState,
) -> Result<Vec<runelink_types::Host>, ApiError> {
    queries::get_all_hosts(&state.db_pool).await
}

/// Get a host by domain.
pub async fn get_host(
    state: &AppState,
    domain: &str,
) -> Result<runelink_types::Host, ApiError> {
    queries::get_host_by_domain(&state.db_pool, domain).await
}
