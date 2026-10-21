//! Installation logic for manifest-driven runtimes.
//!
//! This module handles the `install()` method and all strategy dispatch:
//! - Starlark-driven install_layout (direct download with custom strip_prefix)
//! - Direct download URL (from `download_url_fn` or `InstallStrategy::DirectDownload`)
//! - System package managers (brew, choco, apt, etc.)
//! - Script-based installation
//! - ProvidedBy (executable from another runtime)

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use tracing::{debug, info, warn};
use vx_system_pm::{PackageInstallSpec, PackageManagerRegistry};

use crate::runtime::install_impl::InstallLock;
use crate::{InstallResult, Runtime, RuntimeContext, platform::Platform};

use super::ManifestDrivenRuntime;
use super::types::InstallStrategy;

impl ManifestDrivenRuntime {
    /// Install the runtime using the best available strategy.
    ///
    /// Tries strategies in priority order:
    /// 1. pip package (`uv pip install <pkg>==<version>`)
    /// 2. Starlark-driven `install_layout` (URL + strip_prefix + exe paths)
    /// 3. Direct download URL (from `download_url_fn` or `DirectDownload` strategy)
    /// 4. System package managers (brew, choco, apt, etc.)
    /// 5. Script-based installation
    pub async fn install_impl(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let platform = Platform::current();
        let store_name = self.bundled_with.as_deref().unwrap_or(&self.name);
        let base_path = ctx.paths.version_store_dir(store_name, version);
        // New layout: install directly to version dir (no platform subdirectory).
        // Old layout used base_path.join(platform.as_str()).
        let install_path = base_path;

        // pip package: install via uv pip install <package>==<version>
        if let Some(ref pkg) = self.pip_package {
            return crate::package_runtime::install_pip_package_for_manifest(
                pkg, &self.name, version, ctx,
            )
            .await;
        }

        // Protect the physical store shared by primary and bundled runtimes.
        // Keep the guard through cache checks, cleanup and archive processing.
        let _install_lock = InstallLock::acquire(&install_path).await?;

        // Try Starlark-driven install_layout first (provides URL + strip_prefix + exe paths)
        let layout_hint = self.resolve_layout_hint(version).await?;
        if let Some(ref layout) = layout_hint {
            let url = layout
                .get("url")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            if let Some(url) = url {
                info!(
                    "Installing {} via Starlark install_layout from {}",
                    self.name, url
                );

                if ctx.fs.exists(&install_path)
                    && self.layout_executable_exists(&install_path, layout, ctx)
                {
                    let exe_path = self.resolve_exe_path_from_layout(&install_path, layout);
                    return Ok(InstallResult::already_installed(
                        install_path,
                        exe_path,
                        version.to_string(),
                    ));
                }
                if ctx.fs.exists(&install_path) {
                    ctx.fs.remove_dir_all(&install_path)?;
                }

                self.download_layout_sources(&url, &install_path, Some(layout), ctx)
                    .await?;

                let exe_path = self.resolve_exe_path_from_layout(&install_path, layout);
                return Ok(InstallResult::success(
                    install_path,
                    exe_path,
                    version.to_string(),
                ));
            }
        }

        // Try direct download URL
        if let Some(url) = self.download_url(version, &platform).await? {
            return self
                .install_via_direct_download(
                    version,
                    &url,
                    &install_path,
                    layout_hint.as_ref(),
                    ctx,
                )
                .await;
        }

        // No direct download — try system package managers and scripts
        self.install_via_system_strategies(version, &platform, &install_path, ctx)
            .await
    }

