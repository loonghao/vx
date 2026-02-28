//! # vx-starlark
//!
//! Starlark scripting support for vx providers.
//!
//! This crate provides:
//! - **Starlark runtime integration** for executing provider scripts
//! - **Sandbox security model** for safe script execution
//! - **ProviderContext API** for Starlark scripts to interact with vx
//! - **@vx//stdlib module system** for shared utilities (Buck2-inspired load())
//! - **Two-phase execution** (Analysis → Execution, Buck2-inspired)
//! - **Incremental analysis cache** (content-hash based, Buck2-inspired)
//!
//! ## Overview
//!
//! ```ignore
//! use vx_starlark::{StarlarkProvider, SandboxConfig};
//!
//! // Load a Starlark provider
//! let provider = StarlarkProvider::load("path/to/provider.star").await?;
//!
//! // Call provider functions
//! let versions = provider.fetch_versions().await?;
//! ```

pub mod context;
pub mod engine;
pub mod error;
pub mod handle;
pub mod loader;
pub mod metadata;
pub mod provider;
pub mod sandbox;
pub mod stdlib;

/// Test mocks for provider tests (only available with #[cfg(test)] or in dev builds)
#[cfg(any(test, feature = "test-mocks"))]
pub mod test_mocks;

// Re-exports
pub use context::ProviderContext;
pub use engine::{ProviderLint, StarlarkEngine};
pub use error::{Error, Result};
pub use handle::{
    PostInstallOps, ProviderHandle, ProviderHandleRegistry, VersionFilter, global_registry,
    global_registry_mut,
};
pub use loader::VxModuleLoader;
pub use metadata::{StarMetadata, StarRuntimeMeta};
pub use provider::version_cache::{
    DEFAULT_VERSION_CACHE_TTL_SECS, DEV_VERSION_CACHE_TTL_SECS, VersionCache, VersionCacheStats,
    global_version_cache,
};
pub use provider::{
    EnvOp, InstallLayout, PostExtractAction, ProviderMeta, RuntimeMeta, StarlarkProvider,
    apply_env_ops,
};
pub use sandbox::SandboxConfig;

use std::sync::Arc;

/// Starlark provider file extension
pub const STARLARK_EXTENSION: &str = "star";

/// Default provider filename for Starlark
pub const PROVIDER_FILENAME: &str = "provider.star";

/// Create a `FetchVersionsFn` closure backed by an embedded `provider.star`.
///
/// This is the canonical way for every `ManifestDrivenRuntime`-based provider
/// to wire its embedded Starlark `fetch_versions` into the runtime system.
///
/// # Arguments
///
/// * `name` – Provider name used as a virtual script label (e.g. `"go"`, `"node"`).
/// * `content` – The raw Starlark source, typically `PROVIDER_STAR` from the
///   provider crate's `lib.rs` (`include_str!("../provider.star")`).
///
/// # Returns
///
/// An `Arc`-wrapped async closure compatible with
/// [`ManifestDrivenRuntime::with_fetch_versions`].
///
/// # Example
///
/// ```rust,ignore
/// use vx_runtime::{ManifestDrivenRuntime, ProviderSource};
/// use vx_starlark::make_fetch_versions_fn;
///
/// let runtime = ManifestDrivenRuntime::new("go", "go", ProviderSource::BuiltIn)
///     .with_fetch_versions(make_fetch_versions_fn("go", crate::PROVIDER_STAR));
/// ```
pub fn make_fetch_versions_fn(
    name: impl Into<String>,
    content: impl Into<String>,
) -> impl Fn() -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Vec<vx_runtime::VersionInfo>>> + Send>,
> + Send
+ Sync
+ 'static {
    let name: Arc<str> = Arc::from(name.into());
    let content: Arc<str> = Arc::from(content.into());
    move || {
        let name = Arc::clone(&name);
        let content = Arc::clone(&content);
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*name, &*content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load {} provider.star: {e}", name))?;

            let versions = provider
                .fetch_versions()
                .await
                .map_err(|e| anyhow::anyhow!("{} fetch_versions failed: {e}", name))?;

            Ok(versions
                .into_iter()
                .map(|v| vx_runtime::VersionInfo {
                    version: v.version,
                    released_at: v.date.and_then(|d| {
                        chrono::DateTime::parse_from_rfc3339(&d)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    prerelease: !v.stable,
                    lts: v.lts,
                    download_url: None,
                    checksum: None,
                    metadata: std::collections::HashMap::new(),
                })
                .collect())
        })
    }
}

