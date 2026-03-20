//! Manifest-Driven Runtime implementation.
//!
//! This module provides a [`ManifestDrivenRuntime`] that is driven entirely by
//! `provider.star` configuration. It is designed for system tools that don't
//! require strict version management (git, cmake, curl, etc.) as well as for
//! Starlark-driven providers that inject `fetch_versions`, `download_url`, and
//! `install_layout` closures.
//!
//! # Sub-modules
//!
//! - [`types`] — All data types (`InstallStrategy`, `DetectionConfig`, …)
//! - [`detection`] — Version detection and executable search
//! - [`shell`] — Shell path resolution (RFC 0038)
//! - [`install`] — `install()` strategy dispatch

pub mod detection;
pub mod install;
pub mod shell;
pub mod types;

pub use types::{
    DetectionConfig, InstallStrategy, ProvidedTool, ProviderSource, ScriptType, ShellDefinition,
    SystemDepType, SystemDependency, SystemDepsConfig,
};

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, warn};

/// Search a list of glob patterns and return the first matching **file** path.
///
/// Used to locate system-installed tools (e.g. MSVC `cl.exe`) that are not
/// on the system PATH but reside in well-known directories.
///
/// Only returns paths that are regular files (not directories). This prevents
/// mistakenly treating a directory such as `C:/Program Files/7-Zip` as an
/// executable when the glob pattern matches the directory itself.
pub fn find_first_glob_match(patterns: &[String]) -> Option<PathBuf> {
    for pattern in patterns {
        if let Ok(mut paths) = glob::glob(pattern)
            && let Some(Ok(path)) = paths.next()
            && path.is_file()
        {
            return Some(path);
        }
    }
    None
}

use crate::{Ecosystem, InstallResult, Platform, Runtime, RuntimeContext, VersionInfo};
use vx_runtime_core::{MirrorConfig, NormalizeConfig};

/// Type alias for an async `fetch_versions` function injected from Starlark providers.
///
/// Allows `ManifestDrivenRuntime` to delegate `fetch_versions` to a Starlark-driven
/// implementation without creating a circular dependency between `vx-runtime` and
/// `vx-starlark`.
pub type FetchVersionsFn = Arc<
    dyn Fn()
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VersionInfo>>> + Send>>
        + Send
        + Sync,
>;

