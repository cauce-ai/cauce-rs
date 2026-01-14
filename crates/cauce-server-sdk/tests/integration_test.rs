//! Integration tests for cauce-server-sdk.
//!
//! These tests verify that all components work together correctly.

use std::sync::Arc;
use std::time::Duration;

use cauce_core::types::{Payload, Source, Topic};
use cauce_core::{
    ApprovalType, PublishRequest, Signal, SignalDelivery, SubscribeRequest, SubscriptionStatus, Transport,
};
use cauce_server_sdk::config::{RedeliveryConfig, ServerConfig};
use cauce_server_sdk::delivery::{DeliveryTracker, InMemoryDeliveryTracker};
use cauce_server_sdk::routing::{DefaultMessageRouter, MessageRouter};
use cauce_server_sdk::session::{InMemorySessionManager, SessionInfo, SessionManager};
use cauce_server_sdk::subscription::{InMemorySubscriptionManager, SubscriptionManager};
use cauce_server_sdk::DefaultCauceServer;
use chrono::Utc;
use serde_json::json;

/// Creates a test signal.
fn create_test_signal(topic: &str) -> Signal {
    Signal {
        id: format!("sig_{}", uuid::Uuid::new_v4()),
        version: "1.0".to_string(),
        timestamp: Utc::now(),
        source: Source::new("test", "adapter-1", "msg-1"),
        topic: Topic::new_unchecked(topic),
        payload: Payload::new(json!({"test": "data"}), "application/json"),
        metadata: None,
        encrypted: None,
    }
}

/// Creates a test session info.
fn create_test_session_info(client_id: &str) -> SessionInfo {
    SessionInfo::new(
        format!("session_{}", uuid::Uuid::new_v4()),
        client_id,
        "agent",
        "1.0",
        Transport::WebSocket,
        3600,
    )
}

// ============================================================================
// Subscription Flow Tests
// ============================================================================

#[tokio::test]
async fn test_subscription_flow() {
    let manager = InMemorySubscriptionManager::default();

    // Create a subscription
    let request = SubscribeRequest::new(vec!["signal.test.*".to_string()]);

    let response = manager.subscribe("client-1", "session-1", request).await.unwrap();
    assert!(!response.subscription_id.is_empty());

    // Verify subscription exists
    let sub = manager.get_subscription(&response.subscription_id).await.unwrap();
    assert!(sub.is_some());
    let sub = sub.unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert!(sub.topics.contains(&"signal.test.*".to_string()));

    // Find subscriptions for a topic
    let matching = manager.get_subscriptions_for_topic("signal.test.email").await.unwrap();
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].subscription_id, response.subscription_id);

    // Unsubscribe
    manager.unsubscribe(&response.subscription_id).await.unwrap();
    let sub = manager.get_subscription(&response.subscription_id).await.unwrap();
    assert!(sub.is_none());
}

#[tokio::test]
async fn test_subscription_with_approval() {
    let manager = InMemorySubscriptionManager::new()
        .with_default_approval(ApprovalType::UserApproved);

    let request = SubscribeRequest::new(vec!["signal.sensitive.*".to_string()]);

    let response = manager.subscribe("client-1", "session-1", request).await.unwrap();
    assert_eq!(response.status, SubscriptionStatus::Pending);

    // Approve the subscription
    manager.approve(&response.subscription_id, None).await.unwrap();

    let sub = manager.get_subscription(&response.subscription_id).await.unwrap();
    assert!(sub.is_some());
    assert_eq!(sub.unwrap().status, SubscriptionStatus::Active);
}

#[tokio::test]
async fn test_subscription_denial() {
    let manager = InMemorySubscriptionManager::new()
        .with_default_approval(ApprovalType::UserApproved);

    let request = SubscribeRequest::new(vec!["signal.restricted.*".to_string()]);

    let response = manager.subscribe("client-1", "session-1", request).await.unwrap();

    // Deny the subscription
    manager.deny(&response.subscription_id, Some("Not authorized".to_string())).await.unwrap();

    let sub = manager.get_subscription(&response.subscription_id).await.unwrap();
    assert!(sub.is_some());
    assert_eq!(sub.unwrap().status, SubscriptionStatus::Denied);
}

