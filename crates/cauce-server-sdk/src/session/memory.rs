//! In-memory session manager implementation.

use async_trait::async_trait;
use chrono::{Duration, Utc};
use dashmap::DashMap;

use super::{SessionInfo, SessionManager};
use crate::error::{ServerError, ServerResult};

/// In-memory implementation of [`SessionManager`].
///
/// Uses DashMap for concurrent access. Suitable for single-server
/// deployments or development.
///
/// # Example
///
/// ```ignore
/// use cauce_server_sdk::session::InMemorySessionManager;
///
/// // Create with 1 hour session TTL
/// let manager = InMemorySessionManager::new(3600);
/// ```
pub struct InMemorySessionManager {
    /// Sessions indexed by session ID
    sessions: DashMap<String, SessionInfo>,
    /// Sessions indexed by client ID
    client_sessions: DashMap<String, Vec<String>>,
    /// Default session TTL in seconds
    session_ttl_secs: i64,
}

impl InMemorySessionManager {
    /// Creates a new in-memory session manager.
    ///
    /// # Arguments
    ///
    /// * `session_ttl_secs` - Default session TTL in seconds
    pub fn new(session_ttl_secs: i64) -> Self {
        Self {
            sessions: DashMap::new(),
            client_sessions: DashMap::new(),
            session_ttl_secs,
        }
    }

    /// Creates a session manager with default TTL (1 hour).
    pub fn with_default_ttl() -> Self {
        Self::new(3600)
    }

    /// Gets the configured session TTL.
    pub fn session_ttl(&self) -> i64 {
        self.session_ttl_secs
    }
}

impl Default for InMemorySessionManager {
    fn default() -> Self {
        Self::with_default_ttl()
    }
}

#[async_trait]
impl SessionManager for InMemorySessionManager {
    async fn create_session(&self, mut info: SessionInfo) -> ServerResult<String> {
        let session_id = info.session_id.clone();

        // Check if session already exists
        if self.sessions.contains_key(&session_id) {
            return Err(ServerError::InvalidSessionState {
                message: format!("session already exists: {}", session_id),
            });
        }

        // Update expiration based on configured TTL
        let now = Utc::now();
        info.last_activity = now;
        info.expires_at = now + Duration::seconds(self.session_ttl_secs);

        // Add to sessions
        self.sessions.insert(session_id.clone(), info.clone());

        // Add to client sessions index
        self.client_sessions
            .entry(info.client_id.clone())
            .or_default()
            .push(session_id.clone());

        Ok(session_id)
    }

    async fn get_session(&self, session_id: &str) -> ServerResult<Option<SessionInfo>> {
        match self.sessions.get(session_id) {
            Some(info) => {
                if info.is_expired() {
                    // Don't return expired sessions
                    Ok(None)
                } else {
                    Ok(Some(info.clone()))
                }
            }
            None => Ok(None),
        }
    }

    async fn touch_session(&self, session_id: &str) -> ServerResult<()> {
        match self.sessions.get_mut(session_id) {
            Some(mut info) => {
                if info.is_expired() {
                    return Err(ServerError::SessionExpired {
                        id: session_id.to_string(),
                    });
                }

                let now = Utc::now();
                info.last_activity = now;
                info.expires_at = now + Duration::seconds(self.session_ttl_secs);
                Ok(())
            }
            None => Err(ServerError::SessionNotFound {
                id: session_id.to_string(),
            }),
        }
    }

    async fn remove_session(&self, session_id: &str) -> ServerResult<()> {
        match self.sessions.remove(session_id) {
            Some((_, info)) => {
                // Remove from client sessions index
                if let Some(mut sessions) = self.client_sessions.get_mut(&info.client_id) {
                    sessions.retain(|id| id != session_id);
                }
                Ok(())
            }
            None => Err(ServerError::SessionNotFound {
                id: session_id.to_string(),
            }),
        }
    }

    async fn get_sessions_for_client(&self, client_id: &str) -> ServerResult<Vec<SessionInfo>> {
        let session_ids = self
            .client_sessions
            .get(client_id)
            .map(|ids| ids.clone())
            .unwrap_or_default();

        let mut result = Vec::new();
        for id in session_ids {
            if let Some(info) = self.sessions.get(&id) {
                if !info.is_expired() {
                    result.push(info.clone());
                }
            }
        }

        Ok(result)
    }

    async fn is_valid(&self, session_id: &str) -> ServerResult<bool> {
        match self.sessions.get(session_id) {
            Some(info) => Ok(!info.is_expired()),
            None => Ok(false),
        }
    }

