//! Acknowledgment method types for the Cauce Protocol.
//!
//! Used to acknowledge receipt of signals.

use serde::{Deserialize, Serialize};

/// Request parameters for the `cauce.ack` method.
///
/// Clients send this to acknowledge receipt of signals.
///
/// # Example
///
/// ```
/// use cauce_core::methods::AckRequest;
///
/// let request = AckRequest {
///     subscription_id: "sub_abc123".to_string(),
///     signal_ids: vec![
///         "sig_1704067200_abc123def456".to_string(),
///         "sig_1704067201_xyz789ghi012".to_string(),
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AckRequest {
    /// The subscription that received the signals
    pub subscription_id: String,

    /// IDs of signals being acknowledged
    pub signal_ids: Vec<String>,
}

impl AckRequest {
    /// Creates a new AckRequest.
    pub fn new(subscription_id: impl Into<String>, signal_ids: Vec<String>) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            signal_ids,
        }
    }

    /// Creates an AckRequest for a single signal.
    pub fn single(subscription_id: impl Into<String>, signal_id: impl Into<String>) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            signal_ids: vec![signal_id.into()],
        }
    }
}

/// Response from the `cauce.ack` method.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{AckResponse, AckFailure};
///
/// let response = AckResponse {
///     acknowledged: vec!["sig_123".to_string()],
///     failed: vec![
///         AckFailure {
///             signal_id: "sig_456".to_string(),
///             reason: "Signal already acknowledged".to_string(),
///         }
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AckResponse {
    /// Signal IDs that were successfully acknowledged
    pub acknowledged: Vec<String>,

    /// Signals that failed to acknowledge
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<AckFailure>,
}

impl AckResponse {
    /// Creates a successful response where all signals were acknowledged.
    pub fn all_acknowledged(signal_ids: Vec<String>) -> Self {
        Self {
            acknowledged: signal_ids,
            failed: vec![],
        }
    }

    /// Creates a response with mixed results.
    pub fn mixed(acknowledged: Vec<String>, failed: Vec<AckFailure>) -> Self {
        Self {
            acknowledged,
            failed,
        }
    }
}

/// Details about a failed acknowledgment.
///
/// # Example
///
/// ```
/// use cauce_core::methods::AckFailure;
///
/// let failure = AckFailure {
///     signal_id: "sig_123".to_string(),
///     reason: "Signal not found".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AckFailure {
    /// The signal ID that failed to acknowledge
    pub signal_id: String,

    /// Reason for the failure
    pub reason: String,
}

impl AckFailure {
    /// Creates a new AckFailure.
    pub fn new(signal_id: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            signal_id: signal_id.into(),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== AckRequest Tests =====

    #[test]
    fn test_ack_request_new() {
        let request = AckRequest::new("sub_123", vec!["sig_a".to_string(), "sig_b".to_string()]);
        assert_eq!(request.subscription_id, "sub_123");
        assert_eq!(request.signal_ids.len(), 2);
    }

    #[test]
    fn test_ack_request_single() {
        let request = AckRequest::single("sub_123", "sig_single");
        assert_eq!(request.subscription_id, "sub_123");
        assert_eq!(request.signal_ids.len(), 1);
        assert_eq!(request.signal_ids[0], "sig_single");
    }

    #[test]
    fn test_ack_request_serialization() {
        let request = AckRequest::new("sub_test", vec!["sig_1".to_string(), "sig_2".to_string()]);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"subscription_id\":\"sub_test\""));
        assert!(json.contains("\"signal_ids\":[\"sig_1\",\"sig_2\"]"));
    }

    #[test]
    fn test_ack_request_deserialization() {
        let json = r#"{"subscription_id":"sub_deser","signal_ids":["sig_x","sig_y","sig_z"]}"#;
        let request: AckRequest = serde_json::from_str(json).unwrap();

        assert_eq!(request.subscription_id, "sub_deser");
        assert_eq!(request.signal_ids.len(), 3);
    }

    #[test]
    fn test_ack_request_roundtrip() {
        let request = AckRequest::new("sub_roundtrip", vec!["sig_rt".to_string()]);
        let json = serde_json::to_string(&request).unwrap();
        let restored: AckRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== AckResponse Tests =====

    #[test]
    fn test_ack_response_all_acknowledged() {
        let response =
            AckResponse::all_acknowledged(vec!["sig_1".to_string(), "sig_2".to_string()]);
        assert_eq!(response.acknowledged.len(), 2);
        assert!(response.failed.is_empty());
    }

    #[test]
    fn test_ack_response_mixed() {
        let response = AckResponse::mixed(
            vec!["sig_ok".to_string()],
            vec![AckFailure::new("sig_bad", "Not found")],
        );
        assert_eq!(response.acknowledged.len(), 1);
        assert_eq!(response.failed.len(), 1);
    }

    #[test]
    fn test_ack_response_serialization_all_success() {
        let response = AckResponse::all_acknowledged(vec!["sig_1".to_string()]);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"acknowledged\":[\"sig_1\"]"));
        assert!(!json.contains("\"failed\"")); // Should be omitted when empty
    }

    #[test]
    fn test_ack_response_serialization_with_failures() {
        let response = AckResponse::mixed(
            vec!["sig_ok".to_string()],
            vec![AckFailure::new("sig_fail", "Already acked")],
        );
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"acknowledged\":[\"sig_ok\"]"));
        assert!(json.contains("\"failed\""));
        assert!(json.contains("\"signal_id\":\"sig_fail\""));
        assert!(json.contains("\"reason\":\"Already acked\""));
    }

    #[test]
    fn test_ack_response_deserialization() {
        let json =
            r#"{"acknowledged":["sig_a"],"failed":[{"signal_id":"sig_b","reason":"error"}]}"#;
        let response: AckResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.acknowledged.len(), 1);
        assert_eq!(response.failed.len(), 1);
        assert_eq!(response.failed[0].signal_id, "sig_b");
    }

    #[test]
    fn test_ack_response_deserialization_no_failed() {
        let json = r#"{"acknowledged":["sig_only"]}"#;
        let response: AckResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.acknowledged.len(), 1);
        assert!(response.failed.is_empty());
    }

    #[test]
    fn test_ack_response_roundtrip() {
        let response = AckResponse::mixed(
            vec!["sig_1".to_string(), "sig_2".to_string()],
            vec![AckFailure::new("sig_3", "timeout")],
        );
        let json = serde_json::to_string(&response).unwrap();
        let restored: AckResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, restored);
    }

    // ===== AckFailure Tests =====

    #[test]
    fn test_ack_failure_new() {
        let failure = AckFailure::new("sig_test", "Some reason");
        assert_eq!(failure.signal_id, "sig_test");
        assert_eq!(failure.reason, "Some reason");
    }

    #[test]
    fn test_ack_failure_serialization() {
        let failure = AckFailure::new("sig_ser", "Serialization test");
        let json = serde_json::to_string(&failure).unwrap();

        assert!(json.contains("\"signal_id\":\"sig_ser\""));
        assert!(json.contains("\"reason\":\"Serialization test\""));
    }

    #[test]
    fn test_ack_failure_roundtrip() {
        let failure = AckFailure::new("sig_rt", "roundtrip reason");
        let json = serde_json::to_string(&failure).unwrap();
        let restored: AckFailure = serde_json::from_str(&json).unwrap();
        assert_eq!(failure, restored);
    }
}
