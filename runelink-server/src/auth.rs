#![allow(dead_code)]

use crate::{
    bearer_auth::{ClientAuth, FederationAuth},
    error::{ApiError, ApiResult},
    queries,
    state::AppState,
};
use axum::http::HeaderMap;
use runelink_types::{
    FederationClaims, ServerMembership, ServerRole, User, UserRef,
};
use uuid::Uuid;

/// Macro to construct an `And` requirement.
/// Example: `and!(Requirement::Client, Requirement::User(user_id))`
#[macro_export]
macro_rules! and {
    () => {
        crate::auth::Requirement::And(vec![])
    };
    ( $( $req:expr ),+ $(,)? ) => {
        crate::auth::Requirement::And(vec![ $( $req ),+ ])
    };
}

/// Macro to construct an `Or` requirement.
/// Example: `or!(Requirement::Client, Requirement::Federation)`
#[macro_export]
macro_rules! or {
    () => {
        crate::auth::Requirement::Or(vec![])
    };
    ( $( $req:expr ),+ $(,)? ) => {
        crate::auth::Requirement::Or(vec![ $( $req ),+ ])
    };
}

#[derive(Clone, Debug)]
pub enum Principal {
    Client(ClientAuth),
    Federation(FederationAuth),
}

impl Principal {
    pub fn from_client_headers(
        headers: &HeaderMap,
        state: &AppState,
    ) -> ApiResult<Self> {
        let auth = ClientAuth::from_headers(headers, state)?;
        Ok(Self::Client(auth))
    }

    pub async fn from_federation_headers(
        headers: &HeaderMap,
        state: &AppState,
    ) -> ApiResult<Self> {
        let auth = FederationAuth::from_headers(headers, state).await?;
        Ok(Self::Federation(auth))
    }
}

#[derive(Clone, Debug)]
pub enum Requirement {
    /// Must be authenticated with a client token.
    Client,
    /// Must be authenticated with a federation token.
    Federation,
    /// Must be a user with the referenced ID.
    User(Uuid),
    /// Must be a host admin.
    HostAdmin,
    /// Must be a member of the referenced server.
    ServerMember { server_id: Uuid },
    /// Must be an admin of the referenced server.
    ServerAdmin { server_id: Uuid },
    /// Must satisfy all sub-requirements.
    And(Vec<Requirement>),
    /// Must satisfy at least one sub-requirement.
    Or(Vec<Requirement>),
}

impl Requirement {
    async fn check(
        &self,
        ctx: &mut AuthContext<'_>,
    ) -> ApiResult<Option<String>> {
        match self {
            Requirement::Client => {
                if !matches!(ctx.principal, Principal::Client(_)) {
                    return Ok(Some("Client auth required".into()));
                }
            }

            Requirement::Federation => {
                if !matches!(ctx.principal, Principal::Federation(_)) {
                    return Ok(Some("Federation auth required".into()));
                }
            }

            Requirement::User(user_id) => {
                let user = ctx.get_user().await?;
                if user.is_none() || user.unwrap().id != *user_id {
                    return Ok(Some("Invalid user".into()));
                }
            }

            Requirement::HostAdmin => {
                // TODO: add user roles
                return Ok(Some("Host admin only".into()));
            }

            Requirement::ServerMember { server_id } => {
                let membership = ctx.get_membership(*server_id).await?;
                if membership.is_none() {
                    return Ok(Some("Server member only".into()));
                }
            }

            Requirement::ServerAdmin { server_id } => {
                let membership = ctx.get_membership(*server_id).await?;
                if membership.is_none()
                    || membership.unwrap().role != ServerRole::Admin
                {
                    return Ok(Some("Server admin only".into()));
                }
            }

            Requirement::And(reqs) => {
                for req in reqs {
                    if let Some(error) = Box::pin(req.check(ctx)).await? {
                        return Ok(Some(error));
                    }
                }
            }

            Requirement::Or(reqs) => {
                if reqs.is_empty() {
                    return Ok(Some("No requirements".into()));
                }
                let mut errors = Vec::<String>::new();
                let mut found = false;
                for req in reqs {
                    if let Some(error) = Box::pin(req.check(ctx)).await? {
                        errors.push(error);
                    } else {
                        found = true;
                    }
                }
                if !found {
                    let combined_error = if errors.len() == 1 {
                        errors.first().unwrap().clone()
                    } else {
                        errors
                            .iter()
                            .map(|e| format!("({e})"))
                            .collect::<Vec<String>>()
                            .join(" and ")
                    };
                    return Ok(Some(combined_error));
                }
            }
        }
        Ok(None)
    }
}

#[derive(Clone, Debug)]
struct AuthContext<'a> {
    state: &'a AppState,
    principal: Principal,
    user_ref: Option<UserRef>,
    user: Option<User>,
    memberships: Option<Vec<ServerMembership>>,
}

impl<'a> AuthContext<'a> {
    async fn get_user(&mut self) -> ApiResult<Option<&User>> {
        if self.user.is_none() {
            let Some(user_ref) = self.user_ref.as_ref() else {
                return Ok(None);
            };
            let user =
                queries::users::get_by_id(&self.state.db_pool, user_ref.id)
                    .await?;
            self.user = Some(user);
        }
        Ok(self.user.as_ref())
    }

    async fn get_membership(
        &mut self,
        server_id: Uuid,
    ) -> ApiResult<Option<&ServerMembership>> {
        if self.memberships.is_none() {
            let Some(user_ref) = self.user_ref.as_ref() else {
                return Ok(None);
            };
            let memberships =
                queries::memberships::get_by_user(self.state, user_ref.id)
                    .await?;
            self.memberships = Some(memberships);
        }
        Ok(self.memberships.as_ref().and_then(|memberships| {
            memberships
                .iter()
                .find(|membership| membership.server.id == server_id)
        }))
    }
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
    ) -> ApiResult<Option<User>> {
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
            queries::users::get_by_id(&state.db_pool, user_ref.id).await;
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
    pub async fn require_user(&mut self, state: &AppState) -> ApiResult<User> {
        // Clone the user reference before calling lookup_user (which needs &mut self)
        let user_ref = self.user_ref.clone().ok_or_else(|| {
            ApiError::AuthError("No delegated user in session".into())
        })?;
        let user = self.lookup_user(state).await?.ok_or_else(|| {
            ApiError::AuthError(format!("User {user_ref} not found locally"))
        })?;
        Ok(user)
    }
}

/// Authorization engine (shared). Operation code defines an `AuthSpec` once,
/// and transport adapters supply a `Principal` based on their auth mechanism.
pub async fn authorize(
    state: &AppState,
    principal: Principal,
    req: Requirement,
) -> ApiResult<Session> {
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
    let mut ctx = AuthContext {
        state,
        principal,
        user_ref,
        user: None,
        memberships: None,
    };
    if let Some(error) = req.check(&mut ctx).await? {
        return Err(ApiError::AuthError(error));
    }
    let cached_user = if ctx.user_ref.is_some() {
        if let Some(user) = ctx.user {
            Some(Some(user))
        } else {
            // Not looked up yet
            None
        }
    } else {
        // Not delegated
        Some(None)
    };
    let session = Session {
        principal: ctx.principal,
        user_ref: ctx.user_ref,
        federation: federation_claims,
        cached_user,
    };
    Ok(session)
}
