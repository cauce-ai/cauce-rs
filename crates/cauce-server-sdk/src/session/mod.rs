//! Session management for the Cauce server.
//!
//! This module provides the [`SessionManager`] trait and implementations
//! for managing client sessions and authentication state.

mod memory;

pub use memory::InMemorySessionManager;

use async_trait::async_trait;
use cauce_core::methods::Transport;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::ServerResult;

/// Information about a client session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Unique session identifier.
    pub session_id: String,
    /// Client identifier.
    pub client_id: String,
    /// Client type (adapter, agent, etc.).
    pub client_type: String,
    /// Protocol version negotiated.
    pub protocol_version: String,
    /// Transport used for this session.
    pub transport: Transport,
    /// When the session was created.
    pub created_at: DateTime<Utc>,
    /// When the session was last active.
    pub last_activity: DateTime<Utc>,
    /// When the session expires.
    pub expires_at: DateTime<Utc>,
    /// Additional metadata about the session.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl SessionInfo {
    /// Creates a new session info.
    pub fn new(
        session_id: impl Into<String>,
        client_id: impl Into<String>,
        client_type: impl Into<String>,
        protocol_version: impl Into<String>,
        transport: Transport,
        ttl_secs: i64,
    ) -> Self {
        let now = Utc::now();
        Self {
            session_id: session_id.into(),
            client_id: client_id.into(),
            client_type: client_type.into(),
            protocol_version: protocol_version.into(),
            transport,
            created_at: now,
            last_activity: now,
            expires_at: now + chrono::Duration::seconds(ttl_secs),
            metadata: None,
        }
    }

    /// Checks if the session has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Returns how many seconds until the session expires.
    pub fn ttl_secs(&self) -> i64 {
        let remaining = self.expires_at - Utc::now();
        remaining.num_seconds().max(0)
    }
}

/// Trait for managing client sessions.
///
/// Sessions track connected clients and their state. The session manager
/// handles creation, lookup, updates, and expiration of sessions.
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::session::{SessionManager, InMemorySessionManager, SessionInfo};
/// use cauce_core::methods::Transport;
///
/// let manager = InMemorySessionManager::new(3600); // 1 hour TTL
///
/// // Create a session from hello request info
/// let info = SessionInfo::new(
///     "sess_abc123",
///     "my-client",
///     "agent",
///     "1.0",
///     Transport::WebSocket,
///     3600,
/// );
/// let session_id = manager.create_session(info).await?;
///
/// // Touch to keep alive
/// manager.touch_session(&session_id).await?;
///
/// // Remove when done
/// manager.remove_session(&session_id).await?;
/// ```
#[async_trait]
pub trait SessionManager: Send + Sync + 'static {
    /// Creates a new session.
    ///
    /// # Arguments
    ///
    /// * `info` - Information about the session to create
    ///
    /// # Returns
    ///
    /// The session ID (may differ from info.session_id if auto-generated).
    async fn create_session(&self, info: SessionInfo) -> ServerResult<String>;

    /// Gets session information.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session ID to look up
    ///
    /// # Returns
    ///
    /// The session info if found, or None if not found or expired.
    async fn get_session(&self, session_id: &str) -> ServerResult<Option<SessionInfo>>;

    /// Updates the session's last activity timestamp.
    ///
    /// Also extends the expiration time. Call this on any client activity.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to update
    async fn touch_session(&self, session_id: &str) -> ServerResult<()>;

    /// Removes a session.
    ///
    /// Call this when a client disconnects or on explicit logout.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to remove
    async fn remove_session(&self, session_id: &str) -> ServerResult<()>;

    /// Gets all sessions for a client.
    ///
    /// A client may have multiple active sessions (e.g., multiple connections).
    ///
    /// # Arguments
    ///
    /// * `client_id` - The client ID to look up
    async fn get_sessions_for_client(&self, client_id: &str) -> ServerResult<Vec<SessionInfo>>;

    /// Checks if a session is valid and not expired.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The session to check
    async fn is_valid(&self, session_id: &str) -> ServerResult<bool>;

    /// Cleans up expired sessions.
    ///
    /// Call this periodically to remove stale sessions.
    ///
    /// # Returns
    ///
    /// Number of sessions removed.
    async fn cleanup_expired(&self) -> ServerResult<usize>;
}
