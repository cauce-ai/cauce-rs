//! Authentication configuration for the Cauce Client SDK.
//!
//! This module provides types for configuring client authentication
//! when connecting to a Cauce Hub.

use serde::{Deserialize, Serialize};

/// Authentication configuration for connecting to a Cauce Hub.
///
/// # Example
///
/// ```rust
/// use cauce_client_sdk::AuthConfig;
///
/// // API key authentication
/// let auth = AuthConfig::api_key("my-api-key");
///
/// // Bearer token authentication
/// let auth = AuthConfig::bearer("my-jwt-token");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    /// API key authentication.
    ///
    /// The key is sent via the `X-Cauce-API-Key` header.
    ApiKey {
        /// The API key.
        key: String,
    },

    /// Bearer token authentication.
    ///
    /// The token is sent via the `Authorization: Bearer <token>` header.
    Bearer {
        /// The bearer token (typically a JWT).
        token: String,
    },
}

impl AuthConfig {
    /// Create an API key authentication configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cauce_client_sdk::AuthConfig;
    ///
    /// let auth = AuthConfig::api_key("my-secret-key");
    /// ```
    pub fn api_key(key: impl Into<String>) -> Self {
        AuthConfig::ApiKey { key: key.into() }
    }

    /// Create a bearer token authentication configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cauce_client_sdk::AuthConfig;
    ///
    /// let auth = AuthConfig::bearer("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...");
    /// ```
    pub fn bearer(token: impl Into<String>) -> Self {
        AuthConfig::Bearer {
            token: token.into(),
        }
    }

    /// Returns the authentication type as a string.
    pub fn auth_type(&self) -> &'static str {
        match self {
            AuthConfig::ApiKey { .. } => "api_key",
            AuthConfig::Bearer { .. } => "bearer",
        }
    }

    /// Returns the header name for this authentication type.
    pub fn header_name(&self) -> &'static str {
        match self {
            AuthConfig::ApiKey { .. } => "X-Cauce-API-Key",
            AuthConfig::Bearer { .. } => "Authorization",
        }
    }

    /// Returns the header value for this authentication type.
    pub fn header_value(&self) -> String {
        match self {
            AuthConfig::ApiKey { key } => key.clone(),
            AuthConfig::Bearer { token } => format!("Bearer {}", token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_creation() {
        let auth = AuthConfig::api_key("test-key");
        match auth {
            AuthConfig::ApiKey { key } => assert_eq!(key, "test-key"),
            _ => panic!("Expected ApiKey"),
        }
    }

    #[test]
    fn test_bearer_creation() {
        let auth = AuthConfig::bearer("test-token");
        match auth {
            AuthConfig::Bearer { token } => assert_eq!(token, "test-token"),
            _ => panic!("Expected Bearer"),
        }
    }

    #[test]
    fn test_auth_type() {
        assert_eq!(AuthConfig::api_key("key").auth_type(), "api_key");
        assert_eq!(AuthConfig::bearer("token").auth_type(), "bearer");
    }

    #[test]
    fn test_header_name() {
        assert_eq!(AuthConfig::api_key("key").header_name(), "X-Cauce-API-Key");
        assert_eq!(AuthConfig::bearer("token").header_name(), "Authorization");
    }

    #[test]
    fn test_header_value() {
        assert_eq!(AuthConfig::api_key("my-key").header_value(), "my-key");
        assert_eq!(
            AuthConfig::bearer("my-token").header_value(),
            "Bearer my-token"
        );
    }

    #[test]
    fn test_serialization() {
        let auth = AuthConfig::api_key("test-key");
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains("\"type\":\"api_key\""));
        assert!(json.contains("\"key\":\"test-key\""));
    }

    #[test]
    fn test_deserialization() {
        let json = r#"{"type":"bearer","token":"my-token"}"#;
        let auth: AuthConfig = serde_json::from_str(json).unwrap();
        match auth {
            AuthConfig::Bearer { token } => assert_eq!(token, "my-token"),
            _ => panic!("Expected Bearer"),
        }
    }
}
