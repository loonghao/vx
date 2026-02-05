//! Provider registry setup and management
//!
//! This module provides the registry setup for all providers.
//! It supports two registration modes:
//!
//! 1. **Static registration** (default): All providers are registered at compile time
//! 2. **Manifest-driven registration**: Providers are loaded from manifest files
//!
//! The manifest-driven approach is preferred as it provides a single source of truth
//! for provider metadata, but static registration is kept for backward compatibility.
//!
//! ## Compile-time Manifest Embedding
//!
//! Provider manifests (`provider.toml` files) are collected at compile time by `build.rs`
//! and embedded into the binary. This enables fast startup without filesystem access.
//!
//! See RFC 0013: Manifest-Driven Provider Registration

use std::sync::Arc;
use tracing::trace;
use vx_manifest::{ManifestLoader, ProviderManifest};
use vx_paths::{find_project_root, VxPaths, PROJECT_VX_DIR};
use vx_runtime::{
    create_runtime_context, default_plugin_paths, init_constraints_from_manifests,
    ManifestRegistry, PluginLoader, Provider, ProviderRegistry, Runtime, RuntimeContext,
};

// Include the compile-time generated provider manifests
include!(concat!(env!("OUT_DIR"), "/provider_manifests.rs"));

/// Macro to register all builtin providers
///
/// This macro generates the provider factory registration code,
/// reducing boilerplate and ensuring consistency.
///
/// # Usage
///
/// ```rust,ignore
/// register_providers!(
///     node, go, rust, uv, bun, pnpm, yarn, vscode, just, vite,
///     rez, deno, zig, java, terraform, kubectl, helm, rcedit,
///     git, choco, docker, awscli, azcli, gcloud, ninja, cmake,
///     protoc, task, pre_commit, ollama, spack,
///     release_please, python, msvc,
/// );
/// ```
macro_rules! register_providers {
    // Match provider names, handling both underscore and hyphen variants
    ($registry:expr, $($name:ident),* $(,)?) => {
        $(
            register_providers!(@single $registry, $name);
        )*
    };

    // Single provider registration with name mapping
    (@single $registry:expr, pre_commit) => {
        $registry.register(vx_provider_pre_commit::create_provider());
    };
    (@single $registry:expr, release_please) => {
        $registry.register(vx_provider_release_please::create_provider());
    };
    (@single $registry:expr, $name:ident) => {
        paste::paste! {
            $registry.register([<vx_provider_ $name>]::create_provider());
        }
    };
}

/// Macro to register provider factories for manifest-driven registration
macro_rules! register_provider_factories {
    ($registry:expr, $($name:ident),* $(,)?) => {
        $(
            register_provider_factories!(@single $registry, $name);
        )*
    };

    // Single factory registration with name mapping
    (@single $registry:expr, pre_commit) => {
        $registry.register_factory("pre-commit", || vx_provider_pre_commit::create_provider());
    };
    (@single $registry:expr, release_please) => {
        $registry.register_factory("release-please", || vx_provider_release_please::create_provider());
    };
    (@single $registry:expr, $name:ident) => {
        paste::paste! {
            $registry.register_factory(stringify!($name), || [<vx_provider_ $name>]::create_provider());
        }
    };
}

/// Create and initialize the provider registry with all available providers.
///
/// Prefers manifest-driven registration with override support:
/// 1. Embedded provider manifests (generated at build time)
/// 2. User-level overrides: ~/.vx/providers
/// 3. Project-level overrides: <project>/.vx/providers
///
/// Falls back to static registration if manifests are missing or factories cannot
/// build any providers (backward compatibility).
pub fn create_registry() -> ProviderRegistry {
    let manifests = load_manifests_with_overrides();

    if manifests.is_empty() {
        // No manifests found; fall back to static registration and init constraints
        let _ = init_constraints_from_manifests(get_embedded_manifests().iter().copied());
        return create_static_registry();
    }

    init_constraints_from_manifest_list(&manifests);

    let mut manifest_registry = create_manifest_registry();
    manifest_registry.load_from_manifests(manifests);
    let registry = manifest_registry.build_registry();

    if let Some(loader) = build_plugin_loader() {
        registry.set_provider_loader(Arc::new(loader));
    }

    if registry.providers().is_empty() {
        // Safety net: if factories fail to produce providers, use static registration
        let _ = init_constraints_from_manifests(get_embedded_manifests().iter().copied());
        return create_static_registry();
    }

    registry
}