/// Type alias for an async `download_url` function injected from Starlark providers.
///
/// Signature: `(version: String) -> Option<String>`
pub type DownloadUrlFn = Arc<
    dyn Fn(
            String,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>>> + Send>>
        + Send
        + Sync,
>;

/// Type alias for an async `install_layout` function injected from Starlark providers.
///
/// Returns a serialized JSON value describing the install layout, or `None`.
pub type InstallLayoutFn = Arc<
    dyn Fn(
            String,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Option<serde_json::Value>>> + Send>,
        > + Send
        + Sync,
>;

/// Type alias for an async `deps(version)` function injected from Starlark providers.
pub type DepsFn = Arc<
    dyn Fn(
            String,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Vec<crate::RuntimeDependency>>> + Send>,
        > + Send
        + Sync,
>;

/// A runtime driven by manifest configuration (`provider.star`).
///
/// For Starlark-driven providers, the `fetch_versions_fn`, `download_url_fn`, and
/// `install_layout_fn` fields can be set to delegate to the Starlark engine.
#[derive(Clone)]
pub struct ManifestDrivenRuntime {
    pub name: String,
    pub description: String,
    pub executable: String,
    pub aliases: Vec<String>,
    pub ecosystem_override: Option<Ecosystem>,
    pub bundled_with: Option<String>,
    pub provider_name: String,
    pub source: ProviderSource,
    pub install_strategies: Vec<InstallStrategy>,
    pub provides: Vec<ProvidedTool>,
    pub detection: Option<DetectionConfig>,
    pub system_deps: Option<SystemDepsConfig>,
    pub normalize: Option<NormalizeConfig>,
    pub mirrors: Vec<MirrorConfig>,
    /// Optional Starlark-driven `fetch_versions` implementation.
    pub fetch_versions_fn: Option<FetchVersionsFn>,
    /// Optional Starlark-driven `download_url` implementation.
    pub download_url_fn: Option<DownloadUrlFn>,
    /// Optional Starlark-driven `install_layout` implementation.
    pub install_layout_fn: Option<InstallLayoutFn>,
    /// Optional Starlark-driven `deps(version)` implementation.
    pub deps_fn: Option<DepsFn>,
    /// Optional pip package name for Python-based tools.
    pub pip_package: Option<String>,

    /// Shells provided by this runtime (RFC 0038).
    pub shells: Vec<ShellDefinition>,
    /// Platform OS constraint (e.g. `["macos"]` for macOS-only tools).
    pub platform_os: Vec<String>,
    /// Known system paths (glob patterns) for system-installed tools.
    ///
    /// Used after system package manager installation to locate the executable
    /// (e.g. MSVC cl.exe which is not on PATH).
    pub system_paths: Vec<String>,
}

impl std::fmt::Debug for ManifestDrivenRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManifestDrivenRuntime")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("executable", &self.executable)
            .field("aliases", &self.aliases)
            .field("provider_name", &self.provider_name)
            .field(
                "fetch_versions_fn",
                &self.fetch_versions_fn.as_ref().map(|_| "<fn>"),
            )
            .field("deps_fn", &self.deps_fn.as_ref().map(|_| "<fn>"))
            .finish()
    }
}

// ============================================================================
// Constructor & Builder
// ============================================================================

impl ManifestDrivenRuntime {
    /// Create a new manifest-driven runtime.
    pub fn new(
        name: impl Into<String>,
        provider_name: impl Into<String>,
        source: ProviderSource,
    ) -> Self {
        let name = name.into();
        Self {
            executable: name.clone(),
            name,
            description: String::new(),
            aliases: Vec::new(),
            ecosystem_override: None,
            bundled_with: None,
            provider_name: provider_name.into(),
            source,
            install_strategies: Vec::new(),
            provides: Vec::new(),
            detection: None,
            system_deps: None,
            normalize: None,
            mirrors: Vec::new(),
            fetch_versions_fn: None,
            download_url_fn: None,
            install_layout_fn: None,
            deps_fn: None,
            pip_package: None,

            shells: Vec::new(),
            platform_os: Vec::new(),
            system_paths: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_mirrors(mut self, mirrors: Vec<MirrorConfig>) -> Self {
        self.mirrors = mirrors;
        self
    }

    pub fn with_bundled_with(mut self, bundled_with: impl Into<String>) -> Self {
        self.bundled_with = Some(bundled_with.into());
        self
    }

    pub fn with_platform_os(mut self, platform_os: Vec<String>) -> Self {
        self.platform_os = platform_os;
        self
    }

    pub fn with_executable(mut self, executable: impl Into<String>) -> Self {
        self.executable = executable.into();
        self
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.aliases.push(alias.into());
        self
    }

    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases.extend(aliases);
        self
    }

    pub fn with_ecosystem(mut self, ecosystem: Ecosystem) -> Self {
        self.ecosystem_override = Some(ecosystem);
        self
    }

    pub fn with_strategy(mut self, strategy: InstallStrategy) -> Self {
        self.install_strategies.push(strategy);
        self
    }

    pub fn with_install_strategies(mut self, strategies: Vec<InstallStrategy>) -> Self {
        self.install_strategies.extend(strategies);
        self
    }

    pub fn with_pip_package(mut self, package: impl Into<String>) -> Self {
        self.pip_package = Some(package.into());
        self
    }

    pub fn with_shells(mut self, shells: Vec<ShellDefinition>) -> Self {
        self.shells = shells;
        self
    }

    pub fn with_shell(mut self, name: impl Into<String>, path: impl Into<String>) -> Self {
        self.shells.push(ShellDefinition {
            name: name.into(),
            path: path.into(),
        });
        self
    }

    pub fn with_system_paths(mut self, paths: Vec<String>) -> Self {
        self.system_paths = paths;
        self
    }

    pub fn with_detection(mut self, detection: DetectionConfig) -> Self {
        self.detection = Some(detection);
        self
    }

    pub fn with_normalize(mut self, normalize: NormalizeConfig) -> Self {
        self.normalize = Some(normalize);
        self
    }

    /// Set install dependencies (vx-managed runtimes that must be installed first).
    pub fn with_install_deps(mut self, deps: Vec<String>) -> Self {
        if deps.is_empty() {
            return self;
        }

        let pre_depends: Vec<SystemDependency> = deps
            .into_iter()
            .map(|dep| {
                let (id, version) = if let Some(gt_pos) = dep.find(">=") {
                    (
                        dep[..gt_pos].to_string(),
                        Some(dep[gt_pos + 2..].to_string()),
                    )
                } else if let Some(eq_pos) = dep.find('=') {
                    (
                        dep[..eq_pos].to_string(),
                        Some(dep[eq_pos + 1..].to_string()),
                    )
                } else {
                    (dep, None)
                };

                SystemDependency {
                    dep_type: SystemDepType::Runtime,
                    id,
                    version,
                    reason: Some("Install dependency".to_string()),
                    platforms: vec![],
                    optional: false,
                }
            })
            .collect();

        self.system_deps = Some(SystemDepsConfig {
            pre_depends,
            depends: vec![],
            recommends: vec![],
            suggests: vec![],
        });
        self
    }

    /// Inject a Starlark-driven `fetch_versions` implementation.
    pub fn with_fetch_versions<F>(mut self, f: F) -> Self
    where
        F: Fn() -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Vec<VersionInfo>>> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        self.fetch_versions_fn = Some(Arc::new(f));
        self
    }

    /// Inject a Starlark-driven `download_url` implementation.
    pub fn with_download_url<F>(mut self, f: F) -> Self
    where
        F: Fn(
                String,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Option<String>>> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        self.download_url_fn = Some(Arc::new(f));
        self
    }

    /// Inject a Starlark-driven `deps(version)` implementation.
    pub fn with_deps_fn<F>(mut self, f: F) -> Self
    where
        F: Fn(
                String,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Vec<crate::RuntimeDependency>>> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        self.deps_fn = Some(Arc::new(f));
        self
    }

    /// Inject a Starlark-driven `install_layout` implementation.
    pub fn with_install_layout<F>(mut self, f: F) -> Self
    where
        F: Fn(
                String,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = Result<Option<serde_json::Value>>> + Send>,
            > + Send
            + Sync
            + 'static,
    {
        self.install_layout_fn = Some(Arc::new(f));
        self
    }

    // ========== Internal helpers ==========

    /// Resolve the executable path from a Starlark install_layout descriptor.
    pub(crate) fn resolve_exe_path_from_layout(
        &self,
        install_dir: &std::path::Path,
        layout: &serde_json::Value,
    ) -> PathBuf {
        if let (Some(target_dir), Some(target_name)) = (
            layout.get("target_dir").and_then(|d| d.as_str()),
            layout.get("target_name").and_then(|n| n.as_str()),
        ) {
            let candidate = install_dir.join(target_dir).join(target_name);
            if candidate.exists() {
                return candidate;
            }
        }

        if let Some(paths) = layout.get("executable_paths").and_then(|p| p.as_array()) {
            let mut preferred_rel: Option<&str> = None;
            let mut preferred_names: Vec<String> = vec![self.executable.clone()];
            if cfg!(windows)
                && !self.executable.ends_with(".exe")
                && !self.executable.ends_with(".cmd")
                && !self.executable.ends_with(".bat")
            {
                preferred_names.push(format!("{}.exe", self.executable));
                preferred_names.push(format!("{}.cmd", self.executable));
                preferred_names.push(format!("{}.bat", self.executable));
            }

            for p in paths {
                if let Some(rel) = p.as_str() {
                    let file_name = Path::new(rel).file_name().and_then(|n| n.to_str());
                    if let Some(name) = file_name
                        && preferred_names
                            .iter()
                            .any(|preferred| preferred.eq_ignore_ascii_case(name))
                    {
                        let candidate = install_dir.join(rel);
                        if candidate.exists() {
                            return candidate;
                        }
                        if preferred_rel.is_none() {
                            preferred_rel = Some(rel);
                        }
                    }
                }
            }

            for p in paths {
                if let Some(rel) = p.as_str() {
                    let candidate = install_dir.join(rel);
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }

            if let Some(rel) = preferred_rel {
                return install_dir.join(rel);
            }

            if let Some(first) = paths.first().and_then(|p| p.as_str()) {
                return install_dir.join(first);
            }
        }

        if let (Some(target_dir), Some(target_name)) = (
            layout.get("target_dir").and_then(|d| d.as_str()),
            layout.get("target_name").and_then(|n| n.as_str()),
        ) {
            return install_dir.join(target_dir).join(target_name);
        }

        install_dir.join(vx_paths::with_executable_extension(&self.executable))
    }

    /// Select the best available installation strategy for the current platform.
    pub async fn select_best_strategy(&self, platform: &Platform) -> Option<&InstallStrategy> {
        let mut candidates: Vec<_> = self
            .install_strategies
            .iter()
            .filter(|s| s.matches_platform(platform))
            .collect();

        candidates.sort_by_key(|b| std::cmp::Reverse(b.priority()));

        for strategy in candidates {
            if self.is_strategy_available(strategy).await {
                return Some(strategy);
            }
        }

        None
    }

    async fn is_strategy_available(&self, strategy: &InstallStrategy) -> bool {
        match strategy {
            InstallStrategy::PackageManager { manager, .. } => {
                is_package_manager_available(manager).await
            }
            InstallStrategy::DirectDownload { .. } => true,
            InstallStrategy::Script { .. } => true,
            InstallStrategy::ProvidedBy { provider, .. } => which::which(provider).is_ok(),
        }
    }
}

// ============================================================================
// Runtime trait implementation
// ============================================================================

#[async_trait]
impl Runtime for ManifestDrivenRuntime {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        if self.description.is_empty() {
            "System tool"
        } else {
            &self.description
        }
    }

    fn aliases(&self) -> Vec<&str> {
        self.aliases.iter().map(|s| s.as_str()).collect()
    }

    fn ecosystem(&self) -> Ecosystem {
        self.ecosystem_override.unwrap_or(Ecosystem::System)
    }

    fn supported_platforms(&self) -> Vec<crate::platform::Platform> {
        if self.platform_os.is_empty() {
            return crate::platform::Platform::all_common();
        }
        let mut platforms = Vec::new();
        for os_name in &self.platform_os {
            match os_name.to_lowercase().as_str() {
                "windows" => platforms.extend(crate::platform::Platform::windows_only()),
                "macos" | "darwin" | "osx" => {
                    platforms.extend(crate::platform::Platform::macos_only())
                }
                "linux" => platforms.extend(crate::platform::Platform::linux_only()),
                "unix" => platforms.extend(crate::platform::Platform::unix_only()),
                _ => {}
            }
        }
        if platforms.is_empty() {
            crate::platform::Platform::all_common()
        } else {
            platforms
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("provider".to_string(), self.provider_name.clone());
        meta.insert("source".to_string(), self.source.to_string());
        meta.insert("manifest_driven".to_string(), "true".to_string());
        if let Some(ref bundled) = self.bundled_with {
            meta.insert("bundled_with".to_string(), bundled.clone());
        }
        meta
    }

    async fn versioned_dependencies(
        &self,
        version: &str,
        _ctx: &crate::RuntimeContext,
    ) -> Result<Vec<crate::RuntimeDependency>> {
        match &self.deps_fn {
            Some(f) => f(version.to_string()).await,
            None => Ok(vec![]),
        }
    }

    fn mirror_urls(&self) -> Vec<MirrorConfig> {
        self.mirrors.clone()
    }

    fn store_name(&self) -> &str {
        self.bundled_with.as_deref().unwrap_or(&self.name)
    }

    fn is_version_installable(&self, _version: &str) -> bool {
        self.bundled_with.is_none()
    }

    async fn prepare_execution(
        &self,
        version: &str,
        _ctx: &crate::ExecutionContext,
    ) -> Result<crate::ExecutionPrep> {
        if let Some(ref parent) = self.bundled_with {
            debug!(
                "Preparing bundled runtime {} (bundled with {}) at version {}",
                self.name, parent, version
            );

            let paths = vx_paths::VxPaths::new()
                .map_err(|e| anyhow::anyhow!("Failed to get VxPaths: {}", e))?;
            let store_name = parent;
            let platform = crate::platform::Platform::current();

            let path_manager = vx_paths::PathManager::from_paths(paths.clone());
            let mut candidate_versions: Vec<String> = vec![version.to_string()];
            if let Ok(installed) = path_manager.list_store_versions(store_name) {
                for v in installed {
                    if v != version {
                        candidate_versions.push(v);
                    }
                }
            }

            let exe_name = &self.executable;
            // On Windows, build a list of possible extensions to try.
            // Many bundled runtimes (npm, npx, yarn, corepack) use .cmd on Windows,
            // not .exe. We must try both extensions to find the correct executable.
            let exe_candidates: Vec<String> = if cfg!(windows) {
                if exe_name.ends_with(".exe")
                    || exe_name.ends_with(".cmd")
                    || exe_name.ends_with(".bat")
                {
                    vec![exe_name.to_string()]
                } else {
                    vec![
                        format!("{}.exe", exe_name),
                        format!("{}.cmd", exe_name),
                        exe_name.to_string(),
                    ]
                }
            } else {
                vec![exe_name.clone()]
            };

            for parent_version in &candidate_versions {
                let version_dir = paths.version_store_dir(store_name, parent_version);
                let platform_dir = version_dir.join(platform.as_str());
                let search_dirs = [&platform_dir, &version_dir];

                for dir in &search_dirs {
                    // Build candidates from all possible executable names × locations
                    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
                    for ext_name in &exe_candidates {
                        candidates.push(dir.join(ext_name));
                    }
                    for ext_name in &exe_candidates {
                        candidates.push(dir.join("bin").join(ext_name));
                    }

                    for path in &candidates {
                        if path.exists() {
                            debug!(
                                "Found bundled executable {} at {} (parent version: {})",
                                self.name,
                                path.display(),
                                parent_version
                            );
                            return Ok(crate::ExecutionPrep {
                                executable_override: Some(path.clone()),
                                proxy_ready: true,
                                message: Some(format!(
                                    "Using {} from {} {} installation",
                                    self.name, parent, parent_version
                                )),
                                ..Default::default()
                            });
                        }
                    }

                    let primary_ext = exe_candidates
                        .first()
                        .cloned()
                        .unwrap_or_else(|| exe_name.clone());
                    if dir.exists()
                        && let Some(found) =
                            detection::find_executable_recursive(dir, exe_name, &primary_ext, 4)
                    {
                        debug!(
                            "Found bundled executable {} via recursive search at {} (parent version: {})",
                            self.name,
                            found.display(),
                            parent_version
                        );
                        return Ok(crate::ExecutionPrep {
                            executable_override: Some(found),
                            proxy_ready: true,
                            message: Some(format!(
                                "Using {} from {} {} installation",
                                self.name, parent, parent_version
                            )),
                            ..Default::default()
                        });
                    }
                }
            }

            warn!(
                "Could not find {} executable in {} installation. {} may need to be installed.",
                self.name, parent, parent
            );

            // Before falling back to system PATH, try system_paths glob patterns.
            // This handles tools like `csc` that are bundled with MSVC but live in
            // well-known directories that are NOT on the system PATH.
            if !self.system_paths.is_empty()
                && let Some(found) = find_first_glob_match(&self.system_paths)
            {
                debug!(
                    "Found {} via system_paths glob at {}",
                    self.name,
                    found.display()
                );
                return Ok(crate::ExecutionPrep {
                    executable_override: Some(found),
                    proxy_ready: true,
                    message: Some(format!(
                        "Using {} from system installation (via system_paths)",
                        self.name
                    )),
                    ..Default::default()
                });
            }

            return Ok(crate::ExecutionPrep {
                use_system_path: true,
                message: Some(format!(
                    "{} not found in {} installation, trying system PATH",
                    self.name, parent
                )),
                ..Default::default()
            });
        }

        // Non-bundled runtime: try system_paths glob patterns before giving up.
        // This handles system-installed tools (e.g. MSVC cl.exe) that are not on PATH
        // but reside in well-known directories defined in system_paths.
        if !self.system_paths.is_empty()
            && let Some(found) = find_first_glob_match(&self.system_paths)
        {
            debug!(
                "Found {} via system_paths glob at {}",
                self.name,
                found.display()
            );
            return Ok(crate::ExecutionPrep {
                executable_override: Some(found),
                proxy_ready: true,
                message: Some(format!(
                    "Using {} from system installation (via system_paths)",
                    self.name
                )),
                ..Default::default()
            });
        }

        Ok(crate::ExecutionPrep::default())
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        if let Some(ref f) = self.fetch_versions_fn {
            return f().await;
        }
        // pip package: query PyPI for available versions
        if let Some(ref pkg) = self.pip_package {
            return fetch_pypi_versions(pkg, ctx).await;
        }
        Ok(vec![VersionInfo {
            version: "system".to_string(),
            released_at: None,
            prerelease: false,
            lts: true,
            download_url: None,
            checksum: None,
            metadata: HashMap::new(),
        }])
    }

    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        // 1. Check the vx-managed store first.
        //    Tools installed via `vx install` live in ~/.vx/store/<name>/<version>/
        //    and may NOT be on the system PATH, so which::which() would give a false
        //    negative.  Checking the store first avoids redundant install() calls on
        //    every `vx <tool>` invocation for already-installed tools.
        let runtime_dir = ctx.paths.runtime_store_dir(self.store_name());
        if runtime_dir.exists() {
            if version == "latest" {
                // Any version directory present → tool is installed.
                let has_version = std::fs::read_dir(&runtime_dir)
                    .ok()
                    .map(|entries| entries.filter_map(|e| e.ok()).any(|e| e.path().is_dir()))
                    .unwrap_or(false);
                if has_version {
                    return Ok(true);
                }
            } else if runtime_dir.join(version).is_dir() {
                // Specific version directory exists → that version is installed.
                return Ok(true);
            }
        }

        // 2. Fall back to PATH lookup (system-installed tools, e.g. installed by brew/choco).
        if which::which(&self.executable).is_ok() {
            return Ok(true);
        }

        // 3. Fall back to system_paths glob search (for tools like MSVC cl.exe not on PATH).
        if !self.system_paths.is_empty() {
            return Ok(find_first_glob_match(&self.system_paths).is_some());
        }

        Ok(false)
    }

    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        // 1. Check the vx-managed store first.
        //    Tools installed via `vx install` live in ~/.vx/store/<name>/<version>/
        //    and may not be on the system PATH yet.  Checking the store prevents
        //    companion tools (e.g. uv, prek) from being auto-reinstalled every time
        //    the project environment is prepared.
        let runtime_dir = ctx.paths.runtime_store_dir(self.store_name());
        if runtime_dir.exists() {
            let mut versions: Vec<String> = std::fs::read_dir(&runtime_dir)
                .ok()
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .filter_map(|e| e.file_name().into_string().ok())
                        .collect()
                })
                .unwrap_or_default();
            if !versions.is_empty() {
                // Newest first – mirrors the convention used by ProviderHandle.
                versions.sort_by(|a, b| b.cmp(a));
                return Ok(versions);
            }
        }

        // 2. Fall back to system PATH detection.
        if which::which(&self.executable).is_ok() {
            if let Ok(Some(version)) = detection::detect_version(
                &self.executable,
                self.detection.as_ref().unwrap_or(&DetectionConfig {
                    command: format!("{} --version", self.executable),
                    pattern: String::new(),
                    system_paths: vec![],
                    env_hints: vec![],
                }),
            )
            .await
            {
                return Ok(vec![version]);
            }
            Ok(vec!["system".to_string()])
        } else {
            Ok(vec![])
        }
    }

    async fn get_executable_path_for_version(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<std::path::PathBuf>> {
        let platform = Platform::current();
        let base_path = ctx.paths.version_store_dir(self.store_name(), version);
        let install_path = base_path.join(platform.as_str());
        if !ctx.fs.exists(&install_path) {
            return Ok(None);
        }

        if let Some(ref layout_fn) = self.install_layout_fn
            && let Ok(Some(layout)) = layout_fn(version.to_string()).await
        {
            let exe_path = self.resolve_exe_path_from_layout(&install_path, &layout);
            if ctx.fs.exists(&exe_path) {
                return Ok(Some(exe_path));
            }
        }

        let verification = self.verify_installation(version, &install_path, &platform);
        if verification.valid {
            Ok(verification.executable_path)
        } else {
            Ok(None)
        }
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        if let Some(ref f) = self.download_url_fn {
            return f(version.to_string()).await;
        }

        for strategy in &self.install_strategies {
            if let InstallStrategy::DirectDownload { url, platforms, .. } = strategy
                && (platforms.is_empty()
                    || platforms
                        .iter()
                        .any(|p| p.eq_ignore_ascii_case(platform.os_name())))
            {
                let url = url.replace("{version}", version);
                return Ok(Some(url));
            }
        }
        Ok(None)
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        self.install_impl(version, ctx).await
    }

    fn normalize_config(&self) -> Option<&NormalizeConfig> {
        self.normalize.as_ref()
    }

    fn get_shell_path(
        &self,
        shell_name: &str,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Option<std::path::PathBuf> {
        shell::get_shell_path(
            shell_name,
            &self.shells,
            &self.executable,
            self.store_name(),
            version,
            self.detection.as_ref(),
            ctx,
        )
    }

    fn provided_shells(&self) -> Vec<&'static str> {
        vec![]
    }
}

