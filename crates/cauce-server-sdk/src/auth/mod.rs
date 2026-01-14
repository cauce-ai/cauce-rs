//! Authentication middleware for the Cauce server.
//!
//! This module provides authentication validation via API keys and Bearer tokens.
//!
//! # Example
//!
//! ```ignore
//! use axum::Router;
//! use cauce_server_sdk::auth::{AuthConfig, AuthMiddleware, InMemoryAuthValidator};
//!
//! let validator = InMemoryAuthValidator::new()
//!     .with_api_key("client-1", "sk_live_abc123");
//!
//! let auth = AuthMiddleware::new(validator);
//!
//! let app = Router::new()
//!     .route("/protected", get(handler))
//!     .layer(auth.layer());
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};
use tracing::{debug, warn};

use crate::error::ServerResult;

/// Authentication method used.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    /// API key in X-Cauce-API-Key header.
    ApiKey,
    /// Bearer token in Authorization header.
    BearerToken,
    /// No authentication.
    None,
}

/// Result of authentication validation.
#[derive(Debug, Clone)]
pub struct AuthResult {
    /// Whether authentication succeeded.
    pub authenticated: bool,
    /// The client ID associated with the credentials.
    pub client_id: Option<String>,
    /// The method used for authentication.
    pub method: AuthMethod,
    /// Error message if authentication failed.
    pub error: Option<String>,
}

impl AuthResult {
    /// Creates a successful auth result.
    pub fn success(client_id: impl Into<String>, method: AuthMethod) -> Self {
        Self {
            authenticated: true,
            client_id: Some(client_id.into()),
            method,
            error: None,
        }
    }

    /// Creates a failed auth result.
    pub fn failure(method: AuthMethod, error: impl Into<String>) -> Self {
        Self {
            authenticated: false,
            client_id: None,
            method,
            error: Some(error.into()),
        }
    }

    /// Creates a result for no authentication provided.
    pub fn none() -> Self {
        Self {
            authenticated: false,
            client_id: None,
            method: AuthMethod::None,
            error: Some("No authentication provided".to_string()),
        }
    }
}

/// Trait for validating authentication credentials.
#[async_trait]
pub trait AuthValidator: Send + Sync + 'static {
    /// Validate an API key and return the associated client ID.
    async fn validate_api_key(&self, api_key: &str) -> ServerResult<Option<String>>;

    /// Validate a bearer token and return the associated client ID.
    async fn validate_bearer_token(&self, token: &str) -> ServerResult<Option<String>>;
}

/// In-memory authentication validator.
///
/// Stores API keys and bearer tokens in memory for validation.
#[derive(Debug, Clone, Default)]
pub struct InMemoryAuthValidator {
    /// Map of API key -> client ID.
    api_keys: Arc<DashMap<String, String>>,
    /// Map of bearer token -> client ID.
    bearer_tokens: Arc<DashMap<String, String>>,
}

impl InMemoryAuthValidator {
    /// Creates a new empty validator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an API key for a client.
    pub fn with_api_key(self, client_id: impl Into<String>, api_key: impl Into<String>) -> Self {
        self.api_keys.insert(api_key.into(), client_id.into());
        self
    }

    /// Adds a bearer token for a client.
    pub fn with_bearer_token(
        self,
        client_id: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        self.bearer_tokens.insert(token.into(), client_id.into());
        self
    }

    /// Adds an API key for a client (mutable).
    pub fn add_api_key(&self, client_id: impl Into<String>, api_key: impl Into<String>) {
        self.api_keys.insert(api_key.into(), client_id.into());
    }

    /// Adds a bearer token for a client (mutable).
    pub fn add_bearer_token(&self, client_id: impl Into<String>, token: impl Into<String>) {
        self.bearer_tokens.insert(token.into(), client_id.into());
    }

    /// Removes an API key.
    pub fn remove_api_key(&self, api_key: &str) -> Option<String> {
        self.api_keys.remove(api_key).map(|(_, v)| v)
    }

