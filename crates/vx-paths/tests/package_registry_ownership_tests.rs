//! Removing a package must preserve executable mappings owned by another package.

use std::path::PathBuf;

use vx_paths::global_packages::{GlobalPackage, PackageRegistry};

#[test]
fn test_unregister_preserves_another_packages_executable_index() {
    let mut registry = PackageRegistry::new();
    registry.register(
        GlobalPackage::new("first", "1.0.0", "npm", PathBuf::from("first"))
            .with_executables(vec!["shared".into(), "first-only".into()]),
    );
    registry.register(
        GlobalPackage::new("second", "1.0.0", "npm", PathBuf::from("second"))
            .with_executable("shared"),
    );

    registry.unregister("npm", "first").unwrap();

    assert!(registry.get("npm", "first").is_none());
    assert!(registry.find_by_executable("first-only").is_none());
    assert_eq!(
        registry.find_by_executable("shared").unwrap().name,
        "second"
    );

    registry.unregister("npm", "second").unwrap();
    assert!(registry.find_by_executable("shared").is_none());
}
