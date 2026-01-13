use runelink_types::Host;
use uuid::Uuid;

use crate::{error::ApiError, queries, state::AppState};

/// List all hosts (public).
pub async fn get_all(state: &AppState) -> Result<Vec<Host>, ApiError> {
    let hosts = queries::hosts::get_all(&state.db_pool).await?;
    Ok(hosts)
}

/// Get a host by domain (public).
pub async fn get_by_domain(
    state: &AppState,
    domain: &str,
) -> Result<Host, ApiError> {
    let host = queries::hosts::get_by_domain(&state.db_pool, domain).await?;
    Ok(host)
}

/// Get all domains associated with a user (public).
pub async fn get_user_associated_domains(
    state: &AppState,
    user_id: Uuid,
) -> Result<Vec<String>, ApiError> {
    let domains =
        queries::hosts::get_user_associated_domains(&state.db_pool, user_id)
            .await?;
    Ok(domains)
}