    /// Removes a bearer token.
    pub fn remove_bearer_token(&self, token: &str) -> Option<String> {
        self.bearer_tokens.remove(token).map(|(_, v)| v)
    }

    /// Returns the number of registered API keys.
    pub fn api_key_count(&self) -> usize {
        self.api_keys.len()
    }

    /// Returns the number of registered bearer tokens.
    pub fn bearer_token_count(&self) -> usize {
        self.bearer_tokens.len()
    }
}

#[async_trait]
impl AuthValidator for InMemoryAuthValidator {
    async fn validate_api_key(&self, api_key: &str) -> ServerResult<Option<String>> {
        Ok(self.api_keys.get(api_key).map(|v| v.clone()))
    }

    async fn validate_bearer_token(&self, token: &str) -> ServerResult<Option<String>> {
        Ok(self.bearer_tokens.get(token).map(|v| v.clone()))
    }
}

/// Authentication middleware for axum.
pub struct AuthMiddleware<V: AuthValidator> {
    validator: Arc<V>,
    /// Whether to allow unauthenticated requests.
    allow_anonymous: bool,
}

impl<V: AuthValidator> AuthMiddleware<V> {
    /// Creates a new auth middleware.
    pub fn new(validator: V) -> Self {
        Self {
            validator: Arc::new(validator),
            allow_anonymous: false,
        }
    }

    /// Creates middleware with a shared validator.
    pub fn with_shared(validator: Arc<V>) -> Self {
        Self {
            validator,
            allow_anonymous: false,
        }
    }

    /// Allow unauthenticated requests to pass through.
    pub fn allow_anonymous(mut self) -> Self {
        self.allow_anonymous = true;
        self
    }

    /// Creates a tower Layer for this middleware.
    pub fn layer(self) -> AuthLayer<V> {
        AuthLayer {
            validator: self.validator,
            allow_anonymous: self.allow_anonymous,
        }
    }

    /// Validate authentication from request headers.
    pub async fn validate_request<B>(&self, request: &Request<B>) -> AuthResult {
        // Try API key first
        if let Some(api_key) = request
            .headers()
            .get("X-Cauce-API-Key")
            .and_then(|v| v.to_str().ok())
        {
            match self.validator.validate_api_key(api_key).await {
                Ok(Some(client_id)) => {
                    debug!("API key authenticated for client: {}", client_id);
                    return AuthResult::success(client_id, AuthMethod::ApiKey);
                }
                Ok(None) => {
                    warn!("Invalid API key attempted");
                    return AuthResult::failure(AuthMethod::ApiKey, "Invalid API key");
                }
                Err(e) => {
                    warn!("API key validation error: {}", e);
                    return AuthResult::failure(AuthMethod::ApiKey, e.to_string());
                }
            }
        }

        // Try bearer token
        if let Some(auth_header) = request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
        {
            if let Some(token) = auth_header.strip_prefix("Bearer ") {
                match self.validator.validate_bearer_token(token).await {
                    Ok(Some(client_id)) => {
                        debug!("Bearer token authenticated for client: {}", client_id);
                        return AuthResult::success(client_id, AuthMethod::BearerToken);
                    }
                    Ok(None) => {
                        warn!("Invalid bearer token attempted");
                        return AuthResult::failure(AuthMethod::BearerToken, "Invalid bearer token");
                    }
                    Err(e) => {
                        warn!("Bearer token validation error: {}", e);
                        return AuthResult::failure(AuthMethod::BearerToken, e.to_string());
                    }
                }
            }
        }

        // No authentication provided
        AuthResult::none()
    }
}

impl<V: AuthValidator> Clone for AuthMiddleware<V> {
    fn clone(&self) -> Self {
        Self {
            validator: Arc::clone(&self.validator),
            allow_anonymous: self.allow_anonymous,
        }
    }
}

/// Tower layer for authentication.
pub struct AuthLayer<V: AuthValidator> {
    validator: Arc<V>,
    allow_anonymous: bool,
}

