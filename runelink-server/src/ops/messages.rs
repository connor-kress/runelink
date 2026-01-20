use runelink_client::{requests, util::get_api_url};
use runelink_types::{Message, NewMessage};
use uuid::Uuid;

use crate::{
    auth::{AuthSpec, Requirement, Session},
    error::ApiError,
    queries,
    state::AppState,
};

/// Create a new message in a channel.
/// If target_domain is provided and not the local domain, creates on that remote domain.
/// Otherwise, creates locally.
pub async fn create(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    new_message: &NewMessage,
    target_domain: Option<&str>,
) -> Result<Message, ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        let channel =
            queries::channels::get_by_id(&state.db_pool, channel_id).await?;
        if channel.server_id != server_id {
            return Err(ApiError::AuthError(
                "Channel not found in specified server".into(),
            ));
        }
        let message =
            queries::messages::insert(&state.db_pool, channel_id, new_message)
                .await?;
        Ok(message)
    } else {
        // Create on remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated message creation"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let message = requests::messages::federated::create(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            channel_id,
            new_message,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to create message on {domain}: {e}"
            ))
        })?;
        Ok(message)
    }
}

/// Get all messages.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local messages.
pub async fn get_all(
    state: &AppState,
    session: &Session,
    target_domain: Option<&str>,
) -> Result<Vec<Message>, ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        let messages = queries::messages::get_all(&state.db_pool).await?;
        Ok(messages)
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated message fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let messages = requests::messages::federated::fetch_all(
            &state.http_client,
            &api_url,
            &token,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch messages from {domain}: {e}"
            ))
        })?;
        Ok(messages)
    }
}

/// Get messages in a server.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local messages.
pub async fn get_by_server(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    target_domain: Option<&str>,
) -> Result<Vec<Message>, ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        let messages =
            queries::messages::get_by_server(&state.db_pool, server_id).await?;
        Ok(messages)
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated message fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let messages = requests::messages::federated::fetch_by_server(
            &state.http_client,
            &api_url,
            &token,
            server_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch messages from {domain}: {e}"
            ))
        })?;
        Ok(messages)
    }
}

/// Get messages in a channel.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local messages.
pub async fn get_by_channel(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    target_domain: Option<&str>,
) -> Result<Vec<Message>, ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        let messages =
            queries::messages::get_by_channel(&state.db_pool, channel_id)
                .await?;
        Ok(messages)
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated message fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let messages = requests::messages::federated::fetch_by_channel(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            channel_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch messages from {domain}: {e}"
            ))
        })?;
        Ok(messages)
    }
}

/// Get a message by its ID.
/// If target_domain is provided and not the local domain, fetches from that remote domain.
/// Otherwise, returns local message.
pub async fn get_by_id(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
    target_domain: Option<&str>,
) -> Result<Message, ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        let message =
            queries::messages::get_by_id(&state.db_pool, message_id).await?;
        if message.channel_id != channel_id {
            return Err(ApiError::AuthError(
                "Message not found in specified channel".into(),
            ));
        }
        let channel =
            queries::channels::get_by_id(&state.db_pool, channel_id).await?;
        if channel.server_id != server_id {
            return Err(ApiError::AuthError(
                "Message not found in specified server".into(),
            ));
        }
        Ok(message)
    } else {
        // Fetch from remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated message fetching"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        let message = requests::messages::federated::fetch_by_id(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            channel_id,
            message_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to fetch message from {domain}: {e}"
            ))
        })?;
        Ok(message)
    }
}

/// Delete a message by ID.
/// If target_domain is provided and not the local domain, deletes on that remote domain.
/// Otherwise, deletes locally.
pub async fn delete(
    state: &AppState,
    session: &Session,
    server_id: Uuid,
    channel_id: Uuid,
    message_id: Uuid,
    target_domain: Option<&str>,
) -> Result<(), ApiError> {
    // Handle local case
    if target_domain.is_none()
        || target_domain == Some(state.config.local_domain().as_str())
    {
        // Verify the message belongs to the channel and server
        // TODO: This should be done with one database query
        let message =
            queries::messages::get_by_id(&state.db_pool, message_id).await?;
        if message.channel_id != channel_id {
            return Err(ApiError::AuthError(
                "Message not found in specified channel".into(),
            ));
        }
        let channel =
            queries::channels::get_by_id(&state.db_pool, channel_id).await?;
        if channel.server_id != server_id {
            return Err(ApiError::AuthError(
                "Message not found in specified server".into(),
            ));
        }
        queries::messages::delete(&state.db_pool, message_id).await?;
        Ok(())
    } else {
        // Delete on remote domain using federation
        let domain = target_domain.unwrap();
        let api_url = get_api_url(domain);
        let user_ref = session.user_ref.as_ref().ok_or_else(|| {
            ApiError::Internal(
                "User reference required for federated message deletion"
                    .to_string(),
            )
        })?;
        let token = state.key_manager.issue_federation_jwt_delegated(
            state.config.api_url(),
            api_url.clone(),
            user_ref.id,
            user_ref.domain.clone(),
        )?;
        requests::messages::federated::delete(
            &state.http_client,
            &api_url,
            &token,
            server_id,
            channel_id,
            message_id,
        )
        .await
        .map_err(|e| {
            ApiError::Internal(format!(
                "Failed to delete message on {domain}: {e}"
            ))
        })?;
        Ok(())
    }
}

/// Auth requirements for message operations.
pub mod auth {
    use super::*;

    pub fn create(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
        }
    }

    pub fn get_all() -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::HostAdmin],
        }
    }

    pub fn get_by_server(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
        }
    }

    pub fn get_by_channel(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
        }
    }

    pub fn get_by_id(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            requirements: vec![Requirement::ServerMember { server_id }],
        }
    }

    pub fn delete(server_id: Uuid) -> AuthSpec {
        AuthSpec {
            // TODO: or if they are the message owner
            requirements: vec![Requirement::ServerAdmin { server_id }],
        }
    }

    pub mod federated {
        use super::*;

        pub fn create(server_id: Uuid) -> AuthSpec {
            AuthSpec {
                requirements: vec![
                    Requirement::Federation,
                    Requirement::ServerMember { server_id },
                ],
            }
        }

        pub fn get_all() -> AuthSpec {
            AuthSpec {
                requirements: vec![
                    Requirement::Federation,
                    Requirement::HostAdmin,
                ],
            }
        }

        pub fn get_by_server(server_id: Uuid) -> AuthSpec {
            AuthSpec {
                requirements: vec![
                    Requirement::Federation,
                    Requirement::ServerMember { server_id },
                ],
            }
        }

        pub fn get_by_channel(server_id: Uuid) -> AuthSpec {
            AuthSpec {
                requirements: vec![
                    Requirement::Federation,
                    Requirement::ServerMember { server_id },
                ],
            }
        }

        pub fn get_by_id(server_id: Uuid) -> AuthSpec {
            AuthSpec {
                requirements: vec![
                    Requirement::Federation,
                    Requirement::ServerMember { server_id },
                ],
            }
        }

        pub fn delete(server_id: Uuid) -> AuthSpec {
            AuthSpec {
                // TODO: or if they are the message owner
                requirements: vec![
                    Requirement::Federation,
                    Requirement::ServerAdmin { server_id },
                ],
            }
        }
    }
}
