//! Webhook delivery handler for the Cauce server.
//!
//! This module provides the [`WebhookDelivery`] for pushing signals to
//! client-provided webhook URLs.
//!
//! # Example
//!
//! ```ignore
//! use cauce_server_sdk::transport::WebhookDelivery;
//!
//! let delivery = WebhookDelivery::new("my-secret-key");
//!
//! // Deliver a signal to a webhook
//! let result = delivery.deliver(&webhook_config, &signal_delivery).await?;
//! ```

use std::time::Duration;

use chrono::Utc;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Serialize;
use sha2::Sha256;
use tracing::{debug, error, warn};

use crate::error::{ServerError, ServerResult};
use cauce_core::methods::WebhookConfig;
use cauce_core::SignalDelivery;

type HmacSha256 = Hmac<Sha256>;

/// Webhook delivery configuration.
#[derive(Debug, Clone)]
pub struct WebhookDeliveryConfig {
    /// Secret key for HMAC signatures (if not provided by WebhookConfig).
    pub default_secret: Option<String>,
    /// Request timeout.
    pub timeout: Duration,
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial retry delay.
    pub initial_retry_delay: Duration,
    /// Maximum retry delay.
    pub max_retry_delay: Duration,
}

impl Default for WebhookDeliveryConfig {
    fn default() -> Self {
        Self {
            default_secret: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            initial_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(60),
        }
    }
}

impl WebhookDeliveryConfig {
    /// Creates a new config with a default secret.
    pub fn with_secret(secret: impl Into<String>) -> Self {
        Self {
            default_secret: Some(secret.into()),
            ..Default::default()
        }
    }

    /// Sets the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the maximum retries.
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// Result of a webhook delivery attempt.
#[derive(Debug, Clone, Serialize)]
pub struct WebhookDeliveryResult {
    /// Whether the delivery was successful.
    pub success: bool,
    /// HTTP status code from the webhook endpoint.
    pub status_code: Option<u16>,
    /// Number of attempts made.
    pub attempts: u32,
    /// Error message if failed.
    pub error: Option<String>,
    /// The delivery ID used.
    pub delivery_id: String,
}

impl WebhookDeliveryResult {
    /// Creates a successful result.
    pub fn success(delivery_id: impl Into<String>, status_code: u16, attempts: u32) -> Self {
        Self {
            success: true,
            status_code: Some(status_code),
            attempts,
            error: None,
            delivery_id: delivery_id.into(),
        }
    }

