//! Validation utilities for the Cauce Protocol.
//!
//! This module provides validation functionality:
//!
//! - **Schema validation** - JSON Schema validation using embedded schemas (future)
//! - **Field validation** - Common field validators (IDs, topics, timestamps)
//! - **Message validation** - Complete message validation pipelines
//!
//! ## Schema-Driven Validation
//!
//! Per Constitution Principle II, all protocol messages are validated against
//! JSON schemas. This module will embed and expose those schemas.
//!
//! ## Future Implementation
//!
//! The actual validation implementations will be added in subsequent features (TODO.md 2.7).

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "validation: Validation utilities and schema validation"
}