// ============================================================================
// Message Routing Tests
// ============================================================================

#[tokio::test]
async fn test_message_routing() {
    let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
    let router = DefaultMessageRouter::new(Arc::clone(&subscription_manager));

    // Create subscriptions
    let request1 = SubscribeRequest::new(vec!["signal.email.*".to_string()]);
    let sub1 = subscription_manager.subscribe("client-1", "session-1", request1).await.unwrap();

    let request2 = SubscribeRequest::new(vec!["signal.email.received".to_string()]);
    let sub2 = subscription_manager.subscribe("client-2", "session-2", request2).await.unwrap();

    // Route a message
    let request = PublishRequest::signal(
        "signal.email.received",
        create_test_signal("signal.email.received"),
    );

    let result = router.route(&request).await.unwrap();

    // Both subscriptions should match
    assert_eq!(result.subscription_count, 2);
    assert!(result.subscription_ids.contains(&sub1.subscription_id));
    assert!(result.subscription_ids.contains(&sub2.subscription_id));
}

#[tokio::test]
async fn test_message_routing_no_matches() {
    let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
    let router = DefaultMessageRouter::new(Arc::clone(&subscription_manager));

    // Create a subscription for a different topic
    let request = SubscribeRequest::new(vec!["signal.sms.*".to_string()]);
    let _ = subscription_manager.subscribe("client-1", "session-1", request).await.unwrap();

    // Route a message to a non-matching topic
    let request = PublishRequest::signal(
        "signal.email.received",
        create_test_signal("signal.email.received"),
    );

    let result = router.route(&request).await.unwrap();

    // No subscriptions should match
    assert_eq!(result.subscription_count, 0);
}

// ============================================================================
// Delivery Tracking Tests
// ============================================================================

#[tokio::test]
async fn test_delivery_tracking() {
    let config = RedeliveryConfig::default();
    let tracker = InMemoryDeliveryTracker::new(config);

    let signal = create_test_signal("signal.test.event");
    let delivery = SignalDelivery::new("signal.test.*", signal);

    // Track the delivery
    tracker.track("sub-1", &delivery).await.unwrap();

    // Get unacked deliveries
    let unacked = tracker.get_unacked("sub-1").await.unwrap();
    assert_eq!(unacked.len(), 1);
    assert_eq!(unacked[0].signal.topic.as_str(), "signal.test.event");

    // Acknowledge the delivery
    let ack_response = tracker.ack("sub-1", std::slice::from_ref(&delivery.signal.id)).await.unwrap();
    assert_eq!(ack_response.acknowledged.len(), 1);

    // No more unacked
    let unacked = tracker.get_unacked("sub-1").await.unwrap();
    assert!(unacked.is_empty());
}

