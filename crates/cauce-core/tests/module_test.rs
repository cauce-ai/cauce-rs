//! Integration tests for cauce-core module structure.

/// Verify all 5 modules are accessible from the crate root.
#[test]
fn all_modules_accessible() {
    // Import all modules to verify they're publicly accessible
    use cauce_core::constants;
    use cauce_core::errors;
    use cauce_core::jsonrpc;
    use cauce_core::types;
    use cauce_core::validation;

    // Modules exist and are accessible (compilation proves this)
    let _ = (
        types::module_info(),
        jsonrpc::module_info(),
        validation::module_info(),
        errors::module_info(),
        constants::module_info(),
    );
}

/// Verify types module is accessible.
#[test]
fn types_module_accessible() {
    let info = cauce_core::types::module_info();
    assert!(info.contains("types"));
}

/// Verify jsonrpc module is accessible.
#[test]
fn jsonrpc_module_accessible() {
    let info = cauce_core::jsonrpc::module_info();
    assert!(info.contains("jsonrpc"));
}

/// Verify validation module is accessible.
#[test]
fn validation_module_accessible() {
    let info = cauce_core::validation::module_info();
    assert!(info.contains("validation"));
}

/// Verify errors module is accessible.
#[test]
fn errors_module_accessible() {
    let info = cauce_core::errors::module_info();
    assert!(info.contains("errors"));
}

/// Verify constants module is accessible.
#[test]
fn constants_module_accessible() {
    let info = cauce_core::constants::module_info();
    assert!(info.contains("constants"));
}
