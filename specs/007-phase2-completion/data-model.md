# Data Model: Phase 2 Completion - Cauce Core Library

**Date**: 2026-01-08
**Feature**: 007-phase2-completion
**Status**: Complete

## Overview

This document defines the data model for all new types introduced in Phase 2 of cauce-core. Types are organized by module.

---

## 1. Method Parameter Types (`methods/` module)

### 1.1 Authentication Types (`auth.rs`)

```
┌──────────────────────────────────────────────┐
│ AuthType (enum)                              │
├──────────────────────────────────────────────┤
│ Bearer                                       │
│ ApiKey                                       │
│ Mtls                                         │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ Auth (struct)                                │
├──────────────────────────────────────────────┤
│ type_: AuthType                              │
│ token: Option<String>                        │
│ api_key: Option<String>                      │
└──────────────────────────────────────────────┘
```

**Validation Rules**:
- If `type_ = Bearer`, `token` MUST be Some
- If `type_ = ApiKey`, `api_key` MUST be Some
- If `type_ = Mtls`, both MAY be None (cert-based)

### 1.2 Client Types (`client.rs`)

```
┌──────────────────────────────────────────────┐
│ ClientType (enum)                            │
├──────────────────────────────────────────────┤
│ Adapter                                      │
│ Agent                                        │
│ A2aAgent                                     │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ Capability (enum)                            │
├──────────────────────────────────────────────┤
│ Subscribe                                    │
│ Publish                                      │
│ Ack                                          │
│ E2eEncryption                                │
└──────────────────────────────────────────────┘
```

### 1.3 Transport Types (`transport.rs`)

```
┌──────────────────────────────────────────────┐
│ Transport (enum)                             │
├──────────────────────────────────────────────┤
│ WebSocket                                    │
│ Sse                                          │
│ Polling                                      │
│ LongPolling                                  │
│ Webhook                                      │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ WebhookConfig (struct)                       │
├──────────────────────────────────────────────┤
│ url: String                                  │
│ secret: Option<String>                       │
│ headers: Option<HashMap<String, String>>     │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ E2eConfig (struct)                           │
├──────────────────────────────────────────────┤
│ enabled: bool                                │
│ public_key: Option<String>                   │
│ supported_algorithms: Vec<EncryptionAlgorithm>│
└──────────────────────────────────────────────┘
```

### 1.4 Subscription Enums (`enums.rs`)

```
┌──────────────────────────────────────────────┐
│ ApprovalType (enum)                          │
├──────────────────────────────────────────────┤
│ Automatic                                    │
│ UserApproved                                 │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionStatus (enum)                    │
├──────────────────────────────────────────────┤
│ Pending                                      │
│ Active                                       │
│ Denied                                       │
│ Revoked                                      │
│ Expired                                      │
└──────────────────────────────────────────────┘
```

### 1.5 Hello Method (`hello.rs`)

```
┌──────────────────────────────────────────────┐
│ HelloRequest (struct)                        │
├──────────────────────────────────────────────┤
│ protocol_version: String                     │
│ min_protocol_version: Option<String>         │
│ max_protocol_version: Option<String>         │
│ client_id: String                            │
│ client_type: ClientType                      │
│ capabilities: Vec<Capability>                │
│ auth: Option<Auth>                           │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ HelloResponse (struct)                       │
├──────────────────────────────────────────────┤
│ session_id: String                           │
│ server_version: String                       │
│ capabilities: Vec<Capability>                │
│ session_expires_at: Option<DateTime<Utc>>    │
└──────────────────────────────────────────────┘
```

### 1.6 Subscribe Method (`subscribe.rs`)

```
┌──────────────────────────────────────────────┐
│ SubscribeRequest (struct)                    │
├──────────────────────────────────────────────┤
│ topics: Vec<String>                          │
│ approval_type: Option<ApprovalType>          │
│ reason: Option<String>                       │
│ transport: Option<Transport>                 │
│ webhook: Option<WebhookConfig>               │
│ e2e: Option<E2eConfig>                       │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscribeResponse (struct)                   │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
│ status: SubscriptionStatus                   │
│ topics: Vec<String>                          │
│ created_at: DateTime<Utc>                    │
│ expires_at: Option<DateTime<Utc>>            │
└──────────────────────────────────────────────┘
```

