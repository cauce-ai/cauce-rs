//! SignalBuilder for creating Signal instances.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::constants::PROTOCOL_VERSION;
use crate::errors::BuilderError;
use crate::types::{Encrypted, Metadata, Payload, Signal, Source, Topic};

/// Fluent builder for creating [`Signal`] instances.
///
/// SignalBuilder provides an ergonomic way to construct Signal instances
/// with validation. Required fields must be set before calling `build()`.
///
/// # Required Fields
///
/// - `source` - Origin information
/// - `topic` - Routing topic
/// - `payload` - Message content
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
/// use cauce_core::builders::SignalBuilder;
/// use cauce_core::types::{Source, Payload, Topic, Priority, Metadata};
/// use serde_json::json;
///
/// let signal = SignalBuilder::new()
///     .source(Source::new("email", "email-adapter-1", "msg-12345"))
///     .topic(Topic::new_unchecked("signal.email.received"))
///     .payload(Payload::new(json!({"from": "alice@example.com"}), "application/json"))
///     .metadata(Metadata::new().priority(Priority::High))
///     .build()
///     .expect("valid signal");
///
/// assert!(signal.id.starts_with("sig_"));
/// assert_eq!(signal.source.type_, "email");
/// ```
#[derive(Debug, Default)]
pub struct SignalBuilder {
    id: Option<String>,
    version: Option<String>,
    timestamp: Option<DateTime<Utc>>,
    source: Option<Source>,
    topic: Option<Topic>,
    payload: Option<Payload>,
    metadata: Option<Metadata>,
    encrypted: Option<Encrypted>,
}

impl SignalBuilder {
    /// Creates a new SignalBuilder.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    ///
    /// let builder = SignalBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the signal ID.
    ///
    /// If not called, a unique ID will be auto-generated.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    ///
    /// let builder = SignalBuilder::new()
    ///     .id("sig_1704067200_abc123def456");
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
    /// use cauce_core::builders::SignalBuilder;
    ///
    /// let builder = SignalBuilder::new().version("1.0");
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
    /// use cauce_core::builders::SignalBuilder;
    /// use chrono::Utc;
    ///
    /// let builder = SignalBuilder::new().timestamp(Utc::now());
    /// ```
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Sets the source (required).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    /// use cauce_core::types::Source;
    ///
    /// let builder = SignalBuilder::new()
    ///     .source(Source::new("email", "email-1", "msg-1"));
    /// ```
    pub fn source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    /// Sets the topic (required).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    /// use cauce_core::types::Topic;
    ///
    /// let builder = SignalBuilder::new()
    ///     .topic(Topic::new_unchecked("signal.email.received"));
    /// ```
    pub fn topic(mut self, topic: Topic) -> Self {
        self.topic = Some(topic);
        self
    }

    /// Sets the payload (required).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    /// use cauce_core::types::Payload;
    /// use serde_json::json;
    ///
    /// let builder = SignalBuilder::new()
    ///     .payload(Payload::new(json!({}), "application/json"));
    /// ```
    pub fn payload(mut self, payload: Payload) -> Self {
        self.payload = Some(payload);
        self
    }

