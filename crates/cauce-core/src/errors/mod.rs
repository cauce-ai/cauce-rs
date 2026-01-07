//! Error types for the Cauce Protocol.
//!
//! This module provides error types for validation and building operations:
//!
//! - [`ValidationError`] - Errors during field and format validation
//! - [`BuilderError`] - Errors during builder construction
//!
//! ## Error Categories
//!
//! Errors are categorized by their source:
//! - Validation errors (invalid topics, IDs, fields)
//! - Builder errors (missing required fields)

use thiserror::Error;

/// Errors that occur during validation operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Invalid topic format
    #[error("invalid topic: {reason}")]
    InvalidTopic {
        /// The reason the topic is invalid
        reason: String,
    },

    /// Invalid Signal ID format
    #[error("invalid signal ID: {reason}")]
    InvalidSignalId {
        /// The reason the ID is invalid
        reason: String,
    },

    /// Invalid Action ID format
    #[error("invalid action ID: {reason}")]
    InvalidActionId {
        /// The reason the ID is invalid
        reason: String,
    },

    /// Invalid field value
    #[error("invalid field '{field}': {reason}")]
    InvalidField {
        /// The field that is invalid
        field: String,
        /// The reason it's invalid
        reason: String,
    },
}

/// Errors that occur during builder operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum BuilderError {
    /// A required field was not provided
    #[error("missing required field: {field}")]
    MissingField {
        /// The field that is missing
        field: String,
    },

    /// Multiple required fields were not provided
    #[error("missing required fields: {}", fields.join(", "))]
    MissingFields {
        /// The fields that are missing
        fields: Vec<String>,
    },

    /// Validation failed during build
    #[error("validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),
}

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "errors: Error types and error codes"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::InvalidTopic {
            reason: "contains invalid characters".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "invalid topic: contains invalid characters"
        );
    }

    #[test]
    fn test_builder_error_display() {
        let err = BuilderError::MissingField {
            field: "id".to_string(),
        };
        assert_eq!(err.to_string(), "missing required field: id");
    }

    #[test]
    fn test_builder_error_from_validation() {
        let validation_err = ValidationError::InvalidTopic {
            reason: "too long".to_string(),
        };
        let builder_err: BuilderError = validation_err.into();
        assert!(matches!(builder_err, BuilderError::ValidationFailed(_)));
    }
}
