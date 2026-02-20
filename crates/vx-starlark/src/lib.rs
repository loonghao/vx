//! # vx-starlark
//!
//! Starlark scripting support for vx providers.
//!
//! This crate provides:
//! - **Starlark runtime integration** for executing provider scripts
//! - **Sandbox security model** for safe script execution
//! - **ProviderContext API** for Starlark scripts to interact with vx
//! - **Hybrid format support** for both TOML and Starlark providers
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

// Re-exports
pub use context::ProviderContext;
pub use engine::StarlarkEngine;
pub use error::{Error, Result};
pub use handle::{
    PostInstallOps, ProviderHandle, ProviderHandleRegistry, VersionFilter, global_registry,
    global_registry_mut,
};
pub use loader::VxModuleLoader;
pub use metadata::{StarMetadata, StarRuntimeMeta};
pub use provider::{InstallLayout, PostExtractAction, ProviderMeta, RuntimeMeta, StarlarkProvider};
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
