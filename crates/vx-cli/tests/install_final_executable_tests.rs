//! Regression tests for executable resolution after installation hooks.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use rstest::rstest;

use vx_cli::commands::install::install_quiet;
use vx_runtime::testing::{MockHttpClient, MockInstaller};
use vx_runtime::{
    Ecosystem, InstallResult, Provider, ProviderRegistry, RealFileSystem, RealPathProvider,
    Runtime, RuntimeContext, VersionInfo,
};

const VERSION: &str = "1.0.0";
const RUNTIME: &str = "hook-final-entry";

#[derive(Debug, Clone, Copy)]
enum Lookup {
    Final,
    Missing,
    Error,
}

#[derive(Debug)]
struct HookRuntime {
    result: InstallResult,
    bootstrap_path: PathBuf,
    final_path: PathBuf,
    lookup: Lookup,
    installs: AtomicUsize,
    hooks: AtomicUsize,
    lookup_versions: Mutex<Vec<String>>,
}

impl HookRuntime {
    fn new(ctx: &RuntimeContext, lookup: Lookup) -> Self {
        let install_path = ctx.paths.version_store_dir(RUNTIME, VERSION);
        let bootstrap_path = install_path.join("downloaded.AppImage");
        let final_path = install_path.join("squashfs-root").join("AppRun");
        Self {
            result: InstallResult::success(
                install_path,
                bootstrap_path.clone(),
                VERSION.to_owned(),
            ),
            bootstrap_path,
            final_path,
            lookup,
            installs: AtomicUsize::new(0),
            hooks: AtomicUsize::new(0),
            lookup_versions: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Runtime for HookRuntime {
    fn name(&self) -> &str {
        RUNTIME
    }

    fn description(&self) -> &str {
        "Runtime whose install hook creates the final executable"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Generic
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new(VERSION)])
    }

    async fn install(&self, _version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        self.installs.fetch_add(1, Ordering::Relaxed);
        ctx.fs
            .create_dir_all(self.bootstrap_path.parent().expect("bootstrap parent"))?;
        ctx.fs
            .write_bytes(&self.bootstrap_path, b"downloaded bootstrap")?;
        Ok(self.result.clone())
    }

    async fn post_install(&self, _version: &str, ctx: &RuntimeContext) -> Result<()> {
        self.hooks.fetch_add(1, Ordering::Relaxed);
        ctx.fs
            .create_dir_all(self.final_path.parent().expect("final executable parent"))?;
        ctx.fs
            .write_bytes(&self.final_path, b"final application entry")?;
        Ok(())
    }

    async fn get_executable_path_for_version(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<PathBuf>> {
        self.lookup_versions
            .lock()
            .expect("lookup versions")
            .push(version.to_owned());
        match self.lookup {
            Lookup::Final => Ok(ctx
                .fs
                .exists(&self.final_path)
                .then(|| self.final_path.clone())),
            Lookup::Missing => Ok(None),
            Lookup::Error => Err(anyhow::anyhow!("runtime executable lookup unavailable")),
        }
    }
}

struct HookProvider(Arc<HookRuntime>);

impl Provider for HookProvider {
    fn name(&self) -> &str {
        "hook-provider"
    }

    fn description(&self) -> &str {
        "Installation hook fixture"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![self.0.clone()]
    }
}

fn context(root: &Path) -> RuntimeContext {
    RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(root)),
        Arc::new(MockHttpClient::new()),
        Arc::new(RealFileSystem),
        Arc::new(MockInstaller::new()),
    )
}

fn registry(runtime: Arc<HookRuntime>) -> ProviderRegistry {
    let registry = ProviderRegistry::new();
    registry.register(Arc::new(HookProvider(runtime)));
    registry
}

