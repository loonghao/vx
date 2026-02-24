//! Provider registry setup and management
//!
//! This module provides the registry setup for all providers.
//! All providers are loaded from `provider.star` files (RFC-0037).
//! Static Rust providers are registered directly into the ProviderRegistry.

use std::sync::Arc;
use tracing::trace;
use vx_paths::{PROJECT_VX_DIR, VxPaths, find_project_root};
use vx_runtime::{
    PluginLoader, Provider, ProviderRegistry, Runtime, RuntimeContext, default_plugin_paths,
};
use vx_runtime_http::create_runtime_context;

// ---------------------------------------------------------------------------
// Compile-time embedded provider.star contents (RFC-0037)
//
// build.rs writes provider_stars.rs:
//   - PROVIDER_STAR_<NAME> : &str  (individual star content, via include_str!)
//   - ALL_PROVIDER_STARS   : &[(&str, &str)]  (name, star_content pairs)
// ---------------------------------------------------------------------------
include!(concat!(env!("OUT_DIR"), "/provider_stars.rs"));

// Include the compile-time generated embedded bridge binaries
mod embedded_bridges {
    include!(concat!(env!("OUT_DIR"), "/embedded_bridges.rs"));
}

/// Register embedded bridge binaries into the global bridge registry.
/// This must be called early in startup, before any provider attempts to deploy bridges.
pub fn register_embedded_bridges() {
    vx_bridge::register_embedded_bridge("MSBuild", embedded_bridges::MSBUILD_BRIDGE_BYTES);
}

/// Master list of all builtin providers.
///
/// To add a new provider: add its name here **once** and nowhere else in this file.
macro_rules! for_each_provider {
    ($macro:ident, $registry:expr) => {
        $macro!(
            $registry,
            // Package managers & system
            bun,
            rcedit,
            choco,
            brew,
            make,
            nasm,
            python,
            msvc,
            ffmpeg,
            imagemagick,
            msbuild,
            actrun,
            hadolint,
            meson,
            curl,
            rez,
            // Cloud CLIs
            awscli,
            azcli,
            gcloud,
            // Container & Kubernetes
            docker,
            kubectl,
            helm,
            // CLI utilities
            bat,
            fd,
            fzf,
            jq,
            yq,
            ripgrep,
            // Build systems
            cmake,
            ninja,
            protoc,
            // Runtimes
            deno,
            java,
            dotnet,
            rust,
            zig,
            go,
            node,
            // Node.js package managers
            pnpm,
            yarn,
            // Package managers
            nuget,
            uv,
            prek,
            vcpkg,
            winget,
            // DevOps & CI tools
            terraform,
            gh,
            task,
            dagu,
            // Shell & terminal
            bash,
            pwsh,
            starship,
            // Version control
            git,
            jj,
            // AI & services
            ollama,
            // Editors & IDEs
            vscode,
            // Security & crypto
            openssl,
            // Python ecosystem
            spack,
            // Build tools
            just,
            // Platform-specific
            systemctl,
            xcodebuild,
        );
    };
}

/// Register all builtin providers into a static `ProviderRegistry`.
macro_rules! register_providers {
    ($registry:expr, $($name:ident),* $(,)?) => {
        $(
            register_providers!(@single $registry, $name);
        )*
    };

    (@single $registry:expr, $name:ident) => {
        paste::paste! {
            $registry.register([<vx_provider_ $name>]::create_provider());
        }
    };
}

/// Create and initialize the provider registry with all available providers.
///
/// Uses static registration for Rust-implemented providers.
/// `provider.star` files are loaded separately via `init_provider_handles()`.
pub fn create_registry() -> ProviderRegistry {
    let registry = create_static_registry();

    if let Some(loader) = build_plugin_loader() {
        registry.set_provider_loader(Arc::new(loader));
    }

    registry
}

/// Build a registry using static registration only.
fn create_static_registry() -> ProviderRegistry {
    let registry = ProviderRegistry::new();

    // Register all builtin providers via the unified master list
    for_each_provider!(register_providers, registry);

    // Special case: 7zip cannot be registered via the macro because "7zip" is not
    // a valid Rust identifier. Register it manually here.
    registry.register(vx_provider_7zip::create_provider());

    // Special case: pre-commit and release-please contain hyphens which are not
    // valid Rust identifiers. Register them manually here.
    registry.register(vx_provider_pre_commit::create_provider());
    registry.register(vx_provider_release_please::create_provider());

    registry
}

/// Load user and project-level `provider.star` overrides.
///
/// Returns a list of `(name, star_content)` pairs from:
/// 1. `~/.vx/providers/*/provider.star` (user-level)
/// 2. `<project>/.vx/providers/*/provider.star` (project-level)
pub fn load_star_overrides() -> Vec<(String, String)> {
    let mut overrides = Vec::new();

    // User-level: ~/.vx/providers
    if let Ok(paths) = VxPaths::new() {
        let user_dir = paths.base_dir.join("providers");
        collect_star_files(&user_dir, &mut overrides);
    }

    // Project-level: <project>/.vx/providers
    if let Ok(cwd) = std::env::current_dir()
        && let Some(project_root) = find_project_root(&cwd)
    {
        let project_dir = project_root.join(PROJECT_VX_DIR).join("providers");
        collect_star_files(&project_dir, &mut overrides);
    }

    overrides
}

