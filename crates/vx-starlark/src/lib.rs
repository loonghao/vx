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
    name: &'static str,
    content: &'static str,
) -> impl Fn() -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Vec<vx_runtime::VersionInfo>>> + Send>,
> + Send
+ Sync
+ 'static {
    move || {
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(name, content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load {name} provider.star: {e}"))?;

            let versions = provider
                .fetch_versions()
                .await
                .map_err(|e| anyhow::anyhow!("{name} fetch_versions failed: {e}"))?;

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
    name: &'static str,
    content: &'static str,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send>,
> + Send
+ Sync
+ 'static {
    move |version: String| {
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(name, content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load {name} provider.star: {e}"))?;

            provider
                .download_url(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{name} download_url failed: {e}"))
        })
    }
}

/// Create an `InstallLayoutFn` closure backed by an embedded `provider.star`.
///
/// The returned JSON value is a serialized `InstallLayout` descriptor that
/// `ManifestDrivenRuntime::install()` uses to determine strip_prefix and
/// executable_paths.
///
/// # Arguments
///
/// * `name` – Provider name used as a virtual script label.
/// * `content` – The raw Starlark source (`PROVIDER_STAR`).
pub fn make_install_layout_fn(
    name: &'static str,
    content: &'static str,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<serde_json::Value>>> + Send>,
> + Send
+ Sync
+ 'static {
    move |version: String| {
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(name, content)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load {name} provider.star: {e}"))?;

            // First try the typed InstallLayout path (handles msi_install, archive_install, etc.)
            let layout = provider
                .install_layout(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{name} install_layout failed: {e}"))?;

            if let Some(l) = layout {
                return Ok(Some(
                    serde_json::to_value(l).unwrap_or(serde_json::Value::Null),
                ));
            }

            // Fallback: call the Starlark engine directly to get the raw JSON dict.
            // This handles install_layout() functions that return plain dicts without
            // a __type field (e.g. { "source_name": ..., "target_name": ..., "target_dir": ... }).
            let raw = provider
                .install_layout_raw(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{name} install_layout (raw) failed: {e}"))?;

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
    provider_name: &'static str,
    content: &'static str,
    runtime_name: String,
) -> impl Fn() -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Vec<vx_runtime::VersionInfo>>> + Send>,
> + Send
+ Sync
+ 'static {
    move || {
        let rt_name = runtime_name.clone();
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(provider_name, content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {provider_name} provider.star: {e}")
                })?;

            let versions = provider
                .fetch_versions_for_runtime(Some(&rt_name))
                .await
                .map_err(|e| anyhow::anyhow!("{provider_name} fetch_versions failed: {e}"))?;

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
    provider_name: &'static str,
    content: &'static str,
    runtime_name: String,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<String>>> + Send>,
> + Send
+ Sync
+ 'static {
    move |version: String| {
        let rt_name = runtime_name.clone();
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(provider_name, content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {provider_name} provider.star: {e}")
                })?;

            provider
                .download_url_for_runtime(&version, Some(&rt_name))
                .await
                .map_err(|e| anyhow::anyhow!("{provider_name} download_url failed: {e}"))
        })
    }
}

