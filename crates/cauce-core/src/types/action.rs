//! Action types for the Cauce Protocol.
//!
//! This module provides the [`Action`], [`ActionBody`], and [`ActionContext`] types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{ActionType, Encrypted, Topic};

/// A command from an agent to be executed by an adapter.
///
/// Actions represent outbound commands that agents want adapters
/// to perform, such as sending messages, replying, or reacting.
///
/// # Fields
///
/// - `id` - Unique identifier: `act_<unix_timestamp>_<random_12>`
/// - `version` - Protocol version (e.g., "1.0")
/// - `timestamp` - When the action was created (ISO 8601)
/// - `topic` - Target topic for routing
/// - `action` - Action details (type, target, payload)
/// - `context` - Optional correlation and threading info
/// - `encrypted` - Optional E2E encryption envelope
///
/// # Example
///
/// ```
/// use cauce_core::types::{Action, ActionBody, ActionType, Topic};
/// use chrono::Utc;
/// use serde_json::json;
///
/// let action = Action {
///     id: "act_1704067200_abc123def456".to_string(),
///     version: "1.0".to_string(),
///     timestamp: Utc::now(),
///     topic: Topic::new_unchecked("action.slack.send"),
///     action: ActionBody {
///         type_: ActionType::Send,
///         target: Some("channel-123".to_string()),
///         payload: json!({"text": "Hello!"}),
///     },
///     context: None,
///     encrypted: None,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    /// Unique identifier: `act_<unix_timestamp>_<random_12>`
    pub id: String,

    /// Protocol version (e.g., "1.0")
    pub version: String,

    /// When the action was created
    pub timestamp: DateTime<Utc>,

    /// Target topic for routing
    pub topic: Topic,

    /// Action details
    pub action: ActionBody,

    /// Correlation and threading information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ActionContext>,

    /// End-to-end encryption envelope
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<Encrypted>,
}

/// Details of the action to be performed.
///
/// ActionBody contains the type of action, an optional target,
/// and the action-specific payload data.
///
/// # Fields
///
/// - `type_` - Kind of action (Send, Reply, Forward, etc.)
/// - `target` - Target recipient/destination (optional)
/// - `payload` - Action-specific data as JSON
///
/// # JSON Serialization
///
/// The `type_` field is serialized as `"type"` to match the JSON schema.
///
/// # Example
///
/// ```
/// use cauce_core::types::{ActionBody, ActionType};
/// use serde_json::json;
///
/// let body = ActionBody {
///     type_: ActionType::Send,
///     target: Some("user@example.com".to_string()),
///     payload: json!({"subject": "Hello", "body": "Hi there!"}),
/// };
///
/// let json = serde_json::to_string(&body).unwrap();
/// assert!(json.contains("\"type\":\"send\""));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionBody {
    /// Kind of action to perform
    #[serde(rename = "type")]
    pub type_: ActionType,

    /// Target recipient/destination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,

    /// Action-specific data
    pub payload: Value,
}

impl ActionBody {
    /// Creates a new ActionBody.
    ///
    /// # Arguments
    ///
    /// * `type_` - The kind of action
    /// * `payload` - Action-specific data
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::{ActionBody, ActionType};
    /// use serde_json::json;
    ///
    /// let body = ActionBody::new(ActionType::Send, json!({"text": "Hello"}));
    /// assert_eq!(body.type_, ActionType::Send);
    /// assert!(body.target.is_none());
    /// ```
    pub fn new(type_: ActionType, payload: Value) -> Self {
        Self {
            type_,
            target: None,
            payload,
        }
    }

    /// Creates a new ActionBody with a target.
    ///
    /// # Arguments
    ///
    /// * `type_` - The kind of action
    /// * `target` - Target recipient/destination
    /// * `payload` - Action-specific data
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::{ActionBody, ActionType};
    /// use serde_json::json;
    ///
    /// let body = ActionBody::with_target(
    ///     ActionType::Reply,
    ///     "channel-123",
    ///     json!({"text": "Thanks!"}),
    /// );
    /// assert_eq!(body.target, Some("channel-123".to_string()));
    /// ```
    pub fn with_target(type_: ActionType, target: impl Into<String>, payload: Value) -> Self {
        Self {
            type_,
            target: Some(target.into()),
            payload,
        }
    }
}

/// Correlation and threading information for actions.
///
/// ActionContext provides optional metadata for tracking action
/// relationships, agent attribution, and conversation threading.
///
/// # Fields
///
/// All fields are optional:
/// - `in_reply_to` - Signal ID being responded to
/// - `agent_id` - Identifier of the agent creating the action
/// - `thread_id` - Conversation thread identifier
/// - `correlation_id` - Request correlation for tracking
///
/// # Example
///
/// ```
/// use cauce_core::types::ActionContext;
///
/// let context = ActionContext {
///     in_reply_to: Some("sig_1704067200_abc123def456".to_string()),
///     agent_id: Some("my-agent".to_string()),
///     thread_id: None,
///     correlation_id: Some("req-12345".to_string()),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ActionContext {
    /// Signal ID being responded to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,

    /// Identifier of the agent creating this action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,

    /// Conversation thread identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// Request correlation for tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

