# Data Model: Core Types Module

**Feature**: 005-core-types
**Date**: 2026-01-07

## Entity Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                           Signal                                 │
├─────────────────────────────────────────────────────────────────┤
│ id: String (sig_<ts>_<rand>)                                    │
│ version: String                                                  │
│ timestamp: DateTime<Utc>                                        │
│ source: Source ─────────────────────────────────────────────┐   │
│ topic: Topic ───────────────────────────────────────────┐   │   │
│ payload: Payload ───────────────────────────────────┐   │   │   │
│ metadata: Option<Metadata> ─────────────────────┐   │   │   │   │
│ encrypted: Option<Encrypted> ───────────────┐   │   │   │   │   │
└─────────────────────────────────────────────│───│───│───│───│───┘
                                              │   │   │   │   │
┌─────────────────────────────────────────────┴───┴───┴───┴───┴───┐
│                           Action                                 │
├─────────────────────────────────────────────────────────────────┤
│ id: String (act_<ts>_<rand>)                                    │
│ version: String                                                  │
│ timestamp: DateTime<Utc>                                        │
│ topic: Topic                                                     │
│ action: ActionBody ─────────────────────────────────────────┐   │
│ context: Option<ActionContext> ─────────────────────────┐   │   │
│ encrypted: Option<Encrypted>                             │   │   │
└──────────────────────────────────────────────────────────│───│───┘
                                                           │   │
                                                           ▼   ▼
