use runelink_client::{requests, util::get_api_url};
use runelink_types::{Channel, NewChannel};
use uuid::Uuid;

use crate::{
    auth::Session,
    error::{ApiError, ApiResult},
    queries,
    state::AppState,
};

/// Create a new channel in a server.
/// If target_domain is provided and not the local domain, creates on that remote domain.
/// Otherwise, creates locally.
pub async fn create(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    new_channel: &NewChannel,
    target_domain: Option<&str>,
) -> ApiResult<Channel> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        let channel =
            queries::channels::insert(&state.db_pool, server_id, new_channel)
                .await?;
        Ok(channel)
    } else {
        // Create on remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated channel creation"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let channel = requests::channels::federated::create(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            new_channel,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to create channel on {domain}: {e}"
            ))
        })?;
        Ok(channel)
    }
}

/// Get all channels.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local channels.
pub async fn get_all(
    state: &AppState,
    session: &Session,
    target_domain: Option<&str>,
) -> ApiResult<Vec<Channel>> {
    if !state.config.is_remote_domain(target_domain) {
        // Handle local case
        let channels = queries::channels::get_all(&state.db_pool).await?;
        Ok(channels)
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated channel fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let channels = requests::channels::federated::fetch_all(
            &state.http_client,
            &api_url,
            &token,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch channels from {domain}: {e}"
            ))
        })?;
        Ok(channels)
    }
}

/// Get channels in a server.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local channels.
pub async fn get_by_server(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<Vec<Channel>> {
    if !state.config.is_remote_domain(target_domain) {
        // Handle local case
        queries::channels::get_by_server(&state.db_pool, server_id).await
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated channel fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let channels = requests::channels::federated::fetch_by_server(
            &state.http_client,
            &api_url,
            &token,
            server_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch channels from {domain}: {e}"
            ))
        })?;
        Ok(channels)
    }
}

/// Get a channel by its ID.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local channel.
pub async fn get_by_id(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<Channel> {
    if !state.config.is_remote_domain(target_domain) {
        // Handle local case
        let channel =
            queries::channels::get_by_id(&state.db_pool, channel_id).await?;
        Ok(channel)
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated channel fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let channel = requests::channels::federated::fetch_by_id(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            channel_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch channel from {domain}: {e}"
            ))
        })?;
        Ok(channel)
    }
}

/// Delete a channel by ID.
/// If target_domain is provided and not the local domain, deletes on that remote domain.
/// Otherwise, deletes locally.
pub async fn delete(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    target_domain: Option<&str>,
) -> ApiResult<()> {
    // Handle local case
    if !state.config.is_remote_domain(target_domain) {
        // Verify the channel belongs to the server
        let channel =
            queries::channels::get_by_id(&state.db_pool, channel_id).await?;
        if channel.server_id != server_id {
            return Err(ApiError::AuthError(
                "Channel not found in specified server".into(),
            ));
        }
        queries::channels::delete(&state.db_pool, channel_id).await?;
        Ok(())
    } else {
        // Delete on remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated channel deletion"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        requests::channels::federated::delete(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            channel_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to delete channel on {domain}: {e}"
            ))
        })?;
        Ok(())
    }
}

/// Auth requirements for channel operations.
pub mod auth {
    use super::*;
    use crate::auth::Requirement as Req;

    pub fn create(server_id: Uuid) -> Req {
        Req::And(vec![Req::Client, Req::ServerAdmin { server_id }])
    }

    pub fn get_all() -> Req {
        Req::And(vec![Req::Client, Req::HostAdmin])
    }

    pub fn get_by_server(server_id: Uuid) -> Req {
        Req::And(vec![Req::Client, Req::ServerMember { server_id }])
    }

    pub fn get_by_id(server_id: Uuid) -> Req {
        Req::And(vec![Req::Client, Req::ServerMember { server_id }])
    }

    pub fn delete(server_id: Uuid) -> Req {
        Req::And(vec![Req::Client, Req::ServerAdmin { server_id }])
    }

    pub mod federated {
        use super::*;

        pub fn create(server_id: Uuid) -> Req {
            Req::And(vec![Req::Federation, Req::ServerAdmin { server_id }])
        }

        pub fn get_all() -> Req {
            Req::And(vec![Req::Federation, Req::HostAdmin])
        }

        pub fn get_by_server(server_id: Uuid) -> Req {
            Req::And(vec![Req::Federation, Req::ServerMember { server_id }])
        }

        pub fn get_by_id(server_id: Uuid) -> Req {
            Req::And(vec![Req::Federation, Req::ServerMember { server_id }])
        }

        pub fn delete(server_id: Uuid) -> Req {
            Req::And(vec![Req::Federation, Req::ServerAdmin { server_id }])
        }
    }
}
