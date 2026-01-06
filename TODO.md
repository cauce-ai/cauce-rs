# Cauce-RS Implementation TODO

> **Status**: Planning → Implementation
> **Target**: Full Rust reference implementation for protocol validation and developer demos
> **Coverage Requirement**: 95% code coverage (per project guidelines)

This document outlines the complete implementation plan for cauce-rs, a Rust reference implementation of the Cauce Protocol.

---

## Phase 1: Project Foundation

### 1.1 Repository Setup
- [ ] Initialize Cargo workspace with `Cargo.toml` at root
- [ ] Configure workspace members for all planned crates
- [ ] Set up shared workspace dependencies (tokio, serde, etc.)
- [ ] Create `.gitignore` for Rust/IDE artifacts
- [ ] Add `rustfmt.toml` with project formatting rules
- [ ] Add `clippy.toml` with lint configuration
- [ ] Create `deny.toml` for cargo-deny (license/vulnerability checks)
- [ ] Set up pre-commit hooks (format, clippy, test)

### 1.2 CI/CD Pipeline
- [ ] Create GitHub Actions workflow for CI
  - [ ] Build on multiple platforms (Linux, macOS, Windows)
  - [ ] Run tests with `cargo test --workspace`
  - [ ] Run tests with coverage reporting (`cargo-tarpaulin` or `llvm-cov`)
  - [ ] **Enforce 95% coverage threshold** (fail CI if below)
  - [ ] Run clippy with `--deny warnings`
  - [ ] Check formatting with `cargo fmt --check`
  - [ ] Run cargo-deny for license/security audit
- [ ] Add workflow for release builds
- [ ] Add workflow for Docker image builds

### 1.3 Documentation Structure
- [ ] Create `docs/` directory structure
- [ ] Add `CONTRIBUTING.md` with development guidelines
- [ ] Add `ARCHITECTURE.md` with crate dependency diagram

---

## Phase 2: Core Library (`cauce-core`)

### 2.1 Project Setup
- [ ] Create `crates/cauce-core/Cargo.toml`
- [ ] Add dependencies: `serde`, `serde_json`, `thiserror`, `chrono`, `uuid`, `jsonschema`
- [ ] Create module structure: `lib.rs`, `types/`, `jsonrpc/`, `validation/`, `errors/`, `constants/`

### 2.2 Core Types (`types/` module)
- [ ] Implement `Signal` struct
  - [ ] Fields: `id`, `version`, `timestamp`, `source`, `topic`, `payload`, `metadata`, `encrypted`
  - [ ] ID format validation: `sig_<timestamp>_<random>`
  - [ ] Implement `Serialize`/`Deserialize`
  - [ ] Add builder pattern for construction
- [ ] Implement `Source` struct
  - [ ] Fields: `type_`, `adapter_id`, `native_id`
- [ ] Implement `Payload` struct
  - [ ] Fields: `raw`, `content_type`, `size_bytes`
- [ ] Implement `Metadata` struct
  - [ ] Fields: `thread_id`, `in_reply_to`, `references`, `priority`, `tags`
- [ ] Implement `Priority` enum: `Low`, `Normal`, `High`, `Urgent`
- [ ] Implement `Action` struct
  - [ ] Fields: `id`, `version`, `timestamp`, `topic`, `action`, `context`, `encrypted`
  - [ ] ID format validation: `act_<timestamp>_<random>`
- [ ] Implement `ActionBody` struct
  - [ ] Fields: `type_`, `target`, `payload`
- [ ] Implement `ActionType` enum: `Send`, `Reply`, `Forward`, `React`, `Update`, `Delete`
- [ ] Implement `ActionContext` struct
  - [ ] Fields: `in_reply_to`, `agent_id`, `thread_id`, `correlation_id`
- [ ] Implement `Encrypted` struct
  - [ ] Fields: `algorithm`, `recipient_public_key`, `nonce`, `ciphertext`
- [ ] Implement `EncryptionAlgorithm` enum: `X25519XSalsa20Poly1305`, `A256GCM`, `XChaCha20Poly1305`
- [ ] Implement `Topic` newtype with validation
  - [ ] Length: 1-255 chars
  - [ ] Pattern: alphanumeric, dots, hyphens, underscores
  - [ ] No leading/trailing dots, no consecutive dots

### 2.3 JSON-RPC Types (`jsonrpc/` module)
- [ ] Implement `JsonRpcRequest` struct
  - [ ] Fields: `jsonrpc`, `id`, `method`, `params`
  - [ ] Validate `jsonrpc` is "2.0"
- [ ] Implement `JsonRpcResponse` struct
  - [ ] Variants: success (with `result`) or error (with `error`)
- [ ] Implement `JsonRpcNotification` struct
  - [ ] No `id` field
- [ ] Implement `JsonRpcError` struct
  - [ ] Fields: `code`, `message`, `data`
- [ ] Implement `RequestId` type (string or integer)
- [ ] Add helper methods for creating responses/errors

### 2.4 Method Parameter Types (`methods/` module)
- [ ] `HelloRequest` / `HelloResponse`
  - [ ] Request: `protocol_version`, `min_protocol_version`, `max_protocol_version`, `client_id`, `client_type`, `capabilities`, `auth`
  - [ ] Response: `session_id`, `server_version`, `capabilities`, `session_expires_at`
- [ ] `Auth` struct: `type_`, `token`, `api_key`
- [ ] `AuthType` enum: `Bearer`, `ApiKey`, `Mtls`
- [ ] `ClientType` enum: `Adapter`, `Agent`, `A2aAgent`
- [ ] `Capability` enum: `Subscribe`, `Publish`, `Ack`, `E2eEncryption`
- [ ] `SubscribeRequest` / `SubscribeResponse`
  - [ ] Request: `topics`, `approval_type`, `reason`, `transport`, `webhook`, `e2e`
  - [ ] Response: `subscription_id`, `status`, `topics`, `created_at`, `expires_at`
- [ ] `ApprovalType` enum: `Automatic`, `UserApproved`
- [ ] `Transport` enum: `WebSocket`, `Sse`, `Polling`, `LongPolling`, `Webhook`
- [ ] `WebhookConfig` struct: `url`, `secret`, `headers`
- [ ] `E2eConfig` struct: `enabled`, `public_key`, `supported_algorithms`
- [ ] `UnsubscribeRequest` / `UnsubscribeResponse`
- [ ] `PublishRequest` / `PublishResponse`
  - [ ] Request: `topic`, `message` (Signal or Action)
  - [ ] Response: `message_id`, `delivered_to`, `queued_for`
- [ ] `AckRequest` / `AckResponse`
  - [ ] Request: `signal_ids`, `subscription_id`
  - [ ] Response: `acknowledged`, `failed`
