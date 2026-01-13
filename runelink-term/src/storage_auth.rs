use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::CliError;
use crate::storage::{load_data, save_data};

const AUTH_CACHE_FILENAME: &str = "auth.json";

/// Cached authentication data for a single account.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountAuth {
    /// Long-lived refresh token (required)
    pub refresh_token: String,
    /// Optional cached access token
    pub access_token: Option<String>,
    /// Optional expiration timestamp (Unix timestamp)
    pub expires_at: Option<i64>,
    /// Optional client ID used for token requests
    pub client_id: Option<String>,
    /// Optional scope used for token requests
    pub scope: Option<String>,
}

/// Auth cache storing authentication data for multiple accounts.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct AuthCache {
    /// Map from user_id to account auth data
    pub accounts: HashMap<Uuid, AccountAuth>,
}

impl AuthCache {
    /// Load auth cache from disk.
    pub fn load() -> Result<Self, CliError> {
        load_data(AUTH_CACHE_FILENAME)
    }

    /// Save auth cache to disk.
    pub fn save(&self) -> Result<(), CliError> {
        save_data(self, AUTH_CACHE_FILENAME)
    }

    /// Get auth data for a user.
    pub fn get(&self, user_id: &Uuid) -> Option<&AccountAuth> {
        self.accounts.get(user_id)
    }

    /// Get mutable auth data for a user.
    pub fn get_mut(&mut self, user_id: &Uuid) -> Option<&mut AccountAuth> {
        self.accounts.get_mut(user_id)
    }

    /// Set auth data for a user.
    pub fn set(&mut self, user_id: Uuid, auth: AccountAuth) {
        self.accounts.insert(user_id, auth);
    }

    /// Remove auth data for a user (logout).
    pub fn remove(&mut self, user_id: &Uuid) -> Option<AccountAuth> {
        self.accounts.remove(user_id)
    }

    /// Check if a user has auth data.
    #[allow(dead_code)]
    pub fn has_auth(&self, user_id: &Uuid) -> bool {
        self.accounts.contains_key(user_id)
    }
}