impl<V: AuthValidator> Clone for AuthLayer<V> {
    fn clone(&self) -> Self {
        Self {
            validator: Arc::clone(&self.validator),
            allow_anonymous: self.allow_anonymous,
        }
    }
}

impl<V: AuthValidator, S> Layer<S> for AuthLayer<V> {
    type Service = AuthService<V, S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            validator: Arc::clone(&self.validator),
            allow_anonymous: self.allow_anonymous,
        }
    }
}

/// Tower service for authentication.
pub struct AuthService<V: AuthValidator, S> {
    inner: S,
    validator: Arc<V>,
    allow_anonymous: bool,
}

impl<V: AuthValidator, S: Clone> Clone for AuthService<V, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            validator: Arc::clone(&self.validator),
            allow_anonymous: self.allow_anonymous,
        }
    }
}

impl<V, S> Service<Request<Body>> for AuthService<V, S>
where
    V: AuthValidator,
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let validator = Arc::clone(&self.validator);
        let allow_anonymous = self.allow_anonymous;
        let mut inner = self.inner.clone();

        // Extract headers before entering async block
        let api_key = request
            .headers()
            .get("X-Cauce-API-Key")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let bearer_token = request
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer ").map(String::from));

        Box::pin(async move {
            // Validate authentication
            let auth_result = validate_extracted(&api_key, &bearer_token, &*validator).await;

            if !auth_result.authenticated && !allow_anonymous {
                // Return 401 Unauthorized
                let error_body = serde_json::json!({
                    "error": {
                        "code": "unauthorized",
                        "message": auth_result.error.unwrap_or_else(|| "Unauthorized".to_string())
                    }
                });

                return Ok((
                    StatusCode::UNAUTHORIZED,
                    [("Content-Type", "application/json")],
                    error_body.to_string(),
                )
                    .into_response());
            }

            // Continue to inner service
            inner.call(request).await
        })
    }
}

/// Validate authentication from extracted headers.
async fn validate_extracted<V: AuthValidator>(
    api_key: &Option<String>,
    bearer_token: &Option<String>,
    validator: &V,
) -> AuthResult {
    // Try API key first
    if let Some(ref api_key) = api_key {
        match validator.validate_api_key(api_key).await {
            Ok(Some(client_id)) => {
                debug!("API key authenticated for client: {}", client_id);
                return AuthResult::success(client_id, AuthMethod::ApiKey);
            }
            Ok(None) => {
                warn!("Invalid API key attempted");
                return AuthResult::failure(AuthMethod::ApiKey, "Invalid API key");
            }
            Err(e) => {
                warn!("API key validation error: {}", e);
                return AuthResult::failure(AuthMethod::ApiKey, e.to_string());
            }
        }
    }

    // Try bearer token
    if let Some(ref token) = bearer_token {
        match validator.validate_bearer_token(token).await {
            Ok(Some(client_id)) => {
                debug!("Bearer token authenticated for client: {}", client_id);
                return AuthResult::success(client_id, AuthMethod::BearerToken);
            }
            Ok(None) => {
                warn!("Invalid bearer token attempted");
                return AuthResult::failure(AuthMethod::BearerToken, "Invalid bearer token");
            }
            Err(e) => {
                warn!("Bearer token validation error: {}", e);
                return AuthResult::failure(AuthMethod::BearerToken, e.to_string());
            }
        }
    }

    // No authentication provided
    AuthResult::none()
}

