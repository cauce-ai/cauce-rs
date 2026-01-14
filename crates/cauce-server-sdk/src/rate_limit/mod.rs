//! Rate limiting for the Cauce server.
//!
//! This module provides rate limiting using a token bucket algorithm
//! to prevent abuse and ensure fair resource usage.
//!
//! # Example
//!
//! ```ignore
//! use axum::Router;
//! use cauce_server_sdk::rate_limit::{RateLimitConfig, RateLimitMiddleware, InMemoryRateLimiter};
//!
//! let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
//! let middleware = RateLimitMiddleware::new(limiter);
//!
//! let app = Router::new()
//!     .route("/api", get(handler))
//!     .layer(middleware.layer());
//! ```

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tower::{Layer, Service};
use tracing::{debug, warn};

use crate::error::ServerResult;

/// Rate limit configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum number of requests per window.
    pub max_requests: u64,
    /// Time window duration in seconds.
    pub window_secs: u64,
    /// Bucket capacity (for token bucket algorithm).
    pub bucket_capacity: u64,
    /// Token refill rate per second.
    pub refill_rate: f64,
    /// Whether rate limiting is enabled.
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 1000,
            window_secs: 60,
            bucket_capacity: 100,
            refill_rate: 10.0,
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    /// Creates a strict rate limit config.
    pub fn strict() -> Self {
        Self {
            max_requests: 100,
            window_secs: 60,
            bucket_capacity: 20,
            refill_rate: 2.0,
            enabled: true,
        }
    }

    /// Creates a relaxed rate limit config.
    pub fn relaxed() -> Self {
        Self {
            max_requests: 10000,
            window_secs: 60,
            bucket_capacity: 500,
            refill_rate: 100.0,
            enabled: true,
        }
    }

    /// Disables rate limiting.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Sets the maximum requests per window.
    pub fn with_max_requests(mut self, max_requests: u64) -> Self {
        self.max_requests = max_requests;
        self
    }

    /// Sets the window duration.
    pub fn with_window_secs(mut self, window_secs: u64) -> Self {
        self.window_secs = window_secs;
        self
    }

    /// Sets the bucket capacity.
    pub fn with_bucket_capacity(mut self, capacity: u64) -> Self {
        self.bucket_capacity = capacity;
        self
    }

    /// Sets the refill rate.
    pub fn with_refill_rate(mut self, rate: f64) -> Self {
        self.refill_rate = rate;
        self
    }
}

/// Result of a rate limit check.
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Remaining requests in the current window.
    pub remaining: u64,
    /// Time in milliseconds until the rate limit resets.
    pub retry_after_ms: Option<u64>,
    /// Current bucket level (for token bucket).
    pub bucket_level: u64,
}

impl RateLimitResult {
    /// Creates an allowed result.
    pub fn allowed(remaining: u64, bucket_level: u64) -> Self {
        Self {
            allowed: true,
            remaining,
            retry_after_ms: None,
            bucket_level,
        }
    }

    /// Creates a denied result.
    pub fn denied(retry_after_ms: u64) -> Self {
        Self {
            allowed: false,
            remaining: 0,
            retry_after_ms: Some(retry_after_ms),
            bucket_level: 0,
        }
    }
}

/// Trait for rate limiting.
#[async_trait]
pub trait RateLimiter: Send + Sync + 'static {
    /// Check if a request from the given key is allowed.
    async fn check(&self, key: &str) -> ServerResult<RateLimitResult>;

    /// Check and consume a token for the given key.
    async fn consume(&self, key: &str) -> ServerResult<RateLimitResult>;

    /// Reset the rate limit for a key.
    async fn reset(&self, key: &str) -> ServerResult<()>;

    /// Get the current state for a key.
    async fn get_state(&self, key: &str) -> ServerResult<Option<RateLimitState>>;
}

/// Rate limit state for a key.
#[derive(Debug, Clone)]
pub struct RateLimitState {
    /// Number of requests made in the current window.
    pub requests: u64,
    /// Current token bucket level.
    pub tokens: u64,
    /// When the current window started.
    pub window_start: Instant,
    /// When the bucket was last refilled.
    pub last_refill: Instant,
}

/// Token bucket implementation.
struct TokenBucket {
    /// Current number of tokens.
    tokens: AtomicU64,
    /// Maximum capacity.
    capacity: u64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Last time tokens were added.
    last_refill: std::sync::Mutex<Instant>,
}

