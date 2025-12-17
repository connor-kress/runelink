use ed25519_dalek::{VerifyingKey, pkcs8::EncodePublicKey};

use crate::error::ApiError;

/// Convert a raw Ed25519 public key (32 bytes) into SPKI DER bytes.
///
/// This is useful because JWKS (OKP/Ed25519) represents the public key as raw
/// bytes (`x`), while `jsonwebtoken`'s `DecodingKey::from_ed_der()` expects a
/// DER-encoded public key (SPKI).
pub fn ed25519_public_raw_to_spki_der(raw: &[u8]) -> Result<Vec<u8>, ApiError> {
    let raw_arr: [u8; 32] = raw.try_into().map_err(|_| {
        ApiError::Internal("invalid ed25519 public key length".into())
    })?;
    let vk = VerifyingKey::from_bytes(&raw_arr).map_err(|e| {
        ApiError::Internal(format!("invalid ed25519 public key: {e}"))
    })?;
    let spki = vk.to_public_key_der().map_err(|e| {
        ApiError::Internal(format!("failed to encode public key (spki): {e}"))
    })?;
    Ok(spki.as_bytes().to_vec())
}
