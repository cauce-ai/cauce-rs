# Cauce Hub Reference Implementation (Rust)

**Status**: Planning
**Target**: Full reference implementation for personal use, protocol validation, and developer demos

## Overview

A complete Rust implementation of the Cauce Protocol, including:

- **Hub**: Central pub/sub broker with all protocol features
- **Adapters**: 12 adapters covering email, messaging, notifications, and calendars
- **CLI Agent**: Interactive agent for testing and manual signal management
- **Hub CLI**: Management tool for configuration, keys, and subscription approval

## Architecture

```
cauce-rs/
├── crates/
│   │
│   │ # Core libraries
│   ├── cauce-core/             # Shared: types, schemas, JSON-RPC, validation
│   ├── cauce-client-sdk/       # SDK for building adapters/agents
│   ├── cauce-server-sdk/       # SDK for building Hub implementations
│   │
│   │ # Reference implementation
│   ├── cauce-hub/              # Reference Hub server (uses server-sdk)
│   ├── cauce-cli/              # Hub management CLI
│   ├── cauce-agent-cli/        # Interactive CLI agent (uses client-sdk)
│   │
│   │ # Adapters (all use client-sdk)
│   ├── cauce-adapter-echo/     # Echo adapter (testing)
│   ├── cauce-adapter-cli/      # CLI adapter (stdin/stdout)
│   ├── cauce-adapter-rss/      # RSS feed adapter
│   ├── cauce-adapter-telegram/ # Telegram Bot API
│   ├── cauce-adapter-gmail/    # Gmail (IMAP + OAuth)
│
├── docker/
│   ├── Dockerfile
│   └── docker-compose.yml
│
└── config/
    └── cauce.example.toml
```

## SDK Architecture

### cauce-core

Shared types and utilities used by both SDKs:

```rust
// Types
pub struct Signal { id, version, timestamp, source, topic, payload, metadata }
pub struct Action { id, version, timestamp, topic, action, context }
pub struct Source { type_, adapter_id, native_id }
pub struct Payload { raw, content_type, size_bytes }

// JSON-RPC
pub struct JsonRpcRequest { jsonrpc, id, method, params }
pub struct JsonRpcResponse { jsonrpc, id, result, error }
pub struct JsonRpcError { code, message, data }

// Validation
pub fn validate_signal(value: &Value) -> Result<Signal, ValidationError>
pub fn validate_action(value: &Value) -> Result<Action, ValidationError>
pub fn validate_topic(topic: &str) -> Result<(), ValidationError>

// Constants
pub mod errors { pub const SUBSCRIPTION_NOT_FOUND: i32 = -32001; ... }
pub mod methods { pub const SUBSCRIBE: &str = "cauce.subscribe"; ... }
```

### cauce-client-sdk

For building adapters and agents:

```rust
// Connection
pub struct CauceClient { ... }
impl CauceClient {
    pub async fn connect(config: ClientConfig) -> Result<Self>
    pub async fn disconnect(&self) -> Result<()>
}

// Pub/Sub
impl CauceClient {
    pub async fn subscribe(&self, topics: &[&str]) -> Result<Subscription>
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()>
    pub async fn publish(&self, topic: &str, message: impl Into<Message>) -> Result<()>
    pub async fn ack(&self, subscription_id: &str, signal_ids: &[&str]) -> Result<()>
}

// Receiving signals
impl Subscription {
    pub async fn next(&self) -> Option<Signal>
    pub fn stream(&self) -> impl Stream<Item = Signal>
}

// Transports (client-side)
pub enum Transport { WebSocket, SSE, Polling, LongPolling, Webhook }
```

### cauce-server-sdk

For building Hub implementations:

```rust
// Server
pub struct CauceServer { ... }
impl CauceServer {
    pub fn new(config: ServerConfig) -> Self
    pub fn router(&self) -> axum::Router  // Or generic over framework
}

// Subscription management
pub trait SubscriptionManager {
    fn subscribe(&self, client_id: &str, topics: &[&str]) -> Result<Subscription>
    fn unsubscribe(&self, subscription_id: &str) -> Result<()>
    fn get_subscribers(&self, topic: &str) -> Vec<Subscription>
}

// Message routing
pub trait MessageRouter {
    fn route(&self, message: Message) -> Result<RouteResult>
    fn deliver(&self, subscription_id: &str, signal: Signal) -> Result<()>
}

// Delivery tracking
pub trait DeliveryTracker {
    fn track(&self, subscription_id: &str, signal_id: &str) -> Result<()>
    fn ack(&self, subscription_id: &str, signal_ids: &[&str]) -> Result<()>
    fn get_unacked(&self, subscription_id: &str) -> Vec<Signal>
    fn get_for_redelivery(&self) -> Vec<(String, Signal)>
}

// Transports (server-side)
pub trait TransportHandler {
    fn handle_websocket(&self, socket: WebSocket) -> impl Future
    fn handle_sse(&self, req: Request) -> impl Future<Output = Response>
    fn handle_poll(&self, req: Request) -> impl Future<Output = Response>
}
```