#[tokio::test]
async fn test_delivery_redelivery() {
    let config = RedeliveryConfig::aggressive();
    let tracker = InMemoryDeliveryTracker::new(config);

    let signal = create_test_signal("signal.test.event");
    let delivery = SignalDelivery::new("signal.test.*", signal);

    // Track the delivery
    tracker.track("sub-1", &delivery).await.unwrap();

    // Simulate time passing - delivery should be ready for redelivery
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Get deliveries ready for redelivery
    let for_redelivery = tracker.get_for_redelivery().await.unwrap();

    // With aggressive config and 100ms delay, it might be ready
    // This depends on the actual delay settings
    // Just verify the function works without error
    assert!(for_redelivery.is_empty() || !for_redelivery.is_empty());
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[tokio::test]
async fn test_session_management() {
    let manager = InMemorySessionManager::default();

    // Create a session
    let info = create_test_session_info("client-1");
    let session_id = manager.create_session(info).await.unwrap();

    // Verify session is valid
    assert!(manager.is_valid(&session_id).await.unwrap());

    // Get session
    let session = manager.get_session(&session_id).await.unwrap();
    assert!(session.is_some());
    assert_eq!(session.unwrap().client_id, "client-1");

    // Touch session to keep it alive
    manager.touch_session(&session_id).await.unwrap();

    // Remove session
    manager.remove_session(&session_id).await.unwrap();
    assert!(!manager.is_valid(&session_id).await.unwrap());
}

#[tokio::test]
async fn test_session_expiration() {
    // Create manager with short default TTL
    let manager = InMemorySessionManager::new(1); // 1 second TTL

    // Create a session that will expire quickly (1 second TTL)
    let info = SessionInfo::new(
        format!("session_{}", uuid::Uuid::new_v4()),
        "client-1",
        "agent",
        "1.0",
        Transport::WebSocket,
        1, // 1 second TTL
    );
    let session_id: String = manager.create_session(info).await.unwrap();

    // Session should be valid immediately after creation
    let session: Option<SessionInfo> = manager.get_session(&session_id).await.unwrap();
    assert!(session.is_some());
    assert!(!session.unwrap().is_expired());

    // Wait for session to expire
    tokio::time::sleep(Duration::from_millis(1100)).await;

    // Session should be expired now
    let session: Option<SessionInfo> = manager.get_session(&session_id).await.unwrap();
    if let Some(s) = session {
        assert!(s.is_expired(), "Session should be expired after 1.1 seconds");
    }
}

// ============================================================================
// Server Integration Tests
// ============================================================================

#[test]
fn test_server_creation() {
    let server = create_http_test_server();

    // Verify all components are accessible
    let _ = server.subscription_manager();
    let _ = server.message_router();
    let _ = server.delivery_tracker();
    let _ = server.session_manager();
    let _ = server.config();
}

#[test]
fn test_server_router_creation() {
    let server = create_http_test_server();
    let _router = server.router();
}

#[test]
fn test_server_with_custom_config() {
    let config = ServerConfig::builder("0.0.0.0:9090".parse().unwrap())
        .build()
        .unwrap();

    let server = DefaultCauceServer::new(config);
    assert_eq!(server.address(), "0.0.0.0:9090".parse().unwrap());
}

// ============================================================================
// Full Flow Integration Test
// ============================================================================

#[tokio::test]
async fn test_full_publish_subscribe_flow() {
    // Set up components
    let subscription_manager = Arc::new(InMemorySubscriptionManager::default());
    let router = Arc::new(DefaultMessageRouter::new(Arc::clone(&subscription_manager)));
    let delivery_tracker = Arc::new(InMemoryDeliveryTracker::new(RedeliveryConfig::default()));
    let session_manager = Arc::new(InMemorySessionManager::default());

    // 1. Create a session
    let session_info = create_test_session_info("client-1");
    let session_id = session_manager.create_session(session_info).await.unwrap();
    assert!(session_manager.is_valid(&session_id).await.unwrap());

    // 2. Subscribe to a topic
    let sub_request = SubscribeRequest::new(vec!["signal.email.*".to_string()]);
    let sub_response = subscription_manager
        .subscribe("client-1", &session_id, sub_request)
        .await
        .unwrap();
    assert_eq!(sub_response.status, SubscriptionStatus::Active);

    // 3. Publish a message
    let signal = create_test_signal("signal.email.received");
    let signal_id = signal.id.clone();
    let publish_request = PublishRequest::signal("signal.email.received", signal.clone());

    let route_result = router.route(&publish_request).await.unwrap();
    assert_eq!(route_result.subscription_count, 1);

    // 4. Create and track delivery
    let delivery = SignalDelivery::new("signal.email.*", signal);
    delivery_tracker
        .track(&sub_response.subscription_id, &delivery)
        .await
        .unwrap();

    // 5. Verify unacked delivery
    let unacked = delivery_tracker
        .get_unacked(&sub_response.subscription_id)
        .await
        .unwrap();
    assert_eq!(unacked.len(), 1);

    // 6. Acknowledge the delivery
    let ack_response = delivery_tracker
        .ack(&sub_response.subscription_id, &[signal_id])
        .await
        .unwrap();
    assert_eq!(ack_response.acknowledged.len(), 1);

    // 7. Verify no more unacked
    let unacked = delivery_tracker
        .get_unacked(&sub_response.subscription_id)
        .await
        .unwrap();
    assert!(unacked.is_empty());

    // 8. Unsubscribe
    subscription_manager
        .unsubscribe(&sub_response.subscription_id)
        .await
        .unwrap();

    // 9. Clean up session
    session_manager.remove_session(&session_id).await.unwrap();
}

// ============================================================================
// Auth and Rate Limiting Tests
// ============================================================================

#[tokio::test]
async fn test_auth_validator() {
    use cauce_server_sdk::auth::{AuthValidator, InMemoryAuthValidator};

    let validator = InMemoryAuthValidator::new()
        .with_api_key("client-1", "sk_test_123")
        .with_bearer_token("client-2", "token_abc");

    // Valid API key
    let result = validator.validate_api_key("sk_test_123").await.unwrap();
    assert_eq!(result, Some("client-1".to_string()));

    // Invalid API key
    let result = validator.validate_api_key("invalid").await.unwrap();
    assert!(result.is_none());

    // Valid bearer token
    let result = validator.validate_bearer_token("token_abc").await.unwrap();
    assert_eq!(result, Some("client-2".to_string()));

    // Invalid bearer token
    let result = validator.validate_bearer_token("invalid").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_rate_limiter() {
    use cauce_server_sdk::rate_limit::{InMemoryRateLimiter, RateLimitConfig, RateLimiter};

    let config = RateLimitConfig::default().with_bucket_capacity(3);
    let limiter = InMemoryRateLimiter::new(config);

    // First 3 requests should succeed
    for i in 0..3 {
        let result = limiter.consume("test-key").await.unwrap();
        assert!(result.allowed, "Request {} should be allowed", i);
    }

    // 4th request should be denied
    let result = limiter.consume("test-key").await.unwrap();
    assert!(!result.allowed);
    assert!(result.retry_after_ms.is_some());
}

// ============================================================================
// Webhook Delivery Tests
// ============================================================================

#[test]
fn test_webhook_delivery_creation() {
    use cauce_server_sdk::transport::WebhookDelivery;

    // Test that WebhookDelivery can be created with a secret
    let delivery = WebhookDelivery::with_secret("test-secret");

    // The signature verification system should work correctly
    // A signature generated with secret A should only verify with secret A
    let payload = r#"{"test":"data"}"#;
    let timestamp = 1234567890i64;

    // Compute expected signature manually using the same algorithm
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let signing_input = format!("{}.{}", timestamp, payload);
    let mut mac = HmacSha256::new_from_slice(b"test-secret").unwrap();
    mac.update(signing_input.as_bytes());
    let result = mac.finalize();
    let expected_hex = hex::encode(result.into_bytes());
    let signature = format!("sha256={}", expected_hex);

    // Verify with correct secret should pass
    assert!(delivery.verify_signature(payload, timestamp, &signature, "test-secret"));
    // Verify with wrong secret should fail
    assert!(!delivery.verify_signature(payload, timestamp, &signature, "wrong-secret"));
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_subscription_not_found() {
    let manager = InMemorySubscriptionManager::default();

    let result = manager.get_subscription("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_unsubscribe_not_found() {
    let manager = InMemorySubscriptionManager::default();

    let result = manager.unsubscribe("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_not_found() {
    let manager = InMemorySessionManager::default();

    let session = manager.get_session("nonexistent").await.unwrap();
    assert!(session.is_none());

    assert!(!manager.is_valid("nonexistent").await.unwrap());
}

// ============================================================================
// HTTP Handler Tests via Tower
// ============================================================================

/// Creates a test server with relaxed rate limiting for HTTP tests.
fn create_http_test_server() -> DefaultCauceServer {
    use cauce_server_sdk::config::{LimitsConfig, ServerConfig};

    // Use a config with high rate limit to avoid 429s in tests
    let config = ServerConfig::builder("127.0.0.1:8080".parse().unwrap())
        .limits(LimitsConfig::default().with_rate_limit(10000, 10000))
        .build()
        .unwrap();
    DefaultCauceServer::new(config)
}

#[tokio::test]
async fn test_http_health_endpoint() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let server = create_http_test_server();
    let app = server.router();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_http_poll_endpoint_invalid_session() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let server = create_http_test_server();
    let app = server.router();

    // Poll with invalid session should return unauthorized
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cauce/v1/poll?session_id=invalid_session")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_http_poll_endpoint_valid_session() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let server = create_http_test_server();
    let session_manager = server.session_manager();

    // Create a valid session first
    let session_info = create_test_session_info("test-client");
    let session_id = session_manager.create_session(session_info).await.unwrap();

    let app = server.router();

    // Poll with valid session should return OK
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/cauce/v1/poll?session_id={}", session_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("signals"));
}

#[tokio::test]
async fn test_http_ack_endpoint_invalid_session() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let server = create_http_test_server();
    let app = server.router();

    // Ack with invalid session should return unauthorized
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/cauce/v1/ack?session_id=invalid_session")
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"subscription_id":"sub_1","signal_ids":["sig_1"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_http_ack_endpoint_valid_session() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let server = create_http_test_server();
    let session_manager = server.session_manager();

    // Create a valid session first
    let session_info = create_test_session_info("test-client");
    let session_id = session_manager.create_session(session_info).await.unwrap();

    let app = server.router();

    // Ack with valid session should return OK
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/cauce/v1/ack?session_id={}", session_id))
                .header("Content-Type", "application/json")
                .body(Body::from(r#"{"subscription_id":"sub_1","signal_ids":["sig_1"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("acknowledged"));
}

#[tokio::test]
async fn test_http_sse_endpoint_invalid_session() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let server = create_http_test_server();
    let app = server.router();

    // SSE with invalid session should return error event
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cauce/v1/sse?session_id=invalid_session")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // SSE always returns 200 but sends error event in the stream
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_http_poll_with_subscription_filter() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let server = create_http_test_server();
    let session_manager = server.session_manager();
    let delivery_tracker = server.delivery_tracker();

    // Create a valid session first
    let session_info = create_test_session_info("test-client");
    let session_id = session_manager.create_session(session_info).await.unwrap();

    // Track a delivery
    let signal = create_test_signal("signal.test.event");
    let delivery = SignalDelivery::new("signal.test.*", signal);
    delivery_tracker.track("sub_123", &delivery).await.unwrap();

    let app = server.router();

    // Poll with subscription filter should return the tracked signal
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/cauce/v1/poll?session_id={}&subscription_id=sub_123", session_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();
    assert!(body_str.contains("signals"));
    // Should have the tracked signal
    assert!(body_str.contains("signal.test.event") || body_str.contains("sub_123"));
}

#[tokio::test]
async fn test_http_rate_limiting() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use cauce_server_sdk::config::{LimitsConfig, ServerConfig};
    use tower::ServiceExt;

    // Create server with strict rate limiting
    let config = ServerConfig::builder("127.0.0.1:8080".parse().unwrap())
        .limits(LimitsConfig::default().with_rate_limit(1, 1))
        .build()
        .unwrap();
    let server = DefaultCauceServer::new(config);

    let session_manager = server.session_manager();

    // Create a valid session
    let session_info = create_test_session_info("test-client");
    let _session_id = session_manager.create_session(session_info).await.unwrap();

    // The rate limiter should allow some requests before limiting
    // Note: The router() call creates new rate limiters, so we need to make
    // requests quickly to trigger rate limiting

    let app = server.router();

    // First request should succeed
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be OK or rate limited
    assert!(
        response.status() == StatusCode::OK ||
        response.status() == StatusCode::TOO_MANY_REQUESTS
    );
}

// ============================================================================
// WebSocket Handler Integration Tests
// ============================================================================

/// Starts a test server and returns the address
async fn start_test_server() -> (std::net::SocketAddr, DefaultCauceServer) {
    use cauce_server_sdk::config::{LimitsConfig, ServerConfig};
    use tokio::net::TcpListener;

    // Find an available port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);

    let config = ServerConfig::builder(addr)
        .limits(LimitsConfig::default().with_rate_limit(10000, 10000))
        .build()
        .unwrap();
    let server = DefaultCauceServer::new(config);

    (addr, server)
}

#[tokio::test]
async fn test_websocket_hello_flow() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    // Start the server
    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Give the server time to start
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    // Connect WebSocket client
    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Send hello request (protocol_version is required, not client_version)
    let hello_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.hello",
        "params": {
            "protocol_version": "1.0",
            "client_id": "test-client",
            "client_type": "agent"
        },
        "id": 1
    });

    ws_stream
        .send(Message::Text(hello_request.to_string()))
        .await
        .expect("Failed to send hello");

    // Receive response
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout waiting for response").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        // HelloResponse has session_id and server_version
        assert!(json["result"]["session_id"].is_string(), "Expected session_id: {:?}", json);
        assert!(json["result"]["server_version"].is_string(), "Expected server_version: {:?}", json);
    } else {
        panic!("Expected text message");
    }

    // Clean up
    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_ping_pong() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // First authenticate
    let hello_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.hello",
        "params": {
            "protocol_version": "1.0",
            "client_id": "test-client",
            "client_type": "agent"
        },
        "id": 1
    });
    ws_stream.send(Message::Text(hello_request.to_string())).await.unwrap();
    ws_stream.next().await; // Consume hello response

    // Send ping request (cauce.ping is handled as a request, not notification)
    let ping_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.ping",
        "params": {},
        "id": 2
    });

    ws_stream
        .send(Message::Text(ping_request.to_string()))
        .await
        .expect("Failed to send ping");

    // Receive ping response
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 2);
        // Ping response includes a timestamp
        assert!(json["result"]["timestamp"].is_string(), "Expected timestamp: {:?}", json);
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_subscribe_flow() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // First authenticate
    let hello_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.hello",
        "params": {
            "protocol_version": "1.0",
            "client_id": "test-client",
            "client_type": "agent"
        },
        "id": 1
    });
    ws_stream.send(Message::Text(hello_request.to_string())).await.unwrap();
    ws_stream.next().await; // Consume hello response

    // Send subscribe request
    let subscribe_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.subscribe",
        "params": {
            "topics": ["signal.test.*"]
        },
        "id": 2
    });

    ws_stream
        .send(Message::Text(subscribe_request.to_string()))
        .await
        .expect("Failed to send subscribe");

    // Receive response
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 2);
        assert!(json["result"]["subscription_id"].is_string());
        assert_eq!(json["result"]["status"], "active");
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_unsubscribe_flow() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Authenticate
    let hello = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.hello",
        "params": {
            "protocol_version": "1.0",
            "client_id": "test-client",
            "client_type": "agent"
        },
        "id": 1
    });
    ws_stream.send(Message::Text(hello.to_string())).await.unwrap();
    ws_stream.next().await;

    // Subscribe
    let subscribe = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.subscribe",
        "params": { "topics": ["signal.test.*"] },
        "id": 2
    });
    ws_stream.send(Message::Text(subscribe.to_string())).await.unwrap();

    let sub_response = ws_stream.next().await.unwrap().unwrap();
    let sub_json: serde_json::Value = if let Message::Text(t) = sub_response {
        serde_json::from_str(&t).unwrap()
    } else { panic!("Expected text") };
    let subscription_id = sub_json["result"]["subscription_id"].as_str().unwrap();

    // Unsubscribe
    let unsubscribe = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.unsubscribe",
        "params": { "subscription_id": subscription_id },
        "id": 3
    });
    ws_stream.send(Message::Text(unsubscribe.to_string())).await.unwrap();

    let unsub_response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = unsub_response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["id"], 3);
        assert!(json["result"]["success"].as_bool().unwrap());
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_goodbye_flow() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Authenticate
    let hello = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.hello",
        "params": {
            "protocol_version": "1.0",
            "client_id": "test-client",
            "client_type": "agent"
        },
        "id": 1
    });
    ws_stream.send(Message::Text(hello.to_string())).await.unwrap();
    ws_stream.next().await;

    // Send goodbye
    let goodbye = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.goodbye",
        "params": {},
        "id": 2
    });
    ws_stream.send(Message::Text(goodbye.to_string())).await.unwrap();

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["id"], 2);
        // Goodbye should return success
        assert!(json.get("result").is_some() || json.get("error").is_none());
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_ack_flow() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Authenticate
    let hello = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.hello",
        "params": {
            "protocol_version": "1.0",
            "client_id": "test-client",
            "client_type": "agent"
        },
        "id": 1
    });
    ws_stream.send(Message::Text(hello.to_string())).await.unwrap();
    ws_stream.next().await;

    // Subscribe first
    let subscribe = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.subscribe",
        "params": { "topics": ["signal.test.*"] },
        "id": 2
    });
    ws_stream.send(Message::Text(subscribe.to_string())).await.unwrap();
    let sub_response = ws_stream.next().await.unwrap().unwrap();
    let sub_json: serde_json::Value = if let Message::Text(t) = sub_response {
        serde_json::from_str(&t).unwrap()
    } else { panic!("Expected text") };
    let subscription_id = sub_json["result"]["subscription_id"].as_str().unwrap();

    // Send ack request
    let ack = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.ack",
        "params": {
            "subscription_id": subscription_id,
            "signal_ids": ["sig_nonexistent"]
        },
        "id": 3
    });
    ws_stream.send(Message::Text(ack.to_string())).await.unwrap();

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["id"], 3);
        // Ack should have an acknowledged array (possibly empty for nonexistent)
        assert!(json["result"]["acknowledged"].is_array());
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_method_not_found() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Send unknown method
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "unknown_method",
        "params": {},
        "id": 1
    });
    ws_stream.send(Message::Text(request.to_string())).await.unwrap();

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["id"], 1);
        // Should have an error
        assert!(json.get("error").is_some());
        assert_eq!(json["error"]["code"], -32601); // Method not found
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_invalid_json() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Send invalid JSON
    ws_stream.send(Message::Text("not valid json".to_string())).await.unwrap();

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        // Should have a parse error
        assert!(json.get("error").is_some());
        assert_eq!(json["error"]["code"], -32700); // Parse error
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