### 1.7 Unsubscribe Method (`unsubscribe.rs`)

```
┌──────────────────────────────────────────────┐
│ UnsubscribeRequest (struct)                  │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ UnsubscribeResponse (struct)                 │
├──────────────────────────────────────────────┤
│ success: bool                                │
└──────────────────────────────────────────────┘
```

### 1.8 Publish Method (`publish.rs`)

```
┌──────────────────────────────────────────────┐
│ PublishMessage (enum, untagged)              │
├──────────────────────────────────────────────┤
│ Signal(Signal)                               │
│ Action(Action)                               │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ PublishRequest (struct)                      │
├──────────────────────────────────────────────┤
│ topic: String                                │
│ message: PublishMessage                      │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ PublishResponse (struct)                     │
├──────────────────────────────────────────────┤
│ message_id: String                           │
│ delivered_to: u32                            │
│ queued_for: u32                              │
└──────────────────────────────────────────────┘
```

### 1.9 Ack Method (`ack.rs`)

```
┌──────────────────────────────────────────────┐
│ AckRequest (struct)                          │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
│ signal_ids: Vec<String>                      │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ AckResponse (struct)                         │
├──────────────────────────────────────────────┤
│ acknowledged: Vec<String>                    │
│ failed: Vec<AckFailure>                      │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ AckFailure (struct)                          │
├──────────────────────────────────────────────┤
│ signal_id: String                            │
│ reason: String                               │
└──────────────────────────────────────────────┘
```

### 1.10 Subscription Management (`subscription.rs`)

```
┌──────────────────────────────────────────────┐
│ SubscriptionApproveRequest (struct)          │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
│ restrictions: Option<SubscriptionRestrictions>│
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionRestrictions (struct)            │
├──────────────────────────────────────────────┤
│ allowed_topics: Option<Vec<String>>          │
│ expires_at: Option<DateTime<Utc>>            │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionDenyRequest (struct)             │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
│ reason: Option<String>                       │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionRevokeRequest (struct)           │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
│ reason: Option<String>                       │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionListRequest (struct)             │
├──────────────────────────────────────────────┤
│ status: Option<SubscriptionStatus>           │
│ client_id: Option<String>                    │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionListResponse (struct)            │
├──────────────────────────────────────────────┤
│ subscriptions: Vec<SubscriptionInfo>         │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionInfo (struct)                    │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
│ client_id: String                            │
│ topics: Vec<String>                          │
│ status: SubscriptionStatus                   │
│ transport: Transport                         │
│ created_at: DateTime<Utc>                    │
│ expires_at: Option<DateTime<Utc>>            │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SubscriptionStatusNotification (struct)      │
├──────────────────────────────────────────────┤
│ subscription_id: String                      │
│ status: SubscriptionStatus                   │
│ reason: Option<String>                       │
└──────────────────────────────────────────────┘
```

### 1.11 Ping/Pong (`ping.rs`)

```
┌──────────────────────────────────────────────┐
│ PingParams (struct)                          │
├──────────────────────────────────────────────┤
│ timestamp: DateTime<Utc>                     │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ PongParams (struct)                          │
├──────────────────────────────────────────────┤
│ timestamp: DateTime<Utc>                     │
└──────────────────────────────────────────────┘
```

### 1.12 Signal Delivery (`signal_delivery.rs`)

```
┌──────────────────────────────────────────────┐
│ SignalDelivery (struct)                      │
├──────────────────────────────────────────────┤
│ topic: String                                │
│ signal: Signal                               │
└──────────────────────────────────────────────┘
```

### 1.13 Schema Methods (`schemas.rs`)

