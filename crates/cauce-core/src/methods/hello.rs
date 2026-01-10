//! Hello method types for the Cauce Protocol.
//!
//! The Hello handshake is the first message exchange between a client and the hub.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Auth, Capability, ClientType};

/// Request parameters for the `cauce.hello` method.
///
/// Sent by clients to initiate a connection with the hub.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{HelloRequest, ClientType, Capability, Auth, AuthType};
///
/// let request = HelloRequest {
///     protocol_version: "1.0".to_string(),
///     min_protocol_version: None,
///     max_protocol_version: None,
///     client_id: "my-agent".to_string(),
///     client_type: ClientType::Agent,
///     capabilities: vec![Capability::Subscribe, Capability::Publish],
///     auth: Some(Auth::api_key("my-api-key")),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloRequest {
    /// The protocol version the client prefers
    pub protocol_version: String,

    /// Minimum protocol version the client supports (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_protocol_version: Option<String>,

    /// Maximum protocol version the client supports (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_protocol_version: Option<String>,

    /// Unique identifier for this client
    pub client_id: String,

    /// Type of client (adapter, agent, a2a_agent)
    pub client_type: ClientType,

    /// Capabilities the client supports
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// Authentication credentials (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<Auth>,
}

impl HelloRequest {
    /// Creates a new HelloRequest with required fields.
    pub fn new(
        protocol_version: impl Into<String>,
        client_id: impl Into<String>,
        client_type: ClientType,
    ) -> Self {
        Self {
            protocol_version: protocol_version.into(),
            min_protocol_version: None,
            max_protocol_version: None,
            client_id: client_id.into(),
            client_type,
            capabilities: vec![],
            auth: None,
        }
    }

    /// Adds a capability to the request.
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Sets the authentication.
    pub fn with_auth(mut self, auth: Auth) -> Self {
        self.auth = Some(auth);
        self
    }
}

/// Response from the `cauce.hello` method.
///
/// Returned by the hub to confirm the connection.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{HelloResponse, Capability};
/// use chrono::Utc;
///
/// let response = HelloResponse {
///     session_id: "sess_abc123".to_string(),
///     server_version: "1.0".to_string(),
///     capabilities: vec![Capability::Subscribe, Capability::Publish],
///     session_expires_at: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelloResponse {
    /// Unique session identifier assigned by the hub
    pub session_id: String,

    /// Server protocol version
    pub server_version: String,

    /// Capabilities supported by the server
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// When the session expires (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_expires_at: Option<DateTime<Utc>>,
}

