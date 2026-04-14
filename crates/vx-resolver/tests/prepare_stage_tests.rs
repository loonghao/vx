//! Tests for PrepareStage proxy execution fallback behavior.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tempfile::tempdir;
use vx_manifest::ProviderManifest;
use vx_resolver::{
    ExecutionConfig, ExecutionPlan, PlannedRuntime, PrepareStage, Resolver, ResolverConfig,
    RuntimeMap, Stage,
};
use vx_runtime::{
    ExecutionContext, ExecutionPrep, Provider, ProviderRegistry, Runtime, RuntimeContext,
    VersionInfo,
};

struct BundledRuntime {
    name: &'static str,
    executable: &'static str,
}

#[async_trait]
impl Runtime for BundledRuntime {
    fn name(&self) -> &str {
        self.name
    }

    fn executable_name(&self) -> &str {
        self.executable
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("1.0.0")])
    }

    async fn prepare_execution(
        &self,
        _version: &str,
        _ctx: &ExecutionContext,
    ) -> Result<ExecutionPrep> {
        Ok(ExecutionPrep::proxy_ready()
            .with_prefix("tool")
            .with_prefix("run"))
    }
}

struct TestProvider {
    runtimes: Vec<Arc<dyn Runtime>>,
}

impl Provider for TestProvider {
    fn name(&self) -> &str {
        "test"
    }

    fn description(&self) -> &str {
        "Test provider"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        self.runtimes.clone()
    }
}

#[cfg(unix)]
fn create_mock_executable(dir: &std::path::Path, name: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\nexit 0\n").expect("mock executable should be created");

    let mut perms = fs::metadata(&path)
        .expect("metadata should be available")
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).expect("permissions should be updated");
    path
}

#[cfg(windows)]
fn create_mock_executable(dir: &std::path::Path, name: &str) -> PathBuf {
    let path = dir.join(format!("{}.cmd", name));
    fs::write(&path, "@echo off\r\nexit /b 0\r\n").expect("mock executable should be created");
    path
}

fn create_runtime_map(path_prepend: &str) -> RuntimeMap {
    let escaped_path = path_prepend.replace('\\', "\\\\");
    let manifest = ProviderManifest::parse(&format!(
        r#"
[provider]
name = "test"
ecosystem = "custom"

[[runtimes]]
name = "uvx"
executable = "uv"

[runtimes.env.advanced]
path_prepend = ["{}"]
"#,
        escaped_path
    ))
    .expect("manifest should parse");

    RuntimeMap::from_manifests(&[manifest])
}

#[tokio::test]
async fn prepare_stage_uses_runtime_executable_name_for_system_path_fallback() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let mock_executable = create_mock_executable(temp_dir.path(), "uv");

    let config = ResolverConfig::default();
    let runtime_map = create_runtime_map(&temp_dir.path().to_string_lossy());
    let resolver = Resolver::new(config.clone(), runtime_map).expect("resolver should build");
    let registry = ProviderRegistry::new();
    registry.register(Arc::new(TestProvider {
        runtimes: vec![Arc::new(BundledRuntime {
            name: "uvx",
            executable: "uv",
        })],
    }));

    let stage = PrepareStage::new(&resolver, &config, Some(&registry), None);
    let plan = ExecutionPlan::new(
        PlannedRuntime::needs_install("uvx", "1.0.0".to_string()),
        ExecutionConfig::default(),
    );

    let prepared = stage
        .execute(plan)
        .await
        .expect("prepare stage should resolve bundled runtime via executable name");

    assert_eq!(prepared.executable, mock_executable);
    assert_eq!(prepared.command_prefix, vec!["tool", "run"]);
}
