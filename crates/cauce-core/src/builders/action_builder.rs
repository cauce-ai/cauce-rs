//! ActionBuilder for creating Action instances.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::constants::PROTOCOL_VERSION;
use crate::errors::BuilderError;
use crate::types::{Action, ActionBody, ActionContext, Encrypted, Topic};

/// Fluent builder for creating [`Action`] instances.
///
/// ActionBuilder provides an ergonomic way to construct Action instances
/// with validation. Required fields must be set before calling `build()`.
///
/// # Required Fields
///
/// - `topic` - Target topic for routing
/// - `action` - Action details (type, target, payload)
///
/// # Auto-generated Fields
///
/// - `id` - Auto-generated if not provided
/// - `version` - Defaults to protocol version if not provided
/// - `timestamp` - Defaults to current UTC time if not provided
///
/// # Example
///
/// ```
/// use cauce_core::builders::ActionBuilder;
/// use cauce_core::types::{ActionBody, ActionType, ActionContext, Topic};
/// use serde_json::json;
///
/// let action = ActionBuilder::new()
///     .topic(Topic::new_unchecked("action.email.send"))
///     .action(ActionBody::with_target(
///         ActionType::Send,
///         "bob@example.com",
///         json!({"subject": "Hello", "body": "Hi!"}),
///     ))
///     .context(ActionContext::from_agent("my-agent"))
///     .build()
///     .expect("valid action");
///
/// assert!(action.id.starts_with("act_"));
/// assert_eq!(action.action.type_, ActionType::Send);
/// ```
#[derive(Debug, Default)]
pub struct ActionBuilder {
    id: Option<String>,
    version: Option<String>,
    timestamp: Option<DateTime<Utc>>,
    topic: Option<Topic>,
    action: Option<ActionBody>,
    context: Option<ActionContext>,
    encrypted: Option<Encrypted>,
}

impl ActionBuilder {
    /// Creates a new ActionBuilder.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    ///
    /// let builder = ActionBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the action ID.
    ///
    /// If not called, a unique ID will be auto-generated.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    ///
    /// let builder = ActionBuilder::new()
    ///     .id("act_1704067200_abc123def456");
    /// ```
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the protocol version.
    ///
    /// If not called, defaults to the current protocol version.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    ///
    /// let builder = ActionBuilder::new().version("1.0");
    /// ```
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Sets the timestamp.
    ///
    /// If not called, defaults to the current UTC time.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    /// use chrono::Utc;
    ///
    /// let builder = ActionBuilder::new().timestamp(Utc::now());
    /// ```
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Sets the topic (required).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    /// use cauce_core::types::Topic;
    ///
    /// let builder = ActionBuilder::new()
    ///     .topic(Topic::new_unchecked("action.email.send"));
    /// ```
    pub fn topic(mut self, topic: Topic) -> Self {
        self.topic = Some(topic);
        self
    }

    /// Sets the action body (required).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    /// use cauce_core::types::{ActionBody, ActionType};
    /// use serde_json::json;
    ///
    /// let builder = ActionBuilder::new()
    ///     .action(ActionBody::new(ActionType::Send, json!({"text": "Hello"})));
    /// ```
    pub fn action(mut self, action: ActionBody) -> Self {
        self.action = Some(action);
        self
    }

