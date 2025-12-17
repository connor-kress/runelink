use crate::{error::ApiError, jwks_resolver, queries, state::AppState};
use axum::http::HeaderMap;
use axum::http::header;
use jsonwebtoken::{Algorithm, Validation};
use runelink_types::{ClientAccessClaims, FederationClaims, User};
use uuid::Uuid;

fn extract_bearer_token(headers: &HeaderMap) -> Result<String, ApiError> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| {
            ApiError::AuthError("Missing Authorization header".into())
        })?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| {
            ApiError::AuthError("Invalid Authorization header format".into())
        })?
        .trim();
    Ok(token.into())
}

#[allow(unused)]
fn has_scope(scope_str: &str, required: &str) -> bool {
    scope_str.split_whitespace().any(|s| s == required)
}

#[derive(Clone, Debug)]
pub struct ClientAuth {
    pub claims: ClientAccessClaims,
}

impl ClientAuth {
    pub fn from_headers(
        headers: &HeaderMap,
        state: &AppState,
    ) -> Result<Self, ApiError> {
        let token = extract_bearer_token(headers)?;
        let server_id = state.config.api_url_with_port();
        let mut validation = Validation::new(Algorithm::EdDSA);
        validation.set_audience(&[server_id.clone()]);
        validation.set_issuer(&[server_id]);

        let data = jsonwebtoken::decode::<ClientAccessClaims>(
            &token,
            &state.key_manager.decoding_key,
            &validation,
        )
        .map_err(|_| ApiError::AuthError("Invalid or expired token".into()))?;

        Ok(Self {
            claims: data.claims,
        })
    }
}

#[allow(unused)]
#[derive(Clone, Debug)]
pub struct FederationAuth {
    pub claims: FederationClaims,
}

#[allow(unused)]
impl FederationAuth {
    pub async fn from_headers(
        headers: &HeaderMap,
        state: &AppState,
        required_scopes: &[&str],
    ) -> Result<Self, ApiError> {
        let token = extract_bearer_token(headers)?;
        let server_id = state.config.api_url_with_port();
        let claims =
            jwks_resolver::decode_federation_jwt(state, &token, &server_id)
                .await?;

        for required in required_scopes {
            if !has_scope(&claims.scope, required) {
                return Err(ApiError::AuthError(format!(
                    "Missing required scope: {required}"
                )));
            }
        }

        Ok(Self { claims })
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Session {
    pub user: User,
    pub is_admin: bool, // TODO: this will be inside the User struct
}

#[derive(Clone, Debug)]
pub struct AuthBuilder {
    user_id: Option<Uuid>,
    admin: bool,
    server_member: Option<Uuid>,
    server_admin: Option<Uuid>,
    server_admin_override: Option<Uuid>,
}

impl AuthBuilder {
    pub fn new() -> Self {
        AuthBuilder {
            user_id: None,
            admin: false,
            server_member: None,
            server_admin: None,
            server_admin_override: None,
        }
    }

    #[allow(dead_code)]
    pub fn user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    #[allow(dead_code)]
    pub fn admin(mut self) -> Self {
        self.admin = true;
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

    pub async fn build(
        &self,
        headers: &HeaderMap,
        state: &AppState,
    ) -> Result<Session, ApiError> {
        let auth = ClientAuth::from_headers(headers, state)?;
        let user_id = auth.claims.sub;
        let user = queries::get_user_by_id(&state.db_pool, user_id)
            .await
            .map_err(|_| ApiError::AuthError("Invalid credentials".into()))?;

        let user_is_host_admin = true; // for testing only

        // TODO: early success return for host admins

        // TODO: early success return if admin in server_admin_override

        if self.admin && !user_is_host_admin {
            // redundant check?
            return Err(ApiError::AuthError("Admin only".into()));
        }

        // TODO: check required server member and admin

        Ok(Session {
            user,
            is_admin: user_is_host_admin,
        })
    }

    #[allow(dead_code)]
    pub async fn build_optional(
        self,
        _headers: &HeaderMap,
        _state: &AppState,
    ) -> Result<Option<Session>, ApiError> {
        // allow guests but fetch user info if they are logged in
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_headers(auth_value: Option<&str>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(value) = auth_value {
            headers.insert(header::AUTHORIZATION, value.parse().unwrap());
        }
        headers
    }

    #[test]
    fn test_valid_bearer_token() {
        let headers = make_headers(Some("Bearer abc.def.ghi"));
        let token = extract_bearer_token(&headers).expect("token should parse");
        assert_eq!(token, "abc.def.ghi");
    }

    #[test]
    fn test_missing_header() {
        let headers = make_headers(None);
        let err = extract_bearer_token(&headers).unwrap_err();
        match err {
            ApiError::AuthError(msg) => {
                assert!(msg.contains("Missing Authorization"))
            }
            _ => panic!("unexpected error type"),
        }
    }

    #[test]
    fn test_wrong_prefix() {
        // Missing "Bearer " prefix
        let headers = make_headers(Some("Token abc.def.ghi"));
        let err = extract_bearer_token(&headers).unwrap_err();
        match err {
            ApiError::AuthError(msg) => {
                assert!(msg.contains("Invalid Authorization"))
            }
            _ => panic!("unexpected error type"),
        }
    }

    #[test]
    fn test_empty_bearer_token() {
        let headers = make_headers(Some("Bearer "));
        let token =
            extract_bearer_token(&headers).expect("should parse but be empty");
        assert_eq!(token, "");
    }

    #[test]
    fn test_non_utf8_header() {
        use axum::http::HeaderValue;
        let mut headers = HeaderMap::new();
        // Invalid UTF-8 sequence
        let value = HeaderValue::from_bytes(b"Bearer \xFF\xFE").unwrap();
        headers.insert(header::AUTHORIZATION, value);
        let err = extract_bearer_token(&headers).unwrap_err();
        match err {
            ApiError::AuthError(msg) => {
                assert!(msg.contains("Missing Authorization"));
            }
            _ => panic!("unexpected error type"),
        }
    }
}
