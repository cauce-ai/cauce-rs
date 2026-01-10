//! Subscribe method types for the Cauce Protocol.
//!
//! Used to create subscriptions to topics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ApprovalType, E2eConfig, SubscriptionStatus, Transport, WebhookConfig};

/// Request parameters for the `cauce.subscribe` method.
///
/// Clients send this to subscribe to one or more topics.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{SubscribeRequest, ApprovalType, Transport};
///
/// let request = SubscribeRequest {
///     topics: vec!["signal.email.*".to_string(), "signal.slack.**".to_string()],
///     approval_type: Some(ApprovalType::Automatic),
///     reason: Some("Monitor communications".to_string()),
///     transport: Some(Transport::WebSocket),
///     webhook: None,
///     e2e: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubscribeRequest {
    /// Topics to subscribe to (supports wildcards)
    pub topics: Vec<String>,

    /// How the subscription should be approved
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_type: Option<ApprovalType>,

    /// Reason for the subscription (shown to adapter owner)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Preferred transport for receiving signals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<Transport>,

    /// Webhook configuration (required if transport is Webhook)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<WebhookConfig>,

    /// End-to-end encryption configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub e2e: Option<E2eConfig>,
}

impl SubscribeRequest {
    /// Creates a new SubscribeRequest with the specified topics.
    pub fn new(topics: Vec<String>) -> Self {
        Self {
            topics,
            approval_type: None,
            reason: None,
            transport: None,
            webhook: None,
            e2e: None,
        }
    }

    /// Creates a request for a single topic.
    pub fn single(topic: impl Into<String>) -> Self {
        Self::new(vec![topic.into()])
    }

    /// Sets the approval type.
    pub fn with_approval(mut self, approval_type: ApprovalType) -> Self {
        self.approval_type = Some(approval_type);
        self
    }

    /// Sets the reason for subscription.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Sets the transport type.
    pub fn with_transport(mut self, transport: Transport) -> Self {
        self.transport = Some(transport);
        self
    }

    /// Sets webhook configuration.
    pub fn with_webhook(mut self, webhook: WebhookConfig) -> Self {
        self.transport = Some(Transport::Webhook);
        self.webhook = Some(webhook);
        self
    }
}

