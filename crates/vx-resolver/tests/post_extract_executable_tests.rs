//! Cold-store regression tests for runtimes that bootstrap themselves in
//! `post_install` (`post_extract` in `provider.star`).
//!
//! Rust is the motivating case: `install()` unpacks `bin/rustup-init`, and only
//! the `post_install` hook turns it into the real `cargo/bin/rustup`. The
//! executable resolved during `install()` is therefore stale by the time
//! installation finishes, and dispatching to it fails (`rustup-init` has no
//! `component` subcommand).
//!
//! See <https://github.com/loonghao/vx/issues/1152>.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, bail};
use async_trait::async_trait;
use rstest::rstest;
use vx_resolver::{
    EnsureStage, ExecutionConfig, ExecutionPlan, InstallStatus, PlannedRuntime, Resolver,
    ResolverConfig, RuntimeMap, Stage, VersionResolution,
};
use vx_runtime::{
    Ecosystem, InstallResult, Provider, ProviderRegistry, RealFileSystem, RealPathProvider,
    Runtime, RuntimeContext, VersionInfo,
    testing::{MockHttpClient, MockInstaller},
};

const VERSION: &str = "1.93.1";

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

/// A `RuntimeContext` backed by the real filesystem, rooted at `root`.
fn real_context(root: &Path) -> RuntimeContext {
    RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(root)),
        Arc::new(MockHttpClient::new()),
        Arc::new(RealFileSystem),
        Arc::new(MockInstaller::new()),
    )
}

/// Mimics the Rust provider: `install()` leaves a bootstrapper behind, and the
/// `post_install` hook (`rustup-init`) produces the binary that must be executed.
struct BootstrappingRuntime;

impl BootstrappingRuntime {
    fn install_dir(ctx: &RuntimeContext) -> PathBuf {
        ctx.paths.version_store_dir("rust", VERSION)
    }

    /// `bin/rustup-init` — the bootstrapper written by `install()`.
    fn bootstrapper(ctx: &RuntimeContext) -> PathBuf {
        Self::install_dir(ctx)
            .join("bin")
            .join(exe_name("rustup-init"))
    }

    /// `cargo/bin/rustup` — what `post_install` creates.
    fn real_executable(ctx: &RuntimeContext) -> PathBuf {
        Self::install_dir(ctx)
            .join("cargo")
            .join("bin")
            .join(exe_name("rustup"))
    }

    fn write(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("parent dir")).expect("create parent dir");
        std::fs::write(path, b"binary").expect("write binary");
    }
}

#[async_trait]
impl Runtime for BootstrappingRuntime {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "Mock bootstrapping runtime"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["rustup"]
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new(VERSION)])
    }

    async fn is_installed(&self, _version: &str, ctx: &RuntimeContext) -> Result<bool> {
        Ok(Self::real_executable(ctx).exists())
    }

    /// Reports `false` so the ensure stage treats an incomplete store as a repair.
    fn has_post_extract_hook(&self) -> bool {
        true
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let install_path = Self::install_dir(ctx);
        let bootstrapper = Self::bootstrapper(ctx);
        Self::write(&bootstrapper);

        Ok(InstallResult::success(
            install_path,
            bootstrapper,
            version.to_string(),
        ))
    }

    async fn post_install(&self, _version: &str, ctx: &RuntimeContext) -> Result<()> {
        Self::write(&Self::real_executable(ctx));
        Ok(())
    }

    /// Stands in for `provider.star::get_execute_path`, which declares the real
    /// manager binary rather than the bootstrapper.
    async fn get_executable_path_for_version(
        &self,
        _version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<PathBuf>> {
        let path = Self::real_executable(ctx);
        Ok(path.exists().then_some(path))
    }
}

struct RustProvider;

impl Provider for RustProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "Mock rust provider"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(BootstrappingRuntime)]
    }
}

#[tokio::test]
async fn test_cold_install_dispatches_to_post_extract_executable() {
    let root = tempfile::tempdir().expect("tempdir");
    let context = real_context(root.path());

    let registry = ProviderRegistry::new();
    registry.register(Arc::new(RustProvider));

    let config = ResolverConfig::default();
    let resolver = Resolver::new(config.clone(), RuntimeMap::empty()).expect("resolver");
    let stage = EnsureStage::new(&resolver, &config, Some(&registry), Some(&context));

    // A store that contains nothing yet, so `is_installed` reports false and the
    // ensure stage runs the install + post-install sequence.
    let mut primary = PlannedRuntime::installed("rust", VERSION.to_string(), PathBuf::new());
    primary.status = InstallStatus::Installed;
    primary.version = VersionResolution::Installed {
        version: VERSION.to_string(),
        source: vx_resolver::VersionSource::VxManaged,
    };
    primary.executable = None;

    let plan = ExecutionPlan::new(primary, ExecutionConfig::default());
    let plan = stage
        .execute(plan)
        .await
        .expect("ensure stage should succeed");

    let expected = BootstrappingRuntime::real_executable(&context);
    let bootstrapper = BootstrappingRuntime::bootstrapper(&context);

    assert_eq!(
        plan.primary.executable.as_deref(),
        Some(expected.as_path()),
        "cold install must dispatch to the post-extract binary"
    );
    assert_ne!(
        plan.primary.executable.as_deref(),
        Some(bootstrapper.as_path()),
        "cold install must not dispatch to the bootstrapper"
    );
}

