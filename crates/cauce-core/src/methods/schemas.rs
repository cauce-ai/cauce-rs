//! Schema discovery method types for the Cauce Protocol.
//!
//! Used to list and retrieve JSON schemas for protocol messages.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Request parameters for the `cauce.schemas.list` method.
///
/// This is an empty request - no parameters needed.
///
/// # Example
///
/// ```
/// use cauce_core::methods::SchemasListRequest;
///
/// let request = SchemasListRequest {};
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SchemasListRequest {}

impl SchemasListRequest {
    /// Creates a new SchemasListRequest.
    pub fn new() -> Self {
        Self {}
    }
}

/// Response from the `cauce.schemas.list` method.
///
/// # Example
///
/// ```
/// use cauce_core::methods::{SchemasListResponse, SchemaInfo};
///
/// let response = SchemasListResponse {
///     schemas: vec![
///         SchemaInfo {
///             id: "signal".to_string(),
///             name: "Signal".to_string(),
///             version: "1.0".to_string(),
///         },
///     ],
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemasListResponse {
    /// Available schemas
    pub schemas: Vec<SchemaInfo>,
}

impl SchemasListResponse {
    /// Creates a new SchemasListResponse.
    pub fn new(schemas: Vec<SchemaInfo>) -> Self {
        Self { schemas }
    }

    /// Creates an empty response.
    pub fn empty() -> Self {
        Self { schemas: vec![] }
    }
}

/// Information about a schema.
///
/// # Example
///
/// ```
/// use cauce_core::methods::SchemaInfo;
///
/// let info = SchemaInfo {
///     id: "action".to_string(),
///     name: "Action".to_string(),
///     version: "1.0".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaInfo {
    /// Unique schema identifier (e.g., "signal", "action", "jsonrpc-request")
    pub id: String,

    /// Human-readable schema name
    pub name: String,

    /// Schema version
    pub version: String,
}

impl SchemaInfo {
    /// Creates a new SchemaInfo.
    pub fn new(id: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Request parameters for the `cauce.schemas.get` method.
///
/// # Example
///
/// ```
/// use cauce_core::methods::SchemasGetRequest;
///
/// let request = SchemasGetRequest {
///     schema_id: "signal".to_string(),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemasGetRequest {
    /// ID of the schema to retrieve
    pub schema_id: String,
}

impl SchemasGetRequest {
    /// Creates a new SchemasGetRequest.
    pub fn new(schema_id: impl Into<String>) -> Self {
        Self {
            schema_id: schema_id.into(),
        }
    }
}

/// Response from the `cauce.schemas.get` method.
///
/// # Example
///
/// ```
/// use cauce_core::methods::SchemasGetResponse;
/// use serde_json::json;
///
/// let response = SchemasGetResponse {
///     schema: json!({
///         "$schema": "https://json-schema.org/draft/2020-12/schema",
///         "title": "Signal",
///         "type": "object"
///     }),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemasGetResponse {
    /// The JSON schema
    pub schema: Value,
}