#[tokio::test]
async fn test_install_quiet_returns_final_entry_on_fresh_and_warm_install() {
    let root = tempfile::tempdir().expect("isolated runtime store");
    let ctx = context(root.path());
    let runtime = Arc::new(HookRuntime::new(&ctx, Lookup::Final));
    let registry = registry(runtime.clone());
    assert!(!runtime.bootstrap_path.exists());
    assert!(!runtime.final_path.exists());

    let fresh = install_quiet(&registry, &ctx, RUNTIME)
        .await
        .expect("fresh installation");
    assert!(runtime.bootstrap_path.is_file());
    assert!(runtime.final_path.is_file());
    assert_eq!(fresh.executable_path, runtime.final_path);
    assert_ne!(fresh.executable_path, runtime.bootstrap_path);
    assert_eq!(fresh.install_path, runtime.result.install_path);
    assert_eq!(fresh.version, VERSION);
    assert!(fresh.success);
    assert!(!fresh.already_installed);

    let warm = install_quiet(&registry, &ctx, RUNTIME)
        .await
        .expect("warm installation");
    assert_eq!(warm.executable_path, fresh.executable_path);
    assert_eq!(warm.install_path, fresh.install_path);
    assert_eq!(warm.version, fresh.version);
    assert!(warm.success);
    assert!(warm.already_installed);
    assert_eq!(runtime.installs.load(Ordering::Relaxed), 1);
    assert_eq!(runtime.hooks.load(Ordering::Relaxed), 1);
}

#[rstest]
#[case::managed_missing(Lookup::Missing, false, true)]
#[case::managed_error(Lookup::Error, false, true)]
#[case::system_resolved_path(Lookup::Missing, true, true)]
#[case::system_placeholder(Lookup::Missing, true, false)]
#[tokio::test]
async fn test_install_quiet_preserves_fallback_when_final_lookup_is_unavailable(
    #[case] lookup: Lookup,
    #[case] system_install: bool,
    #[case] known_system_path: bool,
) {
    let root = tempfile::tempdir().expect("isolated runtime store");
    let ctx = context(root.path());
    let mut runtime = HookRuntime::new(&ctx, lookup);
    if system_install {
        runtime.bootstrap_path = root.path().join("system-bin").join("runtime");
        runtime.final_path = root.path().join("system-bin").join("final-runtime");
        runtime.result = InstallResult::system_installed(
            VERSION.to_owned(),
            known_system_path.then(|| runtime.bootstrap_path.clone()),
        );
    }
    let expected = runtime.result.clone();
    let runtime = Arc::new(runtime);
    let registry = registry(runtime.clone());

    let result = install_quiet(&registry, &ctx, RUNTIME)
        .await
        .expect("installation with executable fallback");
    assert_eq!(result.executable_path, expected.executable_path);
    assert_eq!(result.install_path, expected.install_path);
    assert_eq!(result.version, expected.version);
    assert_eq!(result.success, expected.success);
    assert_eq!(result.already_installed, expected.already_installed);
    assert_eq!(
        *runtime.lookup_versions.lock().expect("lookup versions"),
        vec![expected.version]
    );
}

#[tokio::test]
async fn test_final_executable_resolution_preserves_original_install_state() {
    let root = tempfile::tempdir().expect("isolated runtime store");
    let ctx = context(root.path());
    let mut runtime = HookRuntime::new(&ctx, Lookup::Final);
    runtime.result.success = false;
    runtime.result.already_installed = true;
    runtime.result.version = "reported-installed-version".to_owned();
    let expected = runtime.result.clone();
    let final_path = runtime.final_path.clone();
    let runtime = Arc::new(runtime);
    let registry = registry(runtime.clone());

    let result = install_quiet(&registry, &ctx, RUNTIME)
        .await
        .expect("installer result should be preserved");
    assert_eq!(result.executable_path, final_path);
    assert_eq!(result.install_path, expected.install_path);
    assert_eq!(result.version, expected.version);
    assert_eq!(result.success, expected.success);
    assert_eq!(result.already_installed, expected.already_installed);
    assert_eq!(
        *runtime.lookup_versions.lock().expect("lookup versions"),
        vec![expected.version]
    );
}