    async fn cleanup_expired(&self) -> ServerResult<usize> {
        let mut expired_ids = Vec::new();

        // Find expired sessions
        for entry in self.sessions.iter() {
            if entry.is_expired() {
                expired_ids.push(entry.key().clone());
            }
        }

        // Remove them
        for id in &expired_ids {
            let _ = self.remove_session(id).await;
        }

        Ok(expired_ids.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cauce_core::methods::Transport;

    fn create_test_info(session_id: &str, client_id: &str) -> SessionInfo {
        SessionInfo::new(
            session_id,
            client_id,
            "agent",
            "1.0",
            Transport::WebSocket,
            3600,
        )
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = InMemorySessionManager::default();
        let info = create_test_info("sess_1", "client_1");

        let id = manager.create_session(info).await.unwrap();
        assert_eq!(id, "sess_1");

        let retrieved = manager.get_session("sess_1").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().client_id, "client_1");
    }

    #[tokio::test]
    async fn test_create_duplicate_session() {
        let manager = InMemorySessionManager::default();
        let info1 = create_test_info("sess_1", "client_1");
        let info2 = create_test_info("sess_1", "client_2");

        manager.create_session(info1).await.unwrap();
        let result = manager.create_session(info2).await;
        assert!(matches!(
            result,
            Err(ServerError::InvalidSessionState { .. })
        ));
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let manager = InMemorySessionManager::default();

        let result = manager.get_session("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_touch_session() {
        let manager = InMemorySessionManager::default();
        let info = create_test_info("sess_1", "client_1");
        manager.create_session(info).await.unwrap();

        let before = manager.get_session("sess_1").await.unwrap().unwrap();
        let before_activity = before.last_activity;

        // Sleep briefly to ensure time difference
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        manager.touch_session("sess_1").await.unwrap();

        let after = manager.get_session("sess_1").await.unwrap().unwrap();
        assert!(after.last_activity > before_activity);
    }

    #[tokio::test]
    async fn test_touch_nonexistent_session() {
        let manager = InMemorySessionManager::default();

        let result = manager.touch_session("nonexistent").await;
        assert!(matches!(result, Err(ServerError::SessionNotFound { .. })));
    }

    #[tokio::test]
    async fn test_remove_session() {
        let manager = InMemorySessionManager::default();
        let info = create_test_info("sess_1", "client_1");
        manager.create_session(info).await.unwrap();

        manager.remove_session("sess_1").await.unwrap();

        let result = manager.get_session("sess_1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_remove_nonexistent_session() {
        let manager = InMemorySessionManager::default();

        let result = manager.remove_session("nonexistent").await;
        assert!(matches!(result, Err(ServerError::SessionNotFound { .. })));
    }

    #[tokio::test]
    async fn test_get_sessions_for_client() {
        let manager = InMemorySessionManager::default();

        // Create multiple sessions for same client
        manager
            .create_session(create_test_info("sess_1", "client_1"))
            .await
            .unwrap();
        manager
            .create_session(create_test_info("sess_2", "client_1"))
            .await
            .unwrap();
        manager
            .create_session(create_test_info("sess_3", "client_2"))
            .await
            .unwrap();

        let sessions = manager.get_sessions_for_client("client_1").await.unwrap();
        assert_eq!(sessions.len(), 2);

        let ids: Vec<_> = sessions.iter().map(|s| s.session_id.as_str()).collect();
        assert!(ids.contains(&"sess_1"));
        assert!(ids.contains(&"sess_2"));
    }

    #[tokio::test]
    async fn test_is_valid() {
        let manager = InMemorySessionManager::default();
        let info = create_test_info("sess_1", "client_1");
        manager.create_session(info).await.unwrap();

        assert!(manager.is_valid("sess_1").await.unwrap());
        assert!(!manager.is_valid("nonexistent").await.unwrap());
    }

    #[tokio::test]
    async fn test_session_expiration() {
        // Create manager with very short TTL
        let manager = InMemorySessionManager::new(0);

        // Create a session with immediate expiration
        let mut info = create_test_info("sess_1", "client_1");
        info.expires_at = Utc::now() - Duration::seconds(1);

        // Manually insert expired session
        manager.sessions.insert("sess_1".to_string(), info);

        // Should not be returned (expired)
        let result = manager.get_session("sess_1").await.unwrap();
        assert!(result.is_none());

        // is_valid should return false
        assert!(!manager.is_valid("sess_1").await.unwrap());
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let manager = InMemorySessionManager::new(3600);

        // Create an expired session
        let mut info = create_test_info("sess_expired", "client_1");
        info.expires_at = Utc::now() - Duration::seconds(1);
        manager.sessions.insert("sess_expired".to_string(), info);

        // Create a valid session
        manager
            .create_session(create_test_info("sess_valid", "client_2"))
            .await
            .unwrap();

        let removed = manager.cleanup_expired().await.unwrap();
        assert_eq!(removed, 1);

        // Expired should be gone
        assert!(manager.sessions.get("sess_expired").is_none());
        // Valid should remain
        assert!(manager.sessions.get("sess_valid").is_some());
    }

    #[test]
    fn test_session_info_is_expired() {
        let mut info = create_test_info("sess_1", "client_1");
        assert!(!info.is_expired());

        info.expires_at = Utc::now() - Duration::seconds(1);
        assert!(info.is_expired());
    }

    #[test]
    fn test_session_info_ttl_secs() {
        let info = create_test_info("sess_1", "client_1");
        let ttl = info.ttl_secs();
        assert!(ttl > 3500 && ttl <= 3600); // Should be close to 3600
    }
}
