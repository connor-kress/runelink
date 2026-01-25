use runelink_client::{requests::hosts, util::get_api_url};
use runelink_types::Host;
use uuid::Uuid;

use crate::{
    error::{ApiError, ApiResult},
    queries,
    state::AppState,
};

/// List all hosts (public).
pub async fn get_all(state: &AppState) -> ApiResult<Vec<Host>> {
    let hosts = queries::hosts::get_all(&state.db_pool).await?;
    Ok(hosts)
}

/// Get a host by domain (public).
pub async fn get_by_domain(state: &AppState, domain: &str) -> ApiResult<Host> {
    let host = queries::hosts::get_by_domain(&state.db_pool, domain).await?;
    Ok(host)
}

/// Get all domains associated with a user (public).
/// If target_domain is provided and not the local domain, fetches from
/// that remote domain. Otherwise, returns local domains.
pub async fn get_user_associated_domains(
    state: &AppState,
    user_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<Vec<String>> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let domains = queries::hosts::get_user_associated_domains(
            &state.db_pool,
            user_id,
        )
        .await?;
        Ok(domains)
    } else {
        // Fetch from remote domain (public endpoint, no auth needed)
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let domains = hosts::fetch_user_associated_domains(
            &state.http_client,
            &api_url,
            user_id,
            None,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch user associated domains from {domain}: {e}"
            ))
        })?;
        Ok(domains)
    }
}