/// Like `make_install_layout_fn` but accepts an owned runtime name.
fn make_install_layout_fn_owned(
    provider_name: &'static str,
    content: &'static str,
    runtime_name: String,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = anyhow::Result<Option<serde_json::Value>>> + Send>,
> + Send
+ Sync
+ 'static {
    move |version: String| {
        let rt_name = runtime_name.clone();
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(provider_name, content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {provider_name} provider.star: {e}")
                })?;

            let layout = provider
                .install_layout_for_runtime(&version, Some(&rt_name))
                .await
                .map_err(|e| anyhow::anyhow!("{provider_name} install_layout failed: {e}"))?;

            if let Some(l) = layout {
                return Ok(Some(
                    serde_json::to_value(l).unwrap_or(serde_json::Value::Null),
                ));
            }

            let raw = provider
                .install_layout_raw_for_runtime(&version, Some(&rt_name))
                .await
                .map_err(|e| anyhow::anyhow!("{provider_name} install_layout (raw) failed: {e}"))?;

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
    provider_name: &'static str,
    content: &'static str,
) -> std::sync::Arc<dyn vx_runtime::Provider> {
    struct StarOnlyProvider {
        name: &'static str,
        description: String,
        runtimes: Vec<std::sync::Arc<dyn vx_runtime::Runtime>>,
    }

    impl vx_runtime::Provider for StarOnlyProvider {
        fn name(&self) -> &str {
            self.name
        }

        fn description(&self) -> &str {
            &self.description
        }

        fn runtimes(&self) -> Vec<std::sync::Arc<dyn vx_runtime::Runtime>> {
            self.runtimes.clone()
        }
    }

    let meta = StarMetadata::parse(content);
    let description = meta
        .description
        .clone()
        .unwrap_or_else(|| format!("{} provider", provider_name));

    let runtimes = build_runtimes(provider_name, content, None);

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
    provider_name: &'static str,
    content: &'static str,
    primary_name: Option<&'static str>,
) -> Vec<std::sync::Arc<dyn vx_runtime::Runtime>> {
    use vx_runtime::{Ecosystem, ManifestDrivenRuntime, ProviderSource};

    let meta = StarMetadata::parse(content);

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
            ManifestDrivenRuntime::new(provider_name, provider_name, ProviderSource::BuiltIn)
                .with_ecosystem(ecosystem);
        if let Some(ref pkg) = pip_package {
            rt = rt.with_pip_package(pkg.clone());
        } else {
            rt = rt
                .with_fetch_versions(make_fetch_versions_fn(provider_name, content))
                .with_download_url(make_download_url_fn(provider_name, content))
                .with_install_layout(make_install_layout_fn(provider_name, content));
        }
        return vec![std::sync::Arc::new(rt)];
    }

    // Provider-level platform OS constraint (from `platforms = {"os": [...]}`)
    let provider_platform_os: Vec<String> = meta.platforms.unwrap_or_default();

    let _primary = primary_name.unwrap_or_else(|| {
        // Use the first runtime's name as primary if not specified
        meta.runtimes
            .first()
            .and_then(|rt| rt.name.as_deref())
            .unwrap_or(provider_name)
    });

    meta.runtimes
        .iter()
        .map(|rt| {
            let name = rt.name.clone().unwrap_or_else(|| provider_name.to_string());
            let executable = rt.executable.clone().unwrap_or_else(|| name.clone());
            let description = rt.description.clone().unwrap_or_default();

            let mut runtime =
                ManifestDrivenRuntime::new(name.clone(), provider_name, ProviderSource::BuiltIn)
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
            //
            // Multi-runtime providers (e.g. shell-tools with starship/atuin/yazi) define a
            // single `fetch_versions(ctx)` function that dispatches on `ctx.runtime_name`.
            // Every runtime in the provider must have these functions wired so that version
            // resolution and installation work correctly for each individual tool.
            if let Some(ref pkg) = pip_package {
                // pip package: use PyPI version fetching and pip installation
                runtime = runtime.with_pip_package(pkg.clone());
            } else {
                // Pass the runtime name so that multi-runtime providers can dispatch correctly.
                // For single-runtime providers, runtime_name == provider_name, which is fine.
                let rt_name_owned = name.clone();
                runtime = runtime
                    .with_fetch_versions(make_fetch_versions_fn_owned(
                        provider_name,
                        content,
                        rt_name_owned.clone(),
                    ))
                    .with_download_url(make_download_url_fn_owned(
                        provider_name,
                        content,
                        rt_name_owned.clone(),
                    ))
                    .with_install_layout(make_install_layout_fn_owned(
                        provider_name,
                        content,
                        rt_name_owned,
                    ));
            }

            std::sync::Arc::new(runtime) as std::sync::Arc<dyn vx_runtime::Runtime>
        })
        .collect()
}