/// Container for storing auth info in request extensions.
#[derive(Debug, Clone)]
pub struct AuthInfo {
    /// The authenticated client ID.
    pub client_id: String,
    /// The authentication method used.
    pub method: AuthMethod,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_result_success() {
        let result = AuthResult::success("client-1", AuthMethod::ApiKey);
        assert!(result.authenticated);
        assert_eq!(result.client_id, Some("client-1".to_string()));
        assert_eq!(result.method, AuthMethod::ApiKey);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_auth_result_failure() {
        let result = AuthResult::failure(AuthMethod::BearerToken, "Invalid token");
        assert!(!result.authenticated);
        assert!(result.client_id.is_none());
        assert_eq!(result.method, AuthMethod::BearerToken);
        assert_eq!(result.error, Some("Invalid token".to_string()));
    }

    #[test]
    fn test_auth_result_none() {
        let result = AuthResult::none();
        assert!(!result.authenticated);
        assert!(result.client_id.is_none());
        assert_eq!(result.method, AuthMethod::None);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_in_memory_validator_api_key() {
        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_123");

        assert_eq!(validator.api_key_count(), 1);
    }

    #[test]
    fn test_in_memory_validator_bearer_token() {
        let validator = InMemoryAuthValidator::new()
            .with_bearer_token("client-1", "token_abc");

        assert_eq!(validator.bearer_token_count(), 1);
    }

    #[test]
    fn test_in_memory_validator_multiple() {
        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_1")
            .with_api_key("client-2", "sk_test_2")
            .with_bearer_token("client-1", "token_1");

        assert_eq!(validator.api_key_count(), 2);
        assert_eq!(validator.bearer_token_count(), 1);
    }

    #[test]
    fn test_in_memory_validator_add_remove() {
        let validator = InMemoryAuthValidator::new();

        validator.add_api_key("client-1", "sk_test_123");
        assert_eq!(validator.api_key_count(), 1);

        let removed = validator.remove_api_key("sk_test_123");
        assert_eq!(removed, Some("client-1".to_string()));
        assert_eq!(validator.api_key_count(), 0);
    }

    #[tokio::test]
    async fn test_validate_api_key_success() {
        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_123");

        let result = validator.validate_api_key("sk_test_123").await.unwrap();
        assert_eq!(result, Some("client-1".to_string()));
    }

    #[tokio::test]
    async fn test_validate_api_key_failure() {
        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_123");

        let result = validator.validate_api_key("invalid_key").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_validate_bearer_token_success() {
        let validator = InMemoryAuthValidator::new()
            .with_bearer_token("client-1", "token_abc");

        let result = validator.validate_bearer_token("token_abc").await.unwrap();
        assert_eq!(result, Some("client-1".to_string()));
    }

    #[tokio::test]
    async fn test_validate_bearer_token_failure() {
        let validator = InMemoryAuthValidator::new()
            .with_bearer_token("client-1", "token_abc");

        let result = validator.validate_bearer_token("invalid_token").await.unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_auth_middleware_clone() {
        let validator = InMemoryAuthValidator::new();
        let middleware = AuthMiddleware::new(validator);
        let _cloned = middleware.clone();
    }

    #[test]
    fn test_auth_middleware_allow_anonymous() {
        let validator = InMemoryAuthValidator::new();
        let middleware = AuthMiddleware::new(validator).allow_anonymous();
        assert!(middleware.allow_anonymous);
    }

    #[test]
    fn test_auth_method_serialization() {
        let api_key = AuthMethod::ApiKey;
        let json = serde_json::to_string(&api_key).unwrap();
        assert_eq!(json, "\"api_key\"");

        let bearer = AuthMethod::BearerToken;
        let json = serde_json::to_string(&bearer).unwrap();
        assert_eq!(json, "\"bearer_token\"");
    }

    #[test]
    fn test_auth_method_deserialization() {
        let api_key: AuthMethod = serde_json::from_str("\"api_key\"").unwrap();
        assert_eq!(api_key, AuthMethod::ApiKey);

        let bearer: AuthMethod = serde_json::from_str("\"bearer_token\"").unwrap();
        assert_eq!(bearer, AuthMethod::BearerToken);
    }

    #[test]
    fn test_auth_info() {
        let info = AuthInfo {
            client_id: "client-1".to_string(),
            method: AuthMethod::ApiKey,
        };
        assert_eq!(info.client_id, "client-1");
        assert_eq!(info.method, AuthMethod::ApiKey);
    }

    #[test]
    fn test_auth_info_clone() {
        let info = AuthInfo {
            client_id: "client-1".to_string(),
            method: AuthMethod::BearerToken,
        };
        let cloned = info.clone();
        assert_eq!(cloned.client_id, info.client_id);
        assert_eq!(cloned.method, info.method);
    }

    #[test]
    fn test_auth_info_debug() {
        let info = AuthInfo {
            client_id: "client-1".to_string(),
            method: AuthMethod::ApiKey,
        };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("client-1"));
        assert!(debug_str.contains("ApiKey"));
    }

    #[test]
    fn test_auth_method_none() {
        let method = AuthMethod::None;
        let json = serde_json::to_string(&method).unwrap();
        assert_eq!(json, "\"none\"");
    }

    #[test]
    fn test_auth_result_clone() {
        let result = AuthResult::success("client-1", AuthMethod::ApiKey);
        let cloned = result.clone();
        assert_eq!(cloned.authenticated, result.authenticated);
        assert_eq!(cloned.client_id, result.client_id);
    }

    #[test]
    fn test_auth_result_debug() {
        let result = AuthResult::failure(AuthMethod::ApiKey, "test error");
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("test error"));
    }

    #[test]
    fn test_in_memory_validator_remove_nonexistent() {
        let validator = InMemoryAuthValidator::new();

        let removed = validator.remove_api_key("nonexistent");
        assert!(removed.is_none());

        let removed = validator.remove_bearer_token("nonexistent");
        assert!(removed.is_none());
    }

    #[test]
    fn test_in_memory_validator_add_bearer_remove() {
        let validator = InMemoryAuthValidator::new();

        validator.add_bearer_token("client-1", "token_123");
        assert_eq!(validator.bearer_token_count(), 1);

        let removed = validator.remove_bearer_token("token_123");
        assert_eq!(removed, Some("client-1".to_string()));
        assert_eq!(validator.bearer_token_count(), 0);
    }

    #[tokio::test]
    async fn test_validate_extracted_with_api_key() {
        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_123");

        let result = validate_extracted(
            &Some("sk_test_123".to_string()),
            &None,
            &validator,
        ).await;

        assert!(result.authenticated);
        assert_eq!(result.client_id, Some("client-1".to_string()));
        assert_eq!(result.method, AuthMethod::ApiKey);
    }

    #[tokio::test]
    async fn test_validate_extracted_with_bearer_token() {
        let validator = InMemoryAuthValidator::new()
            .with_bearer_token("client-1", "token_abc");

        let result = validate_extracted(
            &None,
            &Some("token_abc".to_string()),
            &validator,
        ).await;

        assert!(result.authenticated);
        assert_eq!(result.client_id, Some("client-1".to_string()));
        assert_eq!(result.method, AuthMethod::BearerToken);
    }

    #[tokio::test]
    async fn test_validate_extracted_invalid_api_key() {
        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_123");

        let result = validate_extracted(
            &Some("invalid_key".to_string()),
            &None,
            &validator,
        ).await;

        assert!(!result.authenticated);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_validate_extracted_invalid_bearer() {
        let validator = InMemoryAuthValidator::new()
            .with_bearer_token("client-1", "token_abc");

        let result = validate_extracted(
            &None,
            &Some("invalid_token".to_string()),
            &validator,
        ).await;

        assert!(!result.authenticated);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_validate_extracted_no_credentials() {
        let validator = InMemoryAuthValidator::new();

        let result = validate_extracted(&None, &None, &validator).await;

        assert!(!result.authenticated);
        assert_eq!(result.method, AuthMethod::None);
    }

    #[tokio::test]
    async fn test_validate_extracted_prefers_api_key() {
        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-api", "sk_test_123")
            .with_bearer_token("client-bearer", "token_abc");

        // If both are provided, API key takes precedence
        let result = validate_extracted(
            &Some("sk_test_123".to_string()),
            &Some("token_abc".to_string()),
            &validator,
        ).await;

        assert!(result.authenticated);
        assert_eq!(result.client_id, Some("client-api".to_string()));
        assert_eq!(result.method, AuthMethod::ApiKey);
    }

    #[test]
    fn test_auth_middleware_with_shared() {
        let validator = Arc::new(InMemoryAuthValidator::new());
        let middleware = AuthMiddleware::with_shared(validator);
        let _layer = middleware.layer();
    }

    #[test]
    fn test_auth_layer_clone() {
        let validator = Arc::new(InMemoryAuthValidator::new());
        let layer = AuthLayer {
            validator,
            allow_anonymous: false,
        };
        let cloned = layer.clone();
        assert_eq!(cloned.allow_anonymous, layer.allow_anonymous);
    }

    #[tokio::test]
    async fn test_validate_request_with_api_key() {
        use axum::http::Request;
        use axum::body::Body;

        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_123");
        let middleware = AuthMiddleware::new(validator);

        let request = Request::builder()
            .header("X-Cauce-API-Key", "sk_test_123")
            .body(Body::empty())
            .unwrap();

        let result = middleware.validate_request(&request).await;
        assert!(result.authenticated);
        assert_eq!(result.client_id, Some("client-1".to_string()));
        assert_eq!(result.method, AuthMethod::ApiKey);
    }

    #[tokio::test]
    async fn test_validate_request_with_invalid_api_key() {
        use axum::http::Request;
        use axum::body::Body;

        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-1", "sk_test_123");
        let middleware = AuthMiddleware::new(validator);

        let request = Request::builder()
            .header("X-Cauce-API-Key", "invalid_key")
            .body(Body::empty())
            .unwrap();

        let result = middleware.validate_request(&request).await;
        assert!(!result.authenticated);
        assert_eq!(result.method, AuthMethod::ApiKey);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_validate_request_with_bearer_token() {
        use axum::http::Request;
        use axum::body::Body;

        let validator = InMemoryAuthValidator::new()
            .with_bearer_token("client-2", "token_abc");
        let middleware = AuthMiddleware::new(validator);

        let request = Request::builder()
            .header("Authorization", "Bearer token_abc")
            .body(Body::empty())
            .unwrap();

        let result = middleware.validate_request(&request).await;
        assert!(result.authenticated);
        assert_eq!(result.client_id, Some("client-2".to_string()));
        assert_eq!(result.method, AuthMethod::BearerToken);
    }

    #[tokio::test]
    async fn test_validate_request_with_invalid_bearer_token() {
        use axum::http::Request;
        use axum::body::Body;

        let validator = InMemoryAuthValidator::new()
            .with_bearer_token("client-2", "token_abc");
        let middleware = AuthMiddleware::new(validator);

        let request = Request::builder()
            .header("Authorization", "Bearer invalid_token")
            .body(Body::empty())
            .unwrap();

        let result = middleware.validate_request(&request).await;
        assert!(!result.authenticated);
        assert_eq!(result.method, AuthMethod::BearerToken);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_validate_request_no_credentials() {
        use axum::http::Request;
        use axum::body::Body;

        let validator = InMemoryAuthValidator::new();
        let middleware = AuthMiddleware::new(validator);

        let request = Request::builder()
            .body(Body::empty())
            .unwrap();

        let result = middleware.validate_request(&request).await;
        assert!(!result.authenticated);
        assert_eq!(result.method, AuthMethod::None);
    }

    #[tokio::test]
    async fn test_validate_request_api_key_takes_precedence() {
        use axum::http::Request;
        use axum::body::Body;

        let validator = InMemoryAuthValidator::new()
            .with_api_key("client-api", "sk_test_123")
            .with_bearer_token("client-bearer", "token_abc");
        let middleware = AuthMiddleware::new(validator);

        // Both credentials provided - API key should take precedence
        let request = Request::builder()
            .header("X-Cauce-API-Key", "sk_test_123")
            .header("Authorization", "Bearer token_abc")
            .body(Body::empty())
            .unwrap();

        let result = middleware.validate_request(&request).await;
        assert!(result.authenticated);
        assert_eq!(result.client_id, Some("client-api".to_string()));
        assert_eq!(result.method, AuthMethod::ApiKey);
    }
}
