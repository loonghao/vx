//! Platform constraint integration tests

use vx_manifest::{PlatformConstraint, ProviderManifest};

#[test]
fn test_msvc_platform_constraint() {
    let toml = r#"
[provider]
name = "msvc"
description = "Microsoft Visual C++ Build Tools"
ecosystem = "system"

[provider.platforms]
os = ["windows"]

[[runtimes]]
name = "cl"
description = "MSVC C/C++ Compiler"
executable = "cl"
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();

    // Check provider-level platform constraint
    assert!(manifest.provider.platform_constraint.is_some());
    let constraint = manifest.provider.platform_constraint.as_ref().unwrap();
    assert_eq!(constraint.os.len(), 1);

    // Check platform description
    assert_eq!(
        manifest.platform_description(),
        Some("Windows only".to_string())
    );
    assert_eq!(manifest.platform_label(), Some("Windows".to_string()));

    // Check supported runtimes on current platform
    let supported = manifest.supported_runtimes();

    #[cfg(target_os = "windows")]
    {
        assert_eq!(supported.len(), 1);
        assert_eq!(supported[0].name, "cl");
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(supported.len(), 0);
    }
}

#[test]
fn test_cross_platform_runtime() {
    let toml = r#"
[provider]
name = "node"
description = "Node.js JavaScript runtime"
ecosystem = "nodejs"

[[runtimes]]
name = "node"
description = "Node.js runtime"
executable = "node"

[[runtimes]]
name = "npm"
description = "Node package manager"
executable = "npm"
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();

    // No platform constraints - should support all platforms
    assert!(manifest.provider.platform_constraint.is_none());
    assert!(manifest.platform_description().is_none());
    assert!(manifest.platform_label().is_none());

    // All runtimes should be supported
    let supported = manifest.supported_runtimes();
    assert_eq!(supported.len(), 2);
}

#[test]
fn test_runtime_level_platform_constraint() {
    let toml = r#"
[provider]
name = "apple-tools"
description = "Apple development tools"

[[runtimes]]
name = "xcodebuild"
executable = "xcodebuild"

[runtimes.platform_constraint]
os = ["macos"]

[[runtimes]]
name = "cross-platform-tool"
executable = "tool"
"#;

    let manifest = ProviderManifest::parse(toml).unwrap();

    // Provider has no platform constraint
    assert!(manifest.provider.platform_constraint.is_none());

    // Check runtime-level constraints
    let xcodebuild = manifest
        .runtimes
        .iter()
        .find(|r| r.name == "xcodebuild")
        .unwrap();
    assert!(xcodebuild.platform_constraint.is_some());
    assert_eq!(
        xcodebuild.platform_description(),
        Some("macOS only".to_string())
    );

    let cross_tool = manifest
        .runtimes
        .iter()
        .find(|r| r.name == "cross-platform-tool")
        .unwrap();
    assert!(cross_tool.platform_constraint.is_none());

    // Check supported runtimes
    let supported = manifest.supported_runtimes();

    #[cfg(target_os = "macos")]
    {
        assert_eq!(supported.len(), 2);
    }

    #[cfg(not(target_os = "macos"))]
    {
        assert_eq!(supported.len(), 1);
        assert_eq!(supported[0].name, "cross-platform-tool");
    }
}

#[test]
fn test_platform_constraint_serialization() {
    use serde_json;
    use vx_manifest::{Arch, Os};

    let constraint = PlatformConstraint {
        os: vec![Os::Windows, Os::Linux],
        arch: vec![Arch::X86_64],
        exclude: vec![],
    };

    // Test JSON serialization
    let json = serde_json::to_string(&constraint).unwrap();
    let deserialized: PlatformConstraint = serde_json::from_str(&json).unwrap();

    assert_eq!(constraint.os, deserialized.os);
    assert_eq!(constraint.arch, deserialized.arch);
    assert_eq!(constraint.exclude, deserialized.exclude);
}