// ============================================================================
// Private helpers
// ============================================================================

/// Check if a package manager is available on the system.
async fn is_package_manager_available(manager: &str) -> bool {
    match manager {
        "choco" | "chocolatey" => which::which("choco").is_ok(),
        "winget" => which::which("winget").is_ok(),
        "scoop" => which::which("scoop").is_ok(),
        "brew" | "homebrew" => which::which("brew").is_ok(),
        "apt" | "apt-get" => which::which("apt").is_ok() || which::which("apt-get").is_ok(),
        "yum" => which::which("yum").is_ok(),
        "dnf" => which::which("dnf").is_ok(),
        "pacman" => which::which("pacman").is_ok(),
        "zypper" => which::which("zypper").is_ok(),
        "apk" => which::which("apk").is_ok(),
        _ => false,
    }
}

/// Fetch available versions from PyPI for a pip package.
async fn fetch_pypi_versions(pkg: &str, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    let url = format!("https://pypi.org/pypi/{}/json", pkg);
    if let Ok(resp) = ctx.http.get_json_value(&url).await {
        let mut versions = Vec::new();
        if let Some(releases) = resp.get("releases").and_then(|v| v.as_object()) {
            for (ver, files) in releases {
                if files.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                    continue;
                }
                let prerelease = ver.contains('a')
                    || ver.contains('b')
                    || ver.contains("rc")
                    || ver.contains("dev");
                versions.push(VersionInfo {
                    version: ver.clone(),
                    released_at: None,
                    prerelease,
                    lts: false,
                    download_url: None,
                    checksum: None,
                    metadata: HashMap::new(),
                });
            }
        }
        versions.sort_by(|a, b| {
            let parse = |v: &str| -> Vec<u64> {
                v.split(|c: char| !c.is_ascii_digit())
                    .filter(|s| !s.is_empty())
                    .filter_map(|s| s.parse::<u64>().ok())
                    .collect()
            };
            parse(&b.version).cmp(&parse(&a.version))
        });
        return Ok(versions);
    }
    Ok(vec![])
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_install_strategy_priority() {
        let strategy = InstallStrategy::PackageManager {
            manager: "choco".to_string(),
            package: "git".to_string(),
            params: None,
            install_args: None,
            priority: 80,
            platforms: vec!["windows".to_string()],
        };
        assert_eq!(strategy.priority(), 80);
    }

    #[test]
    fn test_install_strategy_platform_filter() {
        let strategy = InstallStrategy::PackageManager {
            manager: "brew".to_string(),
            package: "git".to_string(),
            params: None,
            install_args: None,
            priority: 90,
            platforms: vec!["macos".to_string(), "linux".to_string()],
        };

        let macos = Platform::new(crate::Os::MacOS, crate::Arch::Aarch64);
        let windows = Platform::new(crate::Os::Windows, crate::Arch::X86_64);

        assert!(strategy.matches_platform(&macos));
        assert!(!strategy.matches_platform(&windows));
    }

    #[test]
    fn test_manifest_runtime_builder() {
        let runtime = ManifestDrivenRuntime::new("fd", "mytools", ProviderSource::BuiltIn)
            .with_description("A simple, fast alternative to find")
            .with_executable("fd")
            .with_alias("fd-find")
            .with_strategy(InstallStrategy::PackageManager {
                manager: "brew".to_string(),
                package: "fd".to_string(),
                params: None,
                install_args: None,
                priority: 90,
                platforms: vec![],
            });

        assert_eq!(runtime.name(), "fd");
        assert_eq!(runtime.description(), "A simple, fast alternative to find");
        assert_eq!(runtime.install_strategies.len(), 1);
    }
}
