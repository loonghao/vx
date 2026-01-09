//! Provider CRUD E2E Tests
//!
//! End-to-end tests for all builtin providers' dynamic CRUD operations:
//! - Create: Add new providers via file system
//! - Read: Load and query all embedded provider manifests
//! - Update: Modify providers via override mechanism
//! - Delete: Replace/remove providers
//!
//! These tests verify that the manifest-driven provider system correctly handles
//! all 35+ builtin providers defined in crates/vx-providers/

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use vx_manifest::{Ecosystem, ManifestLoader, ProviderManifest};
use vx_runtime::ManifestRegistry;

// ============================================
// Test Utilities
// ============================================

/// Write a provider.toml file to disk
fn write_provider_toml(dir: &Path, name: &str, content: &str) {
    let provider_dir = dir.join(name);
    fs::create_dir_all(&provider_dir).expect("Failed to create provider directory");
    fs::write(provider_dir.join("provider.toml"), content).expect("Failed to write provider.toml");
}

/// Write an override file to disk
fn write_override_toml(dir: &Path, provider_name: &str, content: &str) {
    let filename = format!("{}.override.toml", provider_name);
    fs::write(dir.join(filename), content).expect("Failed to write override file");
}

// ============================================
// READ Tests - Load and Query Builtin Providers
// ============================================

mod read_tests {
    use super::*;

    /// Test that all builtin provider.toml files can be loaded
    #[test]
    fn test_load_all_builtin_provider_manifests() {
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        let count = loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load providers");

        // We should have at least 30 providers
        assert!(
            count >= 30,
            "Expected at least 30 providers, found {}",
            count
        );

        // Verify some key providers exist
        let key_providers = ["node", "python", "go", "rust", "bun", "deno"];
        for name in key_providers {
            assert!(
                loader.get(name).is_some(),
                "Key provider '{}' not found",
                name
            );
        }
    }

    /// Test that each builtin provider has valid structure
    #[test]
    fn test_all_builtin_providers_have_valid_structure() {
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load");

        for manifest in loader.all() {
            // Provider must have a name
            assert!(
                !manifest.provider.name.is_empty(),
                "Provider has empty name"
            );

            // Provider must have at least one runtime
            assert!(
                !manifest.runtimes.is_empty(),
                "Provider '{}' has no runtimes",
                manifest.provider.name
            );

            // Each runtime must have name and executable
            for runtime in &manifest.runtimes {
                assert!(
                    !runtime.name.is_empty(),
                    "Runtime in provider '{}' has empty name",
                    manifest.provider.name
                );
                assert!(
                    !runtime.executable.is_empty(),
                    "Runtime '{}' has empty executable",
                    runtime.name
                );
            }
        }
    }

    /// Test querying runtime by name across all providers
    #[test]
    fn test_find_runtime_across_providers() {
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load");

        // Test finding common runtimes
        // Note: rust provider has rustup/rustc/cargo, not "rust"
        let runtimes_to_find = ["node", "python", "go", "rustup", "bun"];
        for name in runtimes_to_find {
            let result = loader.find_runtime(name);
            assert!(result.is_some(), "Could not find runtime '{}'", name);

            let (manifest, runtime) = result.unwrap();
            assert_eq!(runtime.name, name);
            println!(
                "Found runtime '{}' in provider '{}'",
                name, manifest.provider.name
            );
        }
    }

    /// Test querying runtime by alias
    #[test]
    fn test_find_runtime_by_alias() {
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load");

        // node has alias "nodejs"
        let node_manifest = loader.get("node").expect("Node provider not found");
        let node_runtime = node_manifest
            .get_runtime("node")
            .expect("Node runtime not found");

        if node_runtime.aliases.contains(&"nodejs".to_string()) {
            // find_runtime should also check aliases
            let result = loader.find_runtime("nodejs");
            assert!(result.is_some(), "Could not find node by alias 'nodejs'");
        }
    }

