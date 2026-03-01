//! ISP sub-traits for the `Runtime` trait.
//!
//! These traits provide narrower interface views over [`Runtime`], following the
//! Interface Segregation Principle. Code that only needs a subset of runtime
//! capabilities can declare a narrower bound instead of requiring the full `Runtime`.
//!
//! # Design
//!
//! Each sub-trait is implemented automatically for **every type that impls `Runtime`**
//! via blanket implementations. This means:
//!
//! - **Implementors**: no changes needed — keep your single `impl Runtime` block.
//! - **Callers**: can use narrower bounds (`impl RuntimeIdentity`) or accept
//!   `&dyn RuntimeIdentity` for read-only identity information.
//!
//! # Sub-traits
//!
//! | Trait | Methods |
//! |-------|---------|
//! | [`RuntimeIdentity`] | name, description, aliases, ecosystem, dependencies, metadata |
//! | [`RuntimePlatform`] | supported_platforms, is_platform_supported, check_platform_support |
//! | [`RuntimeVersioning`] | fetch_versions, resolve_version, resolve_installed_version, installed_versions |
//! | [`RuntimeExecutable`] | executable_name, executable_dir_path, executable_relative_path, store_name, … |
//! | [`RuntimeInstallable`] | download_url, install, uninstall, verify_installation, mirror_urls, … |
//! | [`RuntimeHooks`] | pre/post install, execute, switch, update hooks + pre_run |
//! | [`RuntimeEnvironment`] | prepare_environment, execution_environment |
//! | [`RuntimeExecuteOps`] | execute |
//! | [`RuntimeShell`] | get_shell_path, provided_shells |

use crate::context::{ExecutionContext, RuntimeContext};
use crate::ecosystem::Ecosystem;
use crate::layout::ExecutableLayout;
use crate::platform::Platform;
use crate::runtime::Runtime;
use crate::runtime::VerificationResult;
use crate::types::{ExecutionPrep, ExecutionResult, InstallResult, RuntimeDependency, VersionInfo};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use vx_runtime_core::{MirrorConfig, NormalizeConfig};

// ─── RuntimeIdentity ────────────────────────────────────────────────────────

/// Identity and metadata view of a [`Runtime`].
///
/// Callers that only need to display or record tool metadata should use this
/// trait instead of the full `Runtime` to keep their dependencies narrow.
pub trait RuntimeIdentity: Send + Sync {
    /// Primary name used to invoke this runtime (e.g., `"node"`, `"go"`).
    fn name(&self) -> &str;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// Alternative names for this runtime (e.g., `"nodejs"` for `"node"`).
    fn aliases(&self) -> Vec<&str>;
    /// Ecosystem this runtime belongs to.
    fn ecosystem(&self) -> Ecosystem;
    /// Runtime dependencies (e.g., `npm` depends on `node`).
    fn dependencies(&self) -> &[RuntimeDependency];
    /// Arbitrary key-value metadata.
    fn metadata(&self) -> HashMap<String, String>;
}

impl<T: Runtime + ?Sized> RuntimeIdentity for T {
    fn name(&self) -> &str {
        Runtime::name(self)
    }
    fn description(&self) -> &str {
        Runtime::description(self)
    }
    fn aliases(&self) -> Vec<&str> {
        Runtime::aliases(self)
    }
    fn ecosystem(&self) -> Ecosystem {
        Runtime::ecosystem(self)
    }
    fn dependencies(&self) -> &[RuntimeDependency] {
        Runtime::dependencies(self)
    }
    fn metadata(&self) -> HashMap<String, String> {
        Runtime::metadata(self)
    }
}

// ─── RuntimePlatform ────────────────────────────────────────────────────────

/// Platform support view of a [`Runtime`].
pub trait RuntimePlatform: Send + Sync {
    /// List of platforms supported by this runtime.
    fn supported_platforms(&self) -> Vec<Platform>;
    /// Returns `true` if this runtime supports the given platform.
    fn is_platform_supported(&self, platform: &Platform) -> bool;
    /// Returns `Ok(())` if the **current** platform is supported, `Err(msg)` otherwise.
    fn check_platform_support(&self) -> std::result::Result<(), String>;
}

impl<T: Runtime + ?Sized> RuntimePlatform for T {
    fn supported_platforms(&self) -> Vec<Platform> {
        Runtime::supported_platforms(self)
    }
    fn is_platform_supported(&self, platform: &Platform) -> bool {
        Runtime::is_platform_supported(self, platform)
    }
    fn check_platform_support(&self) -> std::result::Result<(), String> {
        Runtime::check_platform_support(self)
    }
}

// ─── RuntimeExecutable ──────────────────────────────────────────────────────

