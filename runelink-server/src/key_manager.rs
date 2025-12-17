use ed25519_dalek::{
    SigningKey, VerifyingKey,
    pkcs8::{
        DecodePrivateKey, DecodePublicKey, EncodePrivateKey, EncodePublicKey,
    },
};
use jsonwebtoken::{DecodingKey, EncodingKey};
use rand::rngs::OsRng;
use runelink_types::auth::PublicJwk;
use std::fs;
use std::path::PathBuf;

use crate::{crypto::ed25519_public_raw_to_spki_der, error::ApiError};

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
        // Store keys in standard formats:
        // - private: PKCS#8 DER
        // - public:  SPKI (SubjectPublicKeyInfo) DER
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

            // Parse keys as PKCS#8 (private) and SPKI (public) DER
            let signing_key =
                SigningKey::from_pkcs8_der(&priv_bytes).map_err(|e| {
                    ApiError::Internal(format!(
                        "invalid private key (expected PKCS#8 DER): {e}"
                    ))
                })?;
            let verifying_key = VerifyingKey::from_public_key_der(&pub_bytes)
                .map_err(|e| {
                ApiError::Internal(format!(
                    "invalid public key (expected SPKI DER): {e}"
                ))
            })?;
            // Ensure the public key matches the private key (avoids confusing runtime failures).
            let derived_pub = signing_key.verifying_key().to_bytes();
            let loaded_pub = verifying_key.to_bytes();
            if derived_pub != loaded_pub {
                return Err(ApiError::Internal(
                    "public key does not match private key".into(),
                ));
            }

            // Ensure we can encode the public key to SPKI DER, since downstream JWT verification
            // expects SPKI DER (via jsonwebtoken's from_ed_der()).
            let _ = ed25519_public_raw_to_spki_der(&loaded_pub)?;

            let kid = "primary".to_string(); // TODO: should this change?
            Ok(Self {
                private_key: EncodingKey::from_ed_der(&priv_bytes),
                decoding_key: DecodingKey::from_ed_der(&pub_bytes),
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
            println!("Generated new ed25519 keypair");

            let kid = "primary".to_string(); // TODO: should this change?
            Ok(Self {
                private_key: EncodingKey::from_ed_der(priv_pkcs8.as_bytes()),
                decoding_key: DecodingKey::from_ed_der(pub_spki.as_bytes()),
                public_jwk: PublicJwk::from_ed25519_bytes(&pub_raw, kid),
                path,
            })
        }
    }
}