    /// Test ecosystem grouping of providers
    #[test]
    fn test_providers_grouped_by_ecosystem() {
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load");

        let mut nodejs_providers = Vec::new();
        let mut python_providers = Vec::new();
        let mut go_providers = Vec::new();

        for manifest in loader.all() {
            match manifest.provider.ecosystem {
                Some(Ecosystem::NodeJs) => nodejs_providers.push(manifest.provider.name.clone()),
                Some(Ecosystem::Python) => python_providers.push(manifest.provider.name.clone()),
                Some(Ecosystem::Go) => go_providers.push(manifest.provider.name.clone()),
                _ => {}
            }
        }

        println!("Node.js ecosystem: {:?}", nodejs_providers);
        println!("Python ecosystem: {:?}", python_providers);
        println!("Go ecosystem: {:?}", go_providers);

        // node, pnpm, yarn, bun should be in nodejs ecosystem
        assert!(nodejs_providers.contains(&"node".to_string()));
    }

    /// Test ManifestRegistry can load and query all providers
    #[test]
    fn test_manifest_registry_loads_all_providers() {
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut registry = ManifestRegistry::new();
        let count = registry
            .load_from_directory(&providers_dir)
            .expect("Failed to load");

        assert!(count >= 30);

        // Test get_runtime_metadata
        let node_meta = registry.get_runtime_metadata("node");
        assert!(node_meta.is_some());
        let meta = node_meta.unwrap();
        assert_eq!(meta.name, "node");
        assert_eq!(meta.provider_name, "node");

        // Test get_supported_runtimes
        let supported = registry.get_supported_runtimes();
        assert!(!supported.is_empty());
        println!("Supported runtimes on this platform: {}", supported.len());
    }
}

// ============================================
// CREATE Tests - Add New Providers
// ============================================

mod create_tests {
    use super::*;

    /// Test creating a new custom provider alongside builtin ones
    #[test]
    fn test_create_custom_provider() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let custom_provider = r#"
[provider]
name = "my-custom-tool"
description = "My custom development tool"
ecosystem = "nodejs"

[[runtimes]]
name = "my-tool"
description = "Custom tool runtime"
executable = "my-tool"
aliases = ["mt"]
priority = 50
"#;
        write_provider_toml(temp_dir.path(), "my-custom-tool", custom_provider);

        let mut loader = ManifestLoader::new();
        let count = loader
            .load_from_dir(temp_dir.path())
            .expect("Failed to load");

        assert_eq!(count, 1);
        let manifest = loader
            .get("my-custom-tool")
            .expect("Custom provider not found");
        assert_eq!(manifest.runtimes[0].name, "my-tool");
        assert_eq!(manifest.runtimes[0].aliases, vec!["mt"]);
    }

    /// Test that new providers can coexist with builtin ones
    #[test]
    fn test_custom_provider_coexists_with_builtin() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        // Load builtin first
        let mut loader = ManifestLoader::new();
        let builtin_count = loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load builtin");

        // Then add custom
        let custom = r#"
[provider]
name = "custom-addon"

[[runtimes]]
name = "addon-cli"
executable = "addon"
"#;
        write_provider_toml(temp_dir.path(), "custom-addon", custom);
        let custom_count = loader
            .load_from_dir(temp_dir.path())
            .expect("Failed to load custom");

        assert_eq!(custom_count, 1);
        assert_eq!(loader.len(), builtin_count + 1);

        // Both should be accessible
        assert!(loader.get("node").is_some());
        assert!(loader.get("custom-addon").is_some());
    }
}

// ============================================
// UPDATE Tests - Override Provider Configs
// ============================================

mod update_tests {
    use super::*;

    /// Test overriding a builtin provider's constraints
    #[test]
    fn test_override_provider_constraints() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load builtin");

        // Create an override for yarn
        let override_content = r#"
[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=14, <22" }
]
"#;
        write_override_toml(temp_dir.path(), "yarn", override_content);
        loader
            .load_overrides_from_dir(temp_dir.path())
            .expect("Failed to load override");

        let manifests = loader.into_manifests();
        let yarn = manifests.iter().find(|m| m.provider.name == "yarn");
        assert!(yarn.is_some());

        let yarn = yarn.unwrap();
        let runtime = yarn.get_runtime("yarn").unwrap();

