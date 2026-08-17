//! Tests for PrepareStage proxy execution fallback behavior.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use tempfile::tempdir;
use vx_manifest::ProviderManifest;
use vx_resolver::{
    ExecutionConfig, ExecutionPlan, PlannedRuntime, PrepareStage, ProjectToolsConfig, Resolver,
    ResolverConfig, RuntimeMap, Stage,
};
use vx_runtime::{
    ExecutionContext, ExecutionPrep, InstallResult, Provider, ProviderRegistry, Runtime,
    RuntimeContext, VersionInfo, mock_context,
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

struct CountingRuntime {
    name: &'static str,
    executable: &'static str,
    installed_versions: Vec<String>,
    installed_error: Option<&'static str>,
    install_count: Arc<AtomicUsize>,
    prepare_count: Arc<AtomicUsize>,
}

#[async_trait]
impl Runtime for CountingRuntime {
    fn name(&self) -> &str {
        self.name
    }

    fn executable_name(&self) -> &str {
        self.executable
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("system")])
    }

    async fn installed_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<String>> {
        if let Some(message) = self.installed_error {
            return Err(anyhow!(message));
        }
        Ok(self.installed_versions.clone())
    }

    async fn install(&self, version: &str, _ctx: &RuntimeContext) -> Result<InstallResult> {
        self.install_count.fetch_add(1, Ordering::SeqCst);
        Ok(InstallResult::success(
            PathBuf::from("unused-install"),
            PathBuf::from("unused-executable"),
            version.to_string(),
        ))
    }

    async fn prepare_environment(
        &self,
        version: &str,
        ctx: &RuntimeContext,
    ) -> Result<HashMap<String, String>> {
        self.prepare_count.fetch_add(1, Ordering::SeqCst);
        let mut environment =
            HashMap::from([("VX_COMPANION_MARKER".to_string(), version.to_string())]);
        if let Some(components) = ctx.get_install_option("VX_MSVC_COMPONENTS") {
            environment.insert(
                "VX_COMPANION_COMPONENTS".to_string(),
                components.to_string(),
            );
        }
        // Simulate a toolchain environment whose PATH/INCLUDE/LIB must be
        // prepended to the primary runtime's environment (not dropped).
        environment.insert("PATH".to_string(), "C:/companion/toolchain/bin".to_string());
        environment.insert(
            "INCLUDE".to_string(),
            "C:/companion/toolchain/include".to_string(),
        );
        environment.insert("LIB".to_string(), "C:/companion/toolchain/lib".to_string());
        Ok(environment)
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
    create_runtime_map_named("uvx", "uv", path_prepend)
}

fn create_runtime_map_named(name: &str, executable: &str, path_prepend: &str) -> RuntimeMap {
    let escaped_path = path_prepend.replace('\\', "\\\\");
    let manifest = ProviderManifest::parse(&format!(
        r#"
[provider]
name = "test"
ecosystem = "custom"

[[runtimes]]
name = "{name}"
executable = "{executable}"

[runtimes.env.advanced]
path_prepend = ["{escaped_path}"]
"#
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

#[tokio::test]
async fn prepare_stage_skips_missing_companion_without_auto_installing() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).expect("resolver should build");
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let install_count = Arc::new(AtomicUsize::new(0));
    let prepare_count = Arc::new(AtomicUsize::new(0));

    registry.register(Arc::new(TestProvider {
        runtimes: vec![
            Arc::new(BundledRuntime {
                name: "git",
                executable: "git",
            }),
            Arc::new(CountingRuntime {
                name: "msvc",
                executable: "cl",
                installed_versions: Vec::new(),
                installed_error: None,
                install_count: install_count.clone(),
                prepare_count: prepare_count.clone(),
            }),
        ],
    }));

    let project_config = ProjectToolsConfig::from_tools_with_install_options(
        HashMap::from([("msvc".to_string(), "14.42".to_string())]),
        HashMap::from([(
            "msvc".to_string(),
            HashMap::from([("VX_MSVC_COMPONENTS".to_string(), "spectre".to_string())]),
        )]),
    );
    let stage = PrepareStage::new(&resolver, &config, Some(&registry), Some(&context))
        .with_project_config(&project_config);
    let plan = ExecutionPlan::new(
        PlannedRuntime::installed("git", "2.51.0".to_string(), PathBuf::from("/usr/bin/git")),
        ExecutionConfig::default(),
    );

    let prepared = stage
        .execute(plan)
        .await
        .expect("missing companion should not fail unrelated command preparation");

    assert_eq!(install_count.load(Ordering::SeqCst), 0);
    assert_eq!(prepare_count.load(Ordering::SeqCst), 0);
    assert!(!prepared.env.contains_key("VX_COMPANION_MARKER"));
}

#[tokio::test]
async fn prepare_stage_injects_already_installed_companion_environment() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).expect("resolver should build");
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let install_count = Arc::new(AtomicUsize::new(0));
    let prepare_count = Arc::new(AtomicUsize::new(0));

    registry.register(Arc::new(TestProvider {
        runtimes: vec![
            Arc::new(BundledRuntime {
                name: "git",
                executable: "git",
            }),
            Arc::new(CountingRuntime {
                name: "msvc",
                executable: "cl",
                installed_versions: vec!["system".to_string()],
                installed_error: None,
                install_count: install_count.clone(),
                prepare_count: prepare_count.clone(),
            }),
        ],
    }));

    let project_config = ProjectToolsConfig::from_tools_with_install_options(
        HashMap::from([("msvc".to_string(), "14.42".to_string())]),
        HashMap::from([(
            "msvc".to_string(),
            HashMap::from([("VX_MSVC_COMPONENTS".to_string(), "spectre".to_string())]),
        )]),
    );
    let stage = PrepareStage::new(&resolver, &config, Some(&registry), Some(&context))
        .with_project_config(&project_config);
    let plan = ExecutionPlan::new(
        PlannedRuntime::installed("git", "2.51.0".to_string(), PathBuf::from("/usr/bin/git")),
        ExecutionConfig::default(),
    );

    let prepared = stage
        .execute(plan)
        .await
        .expect("installed companion should inject environment");

    assert_eq!(install_count.load(Ordering::SeqCst), 0);
    assert_eq!(prepare_count.load(Ordering::SeqCst), 1);
    assert_eq!(
        prepared.env.get("VX_COMPANION_MARKER").map(String::as_str),
        Some("system")
    );
    assert_eq!(
        prepared
            .env
            .get("VX_COMPANION_COMPONENTS")
            .map(String::as_str),
        Some("spectre")
    );
}