impl TokenBucket {
    fn new(capacity: u64, refill_rate: f64) -> Self {
        Self {
            tokens: AtomicU64::new(capacity),
            capacity,
            refill_rate,
            last_refill: std::sync::Mutex::new(Instant::now()),
        }
    }

    fn try_consume(&self) -> bool {
        self.refill();

        loop {
            let current = self.tokens.load(Ordering::Acquire);
            if current == 0 {
                return false;
            }
            if self
                .tokens
                .compare_exchange(current, current - 1, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return true;
            }
        }
    }

    fn refill(&self) {
        let mut last = self.last_refill.lock().unwrap();
        let now = Instant::now();
        let elapsed = now.duration_since(*last).as_secs_f64();

        let tokens_to_add = (elapsed * self.refill_rate) as u64;
        if tokens_to_add > 0 {
            let current = self.tokens.load(Ordering::Acquire);
            let new_tokens = (current + tokens_to_add).min(self.capacity);
            self.tokens.store(new_tokens, Ordering::Release);
            *last = now;
        }
    }

    fn current_tokens(&self) -> u64 {
        self.refill();
        self.tokens.load(Ordering::Acquire)
    }

    fn time_until_refill(&self) -> Duration {
        let tokens = self.tokens.load(Ordering::Acquire);
        if tokens > 0 {
            Duration::ZERO
        } else {
            // Time until at least one token is available
            Duration::from_secs_f64(1.0 / self.refill_rate)
        }
    }
}

/// Sliding window counter.
struct SlidingWindow {
    /// Request counts per time slice.
    counts: AtomicU64,
    /// Window start time.
    window_start: std::sync::Mutex<Instant>,
    /// Window duration.
    window_duration: Duration,
    /// Maximum requests per window.
    max_requests: u64,
}

impl SlidingWindow {
    fn new(max_requests: u64, window_duration: Duration) -> Self {
        Self {
            counts: AtomicU64::new(0),
            window_start: std::sync::Mutex::new(Instant::now()),
            window_duration,
            max_requests,
        }
    }

    fn try_increment(&self) -> bool {
        self.maybe_reset();

        let current = self.counts.fetch_add(1, Ordering::AcqRel);
        if current >= self.max_requests {
            self.counts.fetch_sub(1, Ordering::AcqRel);
            return false;
        }
        true
    }

    fn maybe_reset(&self) {
        let mut start = self.window_start.lock().unwrap();
        let now = Instant::now();
        if now.duration_since(*start) >= self.window_duration {
            self.counts.store(0, Ordering::Release);
            *start = now;
        }
    }

    fn remaining(&self) -> u64 {
        self.maybe_reset();
        let current = self.counts.load(Ordering::Acquire);
        self.max_requests.saturating_sub(current)
    }

    fn time_until_reset(&self) -> Duration {
        let start = self.window_start.lock().unwrap();
        let elapsed = Instant::now().duration_since(*start);
        self.window_duration.saturating_sub(elapsed)
    }
}

/// Entry for a rate-limited client.
struct RateLimitEntry {
    bucket: TokenBucket,
    window: SlidingWindow,
}

/// In-memory rate limiter using token bucket and sliding window.
pub struct InMemoryRateLimiter {
    config: RateLimitConfig,
    entries: Arc<DashMap<String, RateLimitEntry>>,
}

impl InMemoryRateLimiter {
    /// Creates a new rate limiter with the given config.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            entries: Arc::new(DashMap::new()),
        }
    }

    /// Creates a rate limiter with default config.
    pub fn default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    fn get_or_create_entry(&self, key: &str) -> dashmap::mapref::one::RefMut<'_, String, RateLimitEntry> {
        self.entries.entry(key.to_string()).or_insert_with(|| {
            RateLimitEntry {
                bucket: TokenBucket::new(self.config.bucket_capacity, self.config.refill_rate),
                window: SlidingWindow::new(
                    self.config.max_requests,
                    Duration::from_secs(self.config.window_secs),
                ),
            }
        })
    }

    /// Cleanup old entries.
    pub fn cleanup(&self) {
        // Remove entries that haven't been accessed recently
        // For simplicity, this is a no-op in the current implementation
        // A production implementation would track last access time
    }
}