#[derive(Clone, Copy)]
enum ResolutionFailure {
    Missing,
    NonexistentPath,
    Error,
}

/// A bootstrap command can exit successfully without creating the final app.
/// Its resolver may report no path, a nonexistent path, or an evaluation error.
struct IncompleteBootstrappingRuntime {
    failure: ResolutionFailure,
    system_install: bool,
    has_hook: bool,
}

#[async_trait]
impl Runtime for IncompleteBootstrappingRuntime {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "Mock incomplete bootstrap"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new(VERSION)])
    }

    async fn is_installed(&self, version: &str, ctx: &RuntimeContext) -> Result<bool> {
        BootstrappingRuntime.is_installed(version, ctx).await
    }

    fn has_post_extract_hook(&self) -> bool {
        self.has_hook
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let mut result = BootstrappingRuntime.install(version, ctx).await?;
        if self.system_install {
            result.install_path = PathBuf::from("system");
        }
        Ok(result)
    }

    async fn post_install(&self, _version: &str, _ctx: &RuntimeContext) -> Result<()> {
        // Simulate a command that exits zero but leaves only the bootstrapper.
        Ok(())
    }

    async fn get_executable_path_for_version(
        &self,
        _version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<PathBuf>> {
        match self.failure {
            ResolutionFailure::Missing => Ok(None),
            ResolutionFailure::NonexistentPath => {
                Ok(Some(BootstrappingRuntime::real_executable(ctx)))
            }
            ResolutionFailure::Error => bail!("declared executable resolution failed"),
        }
    }
}

struct IncompleteProvider {
    failure: ResolutionFailure,
    system_install: bool,
    has_hook: bool,
}

impl Provider for IncompleteProvider {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "Mock incomplete bootstrap provider"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(IncompleteBootstrappingRuntime {
            failure: self.failure,
            system_install: self.system_install,
            has_hook: self.has_hook,
        })]
    }
}

fn bootstrap_plan(context: &RuntimeContext, warm_cache: bool) -> ExecutionPlan {
    let mut primary = PlannedRuntime::installed("rust", VERSION.to_string(), PathBuf::new());
    primary.version = VersionResolution::Installed {
        version: VERSION.to_string(),
        source: vx_resolver::VersionSource::VxManaged,
    };
    if warm_cache {
        let bootstrapper = BootstrappingRuntime::bootstrapper(context);
        BootstrappingRuntime::write(&bootstrapper);
        primary.executable = Some(bootstrapper);
    } else {
        primary.executable = None;
    }
    ExecutionPlan::new(primary, ExecutionConfig::default())
}

#[rstest]
#[case(false, ResolutionFailure::Missing)]
#[case(true, ResolutionFailure::Missing)]
#[case(false, ResolutionFailure::NonexistentPath)]
#[case(true, ResolutionFailure::NonexistentPath)]
#[case(false, ResolutionFailure::Error)]
#[case(true, ResolutionFailure::Error)]
#[tokio::test]
async fn test_successful_hook_without_final_executable_never_dispatches_bootstrapper(
    #[case] warm_cache: bool,
    #[case] failure: ResolutionFailure,
) {
    let root = tempfile::tempdir().expect("temporary store");
    let context = real_context(root.path());
    let registry = ProviderRegistry::new();
    registry.register(Arc::new(IncompleteProvider {
        failure,
        system_install: false,
        has_hook: true,
    }));
    let config = ResolverConfig::default();
    let resolver = Resolver::new(config.clone(), RuntimeMap::empty()).expect("resolver");
    let stage = EnsureStage::new(&resolver, &config, Some(&registry), Some(&context));

    let result = stage.execute(bootstrap_plan(&context, warm_cache)).await;

    let error = result.expect_err("incomplete bootstrap must not be executable");
    assert!(
        error.to_string().contains("Post-extract hook")
            || error.to_string().contains("post-extract hook"),
        "{error}"
    );
    assert!(BootstrappingRuntime::bootstrapper(&context).is_file());
    assert!(!BootstrappingRuntime::real_executable(&context).exists());
}

#[rstest]
#[case(true, true)]
#[case(false, false)]
#[tokio::test]
async fn test_final_executable_gate_preserves_system_and_no_hook_installs(
    #[case] system_install: bool,
    #[case] has_hook: bool,
) {
    let root = tempfile::tempdir().expect("temporary store");
    let context = real_context(root.path());
    let registry = ProviderRegistry::new();
    registry.register(Arc::new(IncompleteProvider {
        failure: ResolutionFailure::Error,
        system_install,
        has_hook,
    }));
    let config = ResolverConfig::default();
    let resolver = Resolver::new(config.clone(), RuntimeMap::empty()).expect("resolver");
    let stage = EnsureStage::new(&resolver, &config, Some(&registry), Some(&context));

    let result = stage
        .execute(bootstrap_plan(&context, false))
        .await
        .expect("system and no-hook installs retain their existing contract");

    assert_eq!(
        result.primary.executable,
        Some(BootstrappingRuntime::bootstrapper(&context))
    );
}