- [ ] `SignalDelivery` struct (for `cauce.signal` notification)
  - [ ] Fields: `topic`, `signal`
- [ ] `SubscriptionApproveRequest` / `SubscriptionDenyRequest` / `SubscriptionRevokeRequest`
- [ ] `SubscriptionListRequest` / `SubscriptionListResponse`
- [ ] `SubscriptionInfo` struct
- [ ] `SubscriptionStatusNotification` struct
- [ ] `PingParams` / `PongParams` (timestamp)
- [ ] `SchemasListRequest` / `SchemasListResponse`
- [ ] `SchemasGetRequest` / `SchemasGetResponse`

### 2.5 Error Codes (`errors/` module)
- [ ] Define `CauceError` enum with all error variants
- [ ] JSON-RPC standard errors:
  - [ ] `ParseError` (-32700)
  - [ ] `InvalidRequest` (-32600)
  - [ ] `MethodNotFound` (-32601)
  - [ ] `InvalidParams` (-32602)
  - [ ] `InternalError` (-32603)
- [ ] Cauce protocol errors:
  - [ ] `SubscriptionNotFound` (-32001)
  - [ ] `TopicNotFound` (-32002)
  - [ ] `NotAuthorized` (-32003)
  - [ ] `SubscriptionPending` (-32004)
  - [ ] `SubscriptionDenied` (-32005)
  - [ ] `RateLimited` (-32006)
  - [ ] `SignalTooLarge` (-32007)
  - [ ] `EncryptionRequired` (-32008)
  - [ ] `InvalidEncryption` (-32009)
  - [ ] `AdapterUnavailable` (-32010)
  - [ ] `DeliveryFailed` (-32011)
  - [ ] `QueueFull` (-32012)
  - [ ] `SessionExpired` (-32013)
  - [ ] `UnsupportedTransport` (-32014)
  - [ ] `InvalidTopic` (-32015)
- [ ] Implement `From<CauceError> for JsonRpcError`
- [ ] Add error data helpers (details, suggestion, retry_after_ms, field)

### 2.6 Method Constants (`constants/` module)
- [ ] Define method name constants:
  - [ ] `HELLO` = "cauce.hello"
  - [ ] `GOODBYE` = "cauce.goodbye"
  - [ ] `PING` = "cauce.ping"
  - [ ] `PONG` = "cauce.pong"
  - [ ] `PUBLISH` = "cauce.publish"
  - [ ] `SUBSCRIBE` = "cauce.subscribe"
  - [ ] `UNSUBSCRIBE` = "cauce.unsubscribe"
  - [ ] `SIGNAL` = "cauce.signal"
  - [ ] `ACK` = "cauce.ack"
  - [ ] `SUBSCRIPTION_REQUEST` = "cauce.subscription.request"
  - [ ] `SUBSCRIPTION_APPROVE` = "cauce.subscription.approve"
  - [ ] `SUBSCRIPTION_DENY` = "cauce.subscription.deny"
  - [ ] `SUBSCRIPTION_LIST` = "cauce.subscription.list"
  - [ ] `SUBSCRIPTION_REVOKE` = "cauce.subscription.revoke"
  - [ ] `SUBSCRIPTION_STATUS` = "cauce.subscription.status"
  - [ ] `SCHEMAS_LIST` = "cauce.schemas.list"
  - [ ] `SCHEMAS_GET` = "cauce.schemas.get"
- [ ] Define protocol version constant: `PROTOCOL_VERSION` = "1.0"
- [ ] Define size limits:
  - [ ] `MAX_TOPIC_LENGTH` = 255
  - [ ] `MAX_SIGNAL_PAYLOAD_SIZE` = 10 * 1024 * 1024
  - [ ] `MAX_TOPICS_PER_SUBSCRIPTION` = 100
  - [ ] `MAX_SUBSCRIPTIONS_PER_CLIENT` = 1000
  - [ ] `MAX_SIGNALS_PER_BATCH` = 100

### 2.7 Validation (`validation/` module)
- [ ] Embed JSON schemas as static assets using `include_str!`:
  - [ ] `signal.schema.json`
  - [ ] `action.schema.json`
  - [ ] `jsonrpc.schema.json`
  - [ ] `errors.schema.json`
  - [ ] All method schemas (subscribe, unsubscribe, publish, ack, hello, subscription, schemas)
  - [ ] Payload schemas (email, sms, slack, voice)
- [ ] Implement `validate_signal(value: &Value) -> Result<Signal, ValidationError>`
- [ ] Implement `validate_action(value: &Value) -> Result<Action, ValidationError>`
- [ ] Implement `validate_topic(topic: &str) -> Result<(), ValidationError>`
- [ ] Implement `validate_topic_pattern(pattern: &str) -> Result<(), ValidationError>` (allows wildcards)
- [ ] Implement `validate_signal_id(id: &str) -> Result<(), ValidationError>`
- [ ] Implement `validate_action_id(id: &str) -> Result<(), ValidationError>`
- [ ] Implement `validate_subscription_id(id: &str) -> Result<(), ValidationError>`
- [ ] Implement `validate_session_id(id: &str) -> Result<(), ValidationError>`
- [ ] Create `ValidationError` type with detailed error information

### 2.8 ID Generation Utilities
- [ ] Implement `generate_signal_id() -> String`
- [ ] Implement `generate_action_id() -> String`
- [ ] Implement `generate_subscription_id() -> String`
- [ ] Implement `generate_session_id() -> String`
- [ ] Implement `generate_message_id() -> String`

### 2.9 Topic Matching Utilities
- [ ] Implement `TopicMatcher` struct
- [ ] Implement `matches(topic: &str, pattern: &str) -> bool`
  - [ ] Support `*` for single segment match
  - [ ] Support `**` for multi-segment match
- [ ] Implement efficient topic trie for subscription matching

### 2.10 Testing for cauce-core
- [ ] Unit tests for Signal serialization/deserialization
- [ ] Unit tests for Action serialization/deserialization
- [ ] Unit tests for JSON-RPC message parsing
- [ ] Unit tests for all error code conversions
- [ ] Unit tests for topic validation (valid and invalid cases)
- [ ] Unit tests for topic pattern matching (wildcards)
- [ ] Unit tests for ID generation format
- [ ] Unit tests for ID validation
- [ ] Property-based tests for topic matching
- [ ] Integration test: roundtrip serialize/deserialize all types
- [ ] Test against JSON schemas from protocol spec

---

## Phase 3: Client SDK (`cauce-client-sdk`)

### 3.1 Project Setup
- [ ] Create `crates/cauce-client-sdk/Cargo.toml`
- [ ] Add dependencies: `cauce-core`, `tokio`, `tokio-tungstenite`, `reqwest`, `futures`, `async-trait`, `tracing`
- [ ] Create module structure: `lib.rs`, `client/`, `transport/`, `subscription/`, `config/`

