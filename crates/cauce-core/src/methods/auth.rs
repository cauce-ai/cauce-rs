//! Authentication types for the Cauce Protocol.
//!
//! This module provides authentication-related types used in the Hello handshake.

use serde::{Deserialize, Serialize};

/// Authentication type for client connections.
///
/// Indicates the method of authentication used by a client.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `Bearer` → `"bearer"`
/// - `ApiKey` → `"api_key"`
/// - `Mtls` → `"mtls"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    /// Bearer token authentication (OAuth2, JWT)
    Bearer,
    /// API key authentication
    ApiKey,
    /// Mutual TLS (client certificate) authentication
    Mtls,
}

/// Authentication credentials for client connections.
///
/// Contains the authentication type and corresponding credentials.
/// The fields that should be populated depend on the `type_` value:
///
/// - `Bearer` → `token` should be `Some`
/// - `ApiKey` → `api_key` should be `Some`
/// - `Mtls` → Both may be `None` (cert-based auth)
///
/// # Example
///
/// ```
/// use cauce_core::methods::{Auth, AuthType};
///
/// // API key authentication
/// let auth = Auth {
///     type_: AuthType::ApiKey,
///     token: None,
///     api_key: Some("my-api-key".to_string()),
/// };
///
/// // Bearer token authentication
/// let auth = Auth {
///     type_: AuthType::Bearer,
///     token: Some("eyJ...".to_string()),
///     api_key: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Auth {
    /// The authentication type
    #[serde(rename = "type")]
    pub type_: AuthType,

    /// Bearer token (for `AuthType::Bearer`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// API key (for `AuthType::ApiKey`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
}

impl Auth {
    /// Creates a new Bearer token authentication.
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            type_: AuthType::Bearer,
            token: Some(token.into()),
            api_key: None,
        }
    }

    /// Creates a new API key authentication.
    pub fn api_key(key: impl Into<String>) -> Self {
        Self {
            type_: AuthType::ApiKey,
            token: None,
            api_key: Some(key.into()),
        }
    }

    /// Creates a new mTLS authentication.
    pub fn mtls() -> Self {
        Self {
            type_: AuthType::Mtls,
            token: None,
            api_key: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== AuthType Tests =====

    #[test]
    fn test_auth_type_serialization() {
        assert_eq!(
            serde_json::to_string(&AuthType::Bearer).unwrap(),
            "\"bearer\""
        );
        assert_eq!(
            serde_json::to_string(&AuthType::ApiKey).unwrap(),
            "\"api_key\""
        );
        assert_eq!(serde_json::to_string(&AuthType::Mtls).unwrap(), "\"mtls\"");
    }

    #[test]
    fn test_auth_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<AuthType>("\"bearer\"").unwrap(),
            AuthType::Bearer
        );
        assert_eq!(
            serde_json::from_str::<AuthType>("\"api_key\"").unwrap(),
            AuthType::ApiKey
        );
        assert_eq!(
            serde_json::from_str::<AuthType>("\"mtls\"").unwrap(),
            AuthType::Mtls
        );
    }

    #[test]
    fn test_auth_type_roundtrip() {
        for auth_type in [AuthType::Bearer, AuthType::ApiKey, AuthType::Mtls] {
            let json = serde_json::to_string(&auth_type).unwrap();
            let restored: AuthType = serde_json::from_str(&json).unwrap();
            assert_eq!(auth_type, restored);
        }
    }

    // ===== Auth Tests =====

    #[test]
    fn test_auth_bearer() {
        let auth = Auth::bearer("my-token");
        assert_eq!(auth.type_, AuthType::Bearer);
        assert_eq!(auth.token, Some("my-token".to_string()));
        assert_eq!(auth.api_key, None);
    }

    #[test]
    fn test_auth_api_key() {
        let auth = Auth::api_key("my-key");
        assert_eq!(auth.type_, AuthType::ApiKey);
        assert_eq!(auth.token, None);
        assert_eq!(auth.api_key, Some("my-key".to_string()));
    }

    #[test]
    fn test_auth_mtls() {
        let auth = Auth::mtls();
        assert_eq!(auth.type_, AuthType::Mtls);
        assert_eq!(auth.token, None);
        assert_eq!(auth.api_key, None);
    }

    #[test]
    fn test_auth_serialization_bearer() {
        let auth = Auth::bearer("token123");
        let json = serde_json::to_string(&auth).unwrap();

        assert!(json.contains("\"type\":\"bearer\""));
        assert!(json.contains("\"token\":\"token123\""));
        assert!(!json.contains("\"api_key\"")); // Should be omitted
    }

    #[test]
    fn test_auth_serialization_api_key() {
        let auth = Auth::api_key("key456");
        let json = serde_json::to_string(&auth).unwrap();

        assert!(json.contains("\"type\":\"api_key\""));
        assert!(json.contains("\"api_key\":\"key456\""));
        assert!(!json.contains("\"token\"")); // Should be omitted
    }

    #[test]
    fn test_auth_serialization_mtls() {
        let auth = Auth::mtls();
        let json = serde_json::to_string(&auth).unwrap();

        assert!(json.contains("\"type\":\"mtls\""));
        assert!(!json.contains("\"token\"")); // Should be omitted
        assert!(!json.contains("\"api_key\"")); // Should be omitted
    }

    #[test]
    fn test_auth_deserialization() {
        let json = r#"{"type":"bearer","token":"my-token"}"#;
        let auth: Auth = serde_json::from_str(json).unwrap();

        assert_eq!(auth.type_, AuthType::Bearer);
        assert_eq!(auth.token, Some("my-token".to_string()));
        assert_eq!(auth.api_key, None);
    }

    #[test]
    fn test_auth_roundtrip() {
        let auths = vec![
            Auth::bearer("test-token"),
            Auth::api_key("test-key"),
            Auth::mtls(),
        ];

        for auth in auths {
            let json = serde_json::to_string(&auth).unwrap();
            let restored: Auth = serde_json::from_str(&json).unwrap();
            assert_eq!(auth, restored);
        }
    }
}