    /// Sets the metadata (optional).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    /// use cauce_core::types::{Metadata, Priority};
    ///
    /// let builder = SignalBuilder::new()
    ///     .metadata(Metadata::new().priority(Priority::High));
    /// ```
    pub fn metadata(mut self, metadata: Metadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the encrypted envelope (optional).
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    /// use cauce_core::types::{Encrypted, EncryptionAlgorithm};
    ///
    /// let encrypted = Encrypted::new(
    ///     EncryptionAlgorithm::A256Gcm,
    ///     "public_key",
    ///     "nonce",
    ///     "ciphertext",
    /// );
    /// let builder = SignalBuilder::new().encrypted(encrypted);
    /// ```
    pub fn encrypted(mut self, encrypted: Encrypted) -> Self {
        self.encrypted = Some(encrypted);
        self
    }

    /// Builds the Signal, returning an error if required fields are missing.
    ///
    /// # Returns
    ///
    /// - `Ok(Signal)` if all required fields are set
    /// - `Err(BuilderError)` if required fields are missing
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::builders::SignalBuilder;
    /// use cauce_core::types::{Source, Payload, Topic};
    /// use serde_json::json;
    ///
    /// let signal = SignalBuilder::new()
    ///     .source(Source::new("email", "email-1", "msg-1"))
    ///     .topic(Topic::new_unchecked("signal.email.received"))
    ///     .payload(Payload::new(json!({}), "application/json"))
    ///     .build()
    ///     .expect("valid signal");
    /// ```
    pub fn build(self) -> Result<Signal, BuilderError> {
        // Check for missing required fields
        let mut missing = Vec::new();

        if self.source.is_none() {
            missing.push("source".to_string());
        }
        if self.topic.is_none() {
            missing.push("topic".to_string());
        }
        if self.payload.is_none() {
            missing.push("payload".to_string());
        }

        if !missing.is_empty() {
            return Err(BuilderError::MissingFields { fields: missing });
        }

        // Generate ID if not provided
        let id = self.id.unwrap_or_else(generate_signal_id);

        // Use default version if not provided
        let version = self.version.unwrap_or_else(|| PROTOCOL_VERSION.to_string());

        // Use current timestamp if not provided
        let timestamp = self.timestamp.unwrap_or_else(Utc::now);

        Ok(Signal {
            id,
            version,
            timestamp,
            source: self.source.unwrap(),
            topic: self.topic.unwrap(),
            payload: self.payload.unwrap(),
            metadata: self.metadata,
            encrypted: self.encrypted,
        })
    }
}

/// Generates a unique Signal ID.
///
/// Format: `sig_<unix_timestamp>_<random_12_chars>`
fn generate_signal_id() -> String {
    let timestamp = Utc::now().timestamp();
    let random: String = Uuid::new_v4()
        .to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(12)
        .collect();
    format!("sig_{}_{}", timestamp, random)
}

impl Signal {
    /// Creates a new SignalBuilder.
    ///
    /// This is a convenience method equivalent to `SignalBuilder::new()`.
    ///
    /// # Example
    ///
    /// ```
    /// use cauce_core::types::{Signal, Source, Payload, Topic};
    /// use serde_json::json;
    ///
    /// let signal = Signal::builder()
    ///     .source(Source::new("email", "email-1", "msg-1"))
    ///     .topic(Topic::new_unchecked("signal.email.received"))
    ///     .payload(Payload::new(json!({}), "application/json"))
    ///     .build()
    ///     .expect("valid signal");
    /// ```
    pub fn builder() -> SignalBuilder {
        SignalBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Priority;
    use serde_json::json;

    fn test_source() -> Source {
        Source::new("email", "email-adapter-1", "msg-12345")
    }

    fn test_topic() -> Topic {
        Topic::new_unchecked("signal.email.received")
    }

    fn test_payload() -> Payload {
        Payload::new(json!({"from": "alice@example.com"}), "application/json")
    }

    #[test]
    fn test_builder_with_all_required_fields() {
        let signal = SignalBuilder::new()
            .source(test_source())
            .topic(test_topic())
            .payload(test_payload())
            .build();

        assert!(signal.is_ok());
        let signal = signal.unwrap();
        assert!(signal.id.starts_with("sig_"));
        assert_eq!(signal.version, PROTOCOL_VERSION);
        assert_eq!(signal.source.type_, "email");
    }

    #[test]
    fn test_builder_with_custom_id() {
        let signal = SignalBuilder::new()
            .id("sig_1234567890_customid1234")
            .source(test_source())
            .topic(test_topic())
            .payload(test_payload())
            .build()
            .unwrap();

        assert_eq!(signal.id, "sig_1234567890_customid1234");
    }

    #[test]
    fn test_builder_with_custom_version() {
        let signal = SignalBuilder::new()
            .version("2.0")
            .source(test_source())
            .topic(test_topic())
            .payload(test_payload())
            .build()
            .unwrap();

        assert_eq!(signal.version, "2.0");
    }

    #[test]
    fn test_builder_with_custom_timestamp() {
        let timestamp = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let signal = SignalBuilder::new()
            .timestamp(timestamp)
            .source(test_source())
            .topic(test_topic())
            .payload(test_payload())
            .build()
            .unwrap();

        assert_eq!(signal.timestamp, timestamp);
    }

    #[test]
    fn test_builder_with_metadata() {
        let signal = SignalBuilder::new()
            .source(test_source())
            .topic(test_topic())
            .payload(test_payload())
            .metadata(Metadata::new().priority(Priority::High))
            .build()
            .unwrap();

        assert!(signal.metadata.is_some());
        assert_eq!(signal.metadata.unwrap().priority, Some(Priority::High));
    }

    #[test]
    fn test_builder_missing_source() {
        let result = SignalBuilder::new()
            .topic(test_topic())
            .payload(test_payload())
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("source"));
    }

    #[test]
    fn test_builder_missing_topic() {
        let result = SignalBuilder::new()
            .source(test_source())
            .payload(test_payload())
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("topic"));
    }

    #[test]
    fn test_builder_missing_payload() {
        let result = SignalBuilder::new()
            .source(test_source())
            .topic(test_topic())
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("payload"));
    }

    #[test]
    fn test_builder_missing_all_required() {
        let result = SignalBuilder::new().build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("source"));
        assert!(msg.contains("topic"));
        assert!(msg.contains("payload"));
    }

    #[test]
    fn test_signal_builder_method() {
        let signal = Signal::builder()
            .source(test_source())
            .topic(test_topic())
            .payload(test_payload())
            .build()
            .unwrap();

        assert!(signal.id.starts_with("sig_"));
    }

    #[test]
    fn test_generate_signal_id_format() {
        let id = generate_signal_id();
        assert!(id.starts_with("sig_"));
        let parts: Vec<&str> = id.split('_').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "sig");
        assert!(parts[1].parse::<i64>().is_ok()); // timestamp
        assert_eq!(parts[2].len(), 12); // random part
    }

    #[test]
    fn test_generate_signal_id_unique() {
        let id1 = generate_signal_id();
        let id2 = generate_signal_id();
        assert_ne!(id1, id2);
    }
}
