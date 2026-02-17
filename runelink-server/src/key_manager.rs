use ed25519_dalek::{
    SigningKey, VerifyingKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey,
    },
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use rand::rngs::OsRng;
use runelink_types::{FederationClaims, UserRef, auth::PublicJwk};
use std::fs;
use std::path::PathBuf;
use time::Duration;

use crate::error::{ApiError, ApiResult};

/// Handles JWT signing keys and JWKS publication
#[allow(dead_code)]
#[derive(Clone)]
pub struct KeyManager {
    pub private_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub public_jwk: PublicJwk,
    pub path: PathBuf,
}

impl std::fmt::Debug for KeyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyManager")
            .field("private_key", &"[REDACTED]")
            .field("public_jwk", &self.public_jwk)
            .field("path", &self.path)
            .finish()
    }
}

impl KeyManager {
    /// Load keys if they exist under `path` or generate a new Ed25519 keypair
    pub fn load_or_generate(path: PathBuf) -> ApiResult<Self> {
        // Stored formats:
        // - private: PKCS#8 DER
        // - public:  SPKI (SubjectPublicKeyInfo) DER
        //
        // Note: jsonwebtoken's EdDSA verifier expects the *raw 32-byte*
        // Ed25519 public key, so we parse SPKI and convert to raw
        // for `DecodingKey`.
        let priv_path = path.join("private_ed25519.der");
        let pub_path = path.join("public_ed25519.der");

        if priv_path.exists() && pub_path.exists() {
            // Load from disk
            let priv_bytes = fs::read(&priv_path).map_err(|e| {
                ApiError::Internal(format!("failed to read private key: {e}"))
            })?;
            let pub_bytes = fs::read(&pub_path).map_err(|e| {
                ApiError::Internal(format!("failed to read public key: {e}"))
            })?;

            // Parse private key as PKCS#8 DER
            let signing_key =
                SigningKey::from_pkcs8_der(&priv_bytes).map_err(|e| {
                    ApiError::Internal(format!(
                        "invalid private key (expected PKCS#8 DER): {e}"
                    ))
                })?;

            // Parse public key as SPKI DER and convert to raw 32 bytes
            // for jsonwebtoken.
            let loaded_pub: [u8; 32] =
                VerifyingKey::from_public_key_der(&pub_bytes)
                    .map_err(|e| {
                        ApiError::Internal(format!(
                            "invalid public key (expected SPKI DER): {e}"
                        ))
                    })?
                    .to_bytes();

            // Ensure the public key matches the private key
            let derived_pub = signing_key.verifying_key().to_bytes();
            if derived_pub != loaded_pub {
                return Err(ApiError::Internal(
                    "public key does not match private key".into(),
                ));
            }

            let kid = "primary".to_string(); // TODO: should this change?
            Ok(Self {
                private_key: EncodingKey::from_ed_der(&priv_bytes),
                decoding_key: DecodingKey::from_ed_der(&loaded_pub),
                public_jwk: PublicJwk::from_ed25519_bytes(&loaded_pub, kid),
                path,
            })
        } else {
            // Generate new keypair
            let signing_key = SigningKey::generate(&mut OsRng);
            let verify_key = signing_key.verifying_key();
            let priv_pkcs8 = signing_key.to_pkcs8_der().map_err(|e| {
                ApiError::Internal(format!(
                    "failed to encode private key (pkcs8): {e}"
                ))
            })?;
            let pub_spki = verify_key.to_public_key_der().map_err(|e| {
                ApiError::Internal(format!(
                    "failed to encode public key (spki): {e}"
                ))
            })?;
            let pub_raw = verify_key.to_bytes();

            fs::create_dir_all(&path).map_err(|e| {
                ApiError::Internal(format!("failed to create keys dir: {e}"))
            })?;
            fs::write(&priv_path, priv_pkcs8.as_bytes()).map_err(|e| {
                ApiError::Internal(format!("failed to write private key: {e}"))
            })?;
            fs::write(&pub_path, pub_spki.as_bytes()).map_err(|e| {
                ApiError::Internal(format!("failed to write public key: {e}"))
            })?;
            println!("Generated new ed25519 keypair at {:?}", path);

            let kid = "primary".to_string(); // TODO: should this change?

            Ok(Self {
                private_key: EncodingKey::from_ed_der(priv_pkcs8.as_bytes()),
                decoding_key: DecodingKey::from_ed_der(&pub_raw),
                public_jwk: PublicJwk::from_ed25519_bytes(&pub_raw, kid),
                path,
            })
        }
    }

    /// Issue a short-lived server-only federation JWT (no user delegation).
    #[allow(dead_code)]
    pub fn issue_federation_jwt_server_only(
        &self,
        issuer_server_id: String,
        audience_server_id: String,
    ) -> ApiResult<String> {
        let lifetime = Duration::minutes(5); // Short-lived for s2s
        let claims = FederationClaims::new_server_only(
            issuer_server_id,
            audience_server_id,
            lifetime,
        );

        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::EdDSA),
            &claims,
            &self.private_key,
        )
        .map_err(|e| ApiError::Internal(format!("jwt error: {e}")))?;

        Ok(token)
    }

    /// Issue a short-lived federation JWT with explicit user delegation.
    pub fn issue_federation_jwt_delegated(
        &self,
        issuer_server_id: String,
        audience_server_id: String,
        user_ref: UserRef,
    ) -> ApiResult<String> {
        let lifetime = Duration::minutes(5); // Short-lived for s2s
        let claims = FederationClaims::new_delegated(
            issuer_server_id,
            audience_server_id,
            user_ref,
            lifetime,
        );

        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::EdDSA),
            &claims,
            &self.private_key,
        )
        .map_err(|e| ApiError::Internal(format!("jwt error: {e}")))?;

        Ok(token)
    }
}