impl Clone for InMemoryRateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            entries: Arc::clone(&self.entries),
        }
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn check(&self, key: &str) -> ServerResult<RateLimitResult> {
        if !self.config.enabled {
            return Ok(RateLimitResult::allowed(u64::MAX, u64::MAX));
        }

        let entry = self.get_or_create_entry(key);
        let remaining = entry.window.remaining();
        let bucket_level = entry.bucket.current_tokens();

        if remaining == 0 || bucket_level == 0 {
            let retry_after = entry.window.time_until_reset().as_millis() as u64;
            Ok(RateLimitResult::denied(retry_after))
        } else {
            Ok(RateLimitResult::allowed(remaining, bucket_level))
        }
    }

    async fn consume(&self, key: &str) -> ServerResult<RateLimitResult> {
        if !self.config.enabled {
            return Ok(RateLimitResult::allowed(u64::MAX, u64::MAX));
        }

        let entry = self.get_or_create_entry(key);

        // Check both bucket and window
        if !entry.bucket.try_consume() {
            let retry_after = entry.bucket.time_until_refill().as_millis() as u64;
            return Ok(RateLimitResult::denied(retry_after.max(100)));
        }

        if !entry.window.try_increment() {
            // Refund the bucket token since window limit was hit
            entry.bucket.tokens.fetch_add(1, Ordering::AcqRel);
            let retry_after = entry.window.time_until_reset().as_millis() as u64;
            return Ok(RateLimitResult::denied(retry_after));
        }

        let remaining = entry.window.remaining();
        let bucket_level = entry.bucket.current_tokens();
        Ok(RateLimitResult::allowed(remaining, bucket_level))
    }

    async fn reset(&self, key: &str) -> ServerResult<()> {
        self.entries.remove(key);
        Ok(())
    }

    async fn get_state(&self, key: &str) -> ServerResult<Option<RateLimitState>> {
        if let Some(entry) = self.entries.get(key) {
            let window_start = *entry.window.window_start.lock().unwrap();
            let last_refill = *entry.bucket.last_refill.lock().unwrap();
            Ok(Some(RateLimitState {
                requests: entry.window.counts.load(Ordering::Acquire),
                tokens: entry.bucket.current_tokens(),
                window_start,
                last_refill,
            }))
        } else {
            Ok(None)
        }
    }
}

/// Rate limit middleware for axum.
pub struct RateLimitMiddleware<L: RateLimiter> {
    limiter: Arc<L>,
    key_extractor: KeyExtractor,
}

/// Type alias for custom key extraction function.
pub type KeyExtractorFn = Arc<dyn Fn(&Request<Body>) -> String + Send + Sync>;

/// How to extract the rate limit key from a request.
#[derive(Clone, Default)]
pub enum KeyExtractor {
    /// Use client IP address.
    #[default]
    IpAddress,
    /// Use a specific header value.
    Header(String),
    /// Use a constant key (global rate limit).
    Global,
    /// Custom extraction function.
    Custom(KeyExtractorFn),
}

impl KeyExtractor {
    fn extract(&self, request: &Request<Body>) -> String {
        match self {
            Self::IpAddress => {
                // Try X-Forwarded-For first, then fall back to connection info
                request
                    .headers()
                    .get("X-Forwarded-For")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
                    .unwrap_or_else(|| "unknown".to_string())
            }
            Self::Header(name) => request
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(String::from)
                .unwrap_or_else(|| "unknown".to_string()),
            Self::Global => "global".to_string(),
            Self::Custom(f) => f(request),
        }
    }
}

impl<L: RateLimiter> RateLimitMiddleware<L> {
    /// Creates a new rate limit middleware.
    pub fn new(limiter: L) -> Self {
        Self {
            limiter: Arc::new(limiter),
            key_extractor: KeyExtractor::default(),
        }
    }

    /// Creates middleware with a shared limiter.
    pub fn with_shared(limiter: Arc<L>) -> Self {
        Self {
            limiter,
            key_extractor: KeyExtractor::default(),
        }
    }

    /// Sets the key extraction method.
    pub fn with_key_extractor(mut self, extractor: KeyExtractor) -> Self {
        self.key_extractor = extractor;
        self
    }

    /// Use a header for the rate limit key.
    pub fn by_header(mut self, header: impl Into<String>) -> Self {
        self.key_extractor = KeyExtractor::Header(header.into());
        self
    }