/// Create a `DownloadUrlFn` closure backed by an embedded `provider.star`.
///
/// This is the canonical way for every `ManifestDrivenRuntime`-based provider
/// to wire its embedded Starlark `download_url` into the runtime system.
///
/// # Arguments
///
/// * `name` – Provider name used as a virtual script label.
/// * `content` – The raw Starlark source (`PROVIDER_STAR`).
pub fn make_download_url_fn(
    name: impl Into<String>,
    content: impl Into<String>,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send>,
> + Send
+ Sync
+ 'static {
    let name: Arc<str> = Arc::from(name.into());
    let content: Arc<str> = Arc::from(content.into());
    move |version: String| {
        let name = Arc::clone(&name);
        let content = Arc::clone(&content);
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*name, &*content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load {} provider.star: {e}", name))?;

            provider
                .download_url(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{} download_url failed: {e}", name))
        })
    }
}

/// Convert an `InstallLayout` enum into the flat JSON dict that
/// `manifest_runtime` expects (keys at the top level, not nested under
/// the enum variant name).
///
/// `serde_json::to_value` on an untagged enum produces `{"Archive": {...}}`
/// which `manifest_runtime` cannot read directly. This helper flattens it
/// to `{"strip_prefix": "...", "executable_paths": [...], ...}`.
fn install_layout_to_flat_json(layout: crate::provider::types::InstallLayout) -> serde_json::Value {
    use crate::provider::types::InstallLayout;
    match layout {
        InstallLayout::Archive {
            url,
            strip_prefix,
            executable_paths,
        } => {
            let mut map = serde_json::Map::new();
            if let Some(u) = url {
                map.insert("url".into(), serde_json::Value::String(u));
            }
            if let Some(sp) = strip_prefix {
                map.insert("strip_prefix".into(), serde_json::Value::String(sp));
            }
            map.insert(
                "executable_paths".into(),
                serde_json::Value::Array(
                    executable_paths
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
            serde_json::Value::Object(map)
        }
        InstallLayout::Binary {
            url,
            executable_name,
            permissions,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("url".into(), serde_json::Value::String(url));
            if let Some(n) = executable_name {
                map.insert("executable_name".into(), serde_json::Value::String(n));
            }
            map.insert("permissions".into(), serde_json::Value::String(permissions));
            serde_json::Value::Object(map)
        }
        InstallLayout::Msi {
            url,
            executable_paths,
            strip_prefix,
            extra_args,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("url".into(), serde_json::Value::String(url));
            map.insert(
                "executable_paths".into(),
                serde_json::Value::Array(
                    executable_paths
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
            if let Some(sp) = strip_prefix {
                map.insert("strip_prefix".into(), serde_json::Value::String(sp));
            }
            map.insert(
                "extra_args".into(),
                serde_json::Value::Array(
                    extra_args
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
            serde_json::Value::Object(map)
        }
        InstallLayout::SystemFind {
            executable,
            system_paths,
            hint,
        } => {
            let mut map = serde_json::Map::new();
            map.insert("executable".into(), serde_json::Value::String(executable));
            map.insert(
                "system_paths".into(),
                serde_json::Value::Array(
                    system_paths
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );
            if let Some(h) = hint {
                map.insert("hint".into(), serde_json::Value::String(h));
            }
            serde_json::Value::Object(map)
        }
    }
}

/// Create an `InstallLayoutFn` closure backed by an embedded `provider.star`.
///
/// The returned JSON value is a flat dict that `ManifestDrivenRuntime::install()`
/// uses to determine `strip_prefix` and `executable_paths`.
///
/// # Arguments
///
/// * `name` – Provider name used as a virtual script label.
/// * `content` – The raw Starlark source (`PROVIDER_STAR`).
pub fn make_install_layout_fn(
    name: impl Into<String>,
    content: impl Into<String>,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<serde_json::Value>>> + Send>,
> + Send
+ Sync
+ 'static {
    let name: Arc<str> = Arc::from(name.into());
    let content: Arc<str> = Arc::from(content.into());
    move |version: String| {
        let name = Arc::clone(&name);
        let content = Arc::clone(&content);
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*name, &*content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load {} provider.star: {e}", name))?;

            // First try the typed InstallLayout path (handles msi_install, archive_install, etc.)
            let layout = provider
                .install_layout(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{} install_layout failed: {e}", name))?;

            if let Some(l) = layout {
                // Convert to flat JSON so manifest_runtime can read strip_prefix /
                // executable_paths directly without knowing the enum variant name.
                return Ok(Some(install_layout_to_flat_json(l)));
            }

            // Fallback: call the Starlark engine directly to get the raw JSON dict.
            // This handles install_layout() functions that return plain dicts without
            // a __type field (e.g. { "source_name": ..., "target_name": ..., "target_dir": ... }).
            let raw = provider
                .install_layout_raw(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{} install_layout (raw) failed: {e}", name))?;

            Ok(raw)
        })
    }
}

// ---------------------------------------------------------------------------
// Owned-string variants for multi-runtime providers
//
// These are identical to the `&'static str` variants above, but accept an
// owned `String` for the runtime name so that `build_runtimes` can pass the
// per-runtime name captured from the provider.star metadata.
// ---------------------------------------------------------------------------

/// Like `make_fetch_versions_fn` but accepts an owned runtime name.
/// Used by `build_runtimes` to wire each runtime in a multi-runtime provider
/// with the correct `ctx.runtime_name` so that `fetch_versions(ctx)` can
/// dispatch to the right GitHub repo / version source.
fn make_fetch_versions_fn_owned(
    provider_name: Arc<str>,
    content: Arc<str>,
    runtime_name: String,
) -> impl Fn() -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Vec<vx_runtime::VersionInfo>>> + Send>,
> + Send
+ Sync
+ 'static {
    move || {
        let provider_name = Arc::clone(&provider_name);
        let content = Arc::clone(&content);
        let rt_name = runtime_name.clone();
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*provider_name, &*content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {} provider.star: {e}", provider_name)
                })?;

            let versions = provider
                .fetch_versions_for_runtime(Some(&rt_name))
                .await
                .map_err(|e| anyhow::anyhow!("{} fetch_versions failed: {e}", provider_name))?;

            Ok(versions
                .into_iter()
                .map(|v| vx_runtime::VersionInfo {
                    version: v.version,
                    released_at: v.date.and_then(|d| {
                        chrono::DateTime::parse_from_rfc3339(&d)
                            .ok()
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                    }),
                    prerelease: !v.stable,
                    lts: v.lts,
                    download_url: None,
                    checksum: None,
                    metadata: std::collections::HashMap::new(),
                })
                .collect())
        })
    }
}

/// Like `make_download_url_fn` but accepts an owned runtime name.
fn make_download_url_fn_owned(
    provider_name: Arc<str>,
    content: Arc<str>,
    runtime_name: String,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send>,
> + Send
+ Sync
+ 'static {
    move |version: String| {
        let provider_name = Arc::clone(&provider_name);
        let content = Arc::clone(&content);
        let rt_name = runtime_name.clone();
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*provider_name, &*content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {} provider.star: {e}", provider_name)
                })?;

            provider
                .download_url_for_runtime(&version, Some(&rt_name))
                .await
                .map_err(|e| anyhow::anyhow!("{} download_url failed: {e}", provider_name))
        })
    }
}