/// Create a manifest-driven registry with all builtin provider factories
///
/// This is the preferred approach as it uses manifest files as the source of truth.
/// The registry can optionally load additional manifests from a directory.
///
/// # Compile-time Manifests
///
/// Provider manifests are embedded at compile time via `build.rs`.
/// Access them via `PROVIDER_MANIFESTS` constant.
///
/// # Example
///
/// ```rust,ignore
/// // Create registry with builtin factories
/// let manifest_registry = create_manifest_registry();
///
/// // Build the provider registry
/// let provider_registry = manifest_registry.build_registry_from_factories();
///
/// // Access embedded manifests
/// println!("Embedded {} provider manifests", PROVIDER_COUNT);
/// for (name, content) in PROVIDER_MANIFESTS {
///     println!("  - {}", name);
/// }
/// ```
pub fn create_manifest_registry() -> ManifestRegistry {
    let mut registry = ManifestRegistry::new();

    // Register all builtin provider factories using the macro
    register_provider_factories!(
        registry,
        node,
        go,
        rust,
        uv,
        bun,
        pnpm,
        yarn,
        vscode,
        just,
        jq,
        vite,
        rez,
        deno,
        zig,
        java,
        terraform,
        kubectl,
        helm,
        rcedit,
        git,
        choco,
        brew,
        docker,
        awscli,
        azcli,
        gcloud,
        ninja,
        cmake,
        meson,
        make,
        protoc,
        task,
        pre_commit,
        ollama,
        spack,
        release_please,
        python,
        msvc,
        ffmpeg,
        nasm,
        gh,
        imagemagick,
        pwsh,
        dotnet,
        msbuild,
        nuget,
        winget,
    );

    registry
}

/// Build a registry using static registration only (backward compatibility).
fn create_static_registry() -> ProviderRegistry {
    let registry = ProviderRegistry::new();

    register_providers!(
        registry,
        node,
        go,
        rust,
        uv,
        bun,
        pnpm,
        yarn,
        vscode,
        just,
        jq,
        vite,
        rez,
        deno,
        zig,
        java,
        terraform,
        kubectl,
        helm,
        rcedit,
        git,
        choco,
        brew,
        docker,
        awscli,
        azcli,
        gcloud,
        ninja,
        cmake,
        meson,
        make,
        protoc,
        task,
        pre_commit,
        ollama,
        spack,
        release_please,
        python,
        msvc,
        ffmpeg,
        nasm,
        gh,
        imagemagick,
        pwsh,
        dotnet,
        msbuild,
        nuget,
        winget,
    );

    registry
}

/// Load manifests with override order: embedded < user < project.
pub fn load_manifests_with_overrides() -> Vec<ProviderManifest> {
    let mut loader = ManifestLoader::new();

    // 1) Embedded manifests (build.rs generated)
    let _ = loader.load_embedded(get_embedded_manifests().iter().copied());

    // 2) User-level overrides: ~/.vx/providers
    if let Ok(paths) = VxPaths::new() {
        let user_dir = paths.base_dir.join("providers");
        if user_dir.exists() {
            // Load full provider.toml files (for user-defined providers)
            let _ = loader.load_from_dir(&user_dir);
            // Load .override.toml files (for constraint overrides)
            let _ = loader.load_overrides_from_dir(&user_dir);
            trace!("Loaded user provider overrides from {:?}", user_dir);
        }
    }

    // 3) Project-level overrides: <project>/.vx/providers
    if let Ok(cwd) = std::env::current_dir() {
        if let Some(project_root) = find_project_root(&cwd) {
            let project_dir = project_root.join(PROJECT_VX_DIR).join("providers");
            if project_dir.exists() {
                // Load full provider.toml files (for project-specific providers)
                let _ = loader.load_from_dir(&project_dir);
                // Load .override.toml files (for constraint overrides)
                let _ = loader.load_overrides_from_dir(&project_dir);
                trace!("Loaded project provider overrides from {:?}", project_dir);
            }
        }
    }

    loader.into_manifests()
}