    /// Use IP address for the rate limit key (default).
    pub fn by_ip(mut self) -> Self {
        self.key_extractor = KeyExtractor::IpAddress;
        self
    }

    /// Use a global rate limit.
    pub fn global(mut self) -> Self {
        self.key_extractor = KeyExtractor::Global;
        self
    }

    /// Creates a tower Layer for this middleware.
    pub fn layer(self) -> RateLimitLayer<L> {
        RateLimitLayer {
            limiter: self.limiter,
            key_extractor: self.key_extractor,
        }
    }
}

impl<L: RateLimiter> Clone for RateLimitMiddleware<L> {
    fn clone(&self) -> Self {
        Self {
            limiter: Arc::clone(&self.limiter),
            key_extractor: self.key_extractor.clone(),
        }
    }
}

/// Tower layer for rate limiting.
pub struct RateLimitLayer<L: RateLimiter> {
    limiter: Arc<L>,
    key_extractor: KeyExtractor,
}

impl<L: RateLimiter> Clone for RateLimitLayer<L> {
    fn clone(&self) -> Self {
        Self {
            limiter: Arc::clone(&self.limiter),
            key_extractor: self.key_extractor.clone(),
        }
    }
}

impl<L: RateLimiter, S> Layer<S> for RateLimitLayer<L> {
    type Service = RateLimitService<L, S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: Arc::clone(&self.limiter),
            key_extractor: self.key_extractor.clone(),
        }
    }
}

/// Tower service for rate limiting.
pub struct RateLimitService<L: RateLimiter, S> {
    inner: S,
    limiter: Arc<L>,
    key_extractor: KeyExtractor,
}

impl<L: RateLimiter, S: Clone> Clone for RateLimitService<L, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            limiter: Arc::clone(&self.limiter),
            key_extractor: self.key_extractor.clone(),
        }
    }
}