### 3.2 Client Configuration (`config/` module)
- [ ] Implement `ClientConfig` struct
  - [ ] `hub_url`: Hub URL (ws/wss/http/https)
  - [ ] `client_id`: Client identifier
  - [ ] `client_type`: Adapter/Agent/A2aAgent
  - [ ] `auth`: Authentication config
  - [ ] `transport`: Preferred transport
  - [ ] `reconnect`: Reconnection settings
  - [ ] `tls`: TLS configuration
- [ ] Implement `AuthConfig` enum
  - [ ] `ApiKey { key: String }`
  - [ ] `Bearer { token: String }`
- [ ] Implement `ReconnectConfig`
  - [ ] `enabled`: bool
  - [ ] `initial_delay_ms`: u64
  - [ ] `max_delay_ms`: u64
  - [ ] `backoff_multiplier`: f64
- [ ] Implement `TlsConfig`
  - [ ] `accept_invalid_certs`: bool (dev only)
  - [ ] `client_cert`: Option<PathBuf>
  - [ ] `client_key`: Option<PathBuf>
- [ ] Implement `ConfigBuilder` pattern

### 3.3 Transport Trait (`transport/` module)
- [ ] Define `Transport` trait
  ```rust
  #[async_trait]
  pub trait Transport: Send + Sync {
      async fn connect(&mut self) -> Result<()>;
      async fn disconnect(&mut self) -> Result<()>;
      async fn send(&mut self, message: JsonRpcRequest) -> Result<()>;
      async fn receive(&mut self) -> Result<JsonRpcMessage>;
      fn is_connected(&self) -> bool;
  }
  ```
- [ ] Define `JsonRpcMessage` enum (Request, Response, Notification)

### 3.4 WebSocket Transport (`transport/websocket.rs`)
- [ ] Implement `WebSocketTransport` struct
- [ ] Implement connection with TLS support
- [ ] Implement message framing (text frames)
- [ ] Implement ping/pong handling
- [ ] Implement reconnection logic with exponential backoff
- [ ] Handle connection errors and timeouts
- [ ] Add tracing/logging for connection events

### 3.5 SSE Transport (`transport/sse.rs`)
- [ ] Implement `SseTransport` struct
- [ ] Implement GET request with `Accept: text/event-stream`
- [ ] Parse SSE event format (`event:`, `id:`, `data:`)
- [ ] Handle reconnection with `Last-Event-ID` header
- [ ] Implement separate HTTP POST for sending messages
- [ ] Handle keepalive events

### 3.6 HTTP Polling Transport (`transport/polling.rs`)
- [ ] Implement `PollingTransport` struct
- [ ] Implement poll request: `GET /cauce/v1/poll?subscription_id=...&last_id=...`
- [ ] Parse poll response with signals array
- [ ] Implement configurable poll interval
- [ ] Respect `next_poll_after_ms` from server

### 3.7 Long Polling Transport (`transport/long_polling.rs`)
- [ ] Implement `LongPollingTransport` struct
- [ ] Implement long poll: `GET /cauce/v1/poll?subscription_id=...&wait=true&timeout=...`
- [ ] Handle timeout responses (empty signals, reconnect)
- [ ] Implement immediate reconnect on signal delivery

### 3.8 Webhook Transport - Receive Mode (`transport/webhook.rs`)
- [ ] Implement `WebhookTransport` struct
- [ ] Start embedded HTTP server to receive webhooks
- [ ] Verify webhook signatures (`X-Cauce-Signature`)
- [ ] Parse incoming webhook payloads
- [ ] Send acknowledgment responses
- [ ] Handle replay attacks via `X-Cauce-Timestamp`

### 3.9 Client Core (`client/` module)
- [ ] Implement `CauceClient` struct
  - [ ] Holds transport, config, session state
- [ ] Implement `connect(config: ClientConfig) -> Result<Self>`
  - [ ] Create transport based on config
  - [ ] Perform connection
  - [ ] Send `cauce.hello` handshake
  - [ ] Validate version negotiation
  - [ ] Store session_id
- [ ] Implement `disconnect(&self) -> Result<()>`
  - [ ] Send `cauce.goodbye`
  - [ ] Close transport
- [ ] Implement `subscribe(&self, topics: &[&str]) -> Result<Subscription>`
  - [ ] Send `cauce.subscribe` request
  - [ ] Return `Subscription` handle
- [ ] Implement `unsubscribe(&self, subscription_id: &str) -> Result<()>`
- [ ] Implement `publish(&self, topic: &str, message: impl Into<Message>) -> Result<PublishResult>`
- [ ] Implement `ack(&self, subscription_id: &str, signal_ids: &[&str]) -> Result<AckResult>`
- [ ] Implement internal message routing (responses to pending requests)
- [ ] Implement keepalive ping/pong
- [ ] Add automatic reconnection handling

### 3.10 Subscription Handle (`subscription/` module)
- [ ] Implement `Subscription` struct
  - [ ] `id`: subscription ID
  - [ ] `topics`: subscribed topics
  - [ ] `status`: current status
  - [ ] `signal_rx`: channel receiver for signals
- [ ] Implement `next(&self) -> Option<Signal>` - async wait for next signal
- [ ] Implement `stream(&self) -> impl Stream<Item = Signal>` - async stream
- [ ] Implement `try_next(&self) -> Option<Signal>` - non-blocking
- [ ] Implement `status(&self) -> SubscriptionStatus`

### 3.11 Message Builder Helpers
- [ ] Implement `SignalBuilder` for constructing signals
- [ ] Implement `ActionBuilder` for constructing actions
- [ ] Add convenience methods for common action types

### 3.12 Local Queue for Resilience
- [ ] Implement `LocalQueue` for buffering messages when Hub unavailable
- [ ] Persist queue to disk (optional SQLite or file-based)
- [ ] Implement queue size limits and eviction policy
- [ ] Automatic retry with exponential backoff

### 3.13 Testing for cauce-client-sdk
- [ ] Unit tests for ClientConfig validation
- [ ] Unit tests for each transport's message parsing
- [ ] Mock server tests for WebSocket handshake (using `mockall` or test server)
- [ ] Mock server tests for subscribe/publish/ack flow
- [ ] Mock server tests for SSE event parsing
- [ ] Mock server tests for polling responses
- [ ] Integration tests with actual transport connections (localhost)
- [ ] Test reconnection behavior with exponential backoff
- [ ] Test local queue persistence and recovery
- [ ] Test signature verification for webhooks
- [ ] Test timeout handling

---

## Phase 4: Server SDK (`cauce-server-sdk`)

