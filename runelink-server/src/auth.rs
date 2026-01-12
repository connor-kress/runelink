#![allow(dead_code)]

use crate::{
    bearer_auth::{ClientAuth, FederationAuth},
    error::ApiError,
    queries,
    state::AppState,
};
use axum::http::HeaderMap;
use runelink_types::{FederationClaims, ServerRole, User};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Principal {
    Client(ClientAuth),
    Federation(FederationAuth),
}

impl Principal {
    pub fn from_client_headers(
        headers: &HeaderMap,
        state: &AppState,
    ) -> Result<Self, ApiError> {
        let auth = ClientAuth::from_headers(headers, state)?;
        Ok(Self::Client(auth))
    }

    pub async fn from_federation_headers(
        headers: &HeaderMap,
        state: &AppState,
    ) -> Result<Self, ApiError> {
        let auth = FederationAuth::from_headers(headers, state).await?;
        Ok(Self::Federation(auth))
    }
}

#[derive(Clone, Debug)]
pub enum Requirement {
    /// Must be authenticated with a client token.
    Client,
    /// Must be authenticated with a federation token, and contain all required scopes.
    Federation { scopes: Vec<&'static str> },
    /// Must be a host admin.
    HostAdmin,
    /// Must be a member of the referenced server.
    ServerMember { server_id: Uuid },
    /// Must be an admin of the referenced server.
    ServerAdmin { server_id: Uuid },
}

#[derive(Clone, Debug, Default)]
pub struct AuthSpec {
    pub requirements: Vec<Requirement>,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub user: User,
    // TODO: replace with real host-admin logic when roles are modeled
    pub is_admin: bool,
    /// Present only when the request was authenticated via federation.
    pub federation: Option<FederationClaims>,
}

fn has_scope(scope_str: &str, required: &str) -> bool {
    scope_str.split_whitespace().any(|s| s == required)
}

/// Authorization engine (shared). Operation code defines an `AuthSpec` once,
/// and transport adapters supply a `Principal` based on their auth mechanism.
pub async fn authorize(
    state: &AppState,
    principal: Principal,
    spec: AuthSpec,
) -> Result<Session, ApiError> {
    // TODO: handle optional authentication (type state pattern?)
    let (user_id, federation_claims): (Uuid, Option<FederationClaims>) =
        match &principal {
            Principal::Client(auth) => (auth.claims.sub, None),
            Principal::Federation(auth) => {
                (auth.claims.sub, Some(auth.claims.clone()))
            }
        };

    let user = queries::get_user_by_id(&state.db_pool, user_id)
        .await
        .map_err(|_| ApiError::AuthError("Invalid credentials".into()))?;

    // TODO: replace with real host-admin logic when roles are modeled
    let is_host_admin = true;

    for req in &spec.requirements {
        match req {
            Requirement::Client => {
                if !matches!(principal, Principal::Client(_)) {
                    return Err(ApiError::AuthError(
                        "Client auth required".into(),
                    ));
                }
            }
            Requirement::Federation { scopes } => {
                let Principal::Federation(auth) = &principal else {
                    return Err(ApiError::AuthError(
                        "Federation auth required".into(),
                    ));
                };
                for required in scopes {
                    if !has_scope(&auth.claims.scope, required) {
                        return Err(ApiError::AuthError(format!(
                            "Missing required scope: {required}"
                        )));
                    }
                }
            }
            Requirement::HostAdmin => {
                if !is_host_admin {
                    return Err(ApiError::AuthError("Admin only".into()));
                }
            }
            Requirement::ServerMember { server_id } => {
                let _ = queries::get_server_member(
                    &state.db_pool,
                    *server_id,
                    user_id,
                )
                .await
                .map_err(|_| {
                    ApiError::AuthError("Not a server member".into())
                })?;
            }
            Requirement::ServerAdmin { server_id } => {
                let member = queries::get_server_member(
                    &state.db_pool,
                    *server_id,
                    user_id,
                )
                .await
                .map_err(|_| {
                    ApiError::AuthError("Not a server admin".into())
                })?;
                if member.role != ServerRole::Admin {
                    return Err(ApiError::AuthError("Admin only".into()));
                }
            }
        }
    }

    Ok(Session {
        user,
        is_admin: is_host_admin,
        federation: federation_claims,
    })
}