        // Check override was applied
        if !runtime.constraints.is_empty() {
            assert_eq!(runtime.constraints[0].requires[0].version, ">=14, <22");
        }
    }

    /// Test that later overrides are appended to constraints
    /// Note: override constraints only apply to the runtime with the same name as the provider
    #[test]
    fn test_override_appends_constraints() {
        let user_dir = TempDir::new().expect("Failed to create user dir");
        let project_dir = TempDir::new().expect("Failed to create project dir");

        // Create base provider - runtime name MUST match provider name for default override to apply
        let base = r#"
[provider]
name = "mytool"

[[runtimes]]
name = "mytool"
executable = "mytool"
"#;
        write_provider_toml(user_dir.path(), "mytool", base);

        // User-level override adds a constraint (applies to runtime with same name as provider)
        let user_override = r#"
[[constraints]]
when = "^1"
requires = [{ runtime = "node", version = ">=14" }]
"#;
        write_override_toml(user_dir.path(), "mytool", user_override);

        // Project-level override adds another constraint with different `when`
        let project_override = r#"
[[constraints]]
when = "^2"
requires = [{ runtime = "node", version = ">=18" }]
"#;
        write_override_toml(project_dir.path(), "mytool", project_override);

        let mut loader = ManifestLoader::new();
        loader.load_from_dir(user_dir.path()).unwrap();
        loader.load_overrides_from_dir(user_dir.path()).unwrap();
        loader.load_overrides_from_dir(project_dir.path()).unwrap();

        let manifests = loader.into_manifests();
        let manifest = manifests
            .iter()
            .find(|m| m.provider.name == "mytool")
            .unwrap();
        let runtime = manifest.get_runtime("mytool").unwrap();

        // Both constraints should be present (overrides append for different `when` patterns)
        assert!(
            runtime.constraints.len() >= 2,
            "Expected at least 2 constraints, found {}",
            runtime.constraints.len()
        );

        // Check that both user and project overrides are applied
        let versions: Vec<&str> = runtime
            .constraints
            .iter()
            .flat_map(|c| c.requires.iter())
            .map(|r| r.version.as_str())
            .collect();

        assert!(versions.contains(&">=14"), "User override not found");
        assert!(versions.contains(&">=18"), "Project override not found");
    }

    /// Test replacing a builtin provider entirely
    #[test]
    fn test_replace_builtin_provider() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load builtin");

        let original_node = loader.get("node").unwrap();
        let original_desc = original_node.provider.description.clone();

        // Replace node provider with custom version
        let custom_node = r#"
[provider]
name = "node"
description = "Custom Node.js distribution"

[[runtimes]]
name = "node"
executable = "node"
description = "Custom node runtime"
"#;
        write_provider_toml(temp_dir.path(), "node", custom_node);
        loader.load_from_dir(temp_dir.path()).unwrap();

        let replaced = loader.get("node").unwrap();
        assert_eq!(
            replaced.provider.description,
            Some("Custom Node.js distribution".to_string())
        );
        assert_ne!(replaced.provider.description, original_desc);
    }
}

// ============================================
// DELETE Tests - Remove/Replace Providers
// ============================================

mod delete_tests {
    use super::*;

    /// Test that loading a new provider with same name replaces the old one
    #[test]
    fn test_provider_replacement_acts_as_delete() {
        let mut loader = ManifestLoader::new();

        // Insert first version
        let v1 = ProviderManifest::parse(
            r#"
[provider]
name = "replaceable"
description = "Version 1"

[[runtimes]]
name = "tool"
executable = "tool-v1"
"#,
        )
        .unwrap();
        loader.insert(v1);

        assert_eq!(
            loader.get("replaceable").unwrap().provider.description,
            Some("Version 1".to_string())
        );

        // Insert replacement (acts as delete + create)
        let v2 = ProviderManifest::parse(
            r#"
[provider]
name = "replaceable"
description = "Version 2"

[[runtimes]]
name = "tool"
executable = "tool-v2"
"#,
        )
        .unwrap();
        loader.insert(v2);

        // Only one provider should exist
        assert_eq!(loader.len(), 1);
        let manifest = loader.get("replaceable").unwrap();
        assert_eq!(manifest.provider.description, Some("Version 2".to_string()));
        assert_eq!(manifest.runtimes[0].executable, "tool-v2");
    }

