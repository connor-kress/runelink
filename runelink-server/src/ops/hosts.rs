use runelink_client::{requests::hosts, util::get_api_url};
use runelink_types::{Host, UserRef};

use crate::{
    error::{ApiError, ApiResult},
    queries,
    state::AppState,
};

/// Get all domains associated with a user (public).
pub async fn get_user_associated_domains(
    state: &AppState,
    user_ref: UserRef,
    target_domain: Option<&str>,
) -> ApiResult<Vec<String>> {
    if !state.config.is_remote_domain(target_domain) {
        let domains = queries::hosts::get_user_associated_domains(
            &state.db_pool,
            &user_ref,
        )
        .await?;
        Ok(domains)
    } else {
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let domains = hosts::fetch_user_associated_domains(
            &state.http_client,
            &api_url,
            user_ref,
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