/// Like `make_install_layout_fn` but accepts an owned runtime name.
fn make_install_layout_fn_owned(
    provider_name: Arc<str>,
    content: Arc<str>,
    runtime_name: String,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<serde_json::Value>>> + Send>,
> + Send
+ Sync
+ 'static {
    move |version: String| {
        let provider_name = Arc::clone(&provider_name);
        let content = Arc::clone(&content);
        let rt_name = runtime_name.clone();
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*provider_name, &*content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {} provider.star: {e}", provider_name)
                })?;

            let layout = provider
                .install_layout_for_runtime(&version, Some(&rt_name))
                .await
                .map_err(|e| anyhow::anyhow!("{} install_layout failed: {e}", provider_name))?;

            if let Some(l) = layout {
                return Ok(Some(install_layout_to_flat_json(l)));
            }

            let raw = provider
                .install_layout_raw_for_runtime(&version, Some(&rt_name))
                .await
                .map_err(|e| {
                    anyhow::anyhow!("{} install_layout (raw) failed: {e}", provider_name)
                })?;

            Ok(raw)
        })
    }
}

/// Create an `Arc<dyn Provider>` from embedded `provider.star` content.
///
/// This is the canonical way to register a star-only provider into the
/// `ProviderRegistry` without any hand-written Rust `Provider` impl.
///
/// # Arguments
///
/// * `provider_name` – Provider name (e.g. `"go"`, `"node"`).
/// * `content` – The raw Starlark source (`PROVIDER_STAR`).
///
/// # Example
///
/// ```rust,ignore
/// use vx_starlark::create_provider;
///
/// registry.register(create_provider("cmake", vx_provider_cmake::PROVIDER_STAR));
/// ```
pub fn create_provider(
    provider_name: impl Into<String>,
    content: impl Into<String>,
) -> std::sync::Arc<dyn vx_runtime::Provider> {
    struct StarOnlyProvider {
        name: String,
        description: String,
        runtimes: Vec<std::sync::Arc<dyn vx_runtime::Runtime>>,
    }

    impl vx_runtime::Provider for StarOnlyProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn runtimes(&self) -> Vec<std::sync::Arc<dyn vx_runtime::Runtime>> {
            self.runtimes.clone()
        }
    }

    let provider_name = provider_name.into();
    let content = content.into();
    let meta = StarMetadata::parse(&content);
    let description = meta
        .description
        .clone()
        .unwrap_or_else(|| format!("{} provider", provider_name));

    let runtimes = build_runtimes(provider_name.clone(), content, None::<String>);

    std::sync::Arc::new(StarOnlyProvider {
        name: provider_name,
        description,
        runtimes,
    })
}

