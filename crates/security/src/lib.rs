#![forbid(unsafe_code)]

//! Verification of signed requests. No API endpoint trusts a public device ID
//! without proving possession of the corresponding Ed25519 private key.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use seven_protocol::DeviceId;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Mutex};
use thiserror::Error;
use uuid::Uuid;

const MAX_CLOCK_SKEW_MS: u64 = 60_000;
// A signed PNG frame may be as large as 2 MiB. HTTP handlers still enforce
// their own, usually much smaller, per-endpoint limits before verification.
const MAX_BODY_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct RequestProof {
    pub public_key_hex: String,
    pub signature_hex: String,
    pub nonce: Uuid,
    pub timestamp_ms: u64,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AuthError {
    #[error("invalid public key or signature")]
    InvalidSignature,
    #[error("invalid request timestamp")]
    ClockSkew,
    #[error("request nonce has already been used")]
    Replay,
    #[error("request body exceeds maximum size")]
    Oversized,
    #[error("internal verification state unavailable")]
    Internal,
}

#[derive(Default)]
pub struct NonceWindow {
    seen: Mutex<HashMap<(DeviceId, Uuid), u64>>,
}

pub fn device_id(key: &VerifyingKey) -> DeviceId {
    let digest = Sha256::digest(key.as_bytes());
    let mut id = [0_u8; 16];
    id.copy_from_slice(&digest[..16]);
    DeviceId(Uuid::from_bytes(id))
}

pub fn message_to_sign(
    method: &str,
    path: &str,
    body: &[u8],
    nonce: Uuid,
    timestamp_ms: u64,
) -> Result<Vec<u8>, AuthError> {
    if body.len() > MAX_BODY_BYTES {
        return Err(AuthError::Oversized);
    }
    let digest = Sha256::digest(body);
    Ok(format!(
        "seven-desk/request/v1\n{method}\n{path}\n{timestamp_ms}\n{nonce}\n{}",
        hex::encode(digest)
    )
    .into_bytes())
}

pub fn verify(
    proof: &RequestProof,
    method: &str,
    path: &str,
    body: &[u8],
    now_ms: u64,
    nonce_window: &NonceWindow,
) -> Result<(DeviceId, VerifyingKey), AuthError> {
    if now_ms.abs_diff(proof.timestamp_ms) > MAX_CLOCK_SKEW_MS {
        return Err(AuthError::ClockSkew);
    }
    let key_bytes = hex::decode(&proof.public_key_hex).map_err(|_| AuthError::InvalidSignature)?;
    let key: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| AuthError::InvalidSignature)?;
    let verifier = VerifyingKey::from_bytes(&key).map_err(|_| AuthError::InvalidSignature)?;
    let sig_bytes = hex::decode(&proof.signature_hex).map_err(|_| AuthError::InvalidSignature)?;
    let signature = Signature::from_slice(&sig_bytes).map_err(|_| AuthError::InvalidSignature)?;
    let signed = message_to_sign(method, path, body, proof.nonce, proof.timestamp_ms)?;
    verifier
        .verify(&signed, &signature)
        .map_err(|_| AuthError::InvalidSignature)?;
    let id = device_id(&verifier);
    nonce_window.use_once(id, proof.nonce, now_ms)?;
    Ok((id, verifier))
}

impl NonceWindow {
    fn use_once(&self, device_id: DeviceId, nonce: Uuid, now_ms: u64) -> Result<(), AuthError> {
        let mut seen = self.seen.lock().map_err(|_| AuthError::Internal)?;
        seen.retain(|_, timestamp| now_ms.saturating_sub(*timestamp) <= MAX_CLOCK_SKEW_MS * 2);
        if seen.contains_key(&(device_id, nonce)) {
            return Err(AuthError::Replay);
        }
        seen.insert((device_id, nonce), now_ms);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn sign(key: &SigningKey, method: &str, path: &str, body: &[u8], ts: u64) -> RequestProof {
        let nonce = Uuid::new_v4();
        let msg = message_to_sign(method, path, body, nonce, ts).unwrap();
        RequestProof {
            public_key_hex: hex::encode(key.verifying_key().as_bytes()),
            signature_hex: hex::encode(key.sign(&msg).to_bytes()),
            timestamp_ms: ts,
            nonce,
        }
    }

    #[test]
    fn authenticates_then_prevents_replay() {
        let key = SigningKey::from_bytes(&[7_u8; 32]);
        let replay = NonceWindow::default();
        let proof = sign(&key, "POST", "/v1/devices", br#"{}"#, 10_000);
        let result = verify(&proof, "POST", "/v1/devices", br#"{}"#, 10_000, &replay);
        assert_eq!(result.unwrap().0, device_id(&key.verifying_key()));
        assert_eq!(
            verify(&proof, "POST", "/v1/devices", br#"{}"#, 10_000, &replay).unwrap_err(),
            AuthError::Replay
        );
    }

    #[test]
    fn rejects_request_tampering_and_clock_skew() {
        let key = SigningKey::from_bytes(&[9_u8; 32]);
        let replay = NonceWindow::default();
        let proof = sign(&key, "POST", "/v1/sessions", br#"{}"#, 99_999);
        assert_eq!(
            verify(
                &proof,
                "POST",
                "/v1/sessions",
                br#"{"a":1}"#,
                99_999,
                &replay
            )
            .unwrap_err(),
            AuthError::InvalidSignature
        );
        assert_eq!(
            verify(&proof, "POST", "/v1/sessions", br#"{}"#, 180_001, &replay).unwrap_err(),
            AuthError::ClockSkew
        );
    }

    #[test]
    fn endpoint_name_is_covered_by_signature() {
        let key = SigningKey::from_bytes(&[11_u8; 32]);
        let replay = NonceWindow::default();
        let proof = sign(&key, "GET", "/v1/sessions/one", &[], 1_000);
        assert_eq!(
            verify(&proof, "GET", "/v1/sessions/two", &[], 1_000, &replay).unwrap_err(),
            AuthError::InvalidSignature
        );
    }

    #[test]
    fn signs_bounded_frame_bodies_but_rejects_larger_payloads() {
        let key = SigningKey::from_bytes(&[13_u8; 32]);
        let body = vec![0x5a; 64 * 1024 + 1];
        let proof = sign(&key, "POST", "/v1/sessions/frame", &body, 1_000);
        assert!(
            verify(
                &proof,
                "POST",
                "/v1/sessions/frame",
                &body,
                1_000,
                &NonceWindow::default(),
            )
            .is_ok()
        );
        assert_eq!(
            message_to_sign(
                "POST",
                "/v1/sessions/frame",
                &vec![0; MAX_BODY_BYTES + 1],
                Uuid::new_v4(),
                1_000
            ),
            Err(AuthError::Oversized)
        );
    }
}