impl SchemasGetResponse {
    /// Creates a new SchemasGetResponse.
    pub fn new(schema: Value) -> Self {
        Self { schema }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ===== SchemasListRequest Tests =====

    #[test]
    fn test_schemas_list_request_new() {
        let request = SchemasListRequest::new();
        // Just verify it creates successfully
        assert_eq!(request, SchemasListRequest {});
    }

    #[test]
    fn test_schemas_list_request_default() {
        let request: SchemasListRequest = Default::default();
        assert_eq!(request, SchemasListRequest::new());
    }

    #[test]
    fn test_schemas_list_request_serialization() {
        let request = SchemasListRequest::new();
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_schemas_list_request_deserialization() {
        let json = "{}";
        let request: SchemasListRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request, SchemasListRequest::new());
    }

    // ===== SchemasListResponse Tests =====

    #[test]
    fn test_schemas_list_response_empty() {
        let response = SchemasListResponse::empty();
        assert!(response.schemas.is_empty());
    }

    #[test]
    fn test_schemas_list_response_new() {
        let schemas = vec![
            SchemaInfo::new("signal", "Signal", "1.0"),
            SchemaInfo::new("action", "Action", "1.0"),
        ];
        let response = SchemasListResponse::new(schemas);
        assert_eq!(response.schemas.len(), 2);
    }

    #[test]
    fn test_schemas_list_response_serialization() {
        let response =
            SchemasListResponse::new(vec![SchemaInfo::new("test", "Test Schema", "2.0")]);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"schemas\":["));
        assert!(json.contains("\"id\":\"test\""));
        assert!(json.contains("\"name\":\"Test Schema\""));
        assert!(json.contains("\"version\":\"2.0\""));
    }

    #[test]
    fn test_schemas_list_response_roundtrip() {
        let response = SchemasListResponse::new(vec![
            SchemaInfo::new("signal", "Signal", "1.0"),
            SchemaInfo::new("action", "Action", "1.0"),
        ]);

        let json = serde_json::to_string(&response).unwrap();
        let restored: SchemasListResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, restored);
    }

    // ===== SchemaInfo Tests =====

    #[test]
    fn test_schema_info_new() {
        let info = SchemaInfo::new("my-schema", "My Schema", "1.2.3");
        assert_eq!(info.id, "my-schema");
        assert_eq!(info.name, "My Schema");
        assert_eq!(info.version, "1.2.3");
    }

    #[test]
    fn test_schema_info_roundtrip() {
        let info = SchemaInfo::new("roundtrip", "Roundtrip Schema", "0.1.0");
        let json = serde_json::to_string(&info).unwrap();
        let restored: SchemaInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, restored);
    }

    // ===== SchemasGetRequest Tests =====

    #[test]
    fn test_schemas_get_request_new() {
        let request = SchemasGetRequest::new("signal");
        assert_eq!(request.schema_id, "signal");
    }

    #[test]
    fn test_schemas_get_request_serialization() {
        let request = SchemasGetRequest::new("action");
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, r#"{"schema_id":"action"}"#);
    }

    #[test]
    fn test_schemas_get_request_deserialization() {
        let json = r#"{"schema_id":"jsonrpc"}"#;
        let request: SchemasGetRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.schema_id, "jsonrpc");
    }

    #[test]
    fn test_schemas_get_request_roundtrip() {
        let request = SchemasGetRequest::new("test-schema");
        let json = serde_json::to_string(&request).unwrap();
        let restored: SchemasGetRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(request, restored);
    }

    // ===== SchemasGetResponse Tests =====

    #[test]
    fn test_schemas_get_response_new() {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "type": "object"
        });
        let response = SchemasGetResponse::new(schema.clone());
        assert_eq!(response.schema, schema);
    }

    #[test]
    fn test_schemas_get_response_serialization() {
        let response = SchemasGetResponse::new(json!({"type": "string"}));
        let json_str = serde_json::to_string(&response).unwrap();

        assert!(json_str.contains("\"schema\":{"));
        assert!(json_str.contains("\"type\":\"string\""));
    }

    #[test]
    fn test_schemas_get_response_deserialization() {
        let json = r#"{"schema":{"$id":"test","type":"object","properties":{}}}"#;
        let response: SchemasGetResponse = serde_json::from_str(json).unwrap();

        assert_eq!(response.schema["$id"], "test");
        assert_eq!(response.schema["type"], "object");
    }

    #[test]
    fn test_schemas_get_response_roundtrip() {
        let schema = json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "https://example.com/test.json",
            "title": "Test",
            "type": "object",
            "required": ["name"],
            "properties": {
                "name": { "type": "string" }
            }
        });
        let response = SchemasGetResponse::new(schema);

        let json_str = serde_json::to_string(&response).unwrap();
        let restored: SchemasGetResponse = serde_json::from_str(&json_str).unwrap();
        assert_eq!(response, restored);
    }
}
