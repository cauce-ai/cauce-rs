//! Client type and capability definitions for the Cauce Protocol.
//!
//! This module provides types that describe clients connecting to the hub.

use serde::{Deserialize, Serialize};

/// Type of client connecting to the hub.
///
/// Determines the role and permissions of the connecting client.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `Adapter` → `"adapter"`
/// - `Agent` → `"agent"`
/// - `A2aAgent` → `"a2a_agent"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientType {
    /// An adapter that bridges external platforms (email, Slack, etc.)
    Adapter,
    /// An AI agent that processes signals and creates actions
    Agent,
    /// An agent-to-agent communication client
    A2aAgent,
}

/// Capabilities that a client can support or request.
///
/// Used during the Hello handshake to negotiate features.
///
/// # JSON Serialization
///
/// Serializes as lowercase snake_case strings:
/// - `Subscribe` → `"subscribe"`
/// - `Publish` → `"publish"`
/// - `Ack` → `"ack"`
/// - `E2eEncryption` → `"e2e_encryption"`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    /// Ability to subscribe to topics and receive signals
    Subscribe,
    /// Ability to publish signals or actions
    Publish,
    /// Ability to acknowledge signal receipt
    Ack,
    /// Support for end-to-end encryption
    E2eEncryption,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ClientType Tests =====

    #[test]
    fn test_client_type_serialization() {
        assert_eq!(
            serde_json::to_string(&ClientType::Adapter).unwrap(),
            "\"adapter\""
        );
        assert_eq!(
            serde_json::to_string(&ClientType::Agent).unwrap(),
            "\"agent\""
        );
        assert_eq!(
            serde_json::to_string(&ClientType::A2aAgent).unwrap(),
            "\"a2a_agent\""
        );
    }

    #[test]
    fn test_client_type_deserialization() {
        assert_eq!(
            serde_json::from_str::<ClientType>("\"adapter\"").unwrap(),
            ClientType::Adapter
        );
        assert_eq!(
            serde_json::from_str::<ClientType>("\"agent\"").unwrap(),
            ClientType::Agent
        );
        assert_eq!(
            serde_json::from_str::<ClientType>("\"a2a_agent\"").unwrap(),
            ClientType::A2aAgent
        );
    }

    #[test]
    fn test_client_type_roundtrip() {
        for client_type in [ClientType::Adapter, ClientType::Agent, ClientType::A2aAgent] {
            let json = serde_json::to_string(&client_type).unwrap();
            let restored: ClientType = serde_json::from_str(&json).unwrap();
            assert_eq!(client_type, restored);
        }
    }

    // ===== Capability Tests =====

    #[test]
    fn test_capability_serialization() {
        assert_eq!(
            serde_json::to_string(&Capability::Subscribe).unwrap(),
            "\"subscribe\""
        );
        assert_eq!(
            serde_json::to_string(&Capability::Publish).unwrap(),
            "\"publish\""
        );
        assert_eq!(serde_json::to_string(&Capability::Ack).unwrap(), "\"ack\"");
        assert_eq!(
            serde_json::to_string(&Capability::E2eEncryption).unwrap(),
            "\"e2e_encryption\""
        );
    }

    #[test]
    fn test_capability_deserialization() {
        assert_eq!(
            serde_json::from_str::<Capability>("\"subscribe\"").unwrap(),
            Capability::Subscribe
        );
        assert_eq!(
            serde_json::from_str::<Capability>("\"publish\"").unwrap(),
            Capability::Publish
        );
        assert_eq!(
            serde_json::from_str::<Capability>("\"ack\"").unwrap(),
            Capability::Ack
        );
        assert_eq!(
            serde_json::from_str::<Capability>("\"e2e_encryption\"").unwrap(),
            Capability::E2eEncryption
        );
    }

    #[test]
    fn test_capability_roundtrip() {
        for cap in [
            Capability::Subscribe,
            Capability::Publish,
            Capability::Ack,
            Capability::E2eEncryption,
        ] {
            let json = serde_json::to_string(&cap).unwrap();
            let restored: Capability = serde_json::from_str(&json).unwrap();
            assert_eq!(cap, restored);
        }
    }

    #[test]
    fn test_capability_array_serialization() {
        let caps = vec![Capability::Subscribe, Capability::Publish];
        let json = serde_json::to_string(&caps).unwrap();
        assert_eq!(json, "[\"subscribe\",\"publish\"]");
    }

    #[test]
    fn test_capability_array_deserialization() {
        let json = r#"["subscribe", "publish", "e2e_encryption"]"#;
        let caps: Vec<Capability> = serde_json::from_str(json).unwrap();
        assert_eq!(caps.len(), 3);
        assert_eq!(caps[0], Capability::Subscribe);
        assert_eq!(caps[1], Capability::Publish);
        assert_eq!(caps[2], Capability::E2eEncryption);
    }
}