impl HelloResponse {
    /// Creates a new HelloResponse with required fields.
    pub fn new(session_id: impl Into<String>, server_version: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            server_version: server_version.into(),
            capabilities: vec![],
            session_expires_at: None,
        }
    }

    /// Adds a capability to the response.
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Sets the session expiration time.
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.session_expires_at = Some(expires_at);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::methods::AuthType;

    // ===== HelloRequest Tests =====

    #[test]
    fn test_hello_request_new() {
        let request = HelloRequest::new("1.0", "client-1", ClientType::Agent);
        assert_eq!(request.protocol_version, "1.0");
        assert_eq!(request.client_id, "client-1");
        assert_eq!(request.client_type, ClientType::Agent);
        assert!(request.capabilities.is_empty());
        assert!(request.auth.is_none());
    }

    #[test]
    fn test_hello_request_with_capability() {
        let request = HelloRequest::new("1.0", "client-1", ClientType::Agent)
            .with_capability(Capability::Subscribe)
            .with_capability(Capability::Publish);

        assert_eq!(request.capabilities.len(), 2);
        assert!(request.capabilities.contains(&Capability::Subscribe));
        assert!(request.capabilities.contains(&Capability::Publish));
    }

    #[test]
    fn test_hello_request_with_auth() {
        let request = HelloRequest::new("1.0", "client-1", ClientType::Agent)
            .with_auth(Auth::api_key("my-key"));

        assert!(request.auth.is_some());
        let auth = request.auth.unwrap();
        assert_eq!(auth.type_, AuthType::ApiKey);
    }

    #[test]
    fn test_hello_request_serialization() {
        let request = HelloRequest::new("1.0", "my-agent", ClientType::Agent);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"protocol_version\":\"1.0\""));
        assert!(json.contains("\"client_id\":\"my-agent\""));
        assert!(json.contains("\"client_type\":\"agent\""));
        assert!(!json.contains("\"auth\"")); // Should be omitted
        assert!(!json.contains("\"min_protocol_version\"")); // Should be omitted
    }

    #[test]
    fn test_hello_request_serialization_full() {
        let request = HelloRequest {
            protocol_version: "1.0".to_string(),
            min_protocol_version: Some("0.9".to_string()),
            max_protocol_version: Some("1.1".to_string()),
            client_id: "test-client".to_string(),
            client_type: ClientType::Adapter,
            capabilities: vec![Capability::Publish],
            auth: Some(Auth::bearer("token")),
        };
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"min_protocol_version\":\"0.9\""));
        assert!(json.contains("\"max_protocol_version\":\"1.1\""));
        assert!(json.contains("\"client_type\":\"adapter\""));
        assert!(json.contains("\"auth\""));
    }

    #[test]
    fn test_hello_request_deserialization() {
        let json = r#"{
            "protocol_version": "1.0",
            "client_id": "deserialized-client",
            "client_type": "a2a_agent",
            "capabilities": ["subscribe", "ack"]
        }"#;

        let request: HelloRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.protocol_version, "1.0");
        assert_eq!(request.client_id, "deserialized-client");
        assert_eq!(request.client_type, ClientType::A2aAgent);
        assert_eq!(request.capabilities.len(), 2);
    }

    #[test]
    fn test_hello_request_roundtrip() {
        let request = HelloRequest::new("1.0", "roundtrip-client", ClientType::Agent)
            .with_capability(Capability::Subscribe)
            .with_capability(Capability::E2eEncryption)
            .with_auth(Auth::bearer("test-token"));

        let json = serde_json::to_string(&request).unwrap();
        let restored: HelloRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== HelloResponse Tests =====

    #[test]
    fn test_hello_response_new() {
        let response = HelloResponse::new("sess_123", "1.0");
        assert_eq!(response.session_id, "sess_123");
        assert_eq!(response.server_version, "1.0");
        assert!(response.capabilities.is_empty());
        assert!(response.session_expires_at.is_none());
    }

    #[test]
    fn test_hello_response_with_capability() {
        let response = HelloResponse::new("sess_123", "1.0")
            .with_capability(Capability::Subscribe)
            .with_capability(Capability::Publish);

        assert_eq!(response.capabilities.len(), 2);
    }

    #[test]
    fn test_hello_response_with_expiry() {
        let expires = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let response = HelloResponse::new("sess_123", "1.0").with_expiry(expires);

        assert!(response.session_expires_at.is_some());
        assert_eq!(response.session_expires_at.unwrap(), expires);
    }

    #[test]
    fn test_hello_response_serialization() {
        let response = HelloResponse::new("sess_abc", "1.0");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"session_id\":\"sess_abc\""));
        assert!(json.contains("\"server_version\":\"1.0\""));
        assert!(!json.contains("\"session_expires_at\"")); // Should be omitted
    }

    #[test]
    fn test_hello_response_deserialization() {
        let json = r#"{
            "session_id": "sess_xyz",
            "server_version": "1.1",
            "capabilities": ["publish", "e2e_encryption"],
            "session_expires_at": "2024-06-30T12:00:00Z"
        }"#;

        let response: HelloResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.session_id, "sess_xyz");
        assert_eq!(response.server_version, "1.1");
        assert_eq!(response.capabilities.len(), 2);
        assert!(response.session_expires_at.is_some());
    }

    #[test]
    fn test_hello_response_roundtrip() {
        let expires = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let response = HelloResponse::new("sess_roundtrip", "1.0")
            .with_capability(Capability::Ack)
            .with_expiry(expires);

        let json = serde_json::to_string(&response).unwrap();
        let restored: HelloResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, restored);
    }
}
