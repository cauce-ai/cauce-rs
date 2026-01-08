//! Metadata type for the Cauce Protocol.
//!
//! The [`Metadata`] struct contains optional threading and priority information.

use serde::{Deserialize, Serialize};

use super::Priority;

/// Optional threading and priority information.
///
/// Metadata provides additional context for signals, including
/// conversation threading, reply relationships, and priority.
///
/// # Fields
///
/// All fields are optional:
/// - `thread_id` - Conversation thread identifier
/// - `in_reply_to` - Signal/Action ID being replied to
/// - `references` - Related Signal/Action IDs
/// - `priority` - Message priority (defaults to Normal)
/// - `tags` - User-defined labels
///
/// # Example
///
/// ```
/// use cauce_core::types::{Metadata, Priority};
///
/// let metadata = Metadata {
///     thread_id: Some("thread-123".to_string()),
///     in_reply_to: Some("sig_1234_abc".to_string()),
///     references: None,
///     priority: Some(Priority::High),
///     tags: Some(vec!["important".to_string()]),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Metadata {
    /// Conversation thread identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// Signal/Action ID being replied to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,

    /// Related Signal/Action IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<String>>,

    /// Message priority (defaults to Normal if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<Priority>,

    /// User-defined labels
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl Metadata {
    /// Creates a new empty Metadata.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Metadata;
    ///
    /// let metadata = Metadata::new();
    /// assert!(metadata.thread_id.is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates Metadata with a thread ID.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Metadata;
    ///
    /// let metadata = Metadata::with_thread("thread-123");
    /// assert_eq!(metadata.thread_id, Some("thread-123".to_string()));
    /// ```
    pub fn with_thread(thread_id: impl Into<String>) -> Self {
        Self {
            thread_id: Some(thread_id.into()),
            ..Default::default()
        }
    }

    /// Creates Metadata for a reply.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Metadata;
    ///
    /// let metadata = Metadata::reply_to("sig_1234_abc");
    /// assert_eq!(metadata.in_reply_to, Some("sig_1234_abc".to_string()));
    /// ```
    pub fn reply_to(signal_id: impl Into<String>) -> Self {
        Self {
            in_reply_to: Some(signal_id.into()),
            ..Default::default()
        }
    }

    /// Sets the priority.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::{Metadata, Priority};
    ///
    /// let metadata = Metadata::new().priority(Priority::High);
    /// assert_eq!(metadata.priority, Some(Priority::High));
    /// ```
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Adds tags.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::Metadata;
    ///
    /// let metadata = Metadata::new().tags(vec!["urgent".to_string()]);
    /// assert!(metadata.tags.is_some());
    /// ```
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_default() {
        let metadata = Metadata::default();
        assert!(metadata.thread_id.is_none());
        assert!(metadata.in_reply_to.is_none());
        assert!(metadata.references.is_none());
        assert!(metadata.priority.is_none());
        assert!(metadata.tags.is_none());
    }

    #[test]
    fn test_metadata_with_thread() {
        let metadata = Metadata::with_thread("thread-1");
        assert_eq!(metadata.thread_id, Some("thread-1".to_string()));
    }

    #[test]
    fn test_metadata_reply_to() {
        let metadata = Metadata::reply_to("sig_1234_abc123def456");
        assert_eq!(
            metadata.in_reply_to,
            Some("sig_1234_abc123def456".to_string())
        );
    }

    #[test]
    fn test_metadata_builder_chain() {
        let metadata = Metadata::new()
            .priority(Priority::Urgent)
            .tags(vec!["important".to_string(), "urgent".to_string()]);

        assert_eq!(metadata.priority, Some(Priority::Urgent));
        assert_eq!(
            metadata.tags,
            Some(vec!["important".to_string(), "urgent".to_string()])
        );
    }

    #[test]
    fn test_metadata_serialization_skips_none() {
        let metadata = Metadata::new();
        let json = serde_json::to_string(&metadata).unwrap();
        // Empty metadata should serialize to empty object
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_metadata_serialization_with_values() {
        let metadata = Metadata {
            thread_id: Some("thread-1".to_string()),
            priority: Some(Priority::High),
            ..Default::default()
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"thread_id\":\"thread-1\""));
        assert!(json.contains("\"priority\":\"high\""));
        // None fields should not appear
        assert!(!json.contains("in_reply_to"));
    }

    #[test]
    fn test_metadata_round_trip() {
        let metadata = Metadata {
            thread_id: Some("thread-1".to_string()),
            in_reply_to: Some("sig_1_abc123def456".to_string()),
            references: Some(vec!["ref1".to_string(), "ref2".to_string()]),
            priority: Some(Priority::Low),
            tags: Some(vec!["tag1".to_string()]),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let restored: Metadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata, restored);
    }
}