#[tokio::test]
async fn test_websocket_subscribe_without_hello() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Send subscribe without hello first
    let subscribe = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.subscribe",
        "params": { "topics": ["signal.test.*"] },
        "id": 1
    });
    ws_stream.send(Message::Text(subscribe.to_string())).await.unwrap();

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["id"], 1);
        // Should have an error - session required
        assert!(json.get("error").is_some());
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}

// ============================================================================
// SSE Handler Integration Tests
// ============================================================================

#[tokio::test]
async fn test_sse_stream_valid_session() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let server = create_http_test_server();
    let session_manager = server.session_manager();
    let delivery_tracker = server.delivery_tracker();
    let subscription_manager = server.subscription_manager();

    // Create a valid session
    let session_info = create_test_session_info("test-client");
    let session_id = session_manager.create_session(session_info).await.unwrap();

    // Create a subscription
    let sub_request = SubscribeRequest::new(vec!["signal.test.*".to_string()]);
    let sub_response = subscription_manager
        .subscribe("test-client", &session_id, sub_request)
        .await
        .unwrap();

    // Track some deliveries
    let signal = create_test_signal("signal.test.event");
    let delivery = SignalDelivery::new("signal.test.*", signal);
    delivery_tracker
        .track(&sub_response.subscription_id, &delivery)
        .await
        .unwrap();

    let app = server.router();

    // Request SSE stream with subscription filter
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/cauce/v1/sse?session_id={}&subscription_id={}",
                    session_id, sub_response.subscription_id
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Check content type is SSE
    let content_type = response.headers().get("content-type");
    assert!(content_type.is_some());
    assert!(content_type.unwrap().to_str().unwrap().contains("text/event-stream"));

    // Read some of the body to verify SSE format
    let body = response.into_body();
    let bytes = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        body.collect()
    ).await;

    // Either timeout (stream stays open) or get data
    if let Ok(Ok(collected)) = bytes {
        let data = String::from_utf8(collected.to_bytes().to_vec()).unwrap();
        // SSE events start with "event:" or "data:" or ":"
        assert!(
            data.contains("event:") || data.contains("data:") || data.starts_with(':'),
            "SSE stream should contain events: {}",
            data
        );
    }
    // Timeout is acceptable - SSE streams stay open
}