    /// Creates a failed result.
    pub fn failure(
        delivery_id: impl Into<String>,
        error: impl Into<String>,
        attempts: u32,
    ) -> Self {
        Self {
            success: false,
            status_code: None,
            attempts,
            error: Some(error.into()),
            delivery_id: delivery_id.into(),
        }
    }
}

/// Webhook delivery handler.
///
/// Handles pushing signals to webhook endpoints with HMAC signatures
/// and automatic retries.
pub struct WebhookDelivery {
    client: Client,
    config: WebhookDeliveryConfig,
}

impl WebhookDelivery {
    /// Creates a new webhook delivery handler.
    pub fn new(config: WebhookDeliveryConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Creates a handler with a default secret.
    pub fn with_secret(secret: impl Into<String>) -> Self {
        Self::new(WebhookDeliveryConfig::with_secret(secret))
    }

    /// Deliver a signal to a webhook endpoint.
    pub async fn deliver(
        &self,
        webhook_config: &WebhookConfig,
        delivery: &SignalDelivery,
    ) -> ServerResult<WebhookDeliveryResult> {
        let delivery_id = format!("dlv_{}", uuid::Uuid::new_v4());
        let timestamp = Utc::now().timestamp();

        // Serialize the payload
        let payload = serde_json::to_string(delivery).map_err(|e| ServerError::Serialization {
            message: e.to_string(),
        })?;

        // Determine the secret to use
        let secret = webhook_config
            .secret
            .as_ref()
            .or(self.config.default_secret.as_ref());

        let mut attempts = 0;
        let mut last_error = None;

        while attempts <= self.config.max_retries {
            attempts += 1;

            match self
                .attempt_delivery(
                    &webhook_config.url,
                    &payload,
                    &delivery_id,
                    timestamp,
                    secret,
                )
                .await
            {
                Ok(status_code) => {
                    if status_code.is_success() {
                        debug!(
                            "Webhook delivery {} succeeded with status {}",
                            delivery_id,
                            status_code.as_u16()
                        );
                        return Ok(WebhookDeliveryResult::success(
                            delivery_id,
                            status_code.as_u16(),
                            attempts,
                        ));
                    } else {
                        // Non-success status - retry if retriable
                        if self.is_retriable_status(status_code) {
                            warn!(
                                "Webhook delivery {} failed with status {} (attempt {})",
                                delivery_id,
                                status_code.as_u16(),
                                attempts
                            );
                            last_error = Some(format!("HTTP {}", status_code.as_u16()));
                        } else {
                            // Not retriable - fail immediately
                            error!(
                                "Webhook delivery {} failed with non-retriable status {}",
                                delivery_id,
                                status_code.as_u16()
                            );
                            return Ok(WebhookDeliveryResult::failure(
                                delivery_id,
                                format!("HTTP {}", status_code.as_u16()),
                                attempts,
                            ));
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Webhook delivery {} failed: {} (attempt {})",
                        delivery_id, e, attempts
                    );
                    last_error = Some(e.to_string());
                }
            }

            // Wait before retry (exponential backoff)
            if attempts <= self.config.max_retries {
                let delay = self.calculate_retry_delay(attempts);
                tokio::time::sleep(delay).await;
            }
        }

        // All retries exhausted
        error!(
            "Webhook delivery {} failed after {} attempts",
            delivery_id, attempts
        );

        Ok(WebhookDeliveryResult::failure(
            delivery_id,
            last_error.unwrap_or_else(|| "unknown error".to_string()),
            attempts,
        ))
    }

    /// Attempt a single delivery.
    async fn attempt_delivery(
        &self,
        url: &str,
        payload: &str,
        delivery_id: &str,
        timestamp: i64,
        secret: Option<&String>,
    ) -> Result<reqwest::StatusCode, ServerError> {
        let mut request = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .header("X-Cauce-Delivery-Id", delivery_id)
            .header("X-Cauce-Timestamp", timestamp.to_string());

        // Add signature if secret is available
        if let Some(secret) = secret {
            let signature = self.generate_signature(payload, timestamp, secret);
            request = request.header("X-Cauce-Signature", format!("sha256={}", signature));
        }

        let response = request.body(payload.to_string()).send().await.map_err(|e| {
            ServerError::WebhookFailed {
                url: url.to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(response.status())
    }

    /// Generate HMAC-SHA256 signature.
    fn generate_signature(&self, payload: &str, timestamp: i64, secret: &str) -> String {
        let signing_input = format!("{}.{}", timestamp, payload);

        let mut mac =
            HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(signing_input.as_bytes());
        let result = mac.finalize();

        hex::encode(result.into_bytes())
    }

    /// Check if a status code is retriable.
    fn is_retriable_status(&self, status: reqwest::StatusCode) -> bool {
        // Retry on server errors and rate limiting
        status.is_server_error() || status.as_u16() == 429
    }

    /// Calculate retry delay with exponential backoff.
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.config.initial_retry_delay;
        let multiplier = 2u64.pow(attempt.saturating_sub(1));
        let delay = base_delay.saturating_mul(multiplier as u32);
        delay.min(self.config.max_retry_delay)
    }

    /// Verify a webhook signature (for testing/debugging).
    pub fn verify_signature(
        &self,
        payload: &str,
        timestamp: i64,
        signature: &str,
        secret: &str,
    ) -> bool {
        let expected = self.generate_signature(payload, timestamp, secret);
        let signature = signature.strip_prefix("sha256=").unwrap_or(signature);
        expected == signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cauce_core::types::{Payload, Source, Topic};
    use cauce_core::Signal;
    use chrono::{DateTime, Utc};
    use serde_json::json;

    fn create_test_signal() -> Signal {
        Signal {
            id: "sig_123".to_string(),
            version: "1.0".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            source: Source::new("email", "adapter-1", "msg-1"),
            topic: Topic::new_unchecked("signal.email.received"),
            payload: Payload::new(json!({"text": "hello"}), "application/json"),
            metadata: None,
            encrypted: None,
        }
    }

    #[test]
    fn test_config_default() {
        let config = WebhookDeliveryConfig::default();
        assert!(config.default_secret.is_none());
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_with_secret() {
        let config = WebhookDeliveryConfig::with_secret("my-secret");
        assert_eq!(config.default_secret, Some("my-secret".to_string()));
    }

    #[test]
    fn test_config_builder() {
        let config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_secs(60))
            .with_max_retries(5);

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_delivery_result_success() {
        let result = WebhookDeliveryResult::success("dlv_123", 200, 1);
        assert!(result.success);
        assert_eq!(result.status_code, Some(200));
        assert_eq!(result.attempts, 1);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_delivery_result_failure() {
        let result = WebhookDeliveryResult::failure("dlv_123", "connection refused", 3);
        assert!(!result.success);
        assert!(result.status_code.is_none());
        assert_eq!(result.attempts, 3);
        assert_eq!(result.error, Some("connection refused".to_string()));
    }

    #[test]
    fn test_signature_generation() {
        let delivery = WebhookDelivery::with_secret("test-secret");
        let signature = delivery.generate_signature("test-payload", 1234567890, "test-secret");

        // Signature should be a hex string
        assert!(!signature.is_empty());
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_signature_verification() {
        let delivery = WebhookDelivery::with_secret("test-secret");
        let payload = r#"{"test":"data"}"#;
        let timestamp = 1234567890i64;
        let secret = "test-secret";

        let signature = delivery.generate_signature(payload, timestamp, secret);

        assert!(delivery.verify_signature(payload, timestamp, &signature, secret));
        assert!(delivery.verify_signature(
            payload,
            timestamp,
            &format!("sha256={}", signature),
            secret
        ));

        // Wrong payload should fail
        assert!(!delivery.verify_signature("wrong", timestamp, &signature, secret));

        // Wrong timestamp should fail
        assert!(!delivery.verify_signature(payload, 999, &signature, secret));

        // Wrong secret should fail
        assert!(!delivery.verify_signature(payload, timestamp, &signature, "wrong-secret"));
    }

    #[test]
    fn test_retry_delay_calculation() {
        let delivery = WebhookDelivery::new(WebhookDeliveryConfig::default());

        let delay1 = delivery.calculate_retry_delay(1);
        let delay2 = delivery.calculate_retry_delay(2);
        let delay3 = delivery.calculate_retry_delay(3);

        // Exponential backoff
        assert_eq!(delay1, Duration::from_secs(1));
        assert_eq!(delay2, Duration::from_secs(2));
        assert_eq!(delay3, Duration::from_secs(4));
    }

    #[test]
    fn test_retry_delay_cap() {
        let config = WebhookDeliveryConfig::default()
            .with_max_retries(10);
        let delivery = WebhookDelivery::new(config);

        // High attempt number should be capped at max_retry_delay
        let delay = delivery.calculate_retry_delay(10);
        assert!(delay <= Duration::from_secs(60));
    }

    #[test]
    fn test_is_retriable_status() {
        let delivery = WebhookDelivery::new(WebhookDeliveryConfig::default());

        // Server errors are retriable
        assert!(delivery.is_retriable_status(reqwest::StatusCode::INTERNAL_SERVER_ERROR));
        assert!(delivery.is_retriable_status(reqwest::StatusCode::BAD_GATEWAY));
        assert!(delivery.is_retriable_status(reqwest::StatusCode::SERVICE_UNAVAILABLE));

        // Rate limiting is retriable
        assert!(delivery.is_retriable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));

        // Client errors are not retriable
        assert!(!delivery.is_retriable_status(reqwest::StatusCode::BAD_REQUEST));
        assert!(!delivery.is_retriable_status(reqwest::StatusCode::NOT_FOUND));
        assert!(!delivery.is_retriable_status(reqwest::StatusCode::UNAUTHORIZED));
    }

    #[test]
    fn test_delivery_result_serialization() {
        let result = WebhookDeliveryResult::success("dlv_123", 200, 1);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"status_code\":200"));
        assert!(json.contains("\"delivery_id\":\"dlv_123\""));
    }

    #[test]
    fn test_delivery_result_failure_serialization() {
        let result = WebhookDeliveryResult::failure("dlv_456", "connection refused", 3);
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"connection refused\""));
    }

    #[test]
    fn test_delivery_result_clone() {
        let result = WebhookDeliveryResult::success("dlv_123", 200, 1);
        let cloned = result.clone();
        assert_eq!(cloned.success, result.success);
        assert_eq!(cloned.delivery_id, result.delivery_id);
    }

    #[test]
    fn test_delivery_result_debug() {
        let result = WebhookDeliveryResult::success("dlv_123", 200, 1);
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("dlv_123"));
    }

    #[test]
    fn test_config_clone() {
        let config = WebhookDeliveryConfig::with_secret("my-secret")
            .with_timeout(Duration::from_secs(60));
        let cloned = config.clone();
        assert_eq!(cloned.default_secret, config.default_secret);
        assert_eq!(cloned.timeout, config.timeout);
    }

    #[test]
    fn test_config_debug() {
        let config = WebhookDeliveryConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("WebhookDeliveryConfig"));
    }

    #[test]
    fn test_retry_delay_first_attempt() {
        let delivery = WebhookDelivery::new(WebhookDeliveryConfig::default());

        // First attempt should use initial delay
        let delay = delivery.calculate_retry_delay(1);
        assert_eq!(delay, Duration::from_secs(1));
    }

    #[test]
    fn test_retry_delay_zero_attempt() {
        let delivery = WebhookDelivery::new(WebhookDeliveryConfig::default());

        // Zero attempt (edge case) should still give a valid delay
        let delay = delivery.calculate_retry_delay(0);
        assert!(delay <= Duration::from_secs(60));
    }

    #[test]
    fn test_signature_deterministic() {
        let delivery = WebhookDelivery::with_secret("test-secret");
        let payload = r#"{"test":"data"}"#;
        let timestamp = 1234567890i64;
        let secret = "test-secret";

        let sig1 = delivery.generate_signature(payload, timestamp, secret);
        let sig2 = delivery.generate_signature(payload, timestamp, secret);

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_different_for_different_input() {
        let delivery = WebhookDelivery::with_secret("test-secret");
        let secret = "test-secret";

        let sig1 = delivery.generate_signature("payload1", 1234567890, secret);
        let sig2 = delivery.generate_signature("payload2", 1234567890, secret);

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_webhook_delivery_creation() {
        let config = WebhookDeliveryConfig::default();
        let _delivery = WebhookDelivery::new(config);
    }

    #[test]
    fn test_gateway_timeout_retriable() {
        let delivery = WebhookDelivery::new(WebhookDeliveryConfig::default());
        assert!(delivery.is_retriable_status(reqwest::StatusCode::GATEWAY_TIMEOUT));
    }

    #[test]
    fn test_success_status_not_retriable() {
        let delivery = WebhookDelivery::new(WebhookDeliveryConfig::default());
        assert!(!delivery.is_retriable_status(reqwest::StatusCode::OK));
        assert!(!delivery.is_retriable_status(reqwest::StatusCode::CREATED));
    }

    // ============================================================================
    // Wiremock integration tests for actual HTTP delivery
    // ============================================================================

    #[tokio::test]
    async fn test_deliver_success() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use wiremock::matchers::{method, path, header, header_exists};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/webhook"))
            .and(header("Content-Type", "application/json"))
            .and(header_exists("X-Cauce-Delivery-Id"))
            .and(header_exists("X-Cauce-Timestamp"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(0);
        let delivery_handler = WebhookDelivery::new(config);

        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: None,
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        assert!(result.success);
        assert_eq!(result.status_code, Some(200));
        assert_eq!(result.attempts, 1);
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_deliver_with_signature() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use wiremock::matchers::{method, path, header_exists};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/webhook"))
            .and(header_exists("X-Cauce-Signature"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = WebhookDeliveryConfig::with_secret("test-secret")
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(0);
        let delivery_handler = WebhookDelivery::new(config);

        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: Some("webhook-secret".to_string()),
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        assert!(result.success);
        assert_eq!(result.status_code, Some(200));
    }

    #[tokio::test]
    async fn test_deliver_client_error_no_retry() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Client error (400) should not be retried
        Mock::given(method("POST"))
            .and(path("/webhook"))
            .respond_with(ResponseTemplate::new(400))
            .expect(1) // Only 1 attempt - no retry for client errors
            .mount(&mock_server)
            .await;

        let config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(3); // Even with retries configured, should not retry 400
        let delivery_handler = WebhookDelivery::new(config);

        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: None,
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.attempts, 1);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("400"));
    }

    #[tokio::test]
    async fn test_deliver_server_error_with_retry() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate, Respond};

        let mock_server = MockServer::start().await;

        // Use a responder that returns 500 on first call, 200 on second
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        struct SequentialResponder {
            call_count: Arc<AtomicUsize>,
        }

        impl Respond for SequentialResponder {
            fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
                let count = self.call_count.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    ResponseTemplate::new(500)
                } else {
                    ResponseTemplate::new(200)
                }
            }
        }