fn build_plugin_loader() -> Option<PluginLoader> {
    let mut paths = Vec::new();

    if let Ok(vx_paths) = VxPaths::new() {
        paths.extend(default_plugin_paths(std::slice::from_ref(
            &vx_paths.base_dir,
        )));
    }

    if let Ok(cwd) = std::env::current_dir() {
        if let Some(project_root) = find_project_root(&cwd) {
            paths.push(project_root.join(PROJECT_VX_DIR).join("plugins"));
        }
    }

    paths.retain(|p| p.exists());
    if paths.is_empty() {
        return None;
    }

    Some(PluginLoader::new(paths))
}

fn init_constraints_from_manifest_list(manifests: &[ProviderManifest]) {
    let manifest_strings: Vec<(String, String)> = manifests
        .iter()
        .filter_map(|manifest| {
            toml::to_string(manifest)
                .ok()
                .map(|s| (manifest.provider.name.clone(), s))
        })
        .collect();

    if manifest_strings.is_empty() {
        let _ = init_constraints_from_manifests(get_embedded_manifests().iter().copied());
        return;
    }

    let _ = init_constraints_from_manifests(
        manifest_strings
            .iter()
            .map(|(name, content)| (name.as_str(), content.as_str())),
    );
}

/// Get the embedded provider manifests
///
/// Returns a slice of (name, toml_content) tuples for all provider manifests
/// that were embedded at compile time.
pub fn get_embedded_manifests() -> &'static [(&'static str, &'static str)] {
    PROVIDER_MANIFESTS
}

/// Get the number of embedded provider manifests
pub fn get_embedded_manifest_count() -> usize {
    PROVIDER_COUNT
}

/// Get platform label for a runtime from embedded manifests
///
/// Returns the platform label (e.g., "Windows", "macOS") if the runtime
/// has platform constraints, or None if it supports all platforms.
pub fn get_runtime_platform_label(runtime_name: &str) -> Option<String> {
    for (_, content) in PROVIDER_MANIFESTS {
        if let Ok(manifest) = ProviderManifest::parse(content) {
            // Check if provider has platform constraint
            if let Some(ref constraint) = manifest.provider.platform_constraint {
                // Check if any runtime in this provider matches
                for runtime in &manifest.runtimes {
                    if runtime.name == runtime_name
                        || runtime.aliases.contains(&runtime_name.to_string())
                    {
                        return constraint.short_label();
                    }
                }
            }
            // Check runtime-level platform constraint
            for runtime in &manifest.runtimes {
                if runtime.name == runtime_name
                    || runtime.aliases.contains(&runtime_name.to_string())
                {
                    if let Some(ref constraint) = runtime.platform_constraint {
                        return constraint.short_label();
                    }
                }
            }
        }
    }
    None
}

/// Create a runtime context for operations
pub fn create_context() -> anyhow::Result<RuntimeContext> {
    create_runtime_context()
}

/// Extension trait for ProviderRegistry to provide convenience methods
pub trait ProviderRegistryExt {
    /// Get a runtime by name
    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>>;

    /// Check if a runtime is supported
    fn supports_runtime(&self, name: &str) -> bool;

    /// List all runtime names
    fn list_runtimes(&self) -> Vec<String>;

    /// Get all providers
    fn get_providers(&self) -> Vec<Arc<dyn Provider>>;
}

impl ProviderRegistryExt for ProviderRegistry {
    fn get_runtime(&self, name: &str) -> Option<Arc<dyn Runtime>> {
        ProviderRegistry::get_runtime(self, name)
    }

    fn supports_runtime(&self, name: &str) -> bool {
        self.supports(name)
    }

    fn list_runtimes(&self) -> Vec<String> {
        self.runtime_names()
    }