    /// Sets the context (optional).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    /// use cauce_core::types::ActionContext;
    ///
    /// let builder = ActionBuilder::new()
    ///     .context(ActionContext::from_agent("my-agent"));
    /// ```
    pub fn context(mut self, context: ActionContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Sets the encrypted envelope (optional).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    /// use cauce_core::types::{Encrypted, EncryptionAlgorithm};
    ///
    /// let encrypted = Encrypted::new(
    ///     EncryptionAlgorithm::A256Gcm,
    ///     "public_key",
    ///     "nonce",
    ///     "ciphertext",
    /// );
    /// let builder = ActionBuilder::new().encrypted(encrypted);
    /// ```
    pub fn encrypted(mut self, encrypted: Encrypted) -> Self {
        self.encrypted = Some(encrypted);
        self
    }

    /// Builds the Action, returning an error if required fields are missing.
    ///
    /// # Returns
    ///
    /// - `Ok(Action)` if all required fields are set
    /// - `Err(BuilderError)` if required fields are missing
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::ActionBuilder;
    /// use cauce_core::types::{ActionBody, ActionType, Topic};
    /// use serde_json::json;
    ///
    /// let action = ActionBuilder::new()
    ///     .topic(Topic::new_unchecked("action.email.send"))
    ///     .action(ActionBody::new(ActionType::Send, json!({})))
    ///     .build()
    ///     .expect("valid action");
    /// ```
    pub fn build(self) -> Result<Action, BuilderError> {
        // Check for missing required fields
        let mut missing = Vec::new();

        if self.topic.is_none() {
            missing.push("topic".to_string());
        }
        if self.action.is_none() {
            missing.push("action".to_string());
        }

        if !missing.is_empty() {
            return Err(BuilderError::MissingFields { fields: missing });
        }

        // Generate ID if not provided
        let id = self.id.unwrap_or_else(generate_action_id);

        // Use default version if not provided
        let version = self.version.unwrap_or_else(|| PROTOCOL_VERSION.to_string());

        // Use current timestamp if not provided
        let timestamp = self.timestamp.unwrap_or_else(Utc::now);

        Ok(Action {
            id,
            version,
            timestamp,
            topic: self.topic.unwrap(),
            action: self.action.unwrap(),
            context: self.context,
            encrypted: self.encrypted,
        })
    }
}

/// Generates a unique Action ID.
///
/// Format: `act_<unix_timestamp>_<random_12_chars>`
fn generate_action_id() -> String {
    let timestamp = Utc::now().timestamp();
    let random: String = Uuid::new_v4()
        .to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(12)
        .collect();
    format!("act_{}_{}", timestamp, random)
}

impl Action {
    /// Creates a new ActionBuilder.
    ///
    /// This is a convenience method equivalent to `ActionBuilder::new()`.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::{Action, ActionBody, ActionType, Topic};
    /// use serde_json::json;
    ///
    /// let action = Action::builder()
    ///     .topic(Topic::new_unchecked("action.email.send"))
    ///     .action(ActionBody::new(ActionType::Send, json!({})))
    ///     .build()
    ///     .expect("valid action");
    /// ```
    pub fn builder() -> ActionBuilder {
        ActionBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ActionType;
    use serde_json::json;

    fn test_topic() -> Topic {
        Topic::new_unchecked("action.email.send")
    }

    fn test_action_body() -> ActionBody {
        ActionBody::with_target(
            ActionType::Send,
            "bob@example.com",
            json!({"subject": "Hello"}),
        )
    }

    #[test]
    fn test_builder_with_all_required_fields() {
        let action = ActionBuilder::new()
            .topic(test_topic())
            .action(test_action_body())
            .build();

        assert!(action.is_ok());
        let action = action.unwrap();
        assert!(action.id.starts_with("act_"));
        assert_eq!(action.version, PROTOCOL_VERSION);
        assert_eq!(action.action.type_, ActionType::Send);
    }

    #[test]
    fn test_builder_with_custom_id() {
        let action = ActionBuilder::new()
            .id("act_1234567890_customid1234")
            .topic(test_topic())
            .action(test_action_body())
            .build()
            .unwrap();

        assert_eq!(action.id, "act_1234567890_customid1234");
    }

    #[test]
    fn test_builder_with_custom_version() {
        let action = ActionBuilder::new()
            .version("2.0")
            .topic(test_topic())
            .action(test_action_body())
            .build()
            .unwrap();

        assert_eq!(action.version, "2.0");
    }

    #[test]
    fn test_builder_with_custom_timestamp() {
        let timestamp = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let action = ActionBuilder::new()
            .timestamp(timestamp)
            .topic(test_topic())
            .action(test_action_body())
            .build()
            .unwrap();

        assert_eq!(action.timestamp, timestamp);
    }

    #[test]
    fn test_builder_with_context() {
        let action = ActionBuilder::new()
            .topic(test_topic())
            .action(test_action_body())
            .context(ActionContext::from_agent("my-agent"))
            .build()
            .unwrap();

        assert!(action.context.is_some());
        assert_eq!(
            action.context.unwrap().agent_id,
            Some("my-agent".to_string())
        );
    }

    #[test]
    fn test_builder_missing_topic() {
        let result = ActionBuilder::new().action(test_action_body()).build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("topic"));
    }

    #[test]
    fn test_builder_missing_action() {
        let result = ActionBuilder::new().topic(test_topic()).build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("action"));
    }

    #[test]
    fn test_builder_missing_all_required() {
        let result = ActionBuilder::new().build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("topic"));
        assert!(msg.contains("action"));
    }

    #[test]
    fn test_action_builder_method() {
        let action = Action::builder()
            .topic(test_topic())
            .action(test_action_body())
            .build()
            .unwrap();

        assert!(action.id.starts_with("act_"));
    }

    #[test]
    fn test_generate_action_id_format() {
        let id = generate_action_id();
        assert!(id.starts_with("act_"));
        let parts: Vec<&str> = id.split('_').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "act");
        assert!(parts[1].parse::<i64>().is_ok()); // timestamp
        assert_eq!(parts[2].len(), 12); // random part
    }

    #[test]
    fn test_generate_action_id_unique() {
        let id1 = generate_action_id();
        let id2 = generate_action_id();
        assert_ne!(id1, id2);
    }
}
