//! Ping/Pong method types for the Cauce Protocol.
//!
//! Used for connection keep-alive and latency measurement.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Parameters for the `cauce.ping` method.
///
/// Sent by clients to check connection liveness.
///
/// # Example
///
/// ```
/// use cauce_core::methods::PingParams;
/// use chrono::Utc;
///
/// let ping = PingParams { timestamp: Utc::now() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PingParams {
    /// Timestamp when the ping was sent
    pub timestamp: DateTime<Utc>,
}

impl PingParams {
    /// Creates a new PingParams with the current timestamp.
    pub fn now() -> Self {
        Self {
            timestamp: Utc::now(),
        }
    }

    /// Creates a PingParams with a specific timestamp.
    pub fn at(timestamp: DateTime<Utc>) -> Self {
        Self { timestamp }
    }
}

impl Default for PingParams {
    fn default() -> Self {
        Self::now()
    }
}

/// Parameters for the `cauce.pong` response.
///
/// Sent in response to a ping.
///
/// # Example
///
/// ```
/// use cauce_core::methods::PongParams;
/// use chrono::Utc;
///
/// let pong = PongParams { timestamp: Utc::now() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PongParams {
    /// Timestamp when the pong was sent
    pub timestamp: DateTime<Utc>,
}

impl PongParams {
    /// Creates a new PongParams with the current timestamp.
    pub fn now() -> Self {
        Self {
            timestamp: Utc::now(),
        }
    }

    /// Creates a PongParams with a specific timestamp.
    pub fn at(timestamp: DateTime<Utc>) -> Self {
        Self { timestamp }
    }

    /// Creates a PongParams from a PingParams (echoing the ping timestamp or using current time).
    pub fn from_ping(_ping: &PingParams) -> Self {
        Self::now()
    }
}

impl Default for PongParams {
    fn default() -> Self {
        Self::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== PingParams Tests =====

    #[test]
    fn test_ping_params_now() {
        let ping = PingParams::now();
        // Just verify it doesn't panic and has a reasonable timestamp
        assert!(ping.timestamp <= Utc::now());
    }

    #[test]
    fn test_ping_params_at() {
        let ts = DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let ping = PingParams::at(ts);
        assert_eq!(ping.timestamp, ts);
    }

    #[test]
    fn test_ping_params_default() {
        let ping = PingParams::default();
        assert!(ping.timestamp <= Utc::now());
    }

    #[test]
    fn test_ping_params_serialization() {
        let ts = DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let ping = PingParams::at(ts);
        let json = serde_json::to_string(&ping).unwrap();

        assert!(json.contains("\"timestamp\":"));
        assert!(json.contains("2024-01-01"));
    }

    #[test]
    fn test_ping_params_deserialization() {
        let json = r#"{"timestamp":"2024-06-15T10:30:00Z"}"#;
        let ping: PingParams = serde_json::from_str(json).unwrap();

        let expected = DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(ping.timestamp, expected);
    }

    #[test]
    fn test_ping_params_roundtrip() {
        let ts = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let ping = PingParams::at(ts);

        let json = serde_json::to_string(&ping).unwrap();
        let restored: PingParams = serde_json::from_str(&json).unwrap();
        assert_eq!(ping, restored);
    }

    // ===== PongParams Tests =====

    #[test]
    fn test_pong_params_now() {
        let pong = PongParams::now();
        assert!(pong.timestamp <= Utc::now());
    }

    #[test]
    fn test_pong_params_at() {
        let ts = DateTime::parse_from_rfc3339("2024-06-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let pong = PongParams::at(ts);
        assert_eq!(pong.timestamp, ts);
    }

    #[test]
    fn test_pong_params_from_ping() {
        let ping = PingParams::now();
        let pong = PongParams::from_ping(&ping);
        // Pong timestamp should be >= ping timestamp (or very close)
        assert!(
            pong.timestamp >= ping.timestamp
                || (ping.timestamp - pong.timestamp).num_milliseconds() < 1000
        );
    }

    #[test]
    fn test_pong_params_default() {
        let pong = PongParams::default();
        assert!(pong.timestamp <= Utc::now());
    }

    #[test]
    fn test_pong_params_serialization() {
        let ts = DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let pong = PongParams::at(ts);
        let json = serde_json::to_string(&pong).unwrap();

        assert!(json.contains("\"timestamp\":"));
    }

    #[test]
    fn test_pong_params_deserialization() {
        let json = r#"{"timestamp":"2024-06-15T10:30:01Z"}"#;
        let pong: PongParams = serde_json::from_str(json).unwrap();

        let expected = DateTime::parse_from_rfc3339("2024-06-15T10:30:01Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(pong.timestamp, expected);
    }

    #[test]
    fn test_pong_params_roundtrip() {
        let ts = DateTime::parse_from_rfc3339("2024-12-31T23:59:59Z")
            .unwrap()
            .with_timezone(&Utc);
        let pong = PongParams::at(ts);

        let json = serde_json::to_string(&pong).unwrap();
        let restored: PongParams = serde_json::from_str(&json).unwrap();
        assert_eq!(pong, restored);
    }
}