### 4.1 Project Setup
- [ ] Create `crates/cauce-server-sdk/Cargo.toml`
- [ ] Add dependencies: `cauce-core`, `tokio`, `axum`, `tower`, `tokio-tungstenite`, `futures`, `dashmap`, `tracing`
- [ ] Create module structure: `lib.rs`, `server/`, `transport/`, `subscription/`, `routing/`, `delivery/`, `config/`

### 4.2 Server Configuration (`config/` module)
- [ ] Implement `ServerConfig` struct
  - [ ] `address`: Bind address
  - [ ] `tls`: TLS configuration
  - [ ] `transports`: Enabled transports
  - [ ] `limits`: Rate limits, size limits
  - [ ] `auth`: Authentication configuration
  - [ ] `redelivery`: Redelivery configuration
- [ ] Implement `TransportsConfig` (which transports are enabled)
- [ ] Implement `RedeliveryConfig`
  - [ ] `initial_delay_ms`, `max_delay_ms`, `backoff_multiplier`, `max_attempts`
  - [ ] `dead_letter_topic`

### 4.3 Server Core (`server/` module)
- [ ] Implement `CauceServer` struct
- [ ] Implement `new(config: ServerConfig) -> Self`
- [ ] Implement `router(&self) -> axum::Router`
  - [ ] WebSocket endpoint: `/cauce/v1/ws`
  - [ ] SSE endpoint: `/cauce/v1/sse`
  - [ ] Poll endpoint: `/cauce/v1/poll`
  - [ ] Ack endpoint: `/cauce/v1/ack`
  - [ ] Webhook registration endpoint
- [ ] Implement `with_subscription_manager(self, manager: impl SubscriptionManager) -> Self`
- [ ] Implement `with_message_router(self, router: impl MessageRouter) -> Self`
- [ ] Implement `with_delivery_tracker(self, tracker: impl DeliveryTracker) -> Self`

### 4.4 Subscription Manager Trait (`subscription/` module)
- [ ] Define `SubscriptionManager` trait
  ```rust
  #[async_trait]
  pub trait SubscriptionManager: Send + Sync {
      async fn subscribe(&self, client_id: &str, request: SubscribeRequest) -> Result<SubscribeResponse>;
      async fn unsubscribe(&self, subscription_id: &str) -> Result<()>;
      async fn get_subscription(&self, subscription_id: &str) -> Result<Option<SubscriptionInfo>>;
      async fn get_subscriptions_for_topic(&self, topic: &str) -> Result<Vec<SubscriptionInfo>>;
      async fn list_subscriptions(&self, filter: SubscriptionFilter) -> Result<Vec<SubscriptionInfo>>;
      async fn approve(&self, subscription_id: &str, restrictions: Option<Restrictions>) -> Result<()>;
      async fn deny(&self, subscription_id: &str, reason: Option<String>) -> Result<()>;
      async fn revoke(&self, subscription_id: &str, reason: Option<String>) -> Result<()>;
  }
  ```
- [ ] Implement `InMemorySubscriptionManager` (for testing/simple deployments)
- [ ] Implement topic pattern indexing for efficient matching

### 4.5 Message Router Trait (`routing/` module)
- [ ] Define `MessageRouter` trait
  ```rust
  #[async_trait]
  pub trait MessageRouter: Send + Sync {
      async fn route(&self, topic: &str, message: Message) -> Result<RouteResult>;
      async fn deliver(&self, subscription_id: &str, signal: Signal) -> Result<DeliveryResult>;
  }
  ```
- [ ] Implement `RouteResult` (delivered_to, queued_for)
- [ ] Implement `DeliveryResult` (success/failure)
- [ ] Implement `DefaultMessageRouter`
  - [ ] Query subscription manager for matching subscriptions
  - [ ] Use topic trie for efficient matching
  - [ ] Call deliver for each matching subscription

### 4.6 Delivery Tracker Trait (`delivery/` module)
- [ ] Define `DeliveryTracker` trait
  ```rust
  #[async_trait]
  pub trait DeliveryTracker: Send + Sync {
      async fn track(&self, subscription_id: &str, signal: &Signal) -> Result<()>;
      async fn ack(&self, subscription_id: &str, signal_ids: &[&str]) -> Result<AckResult>;
      async fn get_unacked(&self, subscription_id: &str) -> Result<Vec<Signal>>;
      async fn get_for_redelivery(&self) -> Result<Vec<(String, Signal)>>;
      async fn move_to_dead_letter(&self, subscription_id: &str, signal_id: &str) -> Result<()>;
  }
  ```
- [ ] Implement `InMemoryDeliveryTracker`
- [ ] Implement redelivery scheduling with exponential backoff
- [ ] Track delivery attempts and timestamps

### 4.7 WebSocket Transport Handler (`transport/websocket.rs`)
- [ ] Implement `handle_websocket(ws: WebSocketUpgrade) -> Response`
- [ ] Implement connection state management
- [ ] Implement message dispatch (method → handler)
- [ ] Handle `cauce.hello` handshake
- [ ] Handle `cauce.subscribe`, `cauce.unsubscribe`
- [ ] Handle `cauce.publish`
- [ ] Handle `cauce.ack`
- [ ] Handle `cauce.ping`/`cauce.pong`
- [ ] Handle `cauce.goodbye`
- [ ] Send `cauce.signal` notifications to clients
- [ ] Handle client disconnection (cleanup subscriptions)

### 4.8 SSE Transport Handler (`transport/sse.rs`)
- [ ] Implement `handle_sse(req: Request) -> Response`
- [ ] Validate `subscription_id` query parameter
- [ ] Create SSE stream for subscription
- [ ] Format signals as SSE events
- [ ] Send keepalive events periodically
- [ ] Handle client disconnection

### 4.9 Polling Transport Handler (`transport/polling.rs`)
- [ ] Implement `handle_poll(req: Request) -> Response`
- [ ] Parse `subscription_id`, `last_id`, `wait`, `timeout` parameters
- [ ] Return queued signals for subscription
- [ ] For long polling: hold connection until signals or timeout
- [ ] Include `next_poll_after_ms` hint

### 4.10 Webhook Transport Handler (`transport/webhook.rs`)
- [ ] Implement webhook registration storage
- [ ] Implement `send_webhook(subscription_id: &str, signal: &Signal) -> Result<()>`
- [ ] Generate signature: `HMAC-SHA256(secret, timestamp + "." + body)`
- [ ] Set headers: `X-Cauce-Signature`, `X-Cauce-Timestamp`, `X-Cauce-Delivery-Id`
- [ ] Handle webhook response (2xx = ack, other = retry)
- [ ] Implement retry with exponential backoff

### 4.11 Session Management
- [ ] Implement `SessionManager` trait
- [ ] Track active sessions with client info
- [ ] Session timeout and cleanup
- [ ] Session-to-subscription mapping

### 4.12 Authentication Middleware
- [ ] Implement API key authentication
  - [ ] Check `X-Cauce-API-Key` header
  - [ ] Validate against configured keys