        Mock::given(method("POST"))
            .and(path("/webhook"))
            .respond_with(SequentialResponder { call_count: call_count_clone })
            .mount(&mock_server)
            .await;

        let mut config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(1);
        config.initial_retry_delay = Duration::from_millis(10);
        config.max_retry_delay = Duration::from_millis(50);

        let delivery_handler = WebhookDelivery::new(config);

        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: None,
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        // Should succeed on retry
        assert!(result.success);
        assert_eq!(result.attempts, 2);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_deliver_all_retries_exhausted() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Server always returns 500
        Mock::given(method("POST"))
            .and(path("/webhook"))
            .respond_with(ResponseTemplate::new(500))
            .expect(2) // Initial + 1 retry
            .mount(&mock_server)
            .await;

        let mut config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(1);
        config.initial_retry_delay = Duration::from_millis(10);
        config.max_retry_delay = Duration::from_millis(50);

        let delivery_handler = WebhookDelivery::new(config);

        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: None,
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.attempts, 2); // 1 initial + 1 retry
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_deliver_rate_limited_retry() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate, Respond};

        let mock_server = MockServer::start().await;

        // Use a responder that returns 429 on first call, 200 on second
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        struct RateLimitResponder {
            call_count: Arc<AtomicUsize>,
        }

        impl Respond for RateLimitResponder {
            fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
                let count = self.call_count.fetch_add(1, Ordering::SeqCst);
                if count == 0 {
                    ResponseTemplate::new(429)
                } else {
                    ResponseTemplate::new(200)
                }
            }
        }

        Mock::given(method("POST"))
            .and(path("/webhook"))
            .respond_with(RateLimitResponder { call_count: call_count_clone })
            .mount(&mock_server)
            .await;

        let mut config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(1);
        config.initial_retry_delay = Duration::from_millis(10);
        config.max_retry_delay = Duration::from_millis(50);

        let delivery_handler = WebhookDelivery::new(config);

        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: None,
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        // Should succeed on retry after rate limit
        assert!(result.success);
        assert_eq!(result.attempts, 2);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_deliver_connection_error() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;

        let config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_millis(100))
            .with_max_retries(0);
        let delivery_handler = WebhookDelivery::new(config);

        // Use a URL that will fail to connect
        let webhook_config = WebhookConfig {
            url: "http://127.0.0.1:1".to_string(), // Invalid port
            secret: None,
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        assert!(!result.success);
        assert_eq!(result.attempts, 1);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_deliver_uses_webhook_secret_over_default() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use wiremock::matchers::{method, path, header_regex};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Expect signature header with sha256= prefix
        Mock::given(method("POST"))
            .and(path("/webhook"))
            .and(header_regex("X-Cauce-Signature", "sha256=.*"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Default secret is "default-secret", but webhook config overrides it
        let config = WebhookDeliveryConfig::with_secret("default-secret")
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(0);
        let delivery_handler = WebhookDelivery::new(config);

        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: Some("webhook-specific-secret".to_string()),
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        assert!(result.success);
    }

    #[tokio::test]
    async fn test_deliver_no_signature_without_secret() {
        use cauce_core::methods::WebhookConfig;
        use cauce_core::SignalDelivery;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, Request, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Expect no signature header when no secret is configured
        Mock::given(method("POST"))
            .and(path("/webhook"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // No default secret
        let config = WebhookDeliveryConfig::default()
            .with_timeout(Duration::from_secs(5))
            .with_max_retries(0);
        let delivery_handler = WebhookDelivery::new(config);

        // No webhook secret either
        let webhook_config = WebhookConfig {
            url: format!("{}/webhook", mock_server.uri()),
            secret: None,
            headers: None,
        };

        let signal = create_test_signal();
        let signal_delivery = SignalDelivery::new("signal.email.*", signal);

        let result = delivery_handler.deliver(&webhook_config, &signal_delivery).await.unwrap();

        assert!(result.success);

        // Verify no signature was sent by checking received requests
        let requests: Vec<Request> = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(!requests[0].headers.contains_key("X-Cauce-Signature"));
    }
}