/// Response from the `cauce.subscribe` method.
///
/// Contains the subscription details and current status.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{SubscribeResponse, SubscriptionStatus};
/// use chrono::Utc;
///
/// let response = SubscribeResponse {
///     subscription_id: "sub_abc123".to_string(),
///     status: SubscriptionStatus::Active,
///     topics: vec!["signal.email.*".to_string()],
///     created_at: Utc::now(),
///     expires_at: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscribeResponse {
    /// Unique identifier for the subscription
    pub subscription_id: String,

    /// Current status of the subscription
    pub status: SubscriptionStatus,

    /// Topics that were subscribed to
    pub topics: Vec<String>,

    /// When the subscription was created
    pub created_at: DateTime<Utc>,

    /// When the subscription expires (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl SubscribeResponse {
    /// Creates a new SubscribeResponse.
    pub fn new(
        subscription_id: impl Into<String>,
        status: SubscriptionStatus,
        topics: Vec<String>,
    ) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            status,
            topics,
            created_at: Utc::now(),
            expires_at: None,
        }
    }

    /// Sets the expiration time.
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== SubscribeRequest Tests =====

    #[test]
    fn test_subscribe_request_new() {
        let request = SubscribeRequest::new(vec!["topic.a".to_string(), "topic.b".to_string()]);
        assert_eq!(request.topics.len(), 2);
        assert!(request.approval_type.is_none());
        assert!(request.reason.is_none());
    }

    #[test]
    fn test_subscribe_request_single() {
        let request = SubscribeRequest::single("signal.email.*");
        assert_eq!(request.topics.len(), 1);
        assert_eq!(request.topics[0], "signal.email.*");
    }

    #[test]
    fn test_subscribe_request_with_approval() {
        let request = SubscribeRequest::single("topic").with_approval(ApprovalType::UserApproved);
        assert_eq!(request.approval_type, Some(ApprovalType::UserApproved));
    }

    #[test]
    fn test_subscribe_request_with_reason() {
        let request = SubscribeRequest::single("topic").with_reason("Need to monitor emails");
        assert_eq!(request.reason, Some("Need to monitor emails".to_string()));
    }

    #[test]
    fn test_subscribe_request_with_transport() {
        let request = SubscribeRequest::single("topic").with_transport(Transport::WebSocket);
        assert_eq!(request.transport, Some(Transport::WebSocket));
    }

    #[test]
    fn test_subscribe_request_with_webhook() {
        let request = SubscribeRequest::single("topic")
            .with_webhook(WebhookConfig::new("https://example.com/webhook"));

        assert_eq!(request.transport, Some(Transport::Webhook));
        assert!(request.webhook.is_some());
    }

    #[test]
    fn test_subscribe_request_serialization() {
        let request = SubscribeRequest::single("signal.*");
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"topics\":[\"signal.*\"]"));
        assert!(!json.contains("\"approval_type\"")); // Should be omitted
        assert!(!json.contains("\"reason\"")); // Should be omitted
    }

    #[test]
    fn test_subscribe_request_serialization_full() {
        let request = SubscribeRequest {
            topics: vec!["signal.**".to_string()],
            approval_type: Some(ApprovalType::Automatic),
            reason: Some("testing".to_string()),
            transport: Some(Transport::Sse),
            webhook: None,
            e2e: None,
        };
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"approval_type\":\"automatic\""));
        assert!(json.contains("\"reason\":\"testing\""));
        assert!(json.contains("\"transport\":\"sse\""));
    }

    #[test]
    fn test_subscribe_request_deserialization() {
        let json = r#"{
            "topics": ["signal.email.*", "signal.slack.**"],
            "approval_type": "user_approved",
            "transport": "websocket"
        }"#;

        let request: SubscribeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.topics.len(), 2);
        assert_eq!(request.approval_type, Some(ApprovalType::UserApproved));
        assert_eq!(request.transport, Some(Transport::WebSocket));
    }

    #[test]
    fn test_subscribe_request_roundtrip() {
        let request = SubscribeRequest::new(vec!["topic.a".to_string()])
            .with_approval(ApprovalType::Automatic)
            .with_reason("test reason")
            .with_transport(Transport::LongPolling);

        let json = serde_json::to_string(&request).unwrap();
        let restored: SubscribeRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== SubscribeResponse Tests =====

    #[test]
    fn test_subscribe_response_new() {
        let response = SubscribeResponse::new(
            "sub_123",
            SubscriptionStatus::Active,
            vec!["topic".to_string()],
        );
        assert_eq!(response.subscription_id, "sub_123");
        assert_eq!(response.status, SubscriptionStatus::Active);
        assert!(response.expires_at.is_none());
    }

    #[test]
    fn test_subscribe_response_with_expiry() {
        let expires = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let response = SubscribeResponse::new("sub_123", SubscriptionStatus::Pending, vec![])
            .with_expiry(expires);

        assert_eq!(response.expires_at, Some(expires));
    }

    #[test]
    fn test_subscribe_response_serialization() {
        let response = SubscribeResponse {
            subscription_id: "sub_test".to_string(),
            status: SubscriptionStatus::Pending,
            topics: vec!["signal.*".to_string()],
            created_at: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            expires_at: None,
        };
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"subscription_id\":\"sub_test\""));
        assert!(json.contains("\"status\":\"pending\""));
        assert!(json.contains("\"created_at\""));
        assert!(!json.contains("\"expires_at\"")); // Should be omitted
    }

    #[test]
    fn test_subscribe_response_deserialization() {
        let json = r#"{
            "subscription_id": "sub_deser",
            "status": "active",
            "topics": ["topic.a", "topic.b"],
            "created_at": "2024-06-15T10:30:00Z"
        }"#;

        let response: SubscribeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.subscription_id, "sub_deser");
        assert_eq!(response.status, SubscriptionStatus::Active);
        assert_eq!(response.topics.len(), 2);
    }

    #[test]
    fn test_subscribe_response_roundtrip() {
        let response = SubscribeResponse {
            subscription_id: "sub_roundtrip".to_string(),
            status: SubscriptionStatus::Active,
            topics: vec!["signal.email.*".to_string()],
            created_at: DateTime::parse_from_rfc3339("2024-01-15T08:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            expires_at: Some(
                DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&response).unwrap();
        let restored: SubscribeResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, restored);
    }
}
