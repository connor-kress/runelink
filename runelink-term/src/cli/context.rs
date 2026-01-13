use reqwest::Client;
use runelink_client::requests;
use time::OffsetDateTime;

use crate::error::CliError;
use crate::storage::{AccountConfig, AppConfig, TryGetDomain};
use crate::storage_auth::AuthCache;

pub struct CliContext<'a> {
    pub client: &'a Client,
    pub config: &'a mut AppConfig,
    pub auth_cache: &'a mut AuthCache,
    pub account: Option<&'a AccountConfig>,
}

impl<'a> CliContext<'a> {
    /// Get the home server API URL for the current account.
    pub fn home_api_url(&self) -> Result<String, CliError> {
        self.account.try_get_api_url()
    }

    /// Get a valid access token, refreshing if necessary.
    pub async fn get_access_token(&mut self) -> Result<String, CliError> {
        let account = self.account.ok_or(CliError::MissingAccount)?;
        let user_id = account.user_id;
        let api_url = self.home_api_url()?;

        // Get auth data (need to check if refresh is needed)
        let needs_refresh = {
            let auth = self.auth_cache.get(&user_id).ok_or_else(|| {
                CliError::InvalidArgument(
                    "Not logged in. Use 'rune account login' to authenticate."
                        .into(),
                )
            })?;

            // Check if access token is still valid
            let now = OffsetDateTime::now_utc().unix_timestamp();
            if let Some(access_token) = &auth.access_token {
                if let Some(expires_at) = auth.expires_at {
                    // Refresh if token expires in less than 60 seconds
                    if expires_at > now + 60 {
                        return Ok(access_token.clone());
                    }
                } else {
                    // No expiration info, assume valid
                    return Ok(access_token.clone());
                }
            }
            true
        };

        if !needs_refresh {
            // Should have returned above, but handle just in case
            let auth = self.auth_cache.get(&user_id).unwrap();
            return Ok(auth.access_token.as_ref().unwrap().clone());
        }

        // Need to refresh token
        let auth = self.auth_cache.get_mut(&user_id).unwrap();
        let token_response = requests::auth::token_refresh(
            self.client,
            &api_url,
            &auth.refresh_token,
            auth.scope.as_deref(),
            auth.client_id.as_deref(),
        )
        .await?;

        // Update cached auth data
        let now = OffsetDateTime::now_utc().unix_timestamp();
        auth.access_token = Some(token_response.access_token.clone());
        auth.expires_at = Some(now + token_response.expires_in);
        // Refresh token may be rotated, update if returned
        if !token_response.refresh_token.is_empty() {
            auth.refresh_token = token_response.refresh_token;
        }

        self.auth_cache.save()?;

        Ok(token_response.access_token)
    }
}
