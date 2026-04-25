//! Local, non-network coverage for global shim management commands.

mod common;

use common::{assert_success, init_test_env, vx_available, vx_binary};
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;
use vx_paths::VxPaths;
use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::shims;

#[test]
fn test_global_shim_update_recreates_from_registry() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let vx_path = vx_binary();
    if !vx_path.exists() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let paths = VxPaths::with_base_dir(&vx_home);
    paths
        .ensure_dirs()
        .expect("failed to initialize vx directories");

    let package_dir = paths.global_package_dir("npm", "vite", "5.4.0");
    let package_bin_dir = package_dir.join("bin");
    std::fs::create_dir_all(&package_bin_dir).expect("failed to create package bin dir");
    std::fs::write(package_bin_dir.join("vite"), "shim target")
        .expect("failed to create fake executable");

    let mut registry = PackageRegistry::new();
    registry
        .register(GlobalPackage::new("vite", "5.4.0", "npm", package_dir).with_executable("vite"));
    registry
        .save(&paths.packages_registry_file())
        .expect("failed to save registry");

    let stale_target = package_bin_dir.join("stale");
    std::fs::write(&stale_target, "stale").expect("failed to create stale target");
    shims::create_shim(&paths.shims_dir, "stale", &stale_target)
        .expect("failed to create stale shim");

    let output = run_vx_with_home(&vx_path, temp.path(), &vx_home, &["global", "shim-update"])
        .expect("failed to run vx global shim-update");
    assert_success(&output, "vx global shim-update");

    assert!(shims::shim_exists(&paths.shims_dir, "vite"));
    assert!(!shims::shim_exists(&paths.shims_dir, "stale"));
}

#[test]
fn test_global_uninstall_removes_stack_shims_and_registry() {
    init_test_env();
    if !vx_available() {
        return;
    }

    let vx_path = vx_binary();
    if !vx_path.exists() {
        return;
    }

    let temp = TempDir::new().expect("failed to create temp dir");
    let vx_home = temp.path().join("vx-home");
    let paths = VxPaths::with_base_dir(&vx_home);
    paths
        .ensure_dirs()
        .expect("failed to initialize vx directories");

    let package_name = "toolpkg";
    let ecosystem = "npm";
    let version = "1.0.0";
    let executable = "toolx";

    let package_dir = paths.global_package_dir(ecosystem, package_name, version);
    let package_bin_dir = package_dir.join("bin");
    std::fs::create_dir_all(&package_bin_dir).expect("failed to create package bin dir");
    let target_path = package_bin_dir.join(executable);
    std::fs::write(&target_path, "shim target").expect("failed to create fake executable");

    let mut registry = PackageRegistry::new();
    registry.register(
        GlobalPackage::new(package_name, version, ecosystem, package_dir.clone())
            .with_executable(executable),
    );
    registry
        .save(&paths.packages_registry_file())
        .expect("failed to save registry");

    shims::create_shim(&paths.shims_dir, executable, &target_path)
        .expect("failed to create primary shim");

    let vx_bin_dir = vx_path.parent().map(PathBuf::from);
    if let Some(dir) = &vx_bin_dir {
        let _ = shims::create_shim(dir, executable, &target_path);
    }

    let output = run_vx_with_home(
        &vx_path,
        temp.path(),
        &vx_home,
        &["global", "uninstall", "npm:toolpkg", "--force"],
    )
    .expect("failed to run vx global uninstall");
    assert_success(&output, "vx global uninstall npm:toolpkg --force");

    let registry_after = PackageRegistry::load(&paths.packages_registry_file())
        .expect("failed to reload package registry");
    assert!(!registry_after.contains(ecosystem, package_name));
    assert!(!package_dir.exists());
    assert!(!shims::shim_exists(&paths.shims_dir, executable));

    if let Some(dir) = vx_bin_dir {
        assert!(
            !shims::shim_exists(&dir, executable),
            "stacked shim should be removed from vx bin dir"
        );
    }
}

fn run_vx_with_home(
    vx_path: &std::path::Path,
    cwd: &std::path::Path,
    vx_home: &std::path::Path,
    args: &[&str],
) -> std::io::Result<Output> {
    Command::new(vx_path)
        .args(args)
        .current_dir(cwd)
        .env("VX_HOME", vx_home)
        .output()
}
