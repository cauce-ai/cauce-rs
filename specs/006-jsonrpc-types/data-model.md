# Data Model: JSON-RPC Types

**Feature**: 006-jsonrpc-types
**Date**: 2026-01-07

## Entities

### RequestId

Polymorphic identifier that can be either a string or an integer.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| (variant) | String \| i64 | Yes | Either a string ID or numeric ID |

**Validation Rules**:
- Must be one of: String, Number (integer)
- Null is NOT valid for RequestId (null id handled separately in Response)

**JSON Examples**:
```json
"abc-123"
42
```

---

### JsonRpcRequest

A JSON-RPC 2.0 request message.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| jsonrpc | String | Yes | Must be exactly "2.0" |
| id | RequestId | Yes | Request identifier for correlation |
| method | String | Yes | Name of the method to invoke |
| params | Value | No | Method parameters (any JSON value) |

**Validation Rules**:
- `jsonrpc` MUST equal "2.0" (reject on parse if not)
- `id` MUST be present (distinguishes from Notification)
- `method` MUST be non-empty string

**JSON Example**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "cauce.subscribe",
  "params": {
    "topics": ["signal.email.*"]
  }
}
```

---

### JsonRpcNotification

A JSON-RPC 2.0 notification (request without id, no response expected).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| jsonrpc | String | Yes | Must be exactly "2.0" |
| method | String | Yes | Name of the method to invoke |
| params | Value | No | Method parameters (any JSON value) |

**Validation Rules**:
- `jsonrpc` MUST equal "2.0"
- `id` MUST NOT be present
- `method` MUST be non-empty string

**JSON Example**:
```json
{
  "jsonrpc": "2.0",
  "method": "cauce.signal",
  "params": {
    "topic": "signal.email.received",
    "signal": { ... }
  }
}
```

---

### JsonRpcError

A JSON-RPC 2.0 error object.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| code | i32 | Yes | Error code (integer) |
| message | String | Yes | Human-readable error message |
| data | Value | No | Additional error data (any JSON) |

**Validation Rules**:
- `code` MUST be an integer
- `message` MUST be a string
- Standard error codes: -32700 (Parse), -32600 (Invalid Request), -32601 (Method Not Found), -32602 (Invalid Params), -32603 (Internal Error)
- Server errors: -32000 to -32099 (reserved)
- Application errors: Any other integer

**JSON Example**:
```json
{
  "code": -32601,
  "message": "Method not found",
  "data": {
    "method": "unknown.method"
  }
}
```

---

### JsonRpcResponse

A JSON-RPC 2.0 response message (either success or error).

**Variant: Success**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| jsonrpc | String | Yes | Must be exactly "2.0" |
| id | RequestId | Yes | Matches the request id |
| result | Value | Yes | Method result (any JSON value) |

**Variant: Error**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| jsonrpc | String | Yes | Must be exactly "2.0" |
| id | RequestId \| null | Yes | Matches request id, or null if request id unknown |
| error | JsonRpcError | Yes | Error object |

**Validation Rules**:
- `jsonrpc` MUST equal "2.0"
- Response MUST have EITHER `result` OR `error`, never both, never neither
- `id` MUST match the request (or be null for errors where request id is unknown)

**JSON Examples**:

Success:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "subscription_id": "sub_abc123",
    "status": "active"
  }
}
```

Error:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "field": "topics",
      "reason": "must be non-empty array"
    }
  }
}
```

Error with null id (unidentifiable request):
```json
{
  "jsonrpc": "2.0",
  "id": null,
  "error": {
    "code": -32700,
    "message": "Parse error"
  }
}
```

## Entity Relationships

```
┌─────────────────┐
│  JsonRpcRequest │
├─────────────────┤
│ jsonrpc: "2.0"  │
│ id: RequestId   │──────┐
│ method: String  │      │
│ params?: Value  │      │
└─────────────────┘      │
                         │
┌─────────────────────┐  │    ┌─────────────────┐
│  JsonRpcNotification│  │    │   RequestId     │
├─────────────────────┤  │    ├─────────────────┤
│ jsonrpc: "2.0"      │  ├───▶│ String(String)  │
│ method: String      │  │    │ Number(i64)     │
│ params?: Value      │  │    └─────────────────┘
└─────────────────────┘  │
                         │
┌─────────────────────┐  │    ┌─────────────────┐
│   JsonRpcResponse   │  │    │  JsonRpcError   │
├─────────────────────┤  │    ├─────────────────┤
│ Success:            │  │    │ code: i32       │
│   jsonrpc: "2.0"    │──┘    │ message: String │
│   id: RequestId     │       │ data?: Value    │
│   result: Value     │       └─────────────────┘
├─────────────────────┤              ▲
│ Error:              │              │
│   jsonrpc: "2.0"    │              │
│   id: RequestId?    │──────────────┤
│   error: Error      │──────────────┘
└─────────────────────┘
```

## State Transitions

N/A - These are stateless message types with no lifecycle.

## Invariants

1. **Version Invariant**: All JSON-RPC messages have `jsonrpc == "2.0"`
2. **Request/Notification Distinction**: Requests have `id`, Notifications do not
3. **Response Exclusivity**: Responses have exactly one of `result` or `error`
4. **ID Type Preservation**: RequestId maintains its type (string/number) through serialization
5. **Null ID Restriction**: Null `id` only valid in error responses for unidentifiable requests
