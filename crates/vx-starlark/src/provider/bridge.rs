//! Bridge functions that wire Starlark provider scripts into the vx-runtime
//! function-pointer API (`FetchVersionsFn`, `DownloadUrlFn`, `InstallLayoutFn`).
//!
//! The three public functions (`make_fetch_versions_fn`, `make_download_url_fn`,
//! `make_install_layout_fn`) are the canonical entry-points for single-runtime
//! providers.  The three `*_owned` variants are used internally by
//! [`super::builder`] when building multi-runtime providers.

use std::sync::Arc;

use super::StarlarkProvider;

// ---------------------------------------------------------------------------
// Public single-runtime variants
// ---------------------------------------------------------------------------

/// Create a `FetchVersionsFn` closure backed by an embedded `provider.star`.
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

            Ok(versions_to_runtime(versions))
        })
    }
}

/// Create a `DownloadUrlFn` closure backed by an embedded `provider.star`.
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

/// Create an `InstallLayoutFn` closure backed by an embedded `provider.star`.
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

            let layout = provider
                .install_layout(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{} install_layout failed: {e}", name))?;

            if let Some(l) = layout {
                return Ok(Some(l.to_flat_json()));
            }

            let raw = provider
                .install_layout_raw(&version)
                .await
                .map_err(|e| anyhow::anyhow!("{} install_layout (raw) failed: {e}", name))?;

            Ok(raw)
        })
    }
}

// ---------------------------------------------------------------------------
// Owned-string variants for multi-runtime providers (used by builder.rs)
// ---------------------------------------------------------------------------

pub(super) fn make_fetch_versions_fn_owned(
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

            Ok(versions_to_runtime(versions))
        })
    }
}

pub(super) fn make_download_url_fn_owned(
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

pub(super) fn make_install_layout_fn_owned(
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
                return Ok(Some(l.to_flat_json()));
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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn versions_to_runtime(versions: Vec<crate::context::VersionInfo>) -> Vec<vx_runtime::VersionInfo> {
    versions
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
        .collect()
}
