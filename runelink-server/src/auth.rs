use crate::{db::DbPool, error::ApiError, queries, state::AppState};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, SaltString},
    Argon2, PasswordHasher, PasswordVerifier,
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Form, Json, Router,
};
use jsonwebtoken::{EncodingKey, Header};
use reqwest::StatusCode;
use runelink_types::{
    NewUser, RefreshToken, SignupRequest, TokenRequest, TokenResponse, User,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{Duration, OffsetDateTime};
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

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    iss: String,
    sub: String,
    aud: Vec<String>,
    exp: i64,
    iat: i64,
    scope: String,
    client_id: String,
}

impl JWTClaims {
    fn new(
        user_id: Uuid,
        client_id: String,
        api_url: String,
        lifetime: Duration,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        JWTClaims {
            iss: api_url.clone(),
            sub: user_id.to_string(),
            aud: vec![api_url],
            exp: now + lifetime.whole_seconds(),
            iat: now,
            scope: "openid".into(), // TODO: inherit original scopes?
            client_id,
        }
    }
}

pub async fn token(
    State(state): State<AppState>,
    Form(req): Form<TokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    match req.grant_type.as_str() {
        "password" => {
            let username = req
                .username
                .clone()
                .ok_or(ApiError::BadRequest("missing username".into()))?;
            let password = req
                .password
                .clone()
                .ok_or(ApiError::BadRequest("missing password".into()))?;

            // Get user
            let user = queries::get_user_by_name_and_domain(
                &state.db_pool, username, state.config.local_domain.clone()
            ).await?;

            // Verify password hash
            let account = queries::get_local_account(
                &state.db_pool, user.id
            ).await?;
            let parsed_hash = PasswordHash::new(&account.password_hash)
                .map_err(|_| ApiError::AuthError(
                    "invalid password hash".into()
                ))?;
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_err(|_| ApiError::AuthError(
                    "invalid credentials".into()
                ))?;

            // Create JWT
            let claims = JWTClaims::new(
                user.id,
                req.client_id.clone().unwrap_or_else(|| "default".to_string()),
                state.config.api_url_with_port(),
                Duration::hours(1),
            );
            let token = jsonwebtoken::encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret("TODO".as_bytes())
                                          // state.config.jwt_secret.as_ref()
            )
            .map_err(|e| ApiError::Internal(format!("jwt error: {e}")))?;

            // Create refresh token
            let rt = RefreshToken::new(
                user.id,
                "default".into(),
                Duration::days(30)
            );
            queries::insert_refresh_token(&state.db_pool, &rt).await?;

            Ok((StatusCode::OK, Json(TokenResponse {
                access_token: token,
                token_type: "Bearer".into(),
                expires_in: 3600,
                refresh_token: rt.token,
                scope: claims.scope,
            })))
        }

        "refresh_token" => {
            let rtoken = req
                .refresh_token
                .clone()
                .ok_or(ApiError::BadRequest("missing refresh_token".into()))?;
            let rt = queries::get_refresh_token(&state.db_pool, &rtoken).await?;

            // Validate refresh token
            let now = OffsetDateTime::now_utc();
            if rt.revoked || rt.expires_at <= now {
                return Err(ApiError::AuthError(
                    "refresh token expired or revoked".into()
                ));
            }

            let lifetime = Duration::hours(1);
            // Create new JWT
            let claims = JWTClaims::new(
                rt.user_id,
                req.client_id.clone().unwrap_or_else(|| "default".to_string()),
                state.config.api_url_with_port(),
                lifetime,
            );
            let token = jsonwebtoken::encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret("TODO".as_bytes())
                                          // state.config.jwt_secret.as_ref()
            )
            .map_err(|e| ApiError::Internal(format!("jwt error: {e}")))?;

            Ok((StatusCode::OK, Json(TokenResponse {
                access_token: token,
                token_type: "Bearer".into(),
                expires_in: lifetime.whole_seconds(),
                refresh_token: rt.token, // TODO: token rotation
                scope: claims.scope,
            })))
        }

        _ => Err(ApiError::BadRequest("unsupported grant_type".into())),
    }
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

/// POST /auth/signup
pub async fn signup(
    State(state): State<AppState>,
    Json(req): Json<SignupRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Insert user
    let new_user = NewUser {
        name: req.name,
        domain: state.config.local_domain.clone(),
    };
    let user = queries::insert_user(&state.db_pool, &new_user).await?;

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("hashing error: {e}")))?
        .to_string();

    // Insert local account
    let _account = queries::insert_local_account(
        &state.db_pool, user.id, &password_hash
    ).await?;

    Ok((StatusCode::CREATED, Json(user)))
}
