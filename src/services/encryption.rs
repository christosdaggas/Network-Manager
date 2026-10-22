// Network Manager - Profile Encryption
// Copyright (C) 2026 Christos A. Daggas
// SPDX-License-Identifier: MIT

//! Profile data encryption and decryption.
//!
//! Uses AES-256-GCM for authenticated encryption of sensitive profile data.
//! Key derivation uses Argon2id with a random salt (stored alongside ciphertext).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use tracing::debug;
use zeroize::Zeroize;

use crate::tr;

/// Length of the random salt used for key derivation.
const SALT_LEN: usize = 16;
/// Length of the AES-256-GCM nonce.
const NONCE_LEN: usize = 12;

/// Plaintext used to verify a passphrase without decrypting the profile cache.
pub const ENCRYPTION_CHECK_VALUE: &str = "network-manager-profile-encryption-check-v1";

/// Create the persisted verification token for a passphrase.
pub fn make_check_token(passphrase: &str) -> Result<String, EncryptionError> {
    ProfileEncryption::with_key(passphrase).encrypt(ENCRYPTION_CHECK_VALUE)
}

/// Check a passphrase against a persisted verification token.
pub fn verify_check_token(token: &str, passphrase: &str) -> bool {
    ProfileEncryption::with_key(passphrase)
        .decrypt(token)
        .map(|plain| plain == ENCRYPTION_CHECK_VALUE)
        .unwrap_or(false)
}

/// Profile encryption service.
#[allow(dead_code)]
pub struct ProfileEncryption {
    /// Raw key bytes — zeroed on drop via `Zeroize`.
    key_bytes: Option<KeyMaterial>,
}

/// Wrapper around key bytes that zeroes memory on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
struct KeyMaterial([u8; 32]);

impl std::fmt::Debug for KeyMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyMaterial([REDACTED])")
    }
}

impl Default for ProfileEncryption {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl ProfileEncryption {
    /// Create a new encryption service without a key.
    pub fn new() -> Self {
        Self { key_bytes: None }
    }

    /// Create an encryption service with the specified passphrase.
    ///
    /// A random salt is generated per-encryption operation, so this only
    /// stores an in-memory passphrase marker. The real per-message key is
    /// derived at encrypt/decrypt time from that marker and the per-message
    /// salt embedded in the ciphertext.
    pub fn with_key(passphrase: &str) -> Self {
        let mut service = Self::new();
        service.set_key(passphrase);
        service
    }

    /// Set the encryption passphrase.
    ///
    /// The passphrase itself is never stored. We keep a one-way marker in
    /// memory and derive a fresh per-message key with Argon2id and the random
    /// salt that is written alongside every ciphertext.
    pub fn set_key(&mut self, passphrase: &str) {
        if passphrase.is_empty() {
            self.key_bytes = None;
            return;
        }

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(passphrase.as_bytes());
        hasher.update(b"network-manager-profile-encryption-v1");

        let mut key = [0u8; 32];
        key.copy_from_slice(&hasher.finalize());
        self.key_bytes = Some(KeyMaterial(key));
        debug!("Encryption passphrase set successfully");
    }

    /// Clear the encryption key (zeroed via Zeroize on drop).
    pub fn clear_key(&mut self) {
        self.key_bytes = None;
    }

    /// Check if a key is set.
    pub fn has_key(&self) -> bool {
        self.key_bytes.is_some()
    }

    /// Encrypt data and return base64-encoded ciphertext.
    ///
    /// Output format: `base64(salt ‖ nonce ‖ ciphertext)`
    /// where salt is 16 bytes and nonce is 12 bytes.
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        if self.key_bytes.is_none() {
            return Err(EncryptionError::NoKeySet);
        }

        // Generate a random salt for this message
        let mut salt = [0u8; SALT_LEN];
        rand::rngs::OsRng.fill_bytes(&mut salt);

        // Generate a random 96-bit nonce via OsRng (CSPRNG)
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Derive per-message key from passphrase + salt
        // Re-derive using the stored passphrase marker — callers pass passphrase via with_key
        let key = self.key_bytes.as_ref().ok_or(EncryptionError::NoKeySet)?;
        // For backward compat, derive from stored key material + salt
        let mut derived = [0u8; 32];
        Argon2::default()
            .hash_password_into(&key.0, &salt, &mut derived)
            .map_err(|e| EncryptionError::EncryptionFailed(format!("KDF failed: {}", e)))?;

        let cipher = Aes256Gcm::new_from_slice(&derived)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;
        derived.zeroize();

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Prepend salt + nonce to ciphertext and encode as base64
        let mut result = salt.to_vec();
        result.extend_from_slice(&nonce_bytes);
        result.extend(ciphertext);

        Ok(BASE64.encode(&result))
    }