/// Executable path configuration view of a [`Runtime`].
pub trait RuntimeExecutable: Send + Sync {
    /// Base executable name (without extension).
    fn executable_name(&self) -> &str;
    /// Windows-specific extensions to probe (default: `[".exe"]`).
    fn executable_extensions(&self) -> &[&str];
    /// Subdirectory inside the install root where the executable lives.
    fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String>;
    /// Full relative path to the executable within the install directory.
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String;
    /// Declarative layout from provider.toml (RFC 0019).
    fn executable_layout(&self) -> Option<ExecutableLayout>;
    /// Post-install normalization config (RFC 0022).
    fn normalize_config(&self) -> Option<&NormalizeConfig>;
    /// Possible bin directory names (default: `["bin"]`).
    fn possible_bin_dirs(&self) -> Vec<&str>;
    /// Store directory name (canonical, e.g. `"node"` for `"npm"`).
    fn store_name(&self) -> &str;
    /// Mirror download sources.
    fn mirror_urls(&self) -> Vec<MirrorConfig>;
}

impl<T: Runtime + ?Sized> RuntimeExecutable for T {
    fn executable_name(&self) -> &str {
        Runtime::executable_name(self)
    }
    fn executable_extensions(&self) -> &[&str] {
        Runtime::executable_extensions(self)
    }
    fn executable_dir_path(&self, v: &str, p: &Platform) -> Option<String> {
        Runtime::executable_dir_path(self, v, p)
    }
    fn executable_relative_path(&self, v: &str, p: &Platform) -> String {
        Runtime::executable_relative_path(self, v, p)
    }
    fn executable_layout(&self) -> Option<ExecutableLayout> {
        Runtime::executable_layout(self)
    }
    fn normalize_config(&self) -> Option<&NormalizeConfig> {
        Runtime::normalize_config(self)
    }
    fn possible_bin_dirs(&self) -> Vec<&str> {
        Runtime::possible_bin_dirs(self)
    }
    fn store_name(&self) -> &str {
        Runtime::store_name(self)
    }
    fn mirror_urls(&self) -> Vec<MirrorConfig> {
        Runtime::mirror_urls(self)
    }
}

// ─── RuntimeVersioning ──────────────────────────────────────────────────────

