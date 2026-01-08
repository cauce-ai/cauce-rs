//! Integration tests for JSON-RPC types.
//!
//! These tests verify that JSON-RPC types can be used together
//! in realistic scenarios and that all types are properly exported.

use cauce_core::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId, JSONRPC_VERSION,
};
use serde_json::json;

// T065: Write integration test: all types can be imported from crate root
#[test]
fn test_all_jsonrpc_types_importable_from_crate_root() {
    // Verify all types are accessible
    let _id = RequestId::from_number(1);
    let _request = JsonRpcRequest::new(RequestId::from_number(1), "test", None);
    let _response = JsonRpcResponse::success(RequestId::from_number(1), json!({}));
    let _notification = JsonRpcNotification::new("test", None);
    let _error = JsonRpcError::new(-32600, "Invalid Request");

    // Verify constant is accessible
    assert_eq!(JSONRPC_VERSION, "2.0");
}

// T064: Write integration test: request/response roundtrip
#[test]
fn test_request_response_roundtrip() {
    // Create a request
    let request = JsonRpcRequest::new(
        RequestId::from_string("req-001"),
        "cauce.subscribe",
        Some(json!({"topics": ["signal.email.*"]})),
    );

    // Serialize request
    let request_json = serde_json::to_string(&request).unwrap();
    assert!(request_json.contains("\"jsonrpc\":\"2.0\""));
    assert!(request_json.contains("\"id\":\"req-001\""));
    assert!(request_json.contains("\"method\":\"cauce.subscribe\""));

    // Deserialize request
    let parsed_request: JsonRpcRequest = serde_json::from_str(&request_json).unwrap();
    assert_eq!(parsed_request.id(), request.id());
    assert_eq!(parsed_request.method(), request.method());

    // Create matching response
    let response = JsonRpcResponse::success(
        RequestId::from_string("req-001"),
        json!({"subscription_id": "sub_123"}),
    );

    // Verify correlation
    assert_eq!(response.id(), Some(request.id()));
    assert!(response.is_success());

    // Serialize and deserialize response
    let response_json = serde_json::to_string(&response).unwrap();
    let parsed_response: JsonRpcResponse = serde_json::from_str(&response_json).unwrap();
    assert_eq!(parsed_response.id(), response.id());
    assert!(parsed_response.is_success());
}

#[test]
fn test_error_response_flow() {
    // Create a request
    let request = JsonRpcRequest::new(
        RequestId::from_number(42),
        "cauce.invalid_method",
        Some(json!({})),
    );

    // Create an error response
    let error = JsonRpcError::method_not_found();
    let response = JsonRpcResponse::error(Some(request.id().clone()), error);

    // Verify correlation
    assert_eq!(response.id(), Some(request.id()));
    assert!(response.is_error());

    // Verify error details
    let error_obj = response.error_obj().unwrap();
    assert_eq!(error_obj.code, -32601);
    assert_eq!(error_obj.message, "Method not found");
}

#[test]
fn test_notification_has_no_response() {
    // Create a notification
    let notification = JsonRpcNotification::new(
        "cauce.signal",
        Some(json!({"topic": "signal.email.received", "signal": {}})),
    );

    // Serialize notification
    let json = serde_json::to_string(&notification).unwrap();

    // Verify no id field (notifications don't expect responses)
    assert!(!json.contains("\"id\""));
    assert!(json.contains("\"jsonrpc\":\"2.0\""));
    assert!(json.contains("\"method\":\"cauce.signal\""));
}

#[test]
fn test_response_into_result_conversion() {
    // Success response converts to Ok
    let success = JsonRpcResponse::success(RequestId::from_number(1), json!({"data": "test"}));
    let result = success.into_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["data"], "test");

    // Error response converts to Err
    let error_response = JsonRpcResponse::error(
        Some(RequestId::from_number(1)),
        JsonRpcError::invalid_params(),
    );
    let result = error_response.into_result();
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, -32602);
}

#[test]
fn test_parse_error_response_with_null_id() {
    // Parse errors may have null id (couldn't parse request to get id)
    let response = JsonRpcResponse::error(None, JsonRpcError::parse_error());

    assert!(response.is_error());
    assert!(response.id().is_none());

    // Can be serialized and deserialized
    let json = serde_json::to_string(&response).unwrap();
    let parsed: JsonRpcResponse = serde_json::from_str(&json).unwrap();
    assert!(parsed.id().is_none());
}
