//! Subscription management types for the Cauce Protocol.
//!
//! Used for approving, denying, revoking, and listing subscriptions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{SubscriptionStatus, Transport};

/// Request to approve a subscription.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{SubscriptionApproveRequest, SubscriptionRestrictions};
/// use chrono::Utc;
///
/// let request = SubscriptionApproveRequest {
///     subscription_id: "sub_pending123".to_string(),
///     restrictions: Some(SubscriptionRestrictions {
///         allowed_topics: Some(vec!["signal.email.*".to_string()]),
///         expires_at: None,
///     }),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionApproveRequest {
    /// The subscription ID to approve
    pub subscription_id: String,

    /// Optional restrictions to apply
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<SubscriptionRestrictions>,
}

impl SubscriptionApproveRequest {
    /// Creates a new approval request without restrictions.
    pub fn new(subscription_id: impl Into<String>) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            restrictions: None,
        }
    }

    /// Adds restrictions to the approval.
    pub fn with_restrictions(mut self, restrictions: SubscriptionRestrictions) -> Self {
        self.restrictions = Some(restrictions);
        self
    }
}

/// Restrictions that can be applied when approving a subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionRestrictions {
    /// Subset of topics the subscription is allowed to access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_topics: Option<Vec<String>>,

    /// When the subscription should expire
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl SubscriptionRestrictions {
    /// Creates empty restrictions.
    pub fn new() -> Self {
        Self {
            allowed_topics: None,
            expires_at: None,
        }
    }

    /// Sets the allowed topics.
    pub fn with_topics(mut self, topics: Vec<String>) -> Self {
        self.allowed_topics = Some(topics);
        self
    }

    /// Sets the expiration time.
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

impl Default for SubscriptionRestrictions {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to deny a subscription.
///
/// # Example
///
/// ```
/// use cauce_core::methods::SubscriptionDenyRequest;
///
/// let request = SubscriptionDenyRequest {
///     subscription_id: "sub_pending123".to_string(),
///     reason: Some("Not authorized for this data".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionDenyRequest {
    /// The subscription ID to deny
    pub subscription_id: String,

    /// Reason for denial (shown to subscriber)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SubscriptionDenyRequest {
    /// Creates a new denial request.
    pub fn new(subscription_id: impl Into<String>) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            reason: None,
        }
    }

    /// Adds a reason for the denial.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Request to revoke an existing subscription.
///
/// # Example
///
/// ```
/// use cauce_core::methods::SubscriptionRevokeRequest;
///
/// let request = SubscriptionRevokeRequest {
///     subscription_id: "sub_active123".to_string(),
///     reason: Some("Subscription privileges removed".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionRevokeRequest {
    /// The subscription ID to revoke
    pub subscription_id: String,

    /// Reason for revocation (shown to subscriber)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SubscriptionRevokeRequest {
    /// Creates a new revocation request.
    pub fn new(subscription_id: impl Into<String>) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            reason: None,
        }
    }

    /// Adds a reason for the revocation.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Request to list subscriptions.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{SubscriptionListRequest, SubscriptionStatus};
///
/// // List all active subscriptions
/// let request = SubscriptionListRequest {
///     status: Some(SubscriptionStatus::Active),
///     client_id: None,
/// };
///
/// // List subscriptions for a specific client
/// let request = SubscriptionListRequest {
///     status: None,
///     client_id: Some("my-agent".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionListRequest {
    /// Filter by status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SubscriptionStatus>,

    /// Filter by client ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
}

impl SubscriptionListRequest {
    /// Creates a request to list all subscriptions.
    pub fn all() -> Self {
        Self {
            status: None,
            client_id: None,
        }
    }

    /// Filter by status.
    pub fn with_status(mut self, status: SubscriptionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Filter by client ID.
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }
}

impl Default for SubscriptionListRequest {
    fn default() -> Self {
        Self::all()
    }
}

/// Response from listing subscriptions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionListResponse {
    /// List of subscriptions matching the filter
    pub subscriptions: Vec<SubscriptionInfo>,
}

impl SubscriptionListResponse {
    /// Creates a new list response.
    pub fn new(subscriptions: Vec<SubscriptionInfo>) -> Self {
        Self { subscriptions }
    }