/// Version fetching and resolution view of a [`Runtime`].
#[async_trait]
pub trait RuntimeVersioning: Send + Sync {
    /// Fetch all available versions from the upstream registry.
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>>;
    /// Resolve a version spec (e.g. `"latest"`, `"3.11"`) to an installed version.
    async fn resolve_installed_version(
        &self,
        spec: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<String>>;
    /// List all locally installed versions.
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>>;
    /// Resolve a version spec to an actual version string from the registry.
    async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String>;
    /// Returns `true` if the given version is installed.
    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool>;
    /// Returns the executable path for an installed version.
    async fn get_executable_path_for_version(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<PathBuf>>;
}

#[async_trait]
impl<T: Runtime + ?Sized> RuntimeVersioning for T {
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Runtime::fetch_versions(self, ctx).await
    }
    async fn resolve_installed_version(
        &self,
        spec: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<String>> {
        Runtime::resolve_installed_version(self, spec, ctx).await
    }
    async fn installed_versions(&self, ctx: &RuntimeContext) -> Result<Vec<String>> {
        Runtime::installed_versions(self, ctx).await
    }
    async fn resolve_version(&self, v: &str, ctx: &RuntimeContext) -> Result<String> {
        Runtime::resolve_version(self, v, ctx).await
    }
    async fn is_installed(&self, v: &str, ctx: &RuntimeContext) -> Result<bool> {
        Runtime::is_installed(self, v, ctx).await
    }
    async fn get_executable_path_for_version(
        &self,
        v: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<PathBuf>> {
        Runtime::get_executable_path_for_version(self, v, ctx).await
    }
}

// ─── RuntimeInstallable ─────────────────────────────────────────────────────

/// Download and install view of a [`Runtime`].
#[async_trait]
pub trait RuntimeInstallable: Send + Sync {
    /// Returns `true` if vx can directly download and install this version.
    fn is_version_installable(&self, version: &str) -> bool;
    /// Primary download URL for this version and platform.
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>>;
    /// Build the ordered URL chain (mirrors first, original URL last).
    async fn build_download_url_chain(
        &self,
        original: &str,
        version: &str,
        platform: &Platform,
    ) -> Vec<String>;
    /// Verify that an installation is present and valid.
    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult;
    /// Download, extract, and verify the runtime.
    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult>;
    /// Remove an installed version.
    async fn uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()>;
    /// Prepare execution context for proxy-managed / bundled versions.
    async fn prepare_execution(
        &self,
        version: &str,
        ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep>;
}

#[async_trait]
impl<T: Runtime + ?Sized> RuntimeInstallable for T {
    fn is_version_installable(&self, v: &str) -> bool {
        Runtime::is_version_installable(self, v)
    }
    async fn download_url(&self, v: &str, p: &Platform) -> Result<Option<String>> {
        Runtime::download_url(self, v, p).await
    }
    async fn build_download_url_chain(&self, o: &str, v: &str, p: &Platform) -> Vec<String> {
        Runtime::build_download_url_chain(self, o, v, p).await
    }
    fn verify_installation(&self, v: &str, path: &Path, p: &Platform) -> VerificationResult {
        Runtime::verify_installation(self, v, path, p)
    }
    async fn install(&self, v: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        Runtime::install(self, v, ctx).await
    }
    async fn uninstall(&self, v: &str, ctx: &RuntimeContext) -> Result<()> {
        Runtime::uninstall(self, v, ctx).await
    }
    async fn prepare_execution(&self, v: &str, ctx: &ExecutionContext) -> Result<ExecutionPrep> {
        Runtime::prepare_execution(self, v, ctx).await
    }
}

// ─── RuntimeHooks ───────────────────────────────────────────────────────────

/// Lifecycle hooks view of a [`Runtime`].
#[async_trait]
pub trait RuntimeHooks: Send + Sync {
    async fn pre_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()>;
    fn post_extract(&self, version: &str, install_path: &Path) -> Result<()>;
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()>;
    async fn pre_uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()>;
    async fn post_uninstall(&self, version: &str, ctx: &RuntimeContext) -> Result<()>;
    async fn pre_execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<()>;
    async fn post_execute(
        &self,
        args: &[String],
        result: &crate::types::ExecutionResult,
        ctx: &ExecutionContext,
    ) -> Result<()>;
    async fn pre_run(&self, args: &[String], executable: &Path) -> Result<bool>;
}

#[async_trait]
impl<T: Runtime + ?Sized> RuntimeHooks for T {
    async fn pre_install(&self, v: &str, ctx: &RuntimeContext) -> Result<()> {
        Runtime::pre_install(self, v, ctx).await
    }
    fn post_extract(&self, v: &str, p: &Path) -> Result<()> {
        Runtime::post_extract(self, v, p)
    }
    async fn post_install(&self, v: &str, ctx: &RuntimeContext) -> Result<()> {
        Runtime::post_install(self, v, ctx).await
    }
    async fn pre_uninstall(&self, v: &str, ctx: &RuntimeContext) -> Result<()> {
        Runtime::pre_uninstall(self, v, ctx).await
    }
    async fn post_uninstall(&self, v: &str, ctx: &RuntimeContext) -> Result<()> {
        Runtime::post_uninstall(self, v, ctx).await
    }
    async fn pre_execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<()> {
        Runtime::pre_execute(self, args, ctx).await
    }
    async fn post_execute(
        &self,
        args: &[String],
        result: &ExecutionResult,
        ctx: &ExecutionContext,
    ) -> Result<()> {
        Runtime::post_execute(self, args, result, ctx).await
    }
    async fn pre_run(&self, args: &[String], exe: &Path) -> Result<bool> {
        Runtime::pre_run(self, args, exe).await
    }
}

// ─── RuntimeEnvironment ─────────────────────────────────────────────────────

/// Environment variable preparation view of a [`Runtime`].
#[async_trait]
pub trait RuntimeEnvironment: Send + Sync {
    /// Environment variables needed when this runtime is in the environment path.
    async fn prepare_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>>;
    /// Environment variables needed when this runtime is the primary command.
    async fn execution_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>>;
}

#[async_trait]
impl<T: Runtime + ?Sized> RuntimeEnvironment for T {
    async fn prepare_environment(
        &self,
        v: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        Runtime::prepare_environment(self, v, ctx).await
    }
    async fn execution_environment(
        &self,
        v: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        Runtime::execution_environment(self, v, ctx).await
    }
}

// ─── RuntimeExecuteOps ──────────────────────────────────────────────────────

/// Command execution view of a [`Runtime`].
#[async_trait]
pub trait RuntimeExecuteOps: Send + Sync {
    /// Execute the runtime with the given arguments.
    async fn execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<ExecutionResult>;
}

#[async_trait]
impl<T: Runtime + ?Sized> RuntimeExecuteOps for T {
    async fn execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<ExecutionResult> {
        Runtime::execute(self, args, ctx).await
    }
}

// ─── RuntimeShell ───────────────────────────────────────────────────────────

/// Shell provider view of a [`Runtime`] (RFC 0038).
pub trait RuntimeShell: Send + Sync {
    /// Returns the path to a shell executable bundled by this runtime.
    fn get_shell_path(
        &self,
        shell_name: &str,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Option<PathBuf>;
    /// Names of shells this runtime can provide.
    fn provided_shells(&self) -> Vec<&'static str>;
}

impl<T: Runtime + ?Sized> RuntimeShell for T {
    fn get_shell_path(&self, s: &str, v: &str, ctx: &RuntimeContext) -> Option<PathBuf> {
        Runtime::get_shell_path(self, s, v, ctx)
    }
    fn provided_shells(&self) -> Vec<&'static str> {
        Runtime::provided_shells(self)
    }
}
