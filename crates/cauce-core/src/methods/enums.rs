//! Subscription-related enumeration types for the Cauce Protocol.
//!
//! This module provides enums for subscription management.

use serde::{Deserialize, Serialize};

/// Approval type for subscription requests.
///
/// Determines how subscription requests are handled by the adapter owner.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `Automatic` → `"automatic"`
/// - `UserApproved` → `"user_approved"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    /// Subscription is automatically approved
    #[default]
    Automatic,
    /// Subscription requires explicit user approval
    UserApproved,
}

/// Status of a subscription.
///
/// Represents the current state in the subscription lifecycle.
///
/// # State Transitions
///
/// ```text
/// Pending → Active (approved) or Denied (rejected)
/// Active → Revoked (manually) or Expired (time-based)
/// ```
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `Pending` → `"pending"`
/// - `Active` → `"active"`
/// - `Denied` → `"denied"`
/// - `Revoked` → `"revoked"`
/// - `Expired` → `"expired"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    /// Awaiting approval
    Pending,
    /// Subscription is active and receiving signals
    Active,
    /// Subscription was explicitly denied
    Denied,
    /// Subscription was manually revoked
    Revoked,
    /// Subscription has expired
    Expired,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ApprovalType Tests =====

    #[test]
    fn test_approval_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ApprovalType::Automatic).unwrap(),
            "\"automatic\""
        );
        assert_eq!(
            serde_json::to_string(&ApprovalType::UserApproved).unwrap(),
            "\"user_approved\""
        );
    }

    #[test]
    fn test_approval_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<ApprovalType>("\"automatic\"").unwrap(),
            ApprovalType::Automatic
        );
        assert_eq!(
            serde_json::from_str::<ApprovalType>("\"user_approved\"").unwrap(),
            ApprovalType::UserApproved
        );
    }

    #[test]
    fn test_approval_type_default() {
        assert_eq!(ApprovalType::default(), ApprovalType::Automatic);
    }

    #[test]
    fn test_approval_type_roundtrip() {
        for approval_type in [ApprovalType::Automatic, ApprovalType::UserApproved] {
            let json = serde_json::to_string(&approval_type).unwrap();
            let restored: ApprovalType = serde_json::from_str(&json).unwrap();
            assert_eq!(approval_type, restored);
        }
    }

    // ===== SubscriptionStatus Tests =====

    #[test]
    fn test_subscription_status_serialization() {
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::Denied).unwrap(),
            "\"denied\""
        );
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::Revoked).unwrap(),
            "\"revoked\""
        );
        assert_eq!(
            serde_json::to_string(&SubscriptionStatus::Expired).unwrap(),
            "\"expired\""
        );
    }

    #[test]
    fn test_subscription_status_deserialization() {
        assert_eq!(
            serde_json::from_str::<SubscriptionStatus>("\"pending\"").unwrap(),
            SubscriptionStatus::Pending
        );
        assert_eq!(
            serde_json::from_str::<SubscriptionStatus>("\"active\"").unwrap(),
            SubscriptionStatus::Active
        );
        assert_eq!(
            serde_json::from_str::<SubscriptionStatus>("\"denied\"").unwrap(),
            SubscriptionStatus::Denied
        );
        assert_eq!(
            serde_json::from_str::<SubscriptionStatus>("\"revoked\"").unwrap(),
            SubscriptionStatus::Revoked
        );
        assert_eq!(
            serde_json::from_str::<SubscriptionStatus>("\"expired\"").unwrap(),
            SubscriptionStatus::Expired
        );
    }

    #[test]
    fn test_subscription_status_roundtrip() {
        let statuses = [
            SubscriptionStatus::Pending,
            SubscriptionStatus::Active,
            SubscriptionStatus::Denied,
            SubscriptionStatus::Revoked,
            SubscriptionStatus::Expired,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let restored: SubscriptionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, restored);
        }
    }
}
