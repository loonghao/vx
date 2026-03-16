//! Regression tests for bundled runtime installation resolution.
//!
//! These tests cover the CI install path used by `install_quiet()`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use rstest::*;
use vx_cli::commands::install::install_quiet;
use vx_runtime::{
    Ecosystem, ExecutionContext, ExecutionPrep, InstallResult, Platform, Provider,
    ProviderRegistry, Runtime, RuntimeContext, VersionInfo, mock_context,
};

const VERSION: &str = "1.0.0";

fn expected_install_path(ctx: &RuntimeContext) -> PathBuf {
    ctx.paths
        .version_store_dir("rustup", VERSION)
        .join(Platform::current().as_str())
}

fn expected_rustup_path(ctx: &RuntimeContext) -> PathBuf {
    let exe = if cfg!(windows) {
        "rustup.exe"
    } else {
        "rustup"
    };
    expected_install_path(ctx).join("bin").join(exe)
}

fn expected_cargo_path(ctx: &RuntimeContext) -> PathBuf {
    let exe = if cfg!(windows) { "cargo.exe" } else { "cargo" };
    expected_install_path(ctx)
        .join("cargo")
        .join("bin")
        .join(exe)
}

#[derive(Debug)]
struct MockRustupRuntime;

#[async_trait]
impl Runtime for MockRustupRuntime {
    fn name(&self) -> &str {
        "rustup"
    }

    fn description(&self) -> &str {
        "Mock rustup runtime"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new(VERSION)])
    }

    async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
        let install_path = ctx
            .paths
            .version_store_dir(self.store_name(), version)
            .join(Platform::current().as_str());
        let rustup_path = expected_rustup_path(ctx);
        let cargo_path = expected_cargo_path(ctx);

        ctx.fs.create_dir_all(&install_path)?;
        ctx.fs.write_bytes(&rustup_path, b"rustup")?;
        ctx.fs.write_bytes(&cargo_path, b"cargo")?;

        Ok(InstallResult::success(
            install_path,
            rustup_path,
            version.to_string(),
        ))
    }
}

#[derive(Debug)]
struct MockCargoRuntime {
    preferred_path: PathBuf,
    fallback_path: PathBuf,
}

#[async_trait]
impl Runtime for MockCargoRuntime {
    fn name(&self) -> &str {
        "cargo"
    }

    fn description(&self) -> &str {
        "Mock bundled cargo runtime"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn metadata(&self) -> HashMap<String, String> {
        HashMap::from([(String::from("bundled_with"), String::from("rustup"))])
    }

    fn store_name(&self) -> &str {
        "rustup"
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new(VERSION)])
    }

    async fn prepare_execution(
        &self,
        _version: &str,
        _ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        Ok(ExecutionPrep::with_executable(self.preferred_path.clone()))
    }

    async fn get_executable_path_for_version(
        &self,
        _version: &str,
        _ctx: &RuntimeContext,
    ) -> Result<Option<PathBuf>> {
        Ok(Some(self.fallback_path.clone()))
    }
}

struct MockRustProvider {
    runtimes: Vec<Arc<dyn Runtime>>,
}

impl Provider for MockRustProvider {
    fn name(&self) -> &str {
        "mock-rust"
    }

    fn description(&self) -> &str {
        "Mock rust provider"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        self.runtimes.clone()
    }
}

#[fixture]
fn registry_with_bundled_runtime() -> (ProviderRegistry, RuntimeContext, PathBuf, PathBuf) {
    let registry = ProviderRegistry::new();
    let ctx = mock_context();
    let cargo_path = expected_cargo_path(&ctx);
    let rustup_path = expected_rustup_path(&ctx);

    registry.register(Arc::new(MockRustProvider {
        runtimes: vec![
            Arc::new(MockRustupRuntime),
            Arc::new(MockCargoRuntime {
                preferred_path: cargo_path.clone(),
                fallback_path: rustup_path.clone(),
            }),
        ],
    }));

    (registry, ctx, cargo_path, rustup_path)
}

#[rstest]
#[tokio::test]
async fn test_install_quiet_prefers_prepare_execution_for_bundled_runtime(
    registry_with_bundled_runtime: (ProviderRegistry, RuntimeContext, PathBuf, PathBuf),
) {
    let (registry, ctx, cargo_path, rustup_path) = registry_with_bundled_runtime;

    let result = install_quiet(&registry, &ctx, "cargo")
        .await
        .expect("bundled runtime install should succeed");

    assert_eq!(result.version, VERSION);
    assert_eq!(result.executable_path, cargo_path);
    assert_ne!(result.executable_path, rustup_path);
    assert!(result.already_installed);
}