    /// Install via a direct download URL, using install_layout hints for strip_prefix.
    async fn install_via_direct_download(
        &self,
        version: &str,
        url: &str,
        install_path: &std::path::Path,
        layout_hint: Option<&serde_json::Value>,
        ctx: &RuntimeContext,
    ) -> Result<InstallResult> {
        info!("Installing {} via direct download from {}", self.name, url);

        if ctx.fs.exists(install_path)
            && layout_hint
                .is_none_or(|layout| self.layout_executable_exists(install_path, layout, ctx))
        {
            let exe_path = if let Some(layout) = layout_hint {
                self.resolve_exe_path_from_layout(install_path, layout)
            } else {
                install_path.join(vx_paths::with_executable_extension(&self.executable))
            };
            return Ok(InstallResult::already_installed(
                install_path.to_path_buf(),
                exe_path,
                version.to_string(),
            ));
        }
        if ctx.fs.exists(install_path) {
            ctx.fs.remove_dir_all(install_path)?;
        }

        self.download_layout_sources(url, install_path, layout_hint, ctx)
            .await?;

        let exe_path = if let Some(layout) = layout_hint {
            self.resolve_exe_path_from_layout(install_path, layout)
        } else {
            install_path.join(vx_paths::with_executable_extension(&self.executable))
        };
        Ok(InstallResult::success(
            install_path.to_path_buf(),
            exe_path,
            version.to_string(),
        ))
    }

    /// Resolve the install_layout hint (strip_prefix, executable_paths) for a version.
    ///
    /// Calls `install_layout_fn` once. Returns `None` if
    /// no layout function is set or the function returns `None`.
    async fn resolve_layout_hint(&self, version: &str) -> Result<Option<serde_json::Value>> {
        let Some(layout_fn) = self.install_layout_fn.as_ref() else {
            return Ok(None);
        };
        // A malformed integrity contract must never degrade to an unchecked download.
        layout_fn(version.to_string()).await
    }