fn collect_star_files(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
    if !dir.exists() {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let star_path = entry.path().join("provider.star");
        if star_path.exists()
            && let Ok(content) = std::fs::read_to_string(&star_path)
        {
            let name = entry.file_name().to_string_lossy().to_string();
            out.push((name, content));
        }
    }
}

fn build_plugin_loader() -> Option<PluginLoader> {
    let mut paths = Vec::new();

    if let Ok(vx_paths) = VxPaths::new() {
        paths.extend(default_plugin_paths(std::slice::from_ref(
            &vx_paths.base_dir,
        )));
    }

    if let Ok(cwd) = std::env::current_dir()
        && let Some(project_root) = find_project_root(&cwd)
    {
        paths.push(project_root.join(PROJECT_VX_DIR).join("plugins"));
    }

    paths.retain(|p| p.exists());
    if paths.is_empty() {
        return None;
    }

    Some(PluginLoader::new(paths))
}

/// Initialize the global ProviderHandle registry with all built-in providers (RFC-0037)
///
/// This function registers all embedded `provider.star` files into the
/// `global_registry()` so that CLI commands can use `ProviderHandle` for
/// path queries, version management, and post-install operations.
///
/// Should be called once at CLI startup, before any command is dispatched.
pub async fn init_provider_handles() {
    use vx_starlark::handle::global_registry_mut;

    let mut reg = global_registry_mut().await;
    for (name, star_content) in ALL_PROVIDER_STARS {
        match reg.register_builtin(name, star_content).await {
            Ok(()) => {
                trace!(provider = %name, "Registered ProviderHandle");
            }
            Err(e) => {
                // Non-fatal: log and continue so other providers still load
                tracing::warn!(
                    provider = %name,
                    error = %e,
                    "Failed to register ProviderHandle — provider.star may have errors"
                );
            }
        }
    }
}

/// Build a RuntimeMap from the global ProviderHandle registry (RFC-0037)
///
/// This replaces the old `RuntimeMap::from_manifests()` approach.
/// The RuntimeMap is built from `provider.star` metadata loaded into
/// `GLOBAL_REGISTRY` at startup via `init_provider_handles()`.
///
/// Falls back to an empty RuntimeMap if the registry is not yet initialized.
pub fn build_runtime_map() -> vx_resolver::RuntimeMap {
    use vx_resolver::{RuntimeMap, RuntimeSpec};

    let Ok(registry) = vx_starlark::handle::GLOBAL_REGISTRY.try_read() else {
        return RuntimeMap::empty();
    };

    let mut map = RuntimeMap::empty();

    for (_name, handle) in registry.iter() {
        for runtime_meta in handle.runtime_metas() {
            let mut spec = RuntimeSpec::new(&runtime_meta.name, &runtime_meta.description);
            spec.executable = Some(runtime_meta.executable.clone());
            spec.aliases = runtime_meta.aliases.clone();
            spec.priority = runtime_meta.priority as i32;
            spec.command_prefix = runtime_meta.command_prefix.clone();
            map.register(spec);
        }
    }

    map
}

/// Get platform label for a runtime from the ProviderHandle registry.
///
/// Returns the platform label (e.g., "Windows", "macOS") if the runtime
/// has platform constraints, or None if it supports all platforms.
/// This is a best-effort synchronous lookup; for full accuracy use the async API.
pub fn get_runtime_platform_label(runtime_name: &str) -> Option<String> {
    // Try to get from ProviderHandle registry (provider.star)
    // This is a sync wrapper — for now we check ALL_PROVIDER_STARS metadata
    // A full implementation would query ProviderMeta.platforms from the handle
    let _ = runtime_name;
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

// =============================================================================
// Build Diagnostics Storage (RFC 0029 Phase 2)
// =============================================================================

use std::sync::OnceLock;

/// Stored build diagnostics from `create_registry()`
///
/// Accessible via `get_build_diagnostics()` for `vx info --warnings`.
#[derive(Debug, Clone, Default)]
pub struct BuildDiagnostics {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

static BUILD_DIAGNOSTICS: OnceLock<BuildDiagnostics> = OnceLock::new();

/// Get build diagnostics from the last `create_registry()` call
pub fn get_build_diagnostics() -> &'static BuildDiagnostics {
    static EMPTY: OnceLock<BuildDiagnostics> = OnceLock::new();
    BUILD_DIAGNOSTICS
        .get()
        .unwrap_or_else(|| EMPTY.get_or_init(BuildDiagnostics::default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_stars_exist() {
        assert!(
            !ALL_PROVIDER_STARS.is_empty(),
            "Expected embedded provider.star files, found none"
        );
    }

    #[test]
    fn test_create_registry_has_providers() {
        let registry = create_registry();
        assert!(
            !registry.runtime_names().is_empty(),
            "Registry should have providers"
        );
    }
}