```

## Entities

### Signal

Represents an inbound message from an adapter to the hub.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String | Yes | Unique identifier: `sig_<unix_timestamp>_<random_12>` |
| version | String | Yes | Protocol version (e.g., "1.0") |
| timestamp | DateTime<Utc> | Yes | When the signal was created (ISO 8601) |
| source | Source | Yes | Origin information |
| topic | Topic | Yes | Routing topic |
| payload | Payload | Yes | Message content |
| metadata | Option<Metadata> | No | Threading and priority info |
| encrypted | Option<Encrypted> | No | E2E encryption envelope |

**Validation Rules**:
- `id` MUST match pattern `sig_\d+_[a-zA-Z0-9]{12}`
- `version` MUST be non-empty
- `timestamp` MUST be valid UTC datetime

---

### Action

Represents a command from an agent to be executed by an adapter.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String | Yes | Unique identifier: `act_<unix_timestamp>_<random_12>` |
| version | String | Yes | Protocol version (e.g., "1.0") |
| timestamp | DateTime<Utc> | Yes | When the action was created (ISO 8601) |
| topic | Topic | Yes | Target topic |
| action | ActionBody | Yes | Action details |
| context | Option<ActionContext> | No | Correlation and threading |
| encrypted | Option<Encrypted> | No | E2E encryption envelope |

**Validation Rules**:
- `id` MUST match pattern `act_\d+_[a-zA-Z0-9]{12}`
- `version` MUST be non-empty
- `timestamp` MUST be valid UTC datetime

---

### Topic

Validated hierarchical identifier for pub/sub routing.

| Aspect | Description |
|--------|-------------|
| Internal type | String (newtype wrapper) |
| Length | 1-255 characters |
| Valid chars | `[a-zA-Z0-9._-]` |
| Pattern | No leading/trailing dots, no consecutive dots |

**Examples**:
- Valid: `signal.email.received`, `action.slack.send`, `system.health`
- Invalid: `.leading.dot`, `trailing.dot.`, `double..dots`, `space invalid`

**Validation Rules**:
- Length MUST be 1-255 characters
- Characters MUST be alphanumeric, dot, hyphen, or underscore
- MUST NOT start or end with a dot
- MUST NOT contain consecutive dots

---

### Source

Identifies where a signal originated.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type_ | String | Yes | Adapter type (e.g., "email", "slack", "telegram") |
| adapter_id | String | Yes | Unique adapter instance identifier |
| native_id | String | Yes | Platform-specific message ID |

**JSON field name**: `type` (Rust uses `type_` to avoid keyword conflict, renamed via serde)

---

### Payload

The actual message content with type information.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| raw | serde_json::Value | Yes | Arbitrary JSON content |
| content_type | String | Yes | MIME type (e.g., "application/json", "text/plain") |
| size_bytes | u64 | Yes | Size of raw content in bytes |

**Validation Rules**:
- `size_bytes` MUST accurately reflect serialized size of `raw`
- `content_type` SHOULD be valid MIME type

---

### Metadata

Optional threading and priority information.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| thread_id | Option<String> | No | Conversation thread identifier |
| in_reply_to | Option<String> | No | Signal/Action ID being replied to |
| references | Option<Vec<String>> | No | Related Signal/Action IDs |
| priority | Option<Priority> | No | Message priority (default: Normal) |
| tags | Option<Vec<String>> | No | User-defined labels |

---

### ActionBody

Details of the action to be performed.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| type_ | ActionType | Yes | Kind of action |
| target | Option<String> | No | Target recipient/destination |
| payload | serde_json::Value | Yes | Action-specific data |

**JSON field name**: `type` (Rust uses `type_` to avoid keyword conflict)

---

### ActionContext

Correlation and threading information for actions.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| in_reply_to | Option<String> | No | Signal ID being responded to |
| agent_id | Option<String> | No | Identifier of the agent creating action |
| thread_id | Option<String> | No | Conversation thread identifier |
| correlation_id | Option<String> | No | Request correlation for tracking |

---

### Encrypted

End-to-end encryption envelope.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| algorithm | EncryptionAlgorithm | Yes | Encryption algorithm used |
| recipient_public_key | String | Yes | Base64-encoded recipient public key |
| nonce | String | Yes | Base64-encoded nonce/IV |
| ciphertext | String | Yes | Base64-encoded encrypted payload |

---

## Enums

### Priority

Message priority levels.

| Variant | JSON Value | Description |
|---------|------------|-------------|
| Low | `"low"` | Non-urgent, batch processing OK |
| Normal | `"normal"` | Default priority |
| High | `"high"` | Should be processed soon |
| Urgent | `"urgent"` | Requires immediate attention |

**Default**: `Normal`

---

### ActionType

Types of actions that can be performed.

| Variant | JSON Value | Description |
|---------|------------|-------------|
| Send | `"send"` | Send new message |
| Reply | `"reply"` | Reply to existing message |
| Forward | `"forward"` | Forward message to another recipient |
| React | `"react"` | Add reaction to message |
| Update | `"update"` | Edit existing message |
| Delete | `"delete"` | Delete message |

---

### EncryptionAlgorithm

Supported E2E encryption algorithms.

| Variant | JSON Value | Description |
|---------|------------|-------------|
| X25519XSalsa20Poly1305 | `"x25519_xsalsa20_poly1305"` | NaCl box |
| A256Gcm | `"a256gcm"` | AES-256-GCM |
| XChaCha20Poly1305 | `"xchacha20_poly1305"` | XChaCha20-Poly1305 |

---

## Relationships

```
Signal
├── contains → Source (1:1, required)
├── contains → Topic (1:1, required)
├── contains → Payload (1:1, required)
├── contains → Metadata (1:1, optional)
└── contains → Encrypted (1:1, optional)

Action
├── contains → Topic (1:1, required)
├── contains → ActionBody (1:1, required)
│   └── has → ActionType (1:1, required)
├── contains → ActionContext (1:1, optional)
└── contains → Encrypted (1:1, optional)

Metadata
└── has → Priority (1:1, optional, defaults to Normal)
```

## JSON Schema Alignment

These types implement the following Cauce protocol schemas:
- `signal.schema.json` → Signal
- `action.schema.json` → Action

No schema modifications required; types are implementations of existing schemas.
