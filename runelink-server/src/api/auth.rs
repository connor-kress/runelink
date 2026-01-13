use crate::{error::ApiError, queries, state::AppState};
use argon2::{
    Argon2, PasswordHasher, PasswordVerifier,
    password_hash::{PasswordHash, SaltString, rand_core::OsRng},
};
use axum::{
    Form, Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use jsonwebtoken::{Algorithm, Header};
use reqwest::StatusCode;
use runelink_types::{
    ClientAccessClaims, NewUser, RefreshToken, SignupRequest, TokenRequest,
    TokenResponse,
};
use serde_json::json;
use time::{Duration, OffsetDateTime};

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

/// Discovery endpoint for OIDC
pub async fn discovery(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let issuer = state.config.api_url();
    let jwks_uri = format!("{}/.well-known/jwks.json", issuer);
    let token_endpoint = format!("{}/auth/token", issuer);
    let userinfo_endpoint = format!("{}/auth/userinfo", issuer);
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

/// JWKS endpoint publishing public keys
pub async fn jwks(State(state): State<AppState>) -> Json<serde_json::Value> {
    let keys = vec![state.key_manager.public_jwk.clone()];
    Json(json!({ "keys": keys }))
}

pub async fn token(
    State(state): State<AppState>,
    Form(req): Form<TokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: check dynamic client IDs for validity
    let client_id = req.client_id.unwrap_or_else(|| "default".into());
    // TODO: check requested scopes for validity
    let scope = req.scope.unwrap_or_else(|| "openid".into());

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
            let user = queries::users::get_by_name_and_domain(
                &state.db_pool,
                username,
                state.config.local_domain(),
            )
            .await?;

            // Verify password hash
            let account =
                queries::accounts::get_by_user(&state.db_pool, user.id).await?;
            let parsed_hash = PasswordHash::new(&account.password_hash)
                .map_err(|_| {
                    ApiError::AuthError("invalid password hash".into())
                })?;
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .map_err(|_| {
                    ApiError::AuthError("invalid credentials".into())
                })?;

            // Create client access JWT (valid only on this server)
            let lifetime = Duration::hours(1);
            let claims = ClientAccessClaims::new(
                user.id,
                client_id.clone(),
                state.config.api_url(),
                scope,
                lifetime,
            );
            let token = jsonwebtoken::encode(
                &Header::new(Algorithm::EdDSA),
                &claims,
                &state.key_manager.private_key,
            )
            .map_err(|e| ApiError::Internal(format!("jwt error: {e}")))?;

            // Create refresh token
            let rt = RefreshToken::new(user.id, client_id, Duration::days(30));
            queries::tokens::insert_refresh(&state.db_pool, &rt).await?;

            Ok((
                StatusCode::OK,
                Json(TokenResponse {
                    access_token: token,
                    token_type: "Bearer".into(),
                    expires_in: 3600,
                    refresh_token: rt.token,
                    scope: claims.scope,
                }),
            ))
        }

        "refresh_token" => {
            let rtoken = req
                .refresh_token
                .clone()
                .ok_or(ApiError::BadRequest("missing refresh_token".into()))?;
            let rt =
                queries::tokens::get_refresh(&state.db_pool, &rtoken).await?;

            // Validate refresh token
            let now = OffsetDateTime::now_utc();
            if rt.revoked || rt.expires_at <= now {
                return Err(ApiError::AuthError(
                    "refresh token expired or revoked".into(),
                ));
            }

            // Create new client access JWT
            let lifetime = Duration::hours(1);
            let claims = ClientAccessClaims::new(
                rt.user_id,
                client_id,
                state.config.api_url(),
                scope,
                lifetime,
            );
            let token = jsonwebtoken::encode(
                &Header::new(Algorithm::EdDSA),
                &claims,
                &state.key_manager.private_key,
            )
            .map_err(|e| ApiError::Internal(format!("jwt error: {e}")))?;

            Ok((
                StatusCode::OK,
                Json(TokenResponse {
                    access_token: token,
                    token_type: "Bearer".into(),
                    expires_in: lifetime.whole_seconds(),
                    refresh_token: rt.token, // TODO: token rotation
                    scope: claims.scope,
                }),
            ))
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
        domain: state.config.local_domain(),
    };
    let user = queries::users::insert(&state.db_pool, &new_user).await?;

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)
        .map_err(|e| ApiError::Internal(format!("hashing error: {e}")))?
        .to_string();

    // Insert local account
    let _account =
        queries::accounts::insert(&state.db_pool, user.id, &password_hash)
            .await?;

    Ok((StatusCode::CREATED, Json(user)))
}
