# Quickstart: Core Types Module

**Feature**: 005-core-types
**Date**: 2026-01-07

## Overview

This guide shows how to use the Core Types module from `cauce-core` to create, serialize, and work with Signals and Actions in the Cauce Protocol.

## Prerequisites

Add `cauce-core` to your `Cargo.toml`:

```toml
[dependencies]
cauce-core = "0.1"
```

## Creating a Signal

### Using the Builder Pattern

```rust
use cauce_core::{Signal, Source, Payload, Topic, Priority, Metadata};
use chrono::Utc;
use serde_json::json;

// Create a signal with required fields
let signal = Signal::builder()
    .id("sig_1704067200_abc123def456")
    .version("1.0")
    .timestamp(Utc::now())
    .source(Source {
        type_: "email".to_string(),
        adapter_id: "email-adapter-1".to_string(),
        native_id: "msg-12345".to_string(),
    })
    .topic(Topic::try_from("signal.email.received")?)
    .payload(Payload {
        raw: json!({
            "from": "alice@example.com",
            "subject": "Hello",
            "body": "Hi there!"
        }),
        content_type: "application/json".to_string(),
        size_bytes: 64,
    })
    .build();

// Add optional metadata
let signal_with_metadata = Signal::builder()
    .id("sig_1704067200_xyz789abc012")
    .version("1.0")
    .timestamp(Utc::now())
    .source(Source { /* ... */ })
    .topic(Topic::try_from("signal.email.received")?)
    .payload(Payload { /* ... */ })
    .metadata(Metadata {
        thread_id: Some("thread-123".to_string()),
        priority: Some(Priority::High),
        ..Default::default()
    })
    .build();
```

## Creating an Action

```rust
use cauce_core::{Action, ActionBody, ActionType, ActionContext, Topic};
use chrono::Utc;
use serde_json::json;

let action = Action::builder()
    .id("act_1704067200_abc123def456")
    .version("1.0")
    .timestamp(Utc::now())
    .topic(Topic::try_from("action.email.send")?)
    .action(ActionBody {
        type_: ActionType::Send,
        target: Some("bob@example.com".to_string()),
        payload: json!({
            "subject": "Re: Hello",
            "body": "Thanks for your message!"
        }),
    })
    .context(ActionContext {
        in_reply_to: Some("sig_1704067200_abc123def456".to_string()),
        agent_id: Some("my-agent".to_string()),
        ..Default::default()
    })
    .build();
```

## Working with Topics

```rust
use cauce_core::Topic;

// Valid topics
let topic1 = Topic::try_from("signal.email.received")?;
let topic2 = Topic::try_from("action.slack.send")?;
let topic3 = Topic::try_from("system.health-check")?;

// Invalid topics (will return Err)
let invalid1 = Topic::try_from(".leading.dot");      // Err: leading dot
let invalid2 = Topic::try_from("trailing.dot.");     // Err: trailing dot
let invalid3 = Topic::try_from("double..dots");      // Err: consecutive dots
let invalid4 = Topic::try_from("invalid chars!");    // Err: invalid character

// Convert back to string
let topic_str: &str = topic1.as_str();
```

## Serialization

### Serialize to JSON

```rust
use cauce_core::Signal;
use serde_json;

let signal: Signal = /* ... */;

// Serialize to JSON string
let json_string = serde_json::to_string(&signal)?;

// Serialize to pretty JSON
let json_pretty = serde_json::to_string_pretty(&signal)?;

// Serialize to serde_json::Value
let json_value = serde_json::to_value(&signal)?;
```

### Deserialize from JSON

```rust
use cauce_core::Signal;
use serde_json;

let json_str = r#"{
    "id": "sig_1704067200_abc123def456",
    "version": "1.0",
    "timestamp": "2024-01-01T00:00:00Z",
    "source": {
        "type": "email",
        "adapter_id": "email-adapter-1",
        "native_id": "msg-12345"
    },
    "topic": "signal.email.received",
    "payload": {
        "raw": {"from": "alice@example.com"},
        "content_type": "application/json",
        "size_bytes": 32
    }
}"#;

let signal: Signal = serde_json::from_str(json_str)?;
```

## Enums

### Priority Levels

```rust
use cauce_core::Priority;

let priority = Priority::Normal;  // Default
let high = Priority::High;
let urgent = Priority::Urgent;
let low = Priority::Low;

// Serializes as snake_case: "normal", "high", "urgent", "low"
```

### Action Types

```rust
use cauce_core::ActionType;

let send = ActionType::Send;
let reply = ActionType::Reply;
let forward = ActionType::Forward;
let react = ActionType::React;
let update = ActionType::Update;
let delete = ActionType::Delete;

// Serializes as snake_case: "send", "reply", etc.
```

## Error Handling

```rust
use cauce_core::{Topic, ValidationError};

match Topic::try_from("..invalid") {
    Ok(topic) => println!("Valid topic: {}", topic.as_str()),
    Err(ValidationError::InvalidTopic { reason }) => {
        eprintln!("Invalid topic: {}", reason);
    }
}
```

## JSON Output Examples

### Signal JSON

```json
{
  "id": "sig_1704067200_abc123def456",
  "version": "1.0",
  "timestamp": "2024-01-01T00:00:00Z",
  "source": {
    "type": "email",
    "adapter_id": "email-adapter-1",
    "native_id": "msg-12345"
  },
  "topic": "signal.email.received",
  "payload": {
    "raw": {
      "from": "alice@example.com",
      "subject": "Hello",
      "body": "Hi there!"
    },
    "content_type": "application/json",
    "size_bytes": 64
  },
  "metadata": {
    "priority": "high",
    "thread_id": "thread-123"
  }
}
```

### Action JSON

```json
{
  "id": "act_1704067200_abc123def456",
  "version": "1.0",
  "timestamp": "2024-01-01T00:00:00Z",
  "topic": "action.email.send",
  "action": {
    "type": "send",
    "target": "bob@example.com",
    "payload": {
      "subject": "Re: Hello",
      "body": "Thanks for your message!"
    }
  },
  "context": {
    "in_reply_to": "sig_1704067200_abc123def456",
    "agent_id": "my-agent"
  }
}
```

## Next Steps

- See [data-model.md](./data-model.md) for complete field documentation
- See [spec.md](./spec.md) for functional requirements and acceptance criteria
- Run `cargo doc --open -p cauce-core` for API documentation