## Hub Features

### Core Protocol

| Feature | Status | Notes |
|---------|--------|-------|
| JSON-RPC 2.0 wire protocol | Required | All methods per spec |
| Pub/Sub routing | Required | Topic matching with wildcards |
| Subscription management | Required | Auto + user-approved |
| Signal acknowledgment | Required | At-least-once delivery |
| Redelivery with backoff | Required | Exponential backoff, dead-letter queue |
| Version negotiation | Required | Semver, handshake validation |

### Transports

| Transport | Status | Notes |
|-----------|--------|-------|
| WebSocket | Required | Full-duplex, real-time |
| SSE | Required | Unidirectional streaming |
| HTTP Polling | Required | Simple request/response |
| Long Polling | Required | Held connection until signals |
| Webhooks | Required | Server-to-server push |

### Integrations

| Integration | Status | Notes |
|-------------|--------|-------|
| A2A Gateway | Required | Agent Card, SendMessage, push notifications |
| MCP Interface | Required | Resources, Tools, Prompts per spec Section 10 |

### Security

| Feature | Status | Notes |
|---------|--------|-------|
| TLS | Required | TLS 1.2+ for all connections |
| API Key auth | Required | Static keys in config |
| JWT auth | Deferred | Future enhancement |
| E2E encryption | Deferred | Future enhancement |

### Storage

| Backend | Status | Notes |
|---------|--------|-------|
| SQLite | Required | Embedded, single-file database |
| PostgreSQL | Deferred | Future enhancement |

## Hub CLI (`cauce-cli`)

Management commands for the Hub:

```bash
# Configuration
cauce init                    # Create config file
cauce config show             # Display current config
cauce config set <key> <val>  # Update config

# API Keys
cauce keys list               # List all API keys
cauce keys create <name>      # Generate new API key
cauce keys revoke <id>        # Revoke an API key

# Subscriptions
cauce subscriptions list      # List all subscriptions
cauce subscriptions pending   # List pending approvals
cauce subscriptions approve <id>  # Approve subscription
cauce subscriptions deny <id>     # Deny subscription
cauce subscriptions revoke <id>   # Revoke active subscription

# Topics
cauce topics list             # List known topics
cauce topics public           # List public topics (A2A accessible)

# Server
cauce serve                   # Start the Hub server
cauce status                  # Check Hub health
```

## CLI Agent (`cauce-agent-cli`)

Interactive agent for testing and manual operation:

```bash
cauce-agent connect wss://localhost:8080/cauce/v1/ws

# In interactive mode:
> subscribe signal.email.*
Subscribed to signal.email.* (sub_abc123)

> signals
[sig_123] signal.email.received from=alice@example.com subject="Meeting"
[sig_456] signal.slack.message channel=#general "Hey team"

> ack sig_123
Acknowledged sig_123

> send action.email.send to=alice@example.com subject="Re: Meeting" body="Works for me!"
Published act_789

> quit
```

## Adapters

### Tier 1: Testing (Ship First)

| Adapter | Protocol | Signals | Actions |
|---------|----------|---------|---------|
| **Echo** | Internal | Mirrors any signal back | Mirrors any action |
| **CLI** | stdin/stdout | Text lines → signals | Actions → stdout |

### Tier 2: Easy (Good APIs)

| Adapter | Protocol | Signals | Actions |
|---------|----------|---------|---------|
| **RSS** | HTTP polling | Feed items → `signal.rss.item` | N/A (read-only) |
| **GitHub** | REST + Webhooks | Notifications → `signal.github.*` | Create issues, comments |
| **Telegram** | Bot API | Messages → `signal.telegram.message` | Send messages |

### Tier 3: Medium (OAuth Required)

| Adapter | Protocol | Signals | Actions |
|---------|----------|---------|---------|
| **Gmail** | IMAP + OAuth | Emails → `signal.email.received` | Send, reply, forward |
| **Fastmail** | JMAP | Emails → `signal.email.received` | Send, reply, forward |
| **Slack** | Events API | Messages → `signal.slack.message` | Send, react |
| **Discord** | Gateway API | Messages → `signal.discord.message` | Send, react |
| **Google Calendar** | REST + OAuth | Events → `signal.calendar.*` | Accept, decline, create |
| **CalDAV** | CalDAV | Events → `signal.calendar.*` | Accept, decline, create |

### Deferred

| Adapter | Protocol | Notes |
|---------|----------|-------|
| **WhatsApp** | Business API | Requires Meta approval. Revisit post-MVP. |

## Configuration

