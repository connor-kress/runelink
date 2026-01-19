use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::UserRef;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct LocalAccount {
    pub user_id: Uuid,
    pub password_hash: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct RefreshToken {
    pub token: String,
    pub user_id: Uuid,
    pub client_id: String,
    #[serde(with = "time::serde::rfc3339")]
    pub issued_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    pub revoked: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub username: Option<String>, // password grant
    pub password: Option<String>, // password grant
    pub refresh_token: Option<String>, // refresh_token grant
    pub scope: Option<String>,
    pub client_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String, // always "Bearer"
    pub expires_in: i64,
    pub refresh_token: String,
    pub scope: String,
}

impl RefreshToken {
    pub fn new(user_id: Uuid, client_id: String, lifetime: Duration) -> Self {
        let mut bytes = [0u8; 32]; // 256 bits
        OsRng.fill_bytes(&mut bytes);
        let token_str = URL_SAFE_NO_PAD.encode(bytes);
        let now = OffsetDateTime::now_utc();
        Self {
            token: token_str,
            user_id,
            client_id,
            issued_at: now,
            expires_at: now + lifetime,
            revoked: false,
        }
    }
}

/// A single public JSON Web Key (JWK)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicJwk {
    /// JWK key type (e.g. "OKP" for Ed25519, "RSA" for RSA)
    pub kty: String,
    /// Cryptographic curve for the key (e.g. "Ed25519", "P-256")
    pub crv: String,
    /// Algorithm intended for use with the key (e.g. "EdDSA", "RS256")
    pub alg: String,
    /// Unique key identifier used to select this key ("kid" field)
    pub kid: String,
    /// Key usage: "sig" for signatures (as opposed to "enc")
    #[serde(rename = "use")]
    pub use_: String,
    /// Base64url-encoded raw public key bytes
    pub x: String,
}

impl PublicJwk {
    pub fn from_ed25519_bytes(pub_bytes: &[u8], kid: String) -> Self {
        Self {
            kty: "OKP".into(),
            crv: "Ed25519".into(),
            alg: "EdDSA".into(),
            kid,
            use_: "sig".into(),
            x: URL_SAFE_NO_PAD.encode(pub_bytes),
        }
    }
}

/// JWT claims used for client access tokens (valid only on the issuing Home
/// Server).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientAccessClaims {
    /// Token issuer (canonical ServerId; currently `ServerConfig::api_url_with_port()`)
    pub iss: String,
    /// Subject identifier for the user (user UUID)
    pub sub: Uuid,
    /// Intended audience for this token (APIs this token can access)
    pub aud: Vec<String>,
    /// Expiration time as a UNIX timestamp
    pub exp: i64,
    /// Issued-at time as a UNIX timestamp
    pub iat: i64,
    /// Space-separated scopes granted to this token (e.g. "openid")
    pub scope: String,
    /// OAuth2 client identifier that obtained this token (e.g. "default")
    pub client_id: String,
}

impl ClientAccessClaims {
    pub fn new(
        user_id: Uuid,
        client_id: String,
        issuer_server_id: String,
        scope: String,
        lifetime: Duration,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            iss: issuer_server_id.clone(),
            sub: user_id,
            aud: vec![issuer_server_id],
            exp: now + lifetime.whole_seconds(),
            iat: now,
            scope,
            client_id,
        }
    }
}

/// JWT claims used for server-to-server federation requests.
///
/// This token authenticates the **calling server** (`iss` and `sub`).
/// It may optionally include a delegated user identity (`user_id`, `user_domain`)
/// for operations performed "on behalf of" a user.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FederationClaims {
    /// Calling server (canonical ServerId / base URL)
    pub iss: String,
    /// Subject: calling server principal (set equal to `iss` for server authentication)
    pub sub: String,
    /// Target server(s) (canonical ServerId / base URL)
    pub aud: Vec<String>,
    /// Expiration time as a UNIX timestamp
    pub exp: i64,
    /// Issued-at time as a UNIX timestamp
    pub iat: i64,
    /// Optional: Delegated user reference (present when token represents user delegation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_ref: Option<UserRef>,
}

impl FederationClaims {
    /// Create a server-only federation token (no user delegation).
    pub fn new_server_only(
        issuer_server_id: String,
        audience_server_id: String,
        lifetime: Duration,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            iss: issuer_server_id.clone(),
            sub: issuer_server_id,
            aud: vec![audience_server_id],
            exp: now + lifetime.whole_seconds(),
            iat: now,
            user_ref: None,
        }
    }

    /// Create a federation token with explicit user delegation.
    pub fn new_delegated(
        issuer_server_id: String,
        audience_server_id: String,
        user_id: Uuid,
        user_domain: String,
        lifetime: Duration,
    ) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            iss: issuer_server_id.clone(),
            sub: issuer_server_id,
            aud: vec![audience_server_id],
            exp: now + lifetime.whole_seconds(),
            iat: now,
            user_ref: Some(UserRef::new(user_id, user_domain)),
        }
    }
}
