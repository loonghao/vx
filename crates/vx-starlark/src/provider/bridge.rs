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

pub(super) fn make_deps_fn_owned(
    provider_name: Arc<str>,
    content: Arc<str>,
    runtime_name: String,
) -> impl Fn(
    String,
) -> std::pin::Pin<
    Box<
        dyn std::future::Future<Output = anyhow::Result<Vec<vx_runtime::RuntimeDependency>>> + Send,
    >,
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

            let raw = provider
                .deps_for_runtime(&version, Some(&rt_name))
                .await
                .map_err(|e| {
                    anyhow::anyhow!("{} deps failed for {}: {e}", provider_name, rt_name)
                })?;

            Ok(raw_deps_to_runtime(raw))
        })
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn raw_deps_to_runtime(raw: Vec<serde_json::Value>) -> Vec<vx_runtime::RuntimeDependency> {
    raw.into_iter()
        .filter_map(|item| {
            let name = item.get("runtime").and_then(|v| v.as_str())?.to_string();
            let version_req = item
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .filter(|s| !s.is_empty() && s != "*");
            let optional = item
                .get("optional")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let reason = item
                .get("reason")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            let mut dep = if optional {
                vx_runtime::RuntimeDependency::optional(name)
            } else {
                vx_runtime::RuntimeDependency::required(name)
            };

            if let Some(ref version_req) = version_req {
                dep = dep.with_version(version_req.clone());
                if let Some(min) = extract_min_version(version_req) {
                    dep = dep.with_min_version(min);
                }
                if let Some(max) = extract_max_version(version_req) {
                    dep = dep.with_max_version(max);
                }
            }

            if let Some(reason) = reason {
                dep = dep.with_reason(reason);
            }

            Some(dep)
        })
        .collect()
}

fn extract_min_version(constraint: &str) -> Option<String> {
    for part in constraint.split(',') {
        let part = part.trim();
        if let Some(version) = part.strip_prefix(">=") {
            return Some(version.trim().to_string());
        }
        if let Some(version) = part.strip_prefix('=') {
            return Some(version.trim().to_string());
        }
        if part.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Some(part.to_string());
        }
    }
    None
}

fn extract_max_version(constraint: &str) -> Option<String> {
    for part in constraint.split(',') {
        let part = part.trim();
        if let Some(version) = part.strip_prefix("<=") {
            return Some(version.trim().to_string());
        }
        if let Some(version) = part.strip_prefix('=') {
            return Some(version.trim().to_string());
        }
    }
    None
}

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

// ---------------------------------------------------------------------------
// RFC 0040: Toolchain Version Indirection
// ---------------------------------------------------------------------------

/// Type alias for the `version_info(user_version)` function pointer.
///
/// See `make_version_info_fn_owned` for the canonical way to create this.
pub type VersionInfoFn = Arc<
    dyn Fn(
            String,
        ) -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = anyhow::Result<Option<vx_runtime::VersionInfoResult>>,
                    > + Send,
            >,
        > + Send
        + Sync,
>;

/// Create a `VersionInfoFn` closure backed by an embedded `provider.star`.
///
/// The closure calls `version_info(ctx, user_version)` in Starlark and converts
/// the result to `vx_runtime::VersionInfoResult`.
pub fn make_version_info_fn_owned(
    provider_name: Arc<str>,
    content: Arc<str>,
    runtime_name: String,
) -> VersionInfoFn {
    Arc::new(move |user_version: String| {
        let provider_name = Arc::clone(&provider_name);
        let content = Arc::clone(&content);
        let runtime_name = runtime_name.clone();
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*provider_name, &*content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {} provider.star: {e}", provider_name)
                })?;

            let starlark_result = provider.version_info(&user_version).await.map_err(|e| {
                anyhow::anyhow!(
                    "{} version_info({}) failed: {e}",
                    runtime_name,
                    user_version
                )
            })?;

            // Convert from vx-starlark VersionInfoResult to vx-runtime VersionInfoResult
            Ok(starlark_result.map(|sr| vx_runtime::VersionInfoResult {
                store_as: sr.store_as,
                download_version: sr.download_version,
                install_params: sr.install_params,
            }))
        })
    })
}

// ---------------------------------------------------------------------------
// Post-install hooks (post_extract → post_install bridge)
// ---------------------------------------------------------------------------

/// Create a `PostInstallFn` closure backed by an embedded `provider.star`.
///
/// The closure calls `post_extract(ctx, version, install_dir)` in Starlark and converts
/// the returned `PostExtractAction` values to JSON descriptors that can be executed
/// by `ManifestDrivenRuntime::post_install()`.
pub fn make_post_install_fn_owned(
    provider_name: Arc<str>,
    content: Arc<str>,
    _runtime_name: String,
) -> vx_runtime::manifest_runtime::PostInstallFn {
    Arc::new(move |version: String, install_dir: std::path::PathBuf| {
        let provider_name = Arc::clone(&provider_name);
        let content = Arc::clone(&content);
        Box::pin(async move {
            let provider = StarlarkProvider::from_content(&*provider_name, &*content)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to load {} provider.star: {e}", provider_name)
                })?;

            let actions = provider
                .post_extract(&version, &install_dir)
                .await
                .map_err(|e| {
                    anyhow::anyhow!("{} post_extract failed: {e}", provider_name)
                })?;

            // Convert PostExtractAction to JSON descriptors
            Ok(actions
                .into_iter()
                .filter_map(|action| post_extract_action_to_json(action))
                .collect())
        })
    })
}

/// Convert a `PostExtractAction` to a JSON descriptor for `ManifestDrivenRuntime::post_install`.
fn post_extract_action_to_json(
    action: crate::provider::types::PostExtractAction,
) -> Option<serde_json::Value> {
    use crate::provider::types::PostExtractAction;

    match action {
        PostExtractAction::SetPermissions { path, mode } => Some(serde_json::json!({
            "type": "set_permissions",
            "path": path,
            "mode": mode,
        })),
        PostExtractAction::RunCommand {
            executable,
            args,
            working_dir,
            env,
            on_failure,
        } => Some(serde_json::json!({
            "type": "run_command",
            "executable": executable,
            "args": args,
            "env": env,
            "working_dir": working_dir,
            "on_failure": on_failure,
        })),
        PostExtractAction::CreateShim {
            name,
            target,
            args,
            shim_dir,
        } => Some(serde_json::json!({
            "type": "create_shim",
            "name": name,
            "target": target,
            "args": args,
            "shim_dir": shim_dir,
        })),
        PostExtractAction::FlattenDir { pattern, .. } => Some(serde_json::json!({
            "type": "flatten_dir",
            "pattern": pattern,
        })),
    }
}