```
┌──────────────────────────────────────────────┐
│ SchemasListRequest (struct)                  │
├──────────────────────────────────────────────┤
│ (no fields - empty params)                   │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SchemasListResponse (struct)                 │
├──────────────────────────────────────────────┤
│ schemas: Vec<SchemaInfo>                     │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SchemaInfo (struct)                          │
├──────────────────────────────────────────────┤
│ id: String                                   │
│ name: String                                 │
│ version: String                              │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SchemasGetRequest (struct)                   │
├──────────────────────────────────────────────┤
│ schema_id: String                            │
└──────────────────────────────────────────────┘

┌──────────────────────────────────────────────┐
│ SchemasGetResponse (struct)                  │
├──────────────────────────────────────────────┤
│ schema: serde_json::Value                    │
└──────────────────────────────────────────────┘
```

---

## 2. Protocol Errors (`errors/protocol.rs`)

```
┌──────────────────────────────────────────────┐
│ CauceError (enum)                            │
├──────────────────────────────────────────────┤
│ // JSON-RPC Standard Errors                  │
│ ParseError { message: String }         -32700│
│ InvalidRequest { message: String }     -32600│
│ MethodNotFound { method: String }      -32601│
│ InvalidParams { message: String }      -32602│
│ InternalError { message: String }      -32603│
│                                              │
│ // Cauce Protocol Errors                     │
│ SubscriptionNotFound { id: String }    -32001│
│ TopicNotFound { topic: String }        -32002│
│ NotAuthorized { reason: String }       -32003│
│ SubscriptionPending { id: String }     -32004│
│ SubscriptionDenied { id, reason }      -32005│
│ RateLimited { retry_after_ms: u64 }    -32006│
│ SignalTooLarge { size, max }           -32007│
│ EncryptionRequired { topic: String }   -32008│
│ InvalidEncryption { reason: String }   -32009│
│ AdapterUnavailable { adapter: String } -32010│
│ DeliveryFailed { signal_id, reason }   -32011│
│ QueueFull { capacity: usize }          -32012│
│ SessionExpired { session_id: String }  -32013│
│ UnsupportedTransport { transport }     -32014│
│ InvalidTopic { topic, reason }         -32015│
└──────────────────────────────────────────────┘
```

**Relationships**:
- Implements `From<CauceError> for JsonRpcError`
- Each variant maps to specific error code and message

---

## 3. Constants (`constants/mod.rs`)

### Method Names

| Constant | Value |
|----------|-------|
| `METHOD_HELLO` | `"cauce.hello"` |
| `METHOD_GOODBYE` | `"cauce.goodbye"` |
| `METHOD_PING` | `"cauce.ping"` |
| `METHOD_PONG` | `"cauce.pong"` |
| `METHOD_PUBLISH` | `"cauce.publish"` |
| `METHOD_SUBSCRIBE` | `"cauce.subscribe"` |
| `METHOD_UNSUBSCRIBE` | `"cauce.unsubscribe"` |
| `METHOD_SIGNAL` | `"cauce.signal"` |
| `METHOD_ACK` | `"cauce.ack"` |
| `METHOD_SUBSCRIPTION_REQUEST` | `"cauce.subscription.request"` |
| `METHOD_SUBSCRIPTION_APPROVE` | `"cauce.subscription.approve"` |
| `METHOD_SUBSCRIPTION_DENY` | `"cauce.subscription.deny"` |
| `METHOD_SUBSCRIPTION_LIST` | `"cauce.subscription.list"` |
| `METHOD_SUBSCRIPTION_REVOKE` | `"cauce.subscription.revoke"` |
| `METHOD_SUBSCRIPTION_STATUS` | `"cauce.subscription.status"` |
| `METHOD_SCHEMAS_LIST` | `"cauce.schemas.list"` |
| `METHOD_SCHEMAS_GET` | `"cauce.schemas.get"` |

### Size Limits