impl ActionContext {
    /// Creates a new empty ActionContext.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::ActionContext;
    ///
    /// let context = ActionContext::new();
    /// assert!(context.in_reply_to.is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an ActionContext for replying to a signal.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::ActionContext;
    ///
    /// let context = ActionContext::reply_to("sig_1704067200_abc123def456");
    /// assert_eq!(context.in_reply_to, Some("sig_1704067200_abc123def456".to_string()));
    /// ```
    pub fn reply_to(signal_id: impl Into<String>) -> Self {
        Self {
            in_reply_to: Some(signal_id.into()),
            ..Default::default()
        }
    }

    /// Creates an ActionContext with an agent ID.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::ActionContext;
    ///
    /// let context = ActionContext::from_agent("my-agent");
    /// assert_eq!(context.agent_id, Some("my-agent".to_string()));
    /// ```
    pub fn from_agent(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: Some(agent_id.into()),
            ..Default::default()
        }
    }

    /// Sets the correlation ID.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::ActionContext;
    ///
    /// let context = ActionContext::new().with_correlation_id("req-123");
    /// assert_eq!(context.correlation_id, Some("req-123".to_string()));
    /// ```
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Sets the thread ID.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::ActionContext;
    ///
    /// let context = ActionContext::new().with_thread_id("thread-456");
    /// assert_eq!(context.thread_id, Some("thread-456".to_string()));
    /// ```
    pub fn with_thread_id(mut self, id: impl Into<String>) -> Self {
        self.thread_id = Some(id.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_action_body_new() {
        let body = ActionBody::new(ActionType::Send, json!({"text": "hello"}));
        assert_eq!(body.type_, ActionType::Send);
        assert!(body.target.is_none());
        assert_eq!(body.payload, json!({"text": "hello"}));
    }

    #[test]
    fn test_action_body_with_target() {
        let body = ActionBody::with_target(ActionType::Reply, "user-123", json!({"text": "hi"}));
        assert_eq!(body.type_, ActionType::Reply);
        assert_eq!(body.target, Some("user-123".to_string()));
    }

    #[test]
    fn test_action_body_serialization() {
        let body = ActionBody {
            type_: ActionType::Send,
            target: Some("target".to_string()),
            payload: json!({"msg": "test"}),
        };

        let json_str = serde_json::to_string(&body).unwrap();
        assert!(json_str.contains("\"type\":\"send\""));
        assert!(json_str.contains("\"target\":\"target\""));
    }

    #[test]
    fn test_action_body_serialization_skips_none_target() {
        let body = ActionBody::new(ActionType::Delete, json!(null));
        let json_str = serde_json::to_string(&body).unwrap();
        assert!(!json_str.contains("target"));
    }

    #[test]
    fn test_action_body_deserialization() {
        let json_str = r#"{"type":"forward","target":"dest","payload":{"data":1}}"#;
        let body: ActionBody = serde_json::from_str(json_str).unwrap();
        assert_eq!(body.type_, ActionType::Forward);
        assert_eq!(body.target, Some("dest".to_string()));
    }

    #[test]
    fn test_action_context_default() {
        let context = ActionContext::default();
        assert!(context.in_reply_to.is_none());
        assert!(context.agent_id.is_none());
        assert!(context.thread_id.is_none());
        assert!(context.correlation_id.is_none());
    }

    #[test]
    fn test_action_context_reply_to() {
        let context = ActionContext::reply_to("sig_123_abc");
        assert_eq!(context.in_reply_to, Some("sig_123_abc".to_string()));
    }

    #[test]
    fn test_action_context_from_agent() {
        let context = ActionContext::from_agent("agent-1");
        assert_eq!(context.agent_id, Some("agent-1".to_string()));
    }

    #[test]
    fn test_action_context_builder_chain() {
        let context = ActionContext::new()
            .with_correlation_id("corr-1")
            .with_thread_id("thread-1");

        assert_eq!(context.correlation_id, Some("corr-1".to_string()));
        assert_eq!(context.thread_id, Some("thread-1".to_string()));
    }

    #[test]
    fn test_action_context_serialization_skips_none() {
        let context = ActionContext::new();
        let json_str = serde_json::to_string(&context).unwrap();
        assert_eq!(json_str, "{}");
    }

    #[test]
    fn test_action_context_serialization_with_values() {
        let context = ActionContext {
            in_reply_to: Some("sig_1".to_string()),
            agent_id: Some("agent".to_string()),
            thread_id: None,
            correlation_id: None,
        };

        let json_str = serde_json::to_string(&context).unwrap();
        assert!(json_str.contains("\"in_reply_to\":\"sig_1\""));
        assert!(json_str.contains("\"agent_id\":\"agent\""));
        assert!(!json_str.contains("thread_id"));
    }

    #[test]
    fn test_action_context_round_trip() {
        let context = ActionContext {
            in_reply_to: Some("sig_1".to_string()),
            agent_id: Some("agent".to_string()),
            thread_id: Some("thread".to_string()),
            correlation_id: Some("corr".to_string()),
        };

        let json_str = serde_json::to_string(&context).unwrap();
        let restored: ActionContext = serde_json::from_str(&json_str).unwrap();
        assert_eq!(context, restored);
    }
}
