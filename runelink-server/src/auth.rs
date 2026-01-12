#![allow(dead_code)]

use crate::{
    bearer_auth::{ClientAuth, FederationAuth},
    error::ApiError,
    queries,
    state::AppState,
};
use axum::http::HeaderMap;
use runelink_types::{FederationClaims, ServerRole, User, UserRef};
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
    /// Must be authenticated with a federation token (server-level auth, no user delegation required).
    Federation,
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

/// Session represents the authenticated context for a request.
///
/// For client auth, the user is always local and exists in the DB.
/// For federation auth, the user reference may or may not exist locally.
#[derive(Clone, Debug)]
pub struct Session {
    /// The authenticated principal (Client or Federation)
    pub principal: Principal,
    /// Optional delegated user reference (always present for client auth, optional for federation)
    pub user_ref: Option<UserRef>,
    /// Present only when the request was authenticated via federation
    pub federation: Option<FederationClaims>,
    /// Cached user lookup result (None = not looked up, Some(None) = looked up but not found, Some(Some(user)) = found)
    cached_user: Option<Option<User>>,
}

impl Session {
    /// Perform a lazy DB lookup of the delegated user (cached).
    /// Returns Ok(None) if the user does not exist locally.
    pub async fn lookup_user(
        &mut self,
        state: &AppState,
    ) -> Result<Option<User>, ApiError> {
        // If already cached, return the cached result
        if let Some(cached) = &self.cached_user {
            return Ok(cached.clone());
        }
        // No user delegated
        let Some(user_ref) = &self.user_ref else {
            self.cached_user = Some(None);
            return Ok(None);
        };
        // Perform DB lookup
        let user_result =
            queries::get_user_by_id(&state.db_pool, user_ref.id).await;
        let user = match user_result {
            Ok(user) => Some(user),
            Err(ApiError::NotFound) => None,
            Err(e) => return Err(e),
        };
        self.cached_user = Some(user.clone());
        Ok(user)
    }

    /// Require that a delegated user exists locally.
    /// Returns an error if the user reference is missing or the user is not in the DB.
    pub async fn require_user(
        &mut self,
        state: &AppState,
    ) -> Result<User, ApiError> {
        // Clone the user reference before calling lookup_user (which needs &mut self)
        let user_ref = self.user_ref.clone().ok_or_else(|| {
            ApiError::AuthError("No delegated user in session".into())
        })?;
        let user = self.lookup_user(state).await?.ok_or_else(|| {
            ApiError::AuthError(format!("User {user_ref} not found locally"))
        })?;
        Ok(user)
    }

    /// Check if this session represents a host admin.
    /// TODO: Replace with real role-based logic when implemented.
    pub fn is_host_admin(&self) -> bool {
        // For now, only client-authenticated sessions can be admins
        matches!(self.principal, Principal::Client(_))
    }
}

/// Authorization engine (shared). Operation code defines an `AuthSpec` once,
/// and transport adapters supply a `Principal` based on their auth mechanism.
pub async fn authorize(
    state: &AppState,
    principal: Principal,
    spec: AuthSpec,
) -> Result<Session, ApiError> {
    // Extract user identity from the principal (no DB lookups yet)
    let (user_ref, federation_claims) = match &principal {
        Principal::Client(auth) => {
            // Client auth always has a local user
            (
                Some(UserRef::new(
                    auth.claims.sub,
                    state.config.local_domain(),
                )),
                None,
            )
        }
        Principal::Federation(auth) => {
            // Federation auth may have delegated user fields
            (auth.claims.user_ref.clone(), Some(auth.claims.clone()))
        }
    };
    let mut session = Session {
        principal: principal.clone(),
        user_ref,
        federation: federation_claims,
        cached_user: None,
    };

    // Validate requirements
    for req in &spec.requirements {
        match req {
            Requirement::Client => {
                if !matches!(principal, Principal::Client(_)) {
                    return Err(ApiError::AuthError(
                        "Client auth required".into(),
                    ));
                }
            }

            Requirement::Federation => {
                if !matches!(principal, Principal::Federation(_)) {
                    return Err(ApiError::AuthError(
                        "Federation auth required".into(),
                    ));
                }
            }

            Requirement::HostAdmin => {
                if !session.is_host_admin() {
                    return Err(ApiError::AuthError("Admin only".into()));
                }
            }

            Requirement::ServerMember { server_id } => {
                // This requires a user_id to check membership
                let user_ref = session.user_ref.clone().ok_or_else(|| {
                    ApiError::AuthError(
                        "User reference required for membership check".into(),
                    )
                })?;
                let member = queries::get_server_member(
                    &state.db_pool,
                    *server_id,
                    user_ref.id,
                )
                .await
                .map_err(|_| {
                    ApiError::AuthError("Not a server member".into())
                })?;
                session.cached_user = Some(Some(member.user));
            }

            Requirement::ServerAdmin { server_id } => {
                // This requires a user_id to check admin role
                let user_ref = session.user_ref.clone().ok_or_else(|| {
                    ApiError::AuthError(
                        "User reference required for admin check".into(),
                    )
                })?;
                let member = queries::get_server_member(
                    &state.db_pool,
                    *server_id,
                    user_ref.id,
                )
                .await
                .map_err(|_| {
                    ApiError::AuthError("Not a server admin".into())
                })?;
                if member.role != ServerRole::Admin {
                    return Err(ApiError::AuthError("Admin only".into()));
                }
                session.cached_user = Some(Some(member.user));
            }
        }
    }

    Ok(session)
}
