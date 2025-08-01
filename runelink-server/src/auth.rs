use crate::state::AppState;
use crate::{db::DbPool, error::ApiError, queries};
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use runelink_types::User;
use serde_json::json;
use uuid::Uuid;

/// Creates a router for all auth-related endpoints
pub fn router() -> Router<AppState> {
    // Well-known discovery endpoints must be at root level
    Router::new()
        .route("/.well-known/openid-configuration", get(discovery))
        .route("/.well-known/jwks.json", get(jwks))
        // OAuth/OIDC endpoints under /auth
        .nest(
            "/auth",
            Router::new()
                .route("/token", post(token))
                .route("/userinfo", get(userinfo))
                .route("/register", post(register_client))
                .route("/signup", post(signup)),
        )
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Session {
    pub user: User,
    pub is_admin: bool, // TODO: this will be inside the User struct
}

#[derive(Clone, Debug)]
pub struct AuthBuilder {
    user_id: Option<Uuid>, // TODO: replace with tokens/auth info
    admin: bool,
    server_member: Option<Uuid>,
    server_admin: Option<Uuid>,
    expected_user_id: Option<Uuid>,
    server_admin_override: Option<Uuid>,
}

impl AuthBuilder {
    pub fn new(user_id: Option<Uuid>) -> Self {
        AuthBuilder {
            user_id,
            admin: false,
            server_member: None,
            server_admin: None,
            expected_user_id: None,
            server_admin_override: None,
        }
    }

    #[allow(dead_code)]
    pub fn admin(mut self) -> Self {
        self.admin = true;
        self
    }

    #[allow(dead_code)]
    pub fn user(mut self, user_id: Uuid) -> Self {
        self.expected_user_id = Some(user_id);
        self
    }

    #[allow(dead_code)]
    pub fn server_member(mut self, server_id: Uuid) -> Self {
        self.server_member = Some(server_id);
        self
    }

    #[allow(dead_code)]
    pub fn server_admin(mut self, server_id: Uuid) -> Self {
        self.server_admin = Some(server_id);
        self
    }

    #[allow(dead_code)]
    pub fn or_server_admin(mut self, server_id: Uuid) -> Self {
        self.server_admin_override = Some(server_id);
        self
    }

    pub async fn build(&self, pool: &DbPool) -> Result<Session, ApiError> {
        let Some(user_id) = self.user_id else {
            return Err(ApiError::AuthError("No credentials provided".into()));
        };
        let user = queries::get_user_by_id(pool, user_id)
            .await
            .map_err(|_| ApiError::AuthError("Invalid credentials".into()))?;

        let user_is_admin = true; // for testing only

        // TODO: early success return for host admins

        // TODO: early success return if admin in server_admin_override

        if self.admin && !user_is_admin {
            // redundant check?
            return Err(ApiError::AuthError("Admin only".into()));
        }

        // TODO: check required server member and admin

        Ok(Session {
            user,
            is_admin: user_is_admin,
        })
    }

    #[allow(dead_code)]
    pub async fn build_optional(
        self,
        _pool: &DbPool,
    ) -> Result<Option<Session>, ApiError> {
        // allow guests but fetch user info if they are logged in
        todo!()
    }
}

/// Discovery endpoint for OIDC
pub async fn discovery(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let scheme = match state.config.local_domain.as_str() {
        "localhost" | "127.0.0.1" | "0.0.0.0" => "http",
        _ => "https",
    };
    let base_url = format!(
        "{}://{}",
        scheme,
        state.config.local_domain_with_explicit_port()
    );
    let issuer = base_url.clone();
    let jwks_uri = format!("{}/.well-known/jwks.json", base_url);
    let token_endpoint = format!("{}/auth/token", base_url);
    let userinfo_endpoint = format!("{}/auth/userinfo", base_url);
    Json(json!({
        "issuer": issuer,
        "jwks_uri": jwks_uri,
        "token_endpoint": token_endpoint,
        "userinfo_endpoint": userinfo_endpoint,
        "grant_types_supported": ["password", "refresh_token"],
        "response_types_supported": [],
        "scopes_supported": ["openid", "read:messages", "send:files"],
        "token_endpoint_auth_methods_supported": ["none"]
    }))
}

/// JWKS endpoint publishing public keys (stubbed for now)
pub async fn jwks() -> Json<serde_json::Value> {
    // TODO: Implement JWKS endpoint with actual public keys
    Json(json!({ "keys": [] }))
}

/// Token endpoint for password and refresh_token grants (stubbed for now)
pub async fn token() -> Json<serde_json::Value> {
    // TODO: Implement ROPC (password) and refresh_token flows
    Json(json!({
        "error": "not_implemented",
        "error_description": "Token endpoint not yet implemented"
    }))
}

/// Protected endpoint returning user claims (stubbed for now)
pub async fn userinfo() -> Json<serde_json::Value> {
    // TODO: Implement userinfo endpoint with actual user data
    Json(json!({
        "error": "not_implemented",
        "error_description": "Userinfo endpoint not yet implemented"
    }))
}

/// Dynamic Client Registration endpoint (stubbed for now)
pub async fn register_client() -> Json<serde_json::Value> {
    // TODO: Implement client registration, generating client_id
    Json(json!({
        "error": "not_implemented",
        "error_description": "Client registration not yet implemented"
    }))
}

/// User signup endpoint (stubbed for now)
pub async fn signup() -> Json<serde_json::Value> {
    // TODO: Implement user account creation with password hashing
    Json(json!({
        "error": "not_implemented",
        "error_description": "User signup not yet implemented"
    }))
}