/// Build a list of `ManifestDrivenRuntime` instances from a `provider.star` file.
///
/// This is the canonical way for every `ManifestDrivenRuntime`-based provider
/// to build its runtime list from the embedded Starlark metadata.  It reads
/// all runtime definitions from the star file (including aliases) and creates
/// a `ManifestDrivenRuntime` for each one.
///
/// For the **primary** runtime (the first one in the list, or the one whose
/// name matches `primary_name`), `fetch_versions`, `download_url` and
/// `install_layout` functions are all wired in from the Starlark script.
///
/// # Arguments
///
/// * `provider_name` – Provider name (e.g. `"go"`, `"node"`).
/// * `content` – The raw Starlark source (`PROVIDER_STAR`).
/// * `primary_name` – Name of the runtime that should have functions wired.
///   Pass `None` to wire the first runtime.
///
/// # Example
///
/// ```rust,ignore
/// use vx_starlark::build_runtimes;
///
/// fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
///     build_runtimes("cmake", crate::PROVIDER_STAR, None)
/// }
/// ```
pub fn build_runtimes(
    provider_name: impl Into<String>,
    content: impl Into<String>,
    primary_name: Option<impl Into<String>>,
) -> Vec<Arc<dyn vx_runtime::Runtime>> {
    use vx_runtime::{Ecosystem, ManifestDrivenRuntime, ProviderSource};

    let provider_name: Arc<str> = Arc::from(provider_name.into());
    let content: Arc<str> = Arc::from(content.into());
    let _primary_name: Option<String> = primary_name.map(|s| s.into());
    let meta = StarMetadata::parse(&content);

    // Parse ecosystem from provider metadata
    let ecosystem = match meta.ecosystem.as_deref() {
        Some("nodejs") | Some("node") => Ecosystem::NodeJs,
        Some("python") => Ecosystem::Python,
        Some("rust") => Ecosystem::Rust,
        Some("go") => Ecosystem::Go,
        Some("git") => Ecosystem::Git,
        Some("dotnet") => Ecosystem::Dotnet,
        Some("system") => Ecosystem::System,
        Some(other) => Ecosystem::Custom(other.to_string()),
        None => Ecosystem::Unknown,
    };

    // pip_package: Python tools installed via uv pip (e.g. meson, black, ruff)
    let pip_package = meta.pip_package.clone();

    if meta.runtimes.is_empty() {
        // Fallback: create a single runtime with the provider name
        let mut rt =
            ManifestDrivenRuntime::new(&*provider_name, &*provider_name, ProviderSource::BuiltIn)
                .with_ecosystem(ecosystem);
        if let Some(ref pkg) = pip_package {
            rt = rt.with_pip_package(pkg.clone());
        } else {
            rt = rt
                .with_fetch_versions(make_fetch_versions_fn(
                    Arc::clone(&provider_name).to_string(),
                    Arc::clone(&content).to_string(),
                ))
                .with_download_url(make_download_url_fn(
                    Arc::clone(&provider_name).to_string(),
                    Arc::clone(&content).to_string(),
                ))
                .with_install_layout(make_install_layout_fn(
                    Arc::clone(&provider_name).to_string(),
                    Arc::clone(&content).to_string(),
                ));
        }
        return vec![Arc::new(rt)];
    }

    // Provider-level platform OS constraint (from `platforms = {"os": [...]}`)
    let provider_platform_os: Vec<String> = meta.platforms.unwrap_or_default();

    let _primary = _primary_name.unwrap_or_else(|| {
        // Use the first runtime's name as primary if not specified
        meta.runtimes
            .first()
            .and_then(|rt| rt.name.as_deref())
            .unwrap_or(&provider_name)
            .to_string()
    });

    meta.runtimes
        .iter()
        .map(|rt| {
            let name = rt.name.clone().unwrap_or_else(|| provider_name.to_string());
            let executable = rt.executable.clone().unwrap_or_else(|| name.clone());
            let description = rt.description.clone().unwrap_or_default();

            let mut runtime =
                ManifestDrivenRuntime::new(name.clone(), &*provider_name, ProviderSource::BuiltIn)
                    .with_executable(executable)
                    .with_description(description)
                    .with_aliases(rt.aliases.clone())
                    .with_ecosystem(ecosystem.clone());

            // Set bundled_with if present
            if let Some(ref bundled) = rt.bundled_with {
                runtime = runtime.with_bundled_with(bundled.clone());
            }

            // Set platform_os constraint if present (e.g. macOS-only tools).
            // Runtime-level constraint takes priority; fall back to provider-level.
            let effective_platform_os = if !rt.platform_os.is_empty() {
                rt.platform_os.clone()
            } else {
                provider_platform_os.clone()
            };
            if !effective_platform_os.is_empty() {
                runtime = runtime.with_platform_os(effective_platform_os);
            }

            // Set install_deps if present (RFC 0021)
            if !rt.install_deps.is_empty() {
                runtime = runtime.with_install_deps(rt.install_deps.clone());
            }

            // Set shells if present (RFC 0038)
            if !rt.shells.is_empty() {
                use vx_runtime::manifest_runtime::ShellDefinition;
                let shells: Vec<ShellDefinition> = rt
                    .shells
                    .iter()
                    .map(|(name, path)| ShellDefinition {
                        name: name.clone(),
                        path: path.clone(),
                    })
                    .collect();
                runtime = runtime.with_shells(shells);
            }

            // Wire fetch_versions, download_url, install_layout for all runtimes.
            if let Some(ref pkg) = pip_package {
                runtime = runtime.with_pip_package(pkg.clone());
            } else {
                let rt_name_owned = name.clone();
                runtime = runtime
                    .with_fetch_versions(make_fetch_versions_fn_owned(
                        Arc::clone(&provider_name),
                        Arc::clone(&content),
                        rt_name_owned.clone(),
                    ))
                    .with_download_url(make_download_url_fn_owned(
                        Arc::clone(&provider_name),
                        Arc::clone(&content),
                        rt_name_owned.clone(),
                    ))
                    .with_install_layout(make_install_layout_fn_owned(
                        Arc::clone(&provider_name),
                        Arc::clone(&content),
                        rt_name_owned,
                    ));
            }

            Arc::new(runtime) as Arc<dyn vx_runtime::Runtime>
        })
        .collect()
}
