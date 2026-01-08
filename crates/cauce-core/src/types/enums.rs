//! Enumeration types for the Cauce Protocol.
//!
//! This module provides enum types used throughout the protocol:
//!
//! - [`Priority`] - Message priority levels
//! - [`ActionType`] - Types of actions that can be performed

use serde::{Deserialize, Serialize};

/// Message priority levels.
///
/// Priority indicates the urgency of a message and may affect
/// how quickly it is processed or delivered.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `Low` → `"low"`
/// - `Normal` → `"normal"`
/// - `High` → `"high"`
/// - `Urgent` → `"urgent"`
///
/// # Example
///
/// ```
/// use cauce_core::types::Priority;
///
/// let priority = Priority::Normal;
/// assert_eq!(serde_json::to_string(&priority).unwrap(), "\"normal\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Non-urgent, batch processing OK
    Low,
    /// Default priority
    #[default]
    Normal,
    /// Should be processed soon
    High,
    /// Requires immediate attention
    Urgent,
}

/// Types of actions that can be performed.
///
/// ActionType indicates the kind of operation an agent wants
/// an adapter to perform.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `Send` → `"send"`
/// - `Reply` → `"reply"`
/// - `Forward` → `"forward"`
/// - `React` → `"react"`
/// - `Update` → `"update"`
/// - `Delete` → `"delete"`
///
/// # Example
///
/// ```
/// use cauce_core::types::ActionType;
///
/// let action_type = ActionType::Send;
/// assert_eq!(serde_json::to_string(&action_type).unwrap(), "\"send\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Send a new message
    Send,
    /// Reply to an existing message
    Reply,
    /// Forward a message to another recipient
    Forward,
    /// Add a reaction to a message
    React,
    /// Edit an existing message
    Update,
    /// Delete a message
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_default() {
        assert_eq!(Priority::default(), Priority::Normal);
    }

    #[test]
    fn test_priority_serialization() {
        assert_eq!(serde_json::to_string(&Priority::Low).unwrap(), "\"low\"");
        assert_eq!(
            serde_json::to_string(&Priority::Normal).unwrap(),
            "\"normal\""
        );
        assert_eq!(serde_json::to_string(&Priority::High).unwrap(), "\"high\"");
        assert_eq!(
            serde_json::to_string(&Priority::Urgent).unwrap(),
            "\"urgent\""
        );
    }

    #[test]
    fn test_priority_deserialization() {
        assert_eq!(
            serde_json::from_str::<Priority>("\"low\"").unwrap(),
            Priority::Low
        );
        assert_eq!(
            serde_json::from_str::<Priority>("\"normal\"").unwrap(),
            Priority::Normal
        );
        assert_eq!(
            serde_json::from_str::<Priority>("\"high\"").unwrap(),
            Priority::High
        );
        assert_eq!(
            serde_json::from_str::<Priority>("\"urgent\"").unwrap(),
            Priority::Urgent
        );
    }

    #[test]
    fn test_action_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ActionType::Send).unwrap(),
            "\"send\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::Reply).unwrap(),
            "\"reply\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::Forward).unwrap(),
            "\"forward\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::React).unwrap(),
            "\"react\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::Update).unwrap(),
            "\"update\""
        );
        assert_eq!(
            serde_json::to_string(&ActionType::Delete).unwrap(),
            "\"delete\""
        );
    }

    #[test]
    fn test_action_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<ActionType>("\"send\"").unwrap(),
            ActionType::Send
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"reply\"").unwrap(),
            ActionType::Reply
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"forward\"").unwrap(),
            ActionType::Forward
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"react\"").unwrap(),
            ActionType::React
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"update\"").unwrap(),
            ActionType::Update
        );
        assert_eq!(
            serde_json::from_str::<ActionType>("\"delete\"").unwrap(),
            ActionType::Delete
        );
    }
}
