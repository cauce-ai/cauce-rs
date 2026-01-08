//! Encrypted type for the Cauce Protocol.
//!
//! This module provides the [`Encrypted`] envelope and [`EncryptionAlgorithm`] enum
//! for end-to-end encryption support.

use serde::{Deserialize, Serialize};

/// Supported end-to-end encryption algorithms.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `X25519XSalsa20Poly1305` → `"x25519_xsalsa20_poly1305"`
/// - `A256Gcm` → `"a256gcm"`
/// - `XChaCha20Poly1305` → `"xchacha20_poly1305"`
///
/// # Example
///
/// ```
/// use cauce_core::types::EncryptionAlgorithm;
///
/// let algo = EncryptionAlgorithm::X25519XSalsa20Poly1305;
/// let json = serde_json::to_string(&algo).unwrap();
/// assert_eq!(json, "\"x25519_xsalsa20_poly1305\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    /// NaCl box: X25519 key exchange + XSalsa20-Poly1305
    #[serde(rename = "x25519_xsalsa20_poly1305")]
    X25519XSalsa20Poly1305,

    /// AES-256-GCM
    #[serde(rename = "a256gcm")]
    A256Gcm,

    /// XChaCha20-Poly1305
    #[serde(rename = "xchacha20_poly1305")]
    XChaCha20Poly1305,
}

/// End-to-end encryption envelope.
///
/// The Encrypted struct contains all the information needed to
/// decrypt an end-to-end encrypted payload.
///
/// # Fields
///
/// - `algorithm` - The encryption algorithm used
/// - `recipient_public_key` - Base64-encoded recipient public key
/// - `nonce` - Base64-encoded nonce/IV
/// - `ciphertext` - Base64-encoded encrypted payload
///
/// # Example
///
/// ```
/// use cauce_core::types::{Encrypted, EncryptionAlgorithm};
///
/// let encrypted = Encrypted {
///     algorithm: EncryptionAlgorithm::X25519XSalsa20Poly1305,
///     recipient_public_key: "base64_encoded_public_key".to_string(),
///     nonce: "base64_encoded_nonce".to_string(),
///     ciphertext: "base64_encoded_ciphertext".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encrypted {
    /// Encryption algorithm used
    pub algorithm: EncryptionAlgorithm,

    /// Base64-encoded recipient public key
    pub recipient_public_key: String,

    /// Base64-encoded nonce/IV
    pub nonce: String,

    /// Base64-encoded encrypted payload
    pub ciphertext: String,
}

impl Encrypted {
    /// Creates a new Encrypted envelope.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The encryption algorithm
    /// * `recipient_public_key` - Base64-encoded recipient public key
    /// * `nonce` - Base64-encoded nonce/IV
    /// * `ciphertext` - Base64-encoded encrypted payload
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::{Encrypted, EncryptionAlgorithm};
    ///
    /// let encrypted = Encrypted::new(
    ///     EncryptionAlgorithm::A256Gcm,
    ///     "recipient_key",
    ///     "nonce_value",
    ///     "encrypted_data",
    /// );
    /// assert_eq!(encrypted.algorithm, EncryptionAlgorithm::A256Gcm);
    /// ```
    pub fn new(
        algorithm: EncryptionAlgorithm,
        recipient_public_key: impl Into<String>,
        nonce: impl Into<String>,
        ciphertext: impl Into<String>,
    ) -> Self {
        Self {
            algorithm,
            recipient_public_key: recipient_public_key.into(),
            nonce: nonce.into(),
            ciphertext: ciphertext.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_algorithm_serialization() {
        assert_eq!(
            serde_json::to_string(&EncryptionAlgorithm::X25519XSalsa20Poly1305).unwrap(),
            "\"x25519_xsalsa20_poly1305\""
        );
        assert_eq!(
            serde_json::to_string(&EncryptionAlgorithm::A256Gcm).unwrap(),
            "\"a256gcm\""
        );
        assert_eq!(
            serde_json::to_string(&EncryptionAlgorithm::XChaCha20Poly1305).unwrap(),
            "\"xchacha20_poly1305\""
        );
    }

    #[test]
    fn test_encryption_algorithm_deserialization() {
        assert_eq!(
            serde_json::from_str::<EncryptionAlgorithm>("\"x25519_xsalsa20_poly1305\"").unwrap(),
            EncryptionAlgorithm::X25519XSalsa20Poly1305
        );
        assert_eq!(
            serde_json::from_str::<EncryptionAlgorithm>("\"a256gcm\"").unwrap(),
            EncryptionAlgorithm::A256Gcm
        );
        assert_eq!(
            serde_json::from_str::<EncryptionAlgorithm>("\"xchacha20_poly1305\"").unwrap(),
            EncryptionAlgorithm::XChaCha20Poly1305
        );
    }

    #[test]
    fn test_encrypted_new() {
        let encrypted = Encrypted::new(
            EncryptionAlgorithm::X25519XSalsa20Poly1305,
            "pubkey123",
            "nonce456",
            "ciphertext789",
        );

        assert_eq!(
            encrypted.algorithm,
            EncryptionAlgorithm::X25519XSalsa20Poly1305
        );
        assert_eq!(encrypted.recipient_public_key, "pubkey123");
        assert_eq!(encrypted.nonce, "nonce456");
        assert_eq!(encrypted.ciphertext, "ciphertext789");
    }

    #[test]
    fn test_encrypted_serialization() {
        let encrypted = Encrypted {
            algorithm: EncryptionAlgorithm::A256Gcm,
            recipient_public_key: "key".to_string(),
            nonce: "iv".to_string(),
            ciphertext: "data".to_string(),
        };

        let json = serde_json::to_string(&encrypted).unwrap();
        assert!(json.contains("\"algorithm\":\"a256gcm\""));
        assert!(json.contains("\"recipient_public_key\":\"key\""));
        assert!(json.contains("\"nonce\":\"iv\""));
        assert!(json.contains("\"ciphertext\":\"data\""));
    }

    #[test]
    fn test_encrypted_deserialization() {
        let json = r#"{
            "algorithm": "xchacha20_poly1305",
            "recipient_public_key": "pk",
            "nonce": "n",
            "ciphertext": "ct"
        }"#;

        let encrypted: Encrypted = serde_json::from_str(json).unwrap();
        assert_eq!(encrypted.algorithm, EncryptionAlgorithm::XChaCha20Poly1305);
        assert_eq!(encrypted.recipient_public_key, "pk");
        assert_eq!(encrypted.nonce, "n");
        assert_eq!(encrypted.ciphertext, "ct");
    }

    #[test]
    fn test_encrypted_round_trip() {
        let encrypted = Encrypted::new(
            EncryptionAlgorithm::X25519XSalsa20Poly1305,
            "base64_public_key_here",
            "base64_nonce_here",
            "base64_ciphertext_here",
        );

        let json = serde_json::to_string(&encrypted).unwrap();
        let restored: Encrypted = serde_json::from_str(&json).unwrap();
        assert_eq!(encrypted, restored);
    }

    #[test]
    fn test_encryption_algorithm_copy() {
        let algo = EncryptionAlgorithm::A256Gcm;
        let copy = algo;
        assert_eq!(algo, copy);
    }

    #[test]
    fn test_encryption_algorithm_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(EncryptionAlgorithm::A256Gcm);
        set.insert(EncryptionAlgorithm::A256Gcm);
        assert_eq!(set.len(), 1);

        set.insert(EncryptionAlgorithm::XChaCha20Poly1305);
        assert_eq!(set.len(), 2);
    }
}
