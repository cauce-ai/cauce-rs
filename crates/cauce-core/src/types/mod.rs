//! Protocol types for the Cauce Protocol.
//!
//! This module contains the core protocol types used throughout the Cauce ecosystem:
//!
//! - **Signal** - Messages sent from adapters to hubs (future implementation)
//! - **Action** - Commands sent from hubs to agents (future implementation)
//! - **Topic** - Hierarchical topic identifiers for pub/sub (future implementation)
//! - **Subscription** - Topic subscription definitions (future implementation)
//!
//! ## Future Implementation
//!
//! The actual type definitions will be added in subsequent features (TODO.md 2.2-2.6).
//! This module currently provides the structure and documentation for future work.

/// Returns module information for testing purposes.
pub fn module_info() -> &'static str {
    "types: Protocol type definitions"
}

// =============================================================================
// Placeholder Types
// =============================================================================
// These are placeholder types for ergonomic imports. They will be replaced
// with full implementations in subsequent features (TODO.md 2.2-2.6).
// =============================================================================

/// A Signal represents a message sent from an adapter to a hub.
///
/// Signals carry data from external systems into the Cauce Protocol network.
/// They are published to topics and routed to interested agents.
///
/// # Future Implementation
///
/// This placeholder will be replaced with a full implementation including:
/// - Unique signal ID
/// - Source adapter identification
/// - Topic routing information
/// - Payload data
/// - Timestamp and metadata
#[derive(Debug, Clone, Default)]
pub struct Signal;

/// An Action represents a command sent from a hub to an agent.
///
/// Actions instruct agents to perform specific operations in response
/// to signals or other events in the system.
///
/// # Future Implementation
///
/// This placeholder will be replaced with a full implementation including:
/// - Unique action ID
/// - Target agent identification
/// - Command type and parameters
/// - Correlation with triggering signal
#[derive(Debug, Clone, Default)]
pub struct Action;

/// A Topic represents a hierarchical identifier for pub/sub routing.
///
/// Topics use a path-like syntax (e.g., "sensors/temperature/room1")
/// to organize and route messages in the Cauce Protocol.
///
/// # Future Implementation
///
/// This placeholder will be replaced with a full implementation including:
/// - Topic path parsing and validation
/// - Wildcard matching support
/// - Topic hierarchy traversal
#[derive(Debug, Clone, Default)]
pub struct Topic;