- [ ] Implement Bearer token authentication
  - [ ] Check `Authorization: Bearer` header
  - [ ] Validate token
- [ ] Add authentication to all endpoints

### 4.13 Rate Limiting
- [ ] Implement rate limiter (per client, per subscription)
- [ ] Track request counts in sliding window
- [ ] Return `RateLimited` error with `retry_after_ms`

### 4.14 Testing for cauce-server-sdk
- [ ] Unit tests for subscription matching
- [ ] Unit tests for topic trie
- [ ] Unit tests for redelivery scheduling
- [ ] Integration tests for WebSocket handler
- [ ] Integration tests for SSE handler
- [ ] Integration tests for polling handler
- [ ] Integration tests for webhook delivery
- [ ] Test authentication middleware
- [ ] Test rate limiting
- [ ] End-to-end test: client SDK ↔ server SDK

---

## Phase 5: Reference Hub (`cauce-hub`)

### 5.1 Project Setup
- [ ] Create `crates/cauce-hub/Cargo.toml`
- [ ] Add dependencies: `cauce-core`, `cauce-server-sdk`, `tokio`, `axum`, `sqlx` (SQLite), `toml`, `config`, `tracing`, `tracing-subscriber`, `clap`
- [ ] Create module structure: `main.rs`, `storage/`, `config/`, `api_keys/`

### 5.2 Configuration Loading (`config/` module)
- [ ] Define `HubConfig` struct matching `cauce.toml` format
  - [ ] `hub`: address, database path, TLS
  - [ ] `hub.transports`: enabled transports
  - [ ] `hub.a2a`: A2A config (enabled, public_topics)
  - [ ] `hub.mcp`: MCP config (enabled)
  - [ ] `adapters.*`: adapter-specific configs
- [ ] Implement TOML config file loading
- [ ] Support environment variable overrides
- [ ] Validate configuration on load
- [ ] Create example `config/cauce.example.toml`

### 5.3 SQLite Storage Layer (`storage/` module)
- [ ] Design database schema:
  - [ ] `subscriptions` table (id, client_id, topics, status, transport, webhook_url, e2e_public_key, created_at, expires_at)
  - [ ] `signals_pending` table (id, subscription_id, signal_json, delivery_attempts, next_retry_at, created_at)
  - [ ] `signals_dead_letter` table (id, subscription_id, signal_json, failed_at, reason)
  - [ ] `api_keys` table (id, name, key_hash, created_at, revoked_at)
  - [ ] `sessions` table (id, client_id, transport, created_at, expires_at)
- [ ] Implement SQLite migrations
- [ ] Implement `SqliteSubscriptionManager`
- [ ] Implement `SqliteDeliveryTracker`
- [ ] Implement `SqliteApiKeyStore`
- [ ] Implement `SqliteSessionStore`

### 5.4 API Key Management (`api_keys/` module)
- [ ] Implement `ApiKeyStore` trait
- [ ] Generate cryptographically secure API keys
- [ ] Hash keys for storage (argon2 or bcrypt)
- [ ] Validate keys on authentication
- [ ] Support key revocation

### 5.5 Hub Application (`main.rs`)
- [ ] Parse CLI arguments (config file path, log level)
- [ ] Load configuration
- [ ] Initialize SQLite database
- [ ] Create storage implementations
- [ ] Create `CauceServer` with storage backends
- [ ] Configure TLS if enabled
- [ ] Start HTTP server with graceful shutdown
- [ ] Add signal handlers (SIGTERM, SIGINT)

### 5.6 Health Endpoint
- [ ] Implement `/health` endpoint
- [ ] Return server status, version, uptime
- [ ] Check database connectivity

### 5.7 Metrics (Optional)
- [ ] Add Prometheus metrics endpoint `/metrics`
- [ ] Track: connections, subscriptions, signals published/delivered, errors

### 5.8 Testing for cauce-hub
- [ ] Unit tests for config loading
- [ ] Unit tests for SQLite storage operations
- [ ] Unit tests for API key hashing/validation
- [ ] Integration tests for full Hub (in-memory SQLite)
- [ ] Test subscription persistence across restart
- [ ] Test signal redelivery after restart
- [ ] Test API key authentication flow
- [ ] Load tests for concurrent connections

---

## Phase 6: Hub CLI (`cauce-cli`)

### 6.1 Project Setup
- [ ] Create `crates/cauce-cli/Cargo.toml`
- [ ] Add dependencies: `clap` (derive), `tokio`, `toml`, `serde`, `tabled`, `colored`
- [ ] Create module structure: `main.rs`, `commands/`

### 6.2 Configuration Commands
- [ ] `cauce init` - Create default config file
  - [ ] Interactive prompts for basic settings
  - [ ] Generate example config at `~/.config/cauce/cauce.toml`
- [ ] `cauce config show` - Display current config
  - [ ] Pretty-print TOML
  - [ ] Hide sensitive values (API keys)
- [ ] `cauce config set <key> <value>` - Update config
  - [ ] Support nested keys (e.g., `hub.address`)
  - [ ] Validate before saving

### 6.3 API Key Commands
- [ ] `cauce keys list` - List all API keys
  - [ ] Show: ID, name, created date, status
  - [ ] Table format with colors
- [ ] `cauce keys create <name>` - Generate new API key
  - [ ] Display generated key (only once!)
  - [ ] Store hashed in database
- [ ] `cauce keys revoke <id>` - Revoke an API key
  - [ ] Confirm before revoking
  - [ ] Mark as revoked in database

### 6.4 Subscription Commands
- [ ] `cauce subscriptions list` - List all subscriptions
  - [ ] Show: ID, client, topics, status, transport
  - [ ] Filter by status (--status active/pending/all)
- [ ] `cauce subscriptions pending` - List pending approvals
  - [ ] Show reason for subscription request
- [ ] `cauce subscriptions approve <id>` - Approve subscription
  - [ ] Optional topic restriction (--topics)
  - [ ] Optional expiration (--expires)
- [ ] `cauce subscriptions deny <id>` - Deny subscription
  - [ ] Optional reason (--reason)
- [ ] `cauce subscriptions revoke <id>` - Revoke active subscription
  - [ ] Confirm before revoking

### 6.5 Topic Commands
- [ ] `cauce topics list` - List known topics
  - [ ] Show recent activity per topic
  - [ ] Show subscriber count
- [ ] `cauce topics public` - List public topics
  - [ ] Show A2A-accessible topics

### 6.6 Server Commands
- [ ] `cauce serve` - Start the Hub server
  - [ ] Load config from default or --config path
  - [ ] Start server (delegates to cauce-hub)
- [ ] `cauce status` - Check Hub health
  - [ ] Connect to running Hub
  - [ ] Display health info