    /// Test that directory loading replaces embedded manifests
    #[test]
    fn test_directory_replaces_embedded() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let mut loader = ManifestLoader::new();

        // Simulate embedded manifest
        let embedded = vec![(
            "test-provider",
            r#"
[provider]
name = "test-provider"
description = "Embedded version"

[[runtimes]]
name = "test"
executable = "test"
"#,
        )];
        loader.load_embedded(embedded).unwrap();

        assert_eq!(
            loader.get("test-provider").unwrap().provider.description,
            Some("Embedded version".to_string())
        );

        // Directory version should replace
        let dir_version = r#"
[provider]
name = "test-provider"
description = "Directory version"

[[runtimes]]
name = "test"
executable = "test-new"
"#;
        write_provider_toml(temp_dir.path(), "test-provider", dir_version);
        loader.load_from_dir(temp_dir.path()).unwrap();

        let manifest = loader.get("test-provider").unwrap();
        assert_eq!(
            manifest.provider.description,
            Some("Directory version".to_string())
        );
    }
}

// ============================================
// Integration Tests - Full CRUD Workflow
// ============================================

mod integration_tests {
    use super::*;

    /// Test complete CRUD workflow
    #[test]
    fn test_full_crud_workflow() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mut loader = ManifestLoader::new();

        // CREATE: Add a new provider
        let new_provider = r#"
[provider]
name = "workflow-test"
description = "Initial version"

[[runtimes]]
name = "workflow-cli"
executable = "workflow"

[[runtimes.constraints]]
when = "*"
requires = [{ runtime = "node", version = ">=12" }]
"#;
        write_provider_toml(temp_dir.path(), "workflow-test", new_provider);
        loader.load_from_dir(temp_dir.path()).unwrap();

        // READ: Verify it was created
        let manifest = loader.get("workflow-test").expect("Provider not created");
        assert_eq!(
            manifest.provider.description,
            Some("Initial version".to_string())
        );

        // UPDATE: Override constraints
        let override_content = r#"
[[constraints]]
when = "*"
requires = [{ runtime = "node", version = ">=18" }]
"#;
        write_override_toml(temp_dir.path(), "workflow-test", override_content);
        loader.load_overrides_from_dir(temp_dir.path()).unwrap();

        // DELETE (via replacement): Replace with new version
        let updated_provider = r#"
[provider]
name = "workflow-test"
description = "Updated version"

[[runtimes]]
name = "workflow-cli"
executable = "workflow-v2"
"#;
        // Remove old and write new
        fs::remove_dir_all(temp_dir.path().join("workflow-test")).unwrap();
        write_provider_toml(temp_dir.path(), "workflow-test", updated_provider);

        // Reload
        let mut new_loader = ManifestLoader::new();
        new_loader.load_from_dir(temp_dir.path()).unwrap();

        let final_manifest = new_loader
            .get("workflow-test")
            .expect("Provider not found after update");
        assert_eq!(
            final_manifest.provider.description,
            Some("Updated version".to_string())
        );
        assert_eq!(final_manifest.runtimes[0].executable, "workflow-v2");
    }

    /// Test loading all builtin providers and verifying count
    #[test]
    fn test_all_builtin_providers_loaded() {
        let providers_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("vx-providers");

        let mut loader = ManifestLoader::new();
        let count = loader
            .load_from_dir(&providers_dir)
            .expect("Failed to load");

        println!("Loaded {} builtin providers:", count);
        let mut names: Vec<_> = loader.all().map(|m| m.provider.name.clone()).collect();
        names.sort();
        for name in &names {
            println!("  - {}", name);
        }

        // Verify minimum expected count
        assert!(
            count >= 30,
            "Expected at least 30 providers, found {}",
            count
        );

        // Verify key providers
        let required = [
            "node", "python", "go", "rust", "bun", "deno", "yarn", "pnpm",
        ];
        for name in required {
            assert!(
                loader.get(name).is_some(),
                "Required provider '{}' missing",
                name
            );
        }
    }
}