| Constant | Value |
|----------|-------|
| `MAX_TOPIC_LENGTH` | `255` (alias for TOPIC_MAX_LENGTH) |
| `MAX_SIGNAL_PAYLOAD_SIZE` | `10 * 1024 * 1024` (10 MB) |
| `MAX_TOPICS_PER_SUBSCRIPTION` | `100` |
| `MAX_SUBSCRIPTIONS_PER_CLIENT` | `1000` |
| `MAX_SIGNALS_PER_BATCH` | `100` |
| `MAX_TOPIC_DEPTH` | `10` (max segments in topic) |

---

## 4. ID Generators (`id/mod.rs`)

| Function | Format | Example |
|----------|--------|---------|
| `generate_signal_id()` | `sig_{timestamp}_{random12}` | `sig_1704672000_a1b2c3d4e5f6` |
| `generate_action_id()` | `act_{timestamp}_{random12}` | `act_1704672000_a1b2c3d4e5f6` |
| `generate_subscription_id()` | `sub_{uuid}` | `sub_550e8400-e29b-41d4-a716-446655440000` |
| `generate_session_id()` | `sess_{uuid}` | `sess_550e8400-e29b-41d4-a716-446655440000` |
| `generate_message_id()` | `msg_{uuid}` | `msg_550e8400-e29b-41d4-a716-446655440000` |

---

## 5. Topic Matching (`matching/mod.rs`)

```
┌──────────────────────────────────────────────┐
│ TopicMatcher (struct)                        │
├──────────────────────────────────────────────┤
│ matches(topic: &str, pattern: &str) -> bool  │
│ matches_any(topic: &str, patterns: &[&str]) -> bool│
└──────────────────────────────────────────────┘
```

**Wildcard Rules**:
- `*` - matches exactly one segment
- `**` - matches one or more segments

**Examples**:
| Topic | Pattern | Matches |
|-------|---------|---------|
| `signal.email` | `signal.*` | ✓ |
| `signal.email.received` | `signal.*` | ✗ |
| `signal.email.received` | `signal.**` | ✓ |
| `signal.email.received` | `**.received` | ✓ |
| `signal.email` | `signal.email` | ✓ (exact) |

---

## 6. Enhanced Validation (`validation/`)

### New Validation Functions

| Function | Purpose |
|----------|---------|
| `validate_signal(value: &Value)` | Validate JSON against signal schema |
| `validate_action(value: &Value)` | Validate JSON against action schema |
| `validate_topic_pattern(pattern: &str)` | Validate topic with wildcards |
| `is_valid_subscription_id(id: &str)` | Validate `sub_` prefix + UUID |
| `is_valid_session_id(id: &str)` | Validate `sess_` prefix + UUID |

### ID Patterns

| ID Type | Pattern |
|---------|---------|
| Signal ID | `^sig_\d+_[a-zA-Z0-9]{12}$` |
| Action ID | `^act_\d+_[a-zA-Z0-9]{12}$` |
| Subscription ID | `^sub_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$` |
| Session ID | `^sess_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$` |
| Message ID | `^msg_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$` |

---

## Entity Relationships

```
HelloRequest ──uses──> Auth, ClientType, Capability
HelloResponse ──uses──> Capability

SubscribeRequest ──uses──> ApprovalType, Transport, WebhookConfig, E2eConfig
SubscribeResponse ──uses──> SubscriptionStatus

PublishRequest ──uses──> PublishMessage(Signal | Action)

AckResponse ──uses──> AckFailure

SubscriptionInfo ──uses──> SubscriptionStatus, Transport

E2eConfig ──uses──> EncryptionAlgorithm (existing)

CauceError ──converts_to──> JsonRpcError (existing)
```

---

## Summary

| Category | Count |
|----------|-------|
| New Enums | 6 (AuthType, ClientType, Capability, Transport, ApprovalType, SubscriptionStatus) |
| New Structs | 28 (all method request/response types) |
| New Error Variants | 20 (in CauceError) |
| New Constants | 24 (17 methods + 6 limits + 1 alias) |
| New Functions | 10 (5 ID generators + 5 validators) |