    /// Creates an empty list response.
    pub fn empty() -> Self {
        Self {
            subscriptions: vec![],
        }
    }
}

/// Information about a subscription.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionInfo {
    /// Unique subscription identifier
    pub subscription_id: String,

    /// Client that owns this subscription
    pub client_id: String,

    /// Session ID for this subscription's connection
    pub session_id: String,

    /// Topics this subscription covers
    pub topics: Vec<String>,

    /// Current status
    pub status: SubscriptionStatus,

    /// Transport used for delivery
    pub transport: Transport,

    /// When the subscription was created
    pub created_at: DateTime<Utc>,

    /// When the subscription expires (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

impl SubscriptionInfo {
    /// Creates a new SubscriptionInfo.
    pub fn new(
        subscription_id: impl Into<String>,
        client_id: impl Into<String>,
        session_id: impl Into<String>,
        topics: Vec<String>,
        status: SubscriptionStatus,
        transport: Transport,
    ) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            client_id: client_id.into(),
            session_id: session_id.into(),
            topics,
            status,
            transport,
            created_at: Utc::now(),
            expires_at: None,
        }
    }
}

/// Notification about subscription status change.
///
/// Sent as a notification to inform clients about subscription status changes.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{SubscriptionStatusNotification, SubscriptionStatus};
///
/// let notification = SubscriptionStatusNotification {
///     subscription_id: "sub_123".to_string(),
///     status: SubscriptionStatus::Active,
///     reason: Some("Approved by adapter owner".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionStatusNotification {
    /// The subscription ID
    pub subscription_id: String,

    /// New status
    pub status: SubscriptionStatus,

    /// Optional reason for the status change
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl SubscriptionStatusNotification {
    /// Creates a new status notification.
    pub fn new(subscription_id: impl Into<String>, status: SubscriptionStatus) -> Self {
        Self {
            subscription_id: subscription_id.into(),
            status,
            reason: None,
        }
    }

    /// Adds a reason for the status change.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== SubscriptionApproveRequest Tests =====

    #[test]
    fn test_approve_request_new() {
        let request = SubscriptionApproveRequest::new("sub_123");
        assert_eq!(request.subscription_id, "sub_123");
        assert!(request.restrictions.is_none());
    }

    #[test]
    fn test_approve_request_with_restrictions() {
        let restrictions =
            SubscriptionRestrictions::new().with_topics(vec!["signal.*".to_string()]);
        let request = SubscriptionApproveRequest::new("sub_123").with_restrictions(restrictions);

        assert!(request.restrictions.is_some());
    }

    #[test]
    fn test_approve_request_serialization() {
        let request = SubscriptionApproveRequest::new("sub_test");
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"subscription_id\":\"sub_test\""));
        assert!(!json.contains("\"restrictions\"")); // Should be omitted
    }

    #[test]
    fn test_approve_request_roundtrip() {
        let request = SubscriptionApproveRequest::new("sub_rt");
        let json = serde_json::to_string(&request).unwrap();
        let restored: SubscriptionApproveRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== SubscriptionRestrictions Tests =====

    #[test]
    fn test_restrictions_builder() {
        let expires = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let restrictions = SubscriptionRestrictions::new()
            .with_topics(vec!["topic.a".to_string()])
            .with_expiry(expires);

        assert!(restrictions.allowed_topics.is_some());
        assert!(restrictions.expires_at.is_some());
    }

    // ===== SubscriptionDenyRequest Tests =====

    #[test]
    fn test_deny_request_new() {
        let request = SubscriptionDenyRequest::new("sub_deny");
        assert_eq!(request.subscription_id, "sub_deny");
        assert!(request.reason.is_none());
    }

    #[test]
    fn test_deny_request_with_reason() {
        let request = SubscriptionDenyRequest::new("sub_deny").with_reason("Not authorized");
        assert_eq!(request.reason, Some("Not authorized".to_string()));
    }

    #[test]
    fn test_deny_request_roundtrip() {
        let request = SubscriptionDenyRequest::new("sub_rt").with_reason("test reason");
        let json = serde_json::to_string(&request).unwrap();
        let restored: SubscriptionDenyRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== SubscriptionRevokeRequest Tests =====

    #[test]
    fn test_revoke_request_new() {
        let request = SubscriptionRevokeRequest::new("sub_revoke");
        assert_eq!(request.subscription_id, "sub_revoke");
        assert!(request.reason.is_none());
    }

    #[test]
    fn test_revoke_request_with_reason() {
        let request = SubscriptionRevokeRequest::new("sub_revoke").with_reason("Access removed");
        assert_eq!(request.reason, Some("Access removed".to_string()));
    }

    // ===== SubscriptionListRequest Tests =====

    #[test]
    fn test_list_request_all() {
        let request = SubscriptionListRequest::all();
        assert!(request.status.is_none());
        assert!(request.client_id.is_none());
    }

    #[test]
    fn test_list_request_with_filters() {
        let request = SubscriptionListRequest::all()
            .with_status(SubscriptionStatus::Active)
            .with_client_id("my-client");

        assert_eq!(request.status, Some(SubscriptionStatus::Active));
        assert_eq!(request.client_id, Some("my-client".to_string()));
    }

    #[test]
    fn test_list_request_serialization() {
        let request = SubscriptionListRequest::all().with_status(SubscriptionStatus::Pending);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"status\":\"pending\""));
        assert!(!json.contains("\"client_id\"")); // Should be omitted
    }

    // ===== SubscriptionListResponse Tests =====

    #[test]
    fn test_list_response_empty() {
        let response = SubscriptionListResponse::empty();
        assert!(response.subscriptions.is_empty());
    }

    #[test]
    fn test_list_response_new() {
        let info = SubscriptionInfo::new(
            "sub_1",
            "client_1",
            "sess_1",
            vec!["topic.*".to_string()],
            SubscriptionStatus::Active,
            Transport::WebSocket,
        );
        let response = SubscriptionListResponse::new(vec![info]);
        assert_eq!(response.subscriptions.len(), 1);
    }

    // ===== SubscriptionInfo Tests =====

    #[test]
    fn test_subscription_info_new() {
        let info = SubscriptionInfo::new(
            "sub_info",
            "client_info",
            "sess_info",
            vec!["signal.**".to_string()],
            SubscriptionStatus::Active,
            Transport::Sse,
        );

        assert_eq!(info.subscription_id, "sub_info");
        assert_eq!(info.client_id, "client_info");
        assert_eq!(info.session_id, "sess_info");
        assert_eq!(info.status, SubscriptionStatus::Active);
        assert_eq!(info.transport, Transport::Sse);
    }

    #[test]
    fn test_subscription_info_roundtrip() {
        let info = SubscriptionInfo {
            subscription_id: "sub_rt".to_string(),
            client_id: "client_rt".to_string(),
            session_id: "sess_rt".to_string(),
            topics: vec!["topic.a".to_string()],
            status: SubscriptionStatus::Pending,
            transport: Transport::Webhook,
            created_at: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            expires_at: Some(
                DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
                    .unwrap()
                    .with_timezone(&Utc),
            ),
        };

        let json = serde_json::to_string(&info).unwrap();
        let restored: SubscriptionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, restored);
    }

    // ===== SubscriptionStatusNotification Tests =====

    #[test]
    fn test_status_notification_new() {
        let notification =
            SubscriptionStatusNotification::new("sub_notif", SubscriptionStatus::Active);
        assert_eq!(notification.subscription_id, "sub_notif");
        assert_eq!(notification.status, SubscriptionStatus::Active);
        assert!(notification.reason.is_none());
    }

    #[test]
    fn test_status_notification_with_reason() {
        let notification =
            SubscriptionStatusNotification::new("sub_notif", SubscriptionStatus::Denied)
                .with_reason("Policy violation");
        assert_eq!(notification.reason, Some("Policy violation".to_string()));
    }

    #[test]
    fn test_status_notification_roundtrip() {
        let notification =
            SubscriptionStatusNotification::new("sub_rt", SubscriptionStatus::Revoked)
                .with_reason("Manual revocation");
        let json = serde_json::to_string(&notification).unwrap();
        let restored: SubscriptionStatusNotification = serde_json::from_str(&json).unwrap();
        assert_eq!(notification, restored);
    }
}