impl<L, S> Service<Request<Body>> for RateLimitService<L, S>
where
    L: RateLimiter,
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
        let limiter = Arc::clone(&self.limiter);
        let key = self.key_extractor.extract(&request);
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Check rate limit
            let result = match limiter.consume(&key).await {
                Ok(r) => r,
                Err(e) => {
                    warn!("Rate limit check failed: {}", e);
                    // Allow request on error
                    RateLimitResult::allowed(0, 0)
                }
            };

            if !result.allowed {
                debug!("Rate limited: key={}", key);

                let retry_after = result.retry_after_ms.unwrap_or(1000);
                let error_body = serde_json::json!({
                    "error": {
                        "code": "rate_limited",
                        "message": "Too many requests",
                        "retry_after_ms": retry_after
                    }
                });

                return Ok((
                    StatusCode::TOO_MANY_REQUESTS,
                    [
                        ("Content-Type", "application/json"),
                        ("Retry-After", &format!("{}", retry_after / 1000)),
                    ],
                    error_body.to_string(),
                )
                    .into_response());
            }

            // Continue to inner service
            inner.call(request).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.max_requests, 1000);
        assert_eq!(config.window_secs, 60);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_strict() {
        let config = RateLimitConfig::strict();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.bucket_capacity, 20);
    }

    #[test]
    fn test_config_relaxed() {
        let config = RateLimitConfig::relaxed();
        assert_eq!(config.max_requests, 10000);
        assert_eq!(config.bucket_capacity, 500);
    }

    #[test]
    fn test_config_disabled() {
        let config = RateLimitConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_config_builder() {
        let config = RateLimitConfig::default()
            .with_max_requests(500)
            .with_window_secs(120)
            .with_bucket_capacity(50);

        assert_eq!(config.max_requests, 500);
        assert_eq!(config.window_secs, 120);
        assert_eq!(config.bucket_capacity, 50);
    }

    #[test]
    fn test_result_allowed() {
        let result = RateLimitResult::allowed(100, 50);
        assert!(result.allowed);
        assert_eq!(result.remaining, 100);
        assert!(result.retry_after_ms.is_none());
    }

    #[test]
    fn test_result_denied() {
        let result = RateLimitResult::denied(5000);
        assert!(!result.allowed);
        assert_eq!(result.remaining, 0);
        assert_eq!(result.retry_after_ms, Some(5000));
    }

    #[tokio::test]
    async fn test_limiter_disabled() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig::disabled());
        let result = limiter.consume("test").await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_limiter_basic() {
        let config = RateLimitConfig::default()
            .with_bucket_capacity(5)
            .with_max_requests(10);
        let limiter = InMemoryRateLimiter::new(config);

        // First 5 requests should succeed (bucket capacity)
        for i in 0..5 {
            let result = limiter.consume("test").await.unwrap();
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        // 6th request should be denied (bucket empty)
        let result = limiter.consume("test").await.unwrap();
        assert!(!result.allowed);
    }

    #[tokio::test]
    async fn test_limiter_check_without_consume() {
        let config = RateLimitConfig::default()
            .with_bucket_capacity(2);
        let limiter = InMemoryRateLimiter::new(config);

        // Check should not consume
        let result1 = limiter.check("test").await.unwrap();
        assert!(result1.allowed);
        assert_eq!(result1.bucket_level, 2);

        let result2 = limiter.check("test").await.unwrap();
        assert!(result2.allowed);
        assert_eq!(result2.bucket_level, 2);
    }

    #[tokio::test]
    async fn test_limiter_reset() {
        let config = RateLimitConfig::default()
            .with_bucket_capacity(2);
        let limiter = InMemoryRateLimiter::new(config);

        // Consume all tokens
        limiter.consume("test").await.unwrap();
        limiter.consume("test").await.unwrap();

        let result = limiter.consume("test").await.unwrap();
        assert!(!result.allowed);

        // Reset
        limiter.reset("test").await.unwrap();

        // Should be allowed again
        let result = limiter.consume("test").await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_limiter_different_keys() {
        let config = RateLimitConfig::default()
            .with_bucket_capacity(2);
        let limiter = InMemoryRateLimiter::new(config);

        // Consume all tokens for key1
        limiter.consume("key1").await.unwrap();
        limiter.consume("key1").await.unwrap();
        let result = limiter.consume("key1").await.unwrap();
        assert!(!result.allowed);

        // key2 should still be allowed
        let result = limiter.consume("key2").await.unwrap();
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_limiter_get_state() {
        let config = RateLimitConfig::default();
        let limiter = InMemoryRateLimiter::new(config);

        // No state initially
        let state = limiter.get_state("test").await.unwrap();
        assert!(state.is_none());

        // After consuming
        limiter.consume("test").await.unwrap();
        let state = limiter.get_state("test").await.unwrap();
        assert!(state.is_some());

        let state = state.unwrap();
        assert!(state.requests > 0 || state.tokens < 100);
    }

    #[test]
    fn test_middleware_clone() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
        let middleware = RateLimitMiddleware::new(limiter);
        let _cloned = middleware.clone();
    }

    #[test]
    fn test_middleware_key_extractor() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());

        let middleware = RateLimitMiddleware::new(limiter.clone()).by_ip();
        assert!(matches!(middleware.key_extractor, KeyExtractor::IpAddress));

        let middleware = RateLimitMiddleware::new(limiter.clone()).by_header("X-API-Key");
        assert!(matches!(middleware.key_extractor, KeyExtractor::Header(_)));

        let middleware = RateLimitMiddleware::new(limiter).global();
        assert!(matches!(middleware.key_extractor, KeyExtractor::Global));
    }

    #[test]
    fn test_limiter_clone() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
        let _cloned = limiter.clone();
    }

    #[test]
    fn test_limiter_default_config() {
        let limiter = InMemoryRateLimiter::default_config();
        // Should use default config
        assert!(limiter.config.enabled);
    }

    #[test]
    fn test_limiter_cleanup() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
        // cleanup is currently a no-op but should not panic
        limiter.cleanup();
    }

    #[test]
    fn test_config_with_refill_rate() {
        let config = RateLimitConfig::default()
            .with_refill_rate(5.0);
        assert_eq!(config.refill_rate, 5.0);
    }

    #[test]
    fn test_key_extractor_global() {
        let extractor = KeyExtractor::Global;
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let key = extractor.extract(&request);
        assert_eq!(key, "global");
    }

    #[test]
    fn test_key_extractor_ip_address() {
        let extractor = KeyExtractor::IpAddress;

        // Without X-Forwarded-For header
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let key = extractor.extract(&request);
        assert_eq!(key, "unknown");

        // With X-Forwarded-For header
        let request = Request::builder()
            .uri("/test")
            .header("X-Forwarded-For", "192.168.1.1, 10.0.0.1")
            .body(Body::empty())
            .unwrap();
        let key = extractor.extract(&request);
        assert_eq!(key, "192.168.1.1");
    }

    #[test]
    fn test_key_extractor_header() {
        let extractor = KeyExtractor::Header("X-API-Key".to_string());

        // Without the header
        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let key = extractor.extract(&request);
        assert_eq!(key, "unknown");

        // With the header
        let request = Request::builder()
            .uri("/test")
            .header("X-API-Key", "api_key_123")
            .body(Body::empty())
            .unwrap();
        let key = extractor.extract(&request);
        assert_eq!(key, "api_key_123");
    }

    #[test]
    fn test_key_extractor_custom() {
        let extractor = KeyExtractor::Custom(Arc::new(|_req: &Request<Body>| {
            "custom_key".to_string()
        }));

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();
        let key = extractor.extract(&request);
        assert_eq!(key, "custom_key");
    }

    #[test]
    fn test_middleware_with_shared() {
        let limiter = Arc::new(InMemoryRateLimiter::new(RateLimitConfig::default()));
        let middleware = RateLimitMiddleware::with_shared(limiter);
        assert!(matches!(middleware.key_extractor, KeyExtractor::IpAddress));
    }

    #[test]
    fn test_middleware_with_key_extractor() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
        let middleware = RateLimitMiddleware::new(limiter)
            .with_key_extractor(KeyExtractor::Global);
        assert!(matches!(middleware.key_extractor, KeyExtractor::Global));
    }

    #[test]
    fn test_layer_clone() {
        let limiter = InMemoryRateLimiter::new(RateLimitConfig::default());
        let layer = RateLimitMiddleware::new(limiter).layer();
        let _cloned = layer.clone();
    }

    #[test]
    fn test_rate_limit_state_debug() {
        let state = RateLimitState {
            requests: 10,
            tokens: 50,
            window_start: Instant::now(),
            last_refill: Instant::now(),
        };
        let debug = format!("{:?}", state);
        assert!(debug.contains("RateLimitState"));
    }

    #[test]
    fn test_rate_limit_state_clone() {
        let state = RateLimitState {
            requests: 10,
            tokens: 50,
            window_start: Instant::now(),
            last_refill: Instant::now(),
        };
        let cloned = state.clone();
        assert_eq!(cloned.requests, state.requests);
        assert_eq!(cloned.tokens, state.tokens);
    }

    #[test]
    fn test_result_debug() {
        let result = RateLimitResult::allowed(100, 50);
        let debug = format!("{:?}", result);
        assert!(debug.contains("RateLimitResult"));
    }

    #[test]
    fn test_result_clone() {
        let result = RateLimitResult::allowed(100, 50);
        let cloned = result.clone();
        assert_eq!(cloned.remaining, result.remaining);
        assert_eq!(cloned.bucket_level, result.bucket_level);
    }

    #[test]
    fn test_config_serialization() {
        let config = RateLimitConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"max_requests\""));

        let deserialized: RateLimitConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_requests, config.max_requests);
    }

    #[test]
    fn test_config_debug() {
        let config = RateLimitConfig::default();
        let debug = format!("{:?}", config);
        assert!(debug.contains("RateLimitConfig"));
    }

    #[test]
    fn test_key_extractor_default() {
        let extractor = KeyExtractor::default();
        assert!(matches!(extractor, KeyExtractor::IpAddress));
    }

    #[test]
    fn test_key_extractor_clone() {
        let extractor = KeyExtractor::Header("X-API-Key".to_string());
        let cloned = extractor.clone();
        assert!(matches!(cloned, KeyExtractor::Header(_)));
    }

    #[tokio::test]
    async fn test_limiter_check_denied_bucket_empty() {
        let config = RateLimitConfig::default()
            .with_bucket_capacity(2)
            .with_refill_rate(0.001); // Very slow refill
        let limiter = InMemoryRateLimiter::new(config);

        // Consume all bucket tokens
        limiter.consume("test").await.unwrap();
        limiter.consume("test").await.unwrap();

        // Check should show denied
        let result = limiter.check("test").await.unwrap();
        assert!(!result.allowed);
    }
}