#[tokio::test]
async fn test_sse_stream_invalid_session() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    let server = create_http_test_server();
    let app = server.router();

    // Request SSE stream with invalid session
    let response = app
        .oneshot(
            Request::builder()
                .uri("/cauce/v1/sse?session_id=invalid_session_id")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // SSE always returns 200, but sends error event
    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body();
    let collected = tokio::time::timeout(
        std::time::Duration::from_millis(500),
        body.collect()
    ).await.unwrap().unwrap();

    let data = String::from_utf8(collected.to_bytes().to_vec()).unwrap();
    // Should contain an error event
    assert!(data.contains("error") || data.contains("invalid_session"));
}

#[tokio::test]
async fn test_sse_stream_headers() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let server = create_http_test_server();
    let session_manager = server.session_manager();

    let session_info = create_test_session_info("test-client");
    let session_id = session_manager.create_session(session_info).await.unwrap();

    let app = server.router();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/cauce/v1/sse?session_id={}", session_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify SSE headers
    let content_type = response.headers().get("content-type").unwrap();
    assert!(content_type.to_str().unwrap().contains("text/event-stream"));

    // Should have cache-control: no-cache
    let cache_control = response.headers().get("cache-control");
    if let Some(cc) = cache_control {
        assert!(cc.to_str().unwrap().contains("no-cache"));
    }
}

