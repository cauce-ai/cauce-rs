//! Unsubscribe method types for the Cauce Protocol.
//!
//! Used to remove subscriptions.

use serde::{Deserialize, Serialize};

/// Request parameters for the `cauce.unsubscribe` method.
///
/// # Example
///
/// ```
/// use cauce_core::methods::UnsubscribeRequest;
///
/// let request = UnsubscribeRequest {
///     subscription_id: "sub_abc123".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsubscribeRequest {
    /// The subscription ID to remove
    pub subscription_id: String,
}

impl UnsubscribeRequest {
    /// Creates a new UnsubscribeRequest.
    pub fn new(subscription_id: impl Into<String>) -> Self {
        Self {
            subscription_id: subscription_id.into(),
        }
    }
}

/// Response from the `cauce.unsubscribe` method.
///
/// # Example
///
/// ```
/// use cauce_core::methods::UnsubscribeResponse;
///
/// let response = UnsubscribeResponse { success: true };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnsubscribeResponse {
    /// Whether the unsubscription was successful
    pub success: bool,
}

impl UnsubscribeResponse {
    /// Creates a successful response.
    pub fn success() -> Self {
        Self { success: true }
    }

    /// Creates a failure response.
    pub fn failure() -> Self {
        Self { success: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== UnsubscribeRequest Tests =====

    #[test]
    fn test_unsubscribe_request_new() {
        let request = UnsubscribeRequest::new("sub_123");
        assert_eq!(request.subscription_id, "sub_123");
    }

    #[test]
    fn test_unsubscribe_request_serialization() {
        let request = UnsubscribeRequest::new("sub_test");
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, r#"{"subscription_id":"sub_test"}"#);
    }

    #[test]
    fn test_unsubscribe_request_deserialization() {
        let json = r#"{"subscription_id":"sub_deser"}"#;
        let request: UnsubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.subscription_id, "sub_deser");
    }

    #[test]
    fn test_unsubscribe_request_roundtrip() {
        let request = UnsubscribeRequest::new("sub_roundtrip");
        let json = serde_json::to_string(&request).unwrap();
        let restored: UnsubscribeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== UnsubscribeResponse Tests =====

    #[test]
    fn test_unsubscribe_response_success() {
        let response = UnsubscribeResponse::success();
        assert!(response.success);
    }

    #[test]
    fn test_unsubscribe_response_failure() {
        let response = UnsubscribeResponse::failure();
        assert!(!response.success);
    }

    #[test]
    fn test_unsubscribe_response_serialization() {
        let response = UnsubscribeResponse::success();
        let json = serde_json::to_string(&response).unwrap();
        assert_eq!(json, r#"{"success":true}"#);
    }

    #[test]
    fn test_unsubscribe_response_deserialization() {
        let json = r#"{"success":false}"#;
        let response: UnsubscribeResponse = serde_json::from_str(json).unwrap();
        assert!(!response.success);
    }

    #[test]
    fn test_unsubscribe_response_roundtrip() {
        for response in [
            UnsubscribeResponse::success(),
            UnsubscribeResponse::failure(),
        ] {
            let json = serde_json::to_string(&response).unwrap();
            let restored: UnsubscribeResponse = serde_json::from_str(&json).unwrap();
            assert_eq!(response, restored);
        }
    }
}
