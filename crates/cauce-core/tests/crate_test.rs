//! Integration tests for cauce-core crate compilation and basic functionality.

/// Verify the crate compiles and can be imported.
#[test]
fn crate_compiles() {
    // This test passes if the crate compiles successfully.
    // The mere existence of this test file proves the crate is usable.
    assert!(true, "cauce-core crate compiled successfully");
}

/// Verify the crate version is accessible.
#[test]
fn crate_has_version() {
    let version = env!("CARGO_PKG_VERSION");
    assert!(!version.is_empty(), "Crate should have a version");
    assert_eq!(version, "0.1.0");
}