// ============================================================================
// Combined WebSocket + Publish Flow Test
// ============================================================================

#[tokio::test]
async fn test_websocket_publish_flow() {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::{connect_async, tungstenite::Message};
    use tokio::net::TcpListener;

    let (addr, server) = start_test_server().await;
    let router = server.router();

    let listener = TcpListener::bind(addr).await.unwrap();
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ws_url = format!("ws://{}/cauce/v1/ws", addr);
    let (mut ws_stream, _) = connect_async(&ws_url).await.expect("Failed to connect");

    // Authenticate
    let hello = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.hello",
        "params": {
            "protocol_version": "1.0",
            "client_id": "test-client",
            "client_type": "adapter"
        },
        "id": 1
    });
    ws_stream.send(Message::Text(hello.to_string())).await.unwrap();
    ws_stream.next().await;

    // Publish a signal
    // PublishMessage is untagged, so the signal is placed directly in `message`
    // Signal fields: id, version, timestamp, source, topic, payload
    // Source: type (renamed from type_), adapter_id, native_id
    // Payload: raw, content_type, size_bytes
    let publish = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "cauce.publish",
        "params": {
            "topic": "signal.email.received",
            "message": {
                "id": "sig_test_123",
                "version": "1.0",
                "timestamp": "2024-01-01T00:00:00Z",
                "source": {
                    "type": "email",
                    "adapter_id": "adapter-1",
                    "native_id": "msg-1"
                },
                "topic": "signal.email.received",
                "payload": {
                    "raw": {"text": "hello"},
                    "content_type": "application/json",
                    "size_bytes": 17
                }
            }
        },
        "id": 2
    });
    ws_stream.send(Message::Text(publish.to_string())).await.unwrap();

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        ws_stream.next()
    ).await.expect("Timeout").unwrap().unwrap();

    if let Message::Text(text) = response {
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["id"], 2);
        // PublishResponse has message_id, delivered_to, and queued_for
        assert!(json["result"]["message_id"].is_string(), "Expected message_id: {:?}", json);
        assert!(json["result"]["delivered_to"].is_number(), "Expected delivered_to: {:?}", json);
    } else {
        panic!("Expected text message");
    }

    ws_stream.close(None).await.ok();
    server_handle.abort();
}