#[tokio::test]
async fn prepare_stage_skips_broken_companion_with_install_options_without_repairing() {
    let config = ResolverConfig::default();
    let runtime_map = RuntimeMap::empty();
    let resolver = Resolver::new(config.clone(), runtime_map).expect("resolver should build");
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let install_count = Arc::new(AtomicUsize::new(0));
    let prepare_count = Arc::new(AtomicUsize::new(0));

    registry.register(Arc::new(TestProvider {
        runtimes: vec![
            Arc::new(BundledRuntime {
                name: "git",
                executable: "git",
            }),
            Arc::new(CountingRuntime {
                name: "msvc",
                executable: "cl",
                installed_versions: Vec::new(),
                installed_error: Some("msvc system directory exists but executable not found"),
                install_count: install_count.clone(),
                prepare_count: prepare_count.clone(),
            }),
        ],
    }));

    let project_config = ProjectToolsConfig::from_tools_with_install_options(
        HashMap::from([("msvc".to_string(), "14.42".to_string())]),
        HashMap::from([(
            "msvc".to_string(),
            HashMap::from([("VX_MSVC_COMPONENTS".to_string(), "spectre".to_string())]),
        )]),
    );
    let stage = PrepareStage::new(&resolver, &config, Some(&registry), Some(&context))
        .with_project_config(&project_config);
    let plan = ExecutionPlan::new(
        PlannedRuntime::installed("git", "2.51.0".to_string(), PathBuf::from("/usr/bin/git")),
        ExecutionConfig::default(),
    );

    let prepared = stage
        .execute(plan)
        .await
        .expect("broken companion should not fail unrelated command preparation");

    assert_eq!(install_count.load(Ordering::SeqCst), 0);
    assert_eq!(prepare_count.load(Ordering::SeqCst), 0);
    assert!(!prepared.env.contains_key("VX_COMPANION_MARKER"));
}

#[tokio::test]
async fn prepare_stage_prepends_companion_toolchain_environment() {
    let temp_dir = tempdir().expect("temp dir should be created");
    let primary_bin = temp_dir.path().join("primary-bin");

    let config = ResolverConfig::default();
    let runtime_map = create_runtime_map_named("cargo", "cargo", &primary_bin.to_string_lossy());
    let resolver = Resolver::new(config.clone(), runtime_map).expect("resolver should build");
    let registry = ProviderRegistry::new();
    let context = mock_context();
    let install_count = Arc::new(AtomicUsize::new(0));
    let prepare_count = Arc::new(AtomicUsize::new(0));

    registry.register(Arc::new(TestProvider {
        runtimes: vec![
            Arc::new(BundledRuntime {
                name: "cargo",
                executable: "cargo",
            }),
            Arc::new(CountingRuntime {
                name: "msvc",
                executable: "cl",
                installed_versions: vec!["system".to_string()],
                installed_error: None,
                install_count: install_count.clone(),
                prepare_count: prepare_count.clone(),
            }),
        ],
    }));

    let project_config =
        ProjectToolsConfig::from_tools(HashMap::from([("msvc".to_string(), "14.42".to_string())]));
    let stage = PrepareStage::new(&resolver, &config, Some(&registry), Some(&context))
        .with_project_config(&project_config);
    let plan = ExecutionPlan::new(
        PlannedRuntime::installed(
            "cargo",
            "1.0.0".to_string(),
            PathBuf::from("/usr/bin/cargo"),
        ),
        ExecutionConfig::default(),
    );

    let prepared = stage
        .execute(plan)
        .await
        .expect("companion environment should be prepended to primary PATH");

    let path = prepared.env.get("PATH").expect("PATH should be present");
    assert!(
        path.starts_with("C:/companion/toolchain/bin"),
        "companion toolchain bin should be first on PATH, got: {path}"
    );
    assert!(
        path.contains(&primary_bin.to_string_lossy().to_string()),
        "primary bin dir should remain on PATH, got: {path}"
    );
    assert_eq!(
        prepared.env.get("INCLUDE").map(String::as_str),
        Some("C:/companion/toolchain/include")
    );
    assert_eq!(
        prepared.env.get("LIB").map(String::as_str),
        Some("C:/companion/toolchain/lib")
    );
}