### 6.7 Testing for cauce-cli
- [ ] Unit tests for config manipulation
- [ ] Integration tests for each command
- [ ] Test CLI output formatting

---

## Phase 7: CLI Agent (`cauce-agent-cli`)

### 7.1 Project Setup
- [ ] Create `crates/cauce-agent-cli/Cargo.toml`
- [ ] Add dependencies: `cauce-client-sdk`, `clap`, `tokio`, `rustyline`, `colored`, `chrono`
- [ ] Create module structure: `main.rs`, `repl/`, `commands/`, `display/`

### 7.2 Connection Management
- [ ] Parse connection URL from CLI argument
- [ ] Support API key via --api-key or env var
- [ ] Connect using `cauce-client-sdk`
- [ ] Display connection status

### 7.3 Interactive REPL (`repl/` module)
- [ ] Implement REPL loop with rustyline
- [ ] Support command history
- [ ] Support tab completion for commands
- [ ] Handle Ctrl+C gracefully

### 7.4 REPL Commands (`commands/` module)
- [ ] `subscribe <topic>` - Subscribe to topic
  - [ ] Display subscription ID
  - [ ] Support wildcards
- [ ] `unsubscribe <subscription_id>` - Unsubscribe
- [ ] `signals` - List pending signals
  - [ ] Display in formatted table
  - [ ] Show: ID, topic, source, timestamp, preview
- [ ] `show <signal_id>` - Show full signal details
  - [ ] Pretty-print JSON payload
- [ ] `ack <signal_id>` - Acknowledge signal
- [ ] `send <topic> <payload>` - Publish action
  - [ ] Parse JSON payload
  - [ ] Generate action ID
- [ ] `subscriptions` - List active subscriptions
- [ ] `status` - Show connection status
- [ ] `help` - Show available commands
- [ ] `quit` / `exit` - Disconnect and exit

### 7.5 Signal Display (`display/` module)
- [ ] Implement signal list formatter
- [ ] Implement signal detail formatter
- [ ] Color-code by topic/source type
- [ ] Truncate long payloads with "..."

### 7.6 Background Signal Receiver
- [ ] Run async task to receive signals
- [ ] Display incoming signals in REPL
- [ ] Buffer signals for `signals` command

### 7.7 Testing for cauce-agent-cli
- [ ] Unit tests for command parsing
- [ ] Unit tests for display formatting
- [ ] Integration tests with mock server
- [ ] Test REPL command execution

---

## Phase 8: Testing Adapters (Tier 1)

### 8.1 Echo Adapter (`cauce-adapter-echo`)
- [ ] Create `crates/cauce-adapter-echo/Cargo.toml`
- [ ] Implement adapter that mirrors signals back
  - [ ] Subscribe to `signal.**`
  - [ ] For each signal, publish to `signal.echo.<original_topic>`
- [ ] Mirror actions similarly
- [ ] Add configurable delay (for testing async)
- [ ] Add logging of all signals/actions
- [ ] Unit tests for echo logic
- [ ] Integration test with Hub

### 8.2 CLI Adapter (`cauce-adapter-cli`)
- [ ] Create `crates/cauce-adapter-cli/Cargo.toml`
- [ ] Read text lines from stdin
  - [ ] Convert to signals: `signal.cli.input`
- [ ] Subscribe to `action.cli.*`
  - [ ] Write action payloads to stdout
- [ ] Support JSON mode (--json) for structured I/O
- [ ] Unit tests for parsing
- [ ] Integration test with Hub

---

## Phase 9: A2A Integration

### 9.1 A2A Gateway Module
- [ ] Create `crates/cauce-hub/src/a2a/` module
- [ ] Add dependencies: A2A protocol types

### 9.2 Agent Card Endpoint
- [ ] Implement `GET /.well-known/agent.json`
- [ ] Generate Agent Card from config:
  - [ ] name, description, url
  - [ ] protocolVersion
  - [ ] capabilities (streaming, pushNotifications)
  - [ ] skills (send_message, schedule_meeting, subscribe_updates, get_public_topics, get_schemas, get_schema)
  - [ ] securitySchemes

### 9.3 A2A Server Endpoint
- [ ] Implement `POST /a2a` for incoming messages
- [ ] Parse A2A `SendMessage` requests
- [ ] Create A2A Task (SUBMITTED state)
- [ ] Convert to Cauce signal (`signal.a2a.message`)
- [ ] Track task state (WORKING, INPUT_REQUIRED, COMPLETED, FAILED)
- [ ] Return task status updates

### 9.4 Skill Handlers
- [ ] Implement `send_message` skill
  - [ ] Route to appropriate adapter based on channel
- [ ] Implement `schedule_meeting` skill
  - [ ] Create calendar action
- [ ] Implement `subscribe_updates` skill
  - [ ] Validate topics are public
  - [ ] Store push notification URL
- [ ] Implement `get_public_topics` skill
- [ ] Implement `get_schemas` skill
- [ ] Implement `get_schema` skill

### 9.5 Push Notifications
- [ ] Implement push notification sender
- [ ] Format signals as A2A `tasks/pushNotification`
- [ ] Send to registered webhook URLs
- [ ] Handle failures and retries

### 9.6 Multi-Turn Conversations
- [ ] Track conversation context via `contextId`
- [ ] Store conversation state
- [ ] Support `INPUT_REQUIRED` responses
- [ ] Route follow-up messages to same context

### 9.7 Error Mapping
- [ ] Map Cauce errors to A2A errors:
  - [ ] Topic not public → TaskRejectedError
  - [ ] Rate limited → RateLimitExceededError
  - [ ] Invalid skill → InvalidRequestError
  - [ ] Auth required → AuthenticationRequiredError

### 9.8 Testing for A2A Integration
- [ ] Unit tests for Agent Card generation
- [ ] Unit tests for skill handlers
- [ ] Unit tests for error mapping
- [ ] Integration tests for A2A flow
- [ ] Test push notification delivery
- [ ] Test multi-turn conversations

---

## Phase 10: MCP Integration

### 10.1 MCP Server Module
- [ ] Create `crates/cauce-hub/src/mcp/` module
- [ ] Add dependencies: MCP protocol types

### 10.2 MCP Server Endpoint
- [ ] Implement MCP server at `/mcp`
- [ ] Support HTTP transport
- [ ] Support SSE streaming (Streamable HTTP)

### 10.3 MCP Resources
- [ ] `cauce://topics` - List available topics
- [ ] `cauce://subscriptions` - List active subscriptions
- [ ] `cauce://signals/{subscription_id}` - Get pending signals
- [ ] `cauce://schemas` - List payload schemas
- [ ] `cauce://schemas/{schema_id}` - Get schema definition

### 10.4 MCP Tools
- [ ] `subscribe` - Subscribe to topics
  - [ ] Input: topics array
  - [ ] Output: subscription_id, status