    fn get_providers(&self) -> Vec<Arc<dyn Provider>> {
        self.providers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_manifests_exist() {
        // Verify that manifests were embedded at compile time
        let count = PROVIDER_COUNT;
        assert!(count > 0, "Expected embedded manifests, found none");
        assert_eq!(PROVIDER_MANIFESTS.len(), count);
    }

    #[test]
    fn test_embedded_manifests_are_valid_toml() {
        for (name, content) in PROVIDER_MANIFESTS {
            let result: Result<toml::Value, _> = toml::from_str(content);
            assert!(
                result.is_ok(),
                "Invalid TOML in manifest for '{}': {:?}",
                name,
                result.err()
            );
        }
    }

    #[test]
    fn test_get_embedded_manifests() {
        let manifests = get_embedded_manifests();
        assert!(!manifests.is_empty());
    }

    #[test]
    fn test_get_embedded_manifest_count() {
        let count = get_embedded_manifest_count();
        assert!(
            count > 30,
            "Expected at least 30 providers, found {}",
            count
        );
    }

    #[test]
    fn test_nuget_manifest_parses() {
        // Find nuget manifest
        let nuget_manifest = PROVIDER_MANIFESTS
            .iter()
            .find(|(name, _)| *name == "nuget")
            .map(|(_, content)| content);

        assert!(nuget_manifest.is_some(), "nuget manifest not found");

        // Parse as ProviderManifest
        let result = vx_manifest::ProviderManifest::parse(nuget_manifest.unwrap());
        assert!(
            result.is_ok(),
            "Failed to parse nuget manifest: {:?}",
            result.err()
        );

        let manifest = result.unwrap();
        assert_eq!(manifest.provider.name, "nuget");
        assert!(
            !manifest.runtimes.is_empty(),
            "nuget should have at least one runtime"
        );
    }

    #[test]
    fn test_msbuild_manifest_parses() {
        // Find msbuild manifest
        let msbuild_manifest = PROVIDER_MANIFESTS
            .iter()
            .find(|(name, _)| *name == "msbuild")
            .map(|(_, content)| content);

        assert!(msbuild_manifest.is_some(), "msbuild manifest not found");

        // Parse as ProviderManifest
        let result = vx_manifest::ProviderManifest::parse(msbuild_manifest.unwrap());
        assert!(
            result.is_ok(),
            "Failed to parse msbuild manifest: {:?}",
            result.err()
        );

        let manifest = result.unwrap();
        assert_eq!(manifest.provider.name, "msbuild");
        assert!(
            !manifest.runtimes.is_empty(),
            "msbuild should have at least one runtime"
        );
    }

    #[test]
    fn test_winget_manifest_parses() {
        // Find winget manifest
        let winget_manifest = PROVIDER_MANIFESTS
            .iter()
            .find(|(name, _)| *name == "winget")
            .map(|(_, content)| content);

        assert!(winget_manifest.is_some(), "winget manifest not found");

        // Parse as ProviderManifest
        let result = vx_manifest::ProviderManifest::parse(winget_manifest.unwrap());
        assert!(
            result.is_ok(),
            "Failed to parse winget manifest: {:?}",
            result.err()
        );

        let manifest = result.unwrap();
        assert_eq!(manifest.provider.name, "winget");
        assert!(
            !manifest.runtimes.is_empty(),
            "winget should have at least one runtime"
        );
    }

    #[test]
    fn test_nuget_provider_in_registry() {
        let registry = create_registry();
        let runtime = registry.get_runtime("nuget");
        assert!(runtime.is_some(), "nuget runtime should be registered");
    }

    #[test]
    fn test_msbuild_provider_in_registry() {
        let registry = create_registry();
        let runtime = registry.get_runtime("msbuild");
        assert!(runtime.is_some(), "msbuild runtime should be registered");
    }

    #[test]
    fn test_winget_provider_in_registry() {
        let registry = create_registry();
        let runtime = registry.get_runtime("winget");
        assert!(runtime.is_some(), "winget runtime should be registered");
    }
}