```toml
# cauce.toml

[hub]
address = "0.0.0.0:8080"
database = "cauce.db"

[hub.tls]
cert = "/path/to/cert.pem"
key = "/path/to/key.pem"

[hub.transports]
websocket = true
sse = true
polling = true
long_polling = true
webhooks = true

[hub.a2a]
enabled = true
public_topics = ["signal.blog.*", "signal.status.*"]

[hub.mcp]
enabled = true

# Adapter configs
[adapters.gmail]
enabled = true
client_id = "..."
client_secret = "..."
poll_interval = "60s"

[adapters.slack]
enabled = true
app_token = "xapp-..."
bot_token = "xoxb-..."

# ... more adapters
```

## Distribution

### Binary Releases

```bash
# Install from crates.io
cargo install cauce-hub
cargo install cauce-cli
cargo install cauce-agent-cli

# Or download from GitHub releases
curl -L https://github.com/.../releases/latest/download/cauce-hub-x86_64-linux -o cauce-hub
```

### Docker

```bash
# Run Hub
docker run -p 8080:8080 -v ./config:/config cauce/hub

# Docker Compose (Hub + adapters)
docker-compose up
```

```yaml
# docker-compose.yml
services:
  hub:
    image: cauce/hub
    ports:
      - "8080:8080"
    volumes:
      - ./config:/config
      - ./data:/data

  gmail-adapter:
    image: cauce/cauce-adapter-gmail
    environment:
      CAUCE_HUB_URL: ws://hub:8080
      CAUCE_API_KEY: ${GMAIL_ADAPTER_KEY}
    depends_on:
      - hub

  slack-adapter:
    image: cauce/cauce-adapter-slack
    environment:
      CAUCE_HUB_URL: ws://hub:8080
      CAUCE_API_KEY: ${SLACK_ADAPTER_KEY}
    depends_on:
      - hub
```

## Development Phases

### Phase 1: Core Foundation

1. Project setup (Cargo workspace, CI)
2. `cauce-core`: types, JSON-RPC, schema validation
3. `cauce-core`: error codes, method constants

### Phase 2: Client SDK

1. `cauce-client-sdk`: WebSocket transport (client-side)
2. `cauce-client-sdk`: connection, handshake, auth
3. `cauce-client-sdk`: subscribe, publish, ack methods
4. `cauce-client-sdk`: SSE, Polling, Long Polling transports
5. `cauce-client-sdk`: Webhook transport (receive mode)

### Phase 3: Server SDK

1. `cauce-server-sdk`: WebSocket transport (server-side)
2. `cauce-server-sdk`: subscription manager
3. `cauce-server-sdk`: message router
4. `cauce-server-sdk`: delivery tracker
5. `cauce-server-sdk`: SSE, Polling, Long Polling, Webhook transports

### Phase 4: Reference Hub

1. `cauce-hub`: SQLite storage layer
2. `cauce-hub`: wire up server-sdk components
3. `cauce-hub`: configuration loading
4. `cauce-hub`: API key authentication

### Phase 5: Testing Infrastructure

1. `cauce-adapter-echo`: echo adapter
2. `cauce-adapter-cli`: CLI adapter
3. `cauce-agent-cli`: interactive CLI agent
4. `cauce-cli`: Hub management CLI (basic commands)

### Phase 6: Integrations

1. A2A Gateway (Agent Card, SendMessage, push)
2. MCP Interface (Resources, Tools, Prompts)

### Phase 7: Real Adapters (Tier 2)

1. `cauce-adapter-rss`: RSS adapter
2. `cauce-adapter-github`: GitHub adapter
3. `cauce-adapter-telegram`: Telegram adapter

### Phase 8: Real Adapters (Tier 3)

1. `cauce-adapter-gmail`: Gmail adapter
2. `cauce-adapter-fastmail`: Fastmail adapter
3. `cauce-adapter-slack`: Slack adapter
4. `cauce-adapter-discord`: Discord adapter
5. `cauce-adapter-gcal`: Google Calendar adapter
6. `cauce-adapter-caldav`: CalDAV adapter

### Phase 9: Polish

1. `cauce-cli`: full commands
2. Docker images
3. Documentation
4. Publish crates to crates.io

## Tech Stack

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

## Success Criteria

1. **Protocol Compliance**: All spec methods work correctly
2. **Reliable Delivery**: Signals survive Hub restarts, unacked signals redeliver
3. **Multi-Transport**: Same agent can connect via any transport
4. **A2A Interop**: External A2A agents can discover and interact with Hub
5. **MCP Interop**: MCP clients can subscribe and publish via tools
6. **Personal Use**: Can run on a laptop or VPS for real email/Slack/etc.
7. **Developer Experience**: Easy to build new adapters using examples

## Design Decisions

1. **Adapter process model**: Both separate binaries and Hub plugins are valid.
   The protocol is agnostic to deployment model. Start with separate binaries
   for simplicity; plugins can be added later if beneficial.

2. **OAuth token storage**: Each adapter stores its own tokens in a local file.
   Keeps adapters self-contained and simplifies the Hub.

3. **WhatsApp**: Deferred. Business API requires Meta approval which is complex.
   Can be revisited post-MVP via Matrix bridge or official API.
