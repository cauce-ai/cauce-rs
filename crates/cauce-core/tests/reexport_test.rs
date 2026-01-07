//! Integration tests for cauce-core re-exports.

/// Verify Signal type is re-exported at crate root.
#[test]
fn signal_reexported() {
    use cauce_core::Signal;
    let _signal = Signal;
    assert!(true, "Signal is re-exported");
}

/// Verify Action type is re-exported at crate root.
#[test]
fn action_reexported() {
    use cauce_core::Action;
    let _action = Action;
    assert!(true, "Action is re-exported");
}

/// Verify Topic type is re-exported at crate root.
#[test]
fn topic_reexported() {
    use cauce_core::Topic;
    let _topic = Topic;
    assert!(true, "Topic is re-exported");
}

/// Verify multiple types can be imported together.
#[test]
fn multiple_types_importable() {
    use cauce_core::{Action, Signal, Topic};
    let (_signal, _action, _topic) = (Signal, Action, Topic);
    assert!(true, "Multiple types can be imported together");
}
