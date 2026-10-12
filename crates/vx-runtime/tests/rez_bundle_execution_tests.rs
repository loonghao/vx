//! Runtime-level contract tests for Rez bundle activation.

#![cfg(feature = "rez-bundles")]

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use vx_runtime::{
    ExecutionContext, ManifestDrivenRuntime, ProviderSource, RealCommandExecutor, RezBundleRequest,
    Runtime,
};

#[test]
fn rez_bundle_preparation_resolves_the_program_and_inherits_the_parent_environment() {
    let fixture = BundleFixture::new();
    let package_root = fixture.write_package(
        "tool",
        "1.0.0",
        "def commands():\n    env.PATH.prepend(\"{root}/bin\")\n    env.VX_REZ_PACKAGE = \"enabled\"\n",
    );
    fixture.write_host_packages();
    let executable = write_test_executable(&package_root.join("bin"));
    let repository = fixture.repository().to_path_buf();

    let runtime = ManifestDrivenRuntime::new("tool", "test", ProviderSource::BuiltIn)
        .with_store_name("tool-rez")
        .with_rez_bundle(move |_version| {
            let repository = repository.clone();
            Box::pin(async move {
                Ok(Some(RezBundleRequest::new(
                    repository,
                    ["tool-1.0.0"],
                    host_platform(),
                    host_architecture(),
                    "vx-rez-runtime-test-tool",
                )))
            })
        });
    let context = ExecutionContext {
        working_dir: None,
        env: HashMap::from([("VX_REZ_PARENT".to_string(), "preserved".to_string())]),
        capture_output: false,
        timeout: None,
        executor: Arc::new(RealCommandExecutor),
    };

    let prepared = tokio_test::block_on(runtime.prepare_execution("1.0.0", &context))
        .expect("Rez bundle activation should succeed");

    assert!(prepared.proxy_ready);
    assert_same_path(
        prepared
            .executable_override
            .as_deref()
            .expect("Rez program should resolve"),
        &executable,
    );
    assert_eq!(
        prepared.env_vars.get("VX_REZ_PARENT").map(String::as_str),
        Some("preserved")
    );
    assert_eq!(
        prepared.env_vars.get("VX_REZ_PACKAGE").map(String::as_str),
        Some("enabled")
    );
    assert_eq!(runtime.store_name(), "tool-rez");
    assert!(runtime.is_version_installable("1.0.0"));
}

struct BundleFixture {
    _temporary: tempfile::TempDir,
    repository: PathBuf,
}

impl BundleFixture {
    fn new() -> Self {
        let temporary = tempfile::tempdir().expect("temporary directory should be created");
        let repository = temporary.path().join("bundle");
        fs::create_dir_all(&repository).expect("bundle directory should be created");
        Self {
            _temporary: temporary,
            repository,
        }
    }

    fn repository(&self) -> &Path {
        &self.repository
    }

    fn write_host_packages(&self) {
        self.write_package("platform", host_platform(), "");
        self.write_package("arch", host_architecture(), "");
    }

    fn write_package(&self, name: &str, version: &str, commands: &str) -> PathBuf {
        let package_root = self.repository.join(name).join(version);
        fs::create_dir_all(&package_root).expect("package root should be created");
        fs::write(
            package_root.join("package.py"),
            format!("name = \"{name}\"\nversion = \"{version}\"\n\n{commands}"),
        )
        .expect("package definition should be written");
        package_root
    }
}

fn host_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "osx",
        platform => platform,
    }
}

fn host_architecture() -> &'static str {
    match std::env::consts::ARCH {
        "aarch64" => "arm_64",
        architecture => architecture,
    }
}

#[cfg(windows)]
fn write_test_executable(directory: &Path) -> PathBuf {
    fs::create_dir_all(directory).expect("bin directory should be created");
    let executable = directory.join("vx-rez-runtime-test-tool.exe");
    fs::copy(
        std::env::current_exe().expect("test executable should be available"),
        &executable,
    )
    .expect("test executable should be copied");
    executable
}

#[cfg(unix)]
fn write_test_executable(directory: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    fs::create_dir_all(directory).expect("bin directory should be created");
    let executable = directory.join("vx-rez-runtime-test-tool");
    fs::write(&executable, "#!/bin/sh\nexit 0\n").expect("test executable should be written");
    let mut permissions = executable
        .metadata()
        .expect("test executable metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&executable, permissions)
        .expect("test executable permissions should be set");
    executable
}

#[cfg(windows)]
fn assert_same_path(actual: &Path, expected: &Path) {
    assert!(
        actual
            .canonicalize()
            .expect("actual path should canonicalize")
            .to_string_lossy()
            .eq_ignore_ascii_case(
                &expected
                    .canonicalize()
                    .expect("expected path should canonicalize")
                    .to_string_lossy()
            )
    );
}

#[cfg(unix)]
fn assert_same_path(actual: &Path, expected: &Path) {
    assert_eq!(actual, expected);
}
