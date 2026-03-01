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
        let install_path = base_path.join(platform.as_str());

        // pip package: install via uv pip install <package>==<version>
        if let Some(ref pkg) = self.pip_package {
            return crate::package_runtime::install_pip_package_for_manifest(
                pkg, &self.name, version, ctx,
            )
            .await;
        }

        // Try Starlark-driven install_layout first (provides URL + strip_prefix + exe paths)
        if let Some(ref layout_fn) = self.install_layout_fn
            && let Some(layout) = layout_fn(version.to_string()).await?
        {
            let url = layout
                .get("url")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());

            if let Some(url) = url {
                info!(
                    "Installing {} via Starlark install_layout from {}",
                    self.name, url
                );

                if ctx.fs.exists(&install_path) {
                    let exe_path = self.resolve_exe_path_from_layout(&install_path, &layout);
                    return Ok(InstallResult::already_installed(
                        install_path,
                        exe_path,
                        version.to_string(),
                    ));
                }

                let mut layout_meta = HashMap::new();
                if let Some(prefix) = layout.get("strip_prefix").and_then(|s| s.as_str()) {
                    layout_meta.insert("strip_prefix".to_string(), prefix.to_string());
                }

                ctx.installer
                    .download_with_layout(&url, &install_path, &layout_meta)
                    .await?;

                let exe_path = self.resolve_exe_path_from_layout(&install_path, &layout);
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
                .install_via_direct_download(version, &url, &install_path, ctx)
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
        ctx: &RuntimeContext,
    ) -> Result<InstallResult> {
        info!("Installing {} via direct download from {}", self.name, url);

        // Resolve install_layout once for strip_prefix / executable_paths hints
        let layout_hint = self.resolve_layout_hint(version).await;

        if ctx.fs.exists(install_path) {
            let exe_path = if let Some(ref layout) = layout_hint {
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

        // Build layout metadata for download_with_layout
        let layout_meta = build_layout_meta(layout_hint.as_ref());
        debug!("layout_meta for download_with_layout: {:?}", layout_meta);

        ctx.installer
            .download_with_layout(url, install_path, &layout_meta)
            .await?;

        let exe_path = if let Some(ref layout) = layout_hint {
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
    /// Calls `install_layout_fn` once and caches the result. Returns `None` if
    /// no layout function is set or the function returns `None`.
    async fn resolve_layout_hint(&self, version: &str) -> Option<serde_json::Value> {
        let layout_fn = self.install_layout_fn.as_ref()?;
        match layout_fn(version.to_string()).await {
            Ok(Some(layout)) => {
                debug!("install_layout_fn returned: {:?}", layout);
                Some(layout)
            }
            Ok(None) => {
                debug!("install_layout_fn returned None");
                None
            }
            Err(e) => {
                warn!("install_layout_fn failed: {}", e);
                None
            }
        }
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

                        let spec = PackageInstallSpec {
                            package: package.clone(),
                            params: params.clone(),
                            install_args: install_args.clone(),
                            ..Default::default()
                        };

                        match pm.install_package(&spec).await {
                            Ok(_) => {
                                info!("Successfully installed {} via {}", self.name, manager);
                                let exe_path = which::which(&self.executable).ok();
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
fn build_layout_meta(layout: Option<&serde_json::Value>) -> HashMap<String, String> {
    let mut meta = HashMap::new();
    let Some(layout) = layout else {
        return meta;
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
    meta
}
