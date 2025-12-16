use ed25519_dalek::SigningKey;
use jsonwebtoken::{DecodingKey, EncodingKey};
use rand::rngs::OsRng;
use runelink_types::auth::PublicJwk;
use std::fs;
use std::path::PathBuf;

use crate::error::ApiError;

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
    pub fn load_or_generate(path: PathBuf) -> Result<Self, ApiError> {
        // TODO: we are storing the raw key bytes, not DER and not PKCS#8
        // TODO: We will also have to update the serialization code in the /tokens endpoint
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

            let kid = "primary".to_string(); // TODO: should this change?
            Ok(Self {
                private_key: EncodingKey::from_ed_der(&priv_bytes),
                decoding_key: DecodingKey::from_ed_der(&pub_bytes),
                public_jwk: PublicJwk::from_ed25519_bytes(&pub_bytes, kid),
                path,
            })
        } else {
            // Generate new keypair
            let signing_key = SigningKey::generate(&mut OsRng);
            let verify_key = signing_key.verifying_key();
            let priv_bytes = signing_key.to_bytes();
            let pub_bytes = verify_key.to_bytes();

            fs::create_dir_all(&path).map_err(|e| {
                ApiError::Internal(format!("failed to create keys dir: {e}"))
            })?;
            fs::write(&priv_path, &priv_bytes).map_err(|e| {
                ApiError::Internal(format!("failed to write private key: {e}"))
            })?;
            fs::write(&pub_path, &pub_bytes).map_err(|e| {
                ApiError::Internal(format!("failed to write public key: {e}"))
            })?;
            println!("Generated new ed25519 keypair");

            let kid = "primary".to_string(); // TODO: should this change?
            Ok(Self {
                private_key: EncodingKey::from_ed_der(&priv_bytes),
                decoding_key: DecodingKey::from_ed_der(&pub_bytes),
                public_jwk: PublicJwk::from_ed25519_bytes(&pub_bytes, kid),
                path,
            })
        }
    }
}