- [ ] `unsubscribe` - Remove subscription
  - [ ] Input: subscription_id
- [ ] `publish` - Publish signal/action
  - [ ] Input: topic, payload
  - [ ] Output: message_id
- [ ] `ack` - Acknowledge signals
  - [ ] Input: subscription_id, signal_ids
- [ ] `get_signals` - Poll for signals
  - [ ] Input: subscription_id, limit, since_id

### 10.5 MCP Prompts
- [ ] `send_email` - Email sending template
- [ ] `send_sms` - SMS sending template
- [ ] `check_messages` - Check messages template

### 10.6 SSE Streaming
- [ ] Implement SSE endpoint for MCP
- [ ] Stream signals as SSE events
- [ ] Support `Last-Event-ID` for resumption
- [ ] Send keepalive events

### 10.7 Testing for MCP Integration
- [ ] Unit tests for resource handlers
- [ ] Unit tests for tool handlers
- [ ] Integration tests for MCP flow
- [ ] Test SSE streaming
- [ ] Test against MCP client library

---

## Phase 11: Tier 2 Adapters

### 11.1 RSS Adapter (`cauce-adapter-rss`)
- [ ] Create `crates/cauce-adapter-rss/Cargo.toml`
- [ ] Add dependencies: `feed-rs`, `reqwest`, `chrono`
- [ ] Configuration:
  - [ ] List of feed URLs
  - [ ] Poll interval
  - [ ] Topic prefix
- [ ] Parse RSS/Atom feeds
- [ ] Track seen items (by guid/link)
- [ ] Publish new items as `signal.rss.item`
- [ ] Signal payload:
  - [ ] title, link, description, published_at
  - [ ] author, categories
- [ ] Handle feed errors gracefully
- [ ] Unit tests for feed parsing
- [ ] Integration test with Hub

### 11.2 GitHub Adapter (`cauce-adapter-github`)
- [ ] Create `crates/cauce-adapter-github/Cargo.toml`
- [ ] Add dependencies: `octocrab` or `reqwest`
- [ ] Configuration:
  - [ ] Personal access token
  - [ ] Watched repos
  - [ ] Notification types
- [ ] Poll GitHub notifications API
- [ ] Publish signals:
  - [ ] `signal.github.notification`
  - [ ] `signal.github.issue`
  - [ ] `signal.github.pr`
  - [ ] `signal.github.mention`
- [ ] Subscribe to action topics:
  - [ ] `action.github.comment`
  - [ ] `action.github.create_issue`
- [ ] Webhook receiver (optional)
- [ ] Unit tests for GitHub API parsing
- [ ] Integration test with Hub

### 11.3 Telegram Adapter (`cauce-adapter-telegram`)
- [ ] Create `crates/cauce-adapter-telegram/Cargo.toml`
- [ ] Add dependencies: `teloxide` or `telegram-bot`
- [ ] Configuration:
  - [ ] Bot token
  - [ ] Allowed chat IDs
- [ ] Receive messages via long polling or webhook
- [ ] Publish signals: `signal.telegram.message`
- [ ] Signal payload:
  - [ ] from, chat_id, text, message_id
  - [ ] attachments (photos, documents)
- [ ] Subscribe to: `action.telegram.send`
- [ ] Send messages via Bot API
- [ ] Unit tests for message parsing
- [ ] Integration test with Hub

---

## Phase 12: Tier 3 Adapters

### 12.1 Gmail Adapter (`cauce-adapter-gmail`)
- [ ] Create `crates/cauce-adapter-gmail/Cargo.toml`
- [ ] Add dependencies: OAuth2, IMAP client, SMTP client
- [ ] Configuration:
  - [ ] OAuth2 client ID/secret
  - [ ] Token storage path
- [ ] OAuth2 flow:
  - [ ] Implement authorization URL generation
  - [ ] Handle callback/code exchange
  - [ ] Store and refresh tokens
- [ ] IMAP polling:
  - [ ] Connect with OAuth2 XOAUTH2
  - [ ] Poll for new emails
  - [ ] Parse email headers and body
- [ ] Publish signals: `signal.email.received`
- [ ] Signal payload:
  - [ ] from, to, cc, subject, body_text, body_html
  - [ ] attachments metadata
- [ ] Subscribe to actions:
  - [ ] `action.email.send` - Compose and send
  - [ ] `action.email.reply` - Reply to thread
  - [ ] `action.email.forward` - Forward email
- [ ] Send via SMTP with OAuth2
- [ ] Unit tests for email parsing
- [ ] Integration test with Hub

### 12.2 Fastmail Adapter (`cauce-adapter-fastmail`)
- [ ] Create `crates/cauce-adapter-fastmail/Cargo.toml`
- [ ] Add dependencies: JMAP client
- [ ] Configuration:
  - [ ] API token
  - [ ] Account ID
- [ ] JMAP implementation:
  - [ ] Query for new emails
  - [ ] Parse email objects
- [ ] Publish signals: `signal.email.received`
- [ ] Subscribe to actions: same as Gmail
- [ ] Send via JMAP Email/set
- [ ] Unit tests for JMAP parsing
- [ ] Integration test with Hub

### 12.3 Slack Adapter (`cauce-adapter-slack`)
- [ ] Create `crates/cauce-adapter-slack/Cargo.toml`
- [ ] Add dependencies: Slack SDK or reqwest
- [ ] Configuration:
  - [ ] App token, Bot token
  - [ ] Subscribed channels
- [ ] Events API:
  - [ ] Webhook receiver for events
  - [ ] Or Socket Mode connection
- [ ] Publish signals:
  - [ ] `signal.slack.message`
  - [ ] `signal.slack.mention`
  - [ ] `signal.slack.reaction`
- [ ] Signal payload:
  - [ ] channel, user, text, ts
  - [ ] thread_ts (for threads)
- [ ] Subscribe to actions:
  - [ ] `action.slack.send`
  - [ ] `action.slack.reply`
  - [ ] `action.slack.react`
- [ ] Send via Web API
- [ ] Unit tests for event parsing
- [ ] Integration test with Hub

### 12.4 Discord Adapter (`cauce-adapter-discord`)
- [ ] Create `crates/cauce-adapter-discord/Cargo.toml`
- [ ] Add dependencies: `serenity` or `twilight`
- [ ] Configuration:
  - [ ] Bot token
  - [ ] Subscribed guilds/channels
- [ ] Gateway API connection
- [ ] Publish signals:
  - [ ] `signal.discord.message`
  - [ ] `signal.discord.mention`
  - [ ] `signal.discord.reaction`
- [ ] Signal payload:
  - [ ] channel_id, guild_id, author, content
  - [ ] attachments
