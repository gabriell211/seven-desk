#![forbid(unsafe_code)]

//! Persistent Ed25519 device identity. Secrets never enter JavaScript, logs,
//! server metadata or local files outside the OS credential store.

use ed25519_dalek::{Signer, SigningKey};
use seven_protocol::DeviceId;
use seven_security::{RequestProof, device_id, message_to_sign};
use thiserror::Error;
use uuid::Uuid;

const KEY_SERVICE: &str = "dev.seven.desk";
const KEY_USER: &str = "device-signing-key-v1";

#[derive(Debug, Error, Eq, PartialEq)]
pub enum IdentityError {
    #[error("the operating system credential store is unavailable")]
    StoreUnavailable,
    #[error("the stored device key is malformed")]
    CorruptKey,
    #[error("a secure random key could not be generated")]
    RandomUnavailable,
    #[error("failed to prepare the signed request")]
    InvalidRequest,
}

pub trait SecretStore {
    fn read(&self) -> Result<Option<[u8; 32]>, IdentityError>;
    fn write(&self, secret: &[u8; 32]) -> Result<(), IdentityError>;
}

pub struct SystemKeyStore;

impl SecretStore for SystemKeyStore {
    fn read(&self) -> Result<Option<[u8; 32]>, IdentityError> {
        let entry = keyring::Entry::new(KEY_SERVICE, KEY_USER)
            .map_err(|_| IdentityError::StoreUnavailable)?;
        match entry.get_secret() {
            Ok(secret) => {
                let key = secret.try_into().map_err(|_| IdentityError::CorruptKey)?;
                Ok(Some(key))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(IdentityError::StoreUnavailable),
        }
    }

    fn write(&self, secret: &[u8; 32]) -> Result<(), IdentityError> {
        let entry = keyring::Entry::new(KEY_SERVICE, KEY_USER)
            .map_err(|_| IdentityError::StoreUnavailable)?;
        entry
            .set_secret(secret)
            .map_err(|_| IdentityError::StoreUnavailable)
    }
}

pub struct Identity {
    key: SigningKey,
}

impl Identity {
    pub fn load(store: &impl SecretStore) -> Result<Option<Self>, IdentityError> {
        store.read().map(|seed| {
            seed.map(|bytes| Self {
                key: SigningKey::from_bytes(&bytes),
            })
        })
    }

    pub fn load_or_create(store: &impl SecretStore) -> Result<Self, IdentityError> {
        if let Some(identity) = Self::load(store)? {
            return Ok(identity);
        }
        let mut seed = [0_u8; 32];
        getrandom::fill(&mut seed).map_err(|_| IdentityError::RandomUnavailable)?;
        let key = SigningKey::from_bytes(&seed);
        store.write(&seed)?;
        seed.fill(0);
        Ok(Self { key })
    }

    pub fn id(&self) -> DeviceId {
        device_id(&self.key.verifying_key())
    }

    pub fn public_key_hex(&self) -> String {
        hex::encode(self.key.verifying_key().as_bytes())
    }

    pub fn sign(
        &self,
        method: &str,
        path: &str,
        body: &[u8],
        nonce: Uuid,
        timestamp_ms: u64,
    ) -> Result<RequestProof, IdentityError> {
        let data = message_to_sign(method, path, body, nonce, timestamp_ms)
            .map_err(|_| IdentityError::InvalidRequest)?;
        let signature = self.key.sign(&data);
        Ok(RequestProof {
            public_key_hex: self.public_key_hex(),
            signature_hex: hex::encode(signature.to_bytes()),
            nonce,
            timestamp_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use seven_security::{NonceWindow, verify};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStore(Mutex<Option<[u8; 32]>>);

    impl SecretStore for MemoryStore {
        fn read(&self) -> Result<Option<[u8; 32]>, IdentityError> {
            self.0
                .lock()
                .map(|v| *v)
                .map_err(|_| IdentityError::StoreUnavailable)
        }
        fn write(&self, value: &[u8; 32]) -> Result<(), IdentityError> {
            *self.0.lock().map_err(|_| IdentityError::StoreUnavailable)? = Some(*value);
            Ok(())
        }
    }

    #[test]
    fn creates_identity_once_and_keeps_id_after_reload() {
        let store = MemoryStore::default();
        assert!(Identity::load(&store).unwrap().is_none());
        let first = Identity::load_or_create(&store).unwrap();
        let second = Identity::load_or_create(&store).unwrap();
        assert_eq!(first.id(), second.id());
        assert_ne!(first.public_key_hex(), String::new());
    }

    #[test]
    fn signed_proof_interoperates_with_security_verifier() {
        let store = MemoryStore::default();
        let identity = Identity::load_or_create(&store).unwrap();
        let proof = identity
            .sign("POST", "/v1/devices", b"{}", Uuid::new_v4(), 10_000)
            .unwrap();
        let (id, _) = verify(
            &proof,
            "POST",
            "/v1/devices",
            b"{}",
            10_000,
            &NonceWindow::default(),
        )
        .unwrap();
        assert_eq!(id, identity.id());
    }

    #[test]
    fn does_not_silently_reset_invalid_key_material() {
        struct Corrupt;
        impl SecretStore for Corrupt {
            fn read(&self) -> Result<Option<[u8; 32]>, IdentityError> {
                Err(IdentityError::CorruptKey)
            }
            fn write(&self, _: &[u8; 32]) -> Result<(), IdentityError> {
                Ok(())
            }
        }
        assert!(matches!(
            Identity::load_or_create(&Corrupt),
            Err(IdentityError::CorruptKey)
        ));
    }
}