    /// Decrypt base64-encoded ciphertext.
    ///
    /// Expects format: `base64(salt ‖ nonce ‖ ciphertext)`.
    /// Falls back to legacy format (nonce-only, 12 bytes) for backward compatibility.
    pub fn decrypt(&self, ciphertext: &str) -> Result<String, EncryptionError> {
        if self.key_bytes.is_none() {
            return Err(EncryptionError::NoKeySet);
        }

        // Decode base64
        let data = BASE64
            .decode(ciphertext)
            .map_err(|e| EncryptionError::InvalidData(e.to_string()))?;

        let key = self.key_bytes.as_ref().ok_or(EncryptionError::NoKeySet)?;

        // Try new format first: salt(16) + nonce(12) + ciphertext
        if data.len() > SALT_LEN + NONCE_LEN {
            let (salt, rest) = data.split_at(SALT_LEN);
            let (nonce_bytes, encrypted) = rest.split_at(NONCE_LEN);
            let nonce = Nonce::from_slice(nonce_bytes);

            let mut derived = [0u8; 32];
            if Argon2::default()
                .hash_password_into(&key.0, salt, &mut derived)
                .is_ok()
            {
                if let Ok(cipher) = Aes256Gcm::new_from_slice(&derived) {
                    derived.zeroize();
                    if let Ok(plaintext) = cipher.decrypt(nonce, encrypted) {
                        return String::from_utf8(plaintext)
                            .map_err(|e| EncryptionError::InvalidData(e.to_string()));
                    }
                }
                derived.zeroize();
            }
        }

        // Fallback: legacy format — nonce(12) + ciphertext (SHA-256 direct key)
        if data.len() > NONCE_LEN {
            let (nonce_bytes, encrypted) = data.split_at(NONCE_LEN);
            let nonce = Nonce::from_slice(nonce_bytes);

            // Legacy key derivation: SHA-256 of the stored key material
            if let Ok(cipher) = Aes256Gcm::new_from_slice(&key.0) {
                if let Ok(plaintext) = cipher.decrypt(nonce, encrypted) {
                    return String::from_utf8(plaintext)
                        .map_err(|e| EncryptionError::InvalidData(e.to_string()));
                }
            }
        }

        Err(EncryptionError::DecryptionFailed)
    }

    /// Encrypt a JSON-serializable value.
    pub fn encrypt_json<T: serde::Serialize>(&self, value: &T) -> Result<String, EncryptionError> {
        let json = serde_json::to_string(value)
            .map_err(|e| EncryptionError::SerializationFailed(e.to_string()))?;
        self.encrypt(&json)
    }

    /// Decrypt to a JSON-deserializable value.
    pub fn decrypt_json<T: serde::de::DeserializeOwned>(
        &self,
        ciphertext: &str,
    ) -> Result<T, EncryptionError> {
        let json = self.decrypt(ciphertext)?;
        serde_json::from_str(&json).map_err(|e| EncryptionError::SerializationFailed(e.to_string()))
    }
}

/// Errors that can occur during encryption/decryption.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum EncryptionError {
    /// No encryption key has been set.
    NoKeySet,
    /// Encryption operation failed.
    EncryptionFailed(String),
    /// Decryption failed (wrong key or corrupted data).
    DecryptionFailed,
    /// Invalid input data.
    InvalidData(String),
    /// Serialization/deserialization failed.
    SerializationFailed(String),
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoKeySet => write!(f, "No encryption key set"),
            Self::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            Self::DecryptionFailed => write!(f, "Decryption failed (wrong key or corrupted data)"),
            Self::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            Self::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
        }
    }
}

impl std::error::Error for EncryptionError {}

impl EncryptionError {
    /// The error as translated text for the UI. `Display` stays in English
    /// because it is also written to the log.
    pub fn user_message(&self) -> String {
        match self {
            Self::NoKeySet => tr("No encryption key set"),
            Self::EncryptionFailed(msg) => tr("Encryption failed: {error}").replace("{error}", msg),
            Self::DecryptionFailed => tr("Decryption failed (wrong key or corrupted data)"),
            Self::InvalidData(msg) => tr("Invalid data: {error}").replace("{error}", msg),
            Self::SerializationFailed(msg) => {
                tr("Serialization failed: {error}").replace("{error}", msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let enc = ProfileEncryption::with_key("test-password");
        let plaintext = "Hello, World!";

        let encrypted = enc.encrypt(plaintext).expect("encryption failed");
        let decrypted = enc.decrypt(&encrypted).expect("decryption failed");

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_wrong_key() {
        let enc1 = ProfileEncryption::with_key("password1");
        let enc2 = ProfileEncryption::with_key("password2");

        let encrypted = enc1.encrypt("secret data").expect("encryption failed");
        let result = enc2.decrypt(&encrypted);

        assert!(result.is_err());
    }

    #[test]
    fn test_no_key() {
        let enc = ProfileEncryption::new();
        let result = enc.encrypt("test");

        assert!(matches!(result, Err(EncryptionError::NoKeySet)));
    }

    #[test]
    fn test_check_token_verifies_passphrase() {
        let token = make_check_token("right-passphrase").expect("token");
        assert!(verify_check_token(&token, "right-passphrase"));
        assert!(!verify_check_token(&token, "wrong-passphrase"));
    }
}