- [ ] Subscribe to actions:
  - [ ] `action.discord.send`
  - [ ] `action.discord.reply`
  - [ ] `action.discord.react`
- [ ] Unit tests for message parsing
- [ ] Integration test with Hub

### 12.5 Google Calendar Adapter (`cauce-adapter-gcal`)
- [ ] Create `crates/cauce-adapter-gcal/Cargo.toml`
- [ ] Add dependencies: Google Calendar API client, OAuth2
- [ ] Configuration:
  - [ ] OAuth2 client ID/secret
  - [ ] Calendar IDs
- [ ] OAuth2 flow (similar to Gmail)
- [ ] Poll for events:
  - [ ] New events
  - [ ] Event updates
  - [ ] Reminders
- [ ] Publish signals:
  - [ ] `signal.calendar.event`
  - [ ] `signal.calendar.invite`
  - [ ] `signal.calendar.reminder`
- [ ] Signal payload:
  - [ ] summary, start, end, location
  - [ ] attendees, organizer
- [ ] Subscribe to actions:
  - [ ] `action.calendar.accept`
  - [ ] `action.calendar.decline`
  - [ ] `action.calendar.create`
- [ ] Unit tests for event parsing
- [ ] Integration test with Hub

### 12.6 CalDAV Adapter (`cauce-adapter-caldav`)
- [ ] Create `crates/cauce-adapter-caldav/Cargo.toml`
- [ ] Add dependencies: CalDAV client library
- [ ] Configuration:
  - [ ] CalDAV server URL
  - [ ] Username/password or OAuth2
  - [ ] Calendar paths
- [ ] CalDAV sync:
  - [ ] REPORT for changes
  - [ ] Parse iCalendar format
- [ ] Publish signals: same as Google Calendar
- [ ] Subscribe to actions: same as Google Calendar
- [ ] Create/update events via PUT
- [ ] Unit tests for iCalendar parsing
- [ ] Integration test with Hub

---

## Phase 13: Docker & Distribution

### 13.1 Docker Setup
- [ ] Create `docker/Dockerfile` for Hub
  - [ ] Multi-stage build (builder + runtime)
  - [ ] Minimal runtime image (distroless or alpine)
  - [ ] Include migrations and default config
- [ ] Create `docker/Dockerfile.adapter` template for adapters
- [ ] Create `docker-compose.yml`
  - [ ] Hub service
  - [ ] Adapter services
  - [ ] Volume mounts for config and data
  - [ ] Environment variable support
- [ ] Add health checks to containers
- [ ] Document Docker usage

### 13.2 Binary Releases
- [ ] Create GitHub Actions workflow for releases
- [ ] Build binaries for:
  - [ ] Linux x86_64, aarch64
  - [ ] macOS x86_64, aarch64
  - [ ] Windows x86_64
- [ ] Create release archives with README
- [ ] Publish to GitHub Releases
- [ ] Create installation script

### 13.3 Crates.io Publishing
- [ ] Prepare each crate for publishing:
  - [ ] Complete Cargo.toml metadata
  - [ ] Add license files
  - [ ] Add README.md per crate
- [ ] Publish in order:
  1. `cauce-core`
  2. `cauce-client-sdk`
  3. `cauce-server-sdk`
  4. `cauce-hub`
  5. `cauce-cli`
  6. `cauce-agent-cli`
  7. Adapters
- [ ] Set up automated publishing on release

---

## Phase 14: Documentation & Polish

### 14.1 API Documentation
- [ ] Add rustdoc comments to all public APIs
- [ ] Create examples in doc comments
- [ ] Generate and publish docs to docs.rs

### 14.2 User Documentation
- [ ] Write Getting Started guide
- [ ] Write Hub deployment guide
- [ ] Write adapter development guide
- [ ] Write configuration reference
- [ ] Write troubleshooting guide

### 14.3 Examples
- [ ] Create `examples/` directory
- [ ] Simple agent example
- [ ] Custom adapter example
- [ ] MCP client example
- [ ] A2A integration example

### 14.4 Protocol Compliance Testing
- [ ] Create compliance test suite
- [ ] Test all JSON-RPC methods
- [ ] Test all transport bindings
- [ ] Test error codes and responses
- [ ] Test size limits
- [ ] Test topic validation

### 14.5 Performance Testing
- [ ] Create benchmark suite
- [ ] Benchmark message throughput
- [ ] Benchmark subscription matching
- [ ] Benchmark concurrent connections
- [ ] Document performance characteristics

### 14.6 Security Review
- [ ] Review authentication implementation
- [ ] Review TLS configuration
- [ ] Check for injection vulnerabilities
- [ ] Check for DoS vulnerabilities
- [ ] Document security considerations

---

## Success Criteria Checklist

- [ ] **Protocol Compliance**: All spec methods work correctly (run compliance test suite)
- [ ] **Reliable Delivery**: Signals survive Hub restarts, unacked signals redeliver
- [ ] **Multi-Transport**: Same agent can connect via WebSocket, SSE, Polling, Long Polling, Webhook
- [ ] **A2A Interop**: External A2A agents can discover and interact with Hub
- [ ] **MCP Interop**: MCP clients can subscribe and publish via tools
- [ ] **Personal Use**: Can run on a laptop or VPS for real email/Slack/etc.
- [ ] **Developer Experience**: Easy to build new adapters using client-sdk and examples
- [ ] **Test Coverage**: ≥95% code coverage across all crates
- [ ] **Documentation**: All public APIs documented with rustdoc
- [ ] **CI Green**: All tests pass, no clippy warnings, formatted code

---

## Tech Stack Reference

| Component | Technology |
|-----------|------------|
| Language | Rust (stable) |
| Async runtime | Tokio |
| Web framework | Axum |
| WebSocket | tokio-tungstenite |
| Database | SQLite via rusqlite/sqlx |
| Serialization | serde + serde_json |
| Schema validation | jsonschema |
| CLI parsing | clap |
| Logging | tracing |
| Config | toml + config |
| HTTP client | reqwest |
| Testing | tokio-test, mockall |

---

## Deferred Items

The following items are explicitly out of scope for the initial implementation:

- **JWT Authentication**: Start with API keys; JWT can be added later
- **E2E Encryption**: Protocol supports it, but implementation deferred
- **PostgreSQL**: Start with SQLite; PostgreSQL can be added later
- **WhatsApp Adapter**: Requires Meta Business API approval, complex setup

---

## Notes

- **Process model**: Start with separate adapter binaries; consider Hub plugins later
- **OAuth tokens**: Each adapter stores its own tokens locally (keeps adapters self-contained)
- **Coverage**: 95% coverage is mandatory for all crates

---

## Version History

- **v0.2** - Merged items from previous TODO, added coverage requirement, tech stack table
- **v0.1** - Initial TODO created from SPEC.md and cauce-protocol spec