    async fn download_layout_sources(
        &self,
        origin: &str,
        install_path: &std::path::Path,
        layout: Option<&serde_json::Value>,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        let metadata = build_layout_meta(layout)?;
        let mut sources = Vec::new();
        if let Some(mirrors) = layout.and_then(|value| value.get("mirror_urls")) {
            let mirrors = mirrors
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("mirror_urls must be a list"))?;
            if !mirrors.is_empty() && !metadata.contains_key("sha256") {
                anyhow::bail!("Artifact mirrors require a fixed SHA256 digest");
            }
            for mirror in mirrors {
                let mirror = mirror
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("mirror_urls must contain strings"))?;
                if !sources.iter().any(|source| source == mirror) {
                    sources.push(mirror.to_string());
                }
            }
        }
        if !sources.iter().any(|source| source == origin) {
            sources.push(origin.to_string());
        }
        let mut last_error = None;
        for source in sources {
            let result = ctx
                .installer
                .download_with_layout(&source, install_path, &metadata)
                .await.and_then(|()| {
                    if let Some(digest) = metadata.get("sha256") {
                        let layout = layout.expect("a digest originates in a layout");
                        if !self.layout_files_exist(install_path, layout, ctx) {
                            anyhow::bail!("Verified artifact is missing its executable or required build files");
                        }
                        // Readers hold this same store lock. An interrupted write
                        // leaves an invalid receipt and will force a fresh install.
                        ctx.fs.write(&install_path.join(".vx-artifact.json"), &serde_json::json!({
                            "schema_version": 1,
                            "sha256": digest,
                            "source": source,
                        }).to_string())?;
                    }
                    Ok(())
                });
            match result {
                Ok(()) => return Ok(()),
                Err(error) => {
                    warn!(%source, %error, "Artifact source failed; retaining the fixed digest for fallback");
                    if ctx.fs.exists(install_path) {
                        ctx.fs.remove_dir_all(install_path)?;
                    }
                    last_error = Some(error);
                }
            }
        }
        Err(last_error.expect("the origin always supplies one source"))
    }

    /// Install via system package managers and script strategies.
    async fn install_via_system_strategies(
        &self,
        _version: &str,
        platform: &Platform,
        _install_path: &std::path::Path,
        _ctx: &RuntimeContext,
    ) -> Result<InstallResult> {
        info!(
            "No direct download for {} on {:?}, trying system package managers",
            self.name, platform.os
        );

        let mut strategies: Vec<_> = self
            .install_strategies
            .iter()
            .filter(|s| s.matches_platform(platform))
            .collect();
        strategies.sort_by_key(|s| std::cmp::Reverse(s.priority()));

        let registry = PackageManagerRegistry::new();
        let available_managers = registry.get_available().await;

        for strategy in strategies {
            match strategy {
                InstallStrategy::PackageManager {
                    manager,
                    package,
                    params,
                    install_args,
                    ..
                } => {
                    let pm = available_managers
                        .iter()
                        .find(|pm| pm.name().eq_ignore_ascii_case(manager));

                    if let Some(pm) = pm {
                        debug!(
                            "Trying to install {} via {} (package: {})",
                            self.name, manager, package
                        );

                        let mut spec = PackageInstallSpec::new(package.clone());
                        spec.params = params.clone();
                        spec.install_args = install_args.clone();

                        match pm.install_package(&spec).await {
                            Ok(_) => {
                                info!("Successfully installed {} via {}", self.name, manager);
                                // For tools like MSVC cl.exe that are not on PATH,
                                // search system_paths glob patterns first, then fall back to which.
                                let exe_path = if !self.system_paths.is_empty() {
                                    super::find_first_glob_match(&self.system_paths)
                                        .or_else(|| which::which(&self.executable).ok())
                                } else {
                                    which::which(&self.executable).ok()
                                };
                                return Ok(InstallResult::system_installed(
                                    format!("system ({})", manager),
                                    exe_path,
                                ));
                            }
                            Err(e) => {
                                warn!("Failed to install {} via {}: {}", self.name, manager, e);
                                continue;
                            }
                        }
                    } else {
                        debug!("Package manager {} not available, skipping", manager);
                    }
                }
                InstallStrategy::Script {
                    url,
                    script_type,
                    args,
                    ..
                } => {
                    debug!("Script installation not yet implemented for {}", self.name);
                    // TODO: Implement script-based installation
                    let _ = (url, script_type, args);
                }
                InstallStrategy::ProvidedBy {
                    provider,
                    relative_path,
                    ..
                } => {
                    if which::which(provider).is_ok() {
                        debug!("{} is provided by {}", self.name, provider);
                        let exe_path = PathBuf::from(relative_path);
                        return Ok(InstallResult::system_installed(
                            format!("provided by {}", provider),
                            Some(exe_path),
                        ));
                    }
                }
                InstallStrategy::DirectDownload { .. } => {
                    // Already tried above
                }
            }
        }

        // All strategies failed
        let tried_managers: Vec<_> = self
            .install_strategies
            .iter()
            .filter_map(|s| match s {
                InstallStrategy::PackageManager { manager, .. } => Some(manager.as_str()),
                _ => None,
            })
            .collect();

        if tried_managers.is_empty() {
            Err(anyhow::anyhow!(
                "No installation strategy available for {} on this platform",
                self.name
            ))
        } else {
            Err(anyhow::anyhow!(
                "Failed to install {}. Tried package managers: {}.\n\
                 Please ensure a package manager is installed (brew, choco, scoop, apt, etc.) \
                 and try again.",
                self.name,
                tried_managers.join(", ")
            ))
        }
    }
}

/// Build layout metadata HashMap from an optional Starlark layout descriptor.
fn build_layout_meta(layout: Option<&serde_json::Value>) -> Result<HashMap<String, String>> {
    let mut meta = HashMap::new();
    let Some(layout) = layout else {
        return Ok(meta);
    };

    if let Some(prefix) = layout.get("strip_prefix").and_then(|s| s.as_str()) {
        debug!("Using strip_prefix: {}", prefix);
        meta.insert("strip_prefix".to_string(), prefix.to_string());
    }
    if let Some(source) = layout.get("source_name").and_then(|s| s.as_str()) {
        meta.insert("source_name".to_string(), source.to_string());
    }
    if let Some(target) = layout.get("target_name").and_then(|s| s.as_str()) {
        meta.insert("target_name".to_string(), target.to_string());
    }
    if let Some(dir) = layout.get("target_dir").and_then(|s| s.as_str()) {
        meta.insert("target_dir".to_string(), dir.to_string());
    }
    if let Some(digest) = layout.get("sha256") {
        let digest = digest
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("sha256 must be a string"))?;
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            anyhow::bail!("Invalid artifact SHA256: expected exactly 64 hexadecimal characters");
        }
        meta.insert("sha256".to_string(), digest.to_string());
    }
    Ok(meta)
}
