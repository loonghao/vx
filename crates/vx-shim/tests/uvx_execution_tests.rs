//! UVX executes requested commands inside package environments without global shims.

use std::path::{Path, PathBuf};
use std::process::Command;

use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_shim::{PackageRequest, ShimExecutor};

fn fake_uv(bin: &Path) {
    std::fs::create_dir_all(bin).unwrap();
    #[cfg(windows)]
    {
        std::fs::write(
            bin.join("capture.ps1"),
            "[IO.File]::WriteAllLines($env:VX_UVX_TEST_CAPTURE, @($PWD.Path, $env:PATH) + $args)\nexit 23\n",
        ).unwrap();
        let system_root = std::env::var("SystemRoot").unwrap();
        let powershell =
            Path::new(&system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
        std::fs::write(bin.join("uv.cmd"), format!(
            "@echo off\r\n\"{}\" -NoProfile -ExecutionPolicy Bypass -File \"%~dp0capture.ps1\" %*\r\nexit /b %errorlevel%\r\n",
            powershell.display()
        )).unwrap();
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let executable = bin.join("uv");
        std::fs::write(&executable, "#!/bin/sh\nprintf '%s\\n' \"$PWD\" \"$PATH\" \"$@\" > \"$VX_UVX_TEST_CAPTURE\"\nexit 23\n").unwrap();
        std::fs::set_permissions(executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn check_execution(request: &str, expected_spec: &str, expected_executable: &str) {
    let temp = tempfile::tempdir().unwrap();
    let working_dir = temp.path().join("working directory with spaces");
    std::fs::create_dir_all(&working_dir).unwrap();
    let vx_home = temp.path().join("vx-home");
    // uv's portable distribution keeps its executable at the version root.
    // Windows tool environment discovery recognizes .exe when selecting bin/;
    // our .cmd test double therefore uses the supported root layout.
    let uv_bin = vx_home.join("store/uv/9.8.7");
    fake_uv(&uv_bin);
    let registry_path = temp.path().join("packages.json");
    let shims_dir = temp.path().join("shims");
    let mut registry = PackageRegistry::new();
    registry.register(
        GlobalPackage::new(
            "example-library",
            "1.0.0",
            "uvx",
            temp.path().join("package"),
        )
        .with_executable("example-library")
        .with_runtime_dependency("uv", "9.8.7"),
    );
    registry.save(&registry_path).unwrap();
    let capture = temp.path().join("arguments.txt");
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "test_uvx_execution_child",
            "--ignored",
            "--nocapture",
        ])
        .current_dir(&working_dir)
        .env("VX_HOME", &vx_home)
        .env("VX_UVX_TEST_REGISTRY", &registry_path)
        .env("VX_UVX_TEST_SHIMS", &shims_dir)
        .env("VX_UVX_TEST_REQUEST", request)
        .env("VX_UVX_TEST_CAPTURE", &capture)
        .status()
        .unwrap();
    assert!(status.success());
    let output = std::fs::read_to_string(capture).unwrap();
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(
        std::fs::canonicalize(lines[0]).unwrap(),
        std::fs::canonicalize(&working_dir).unwrap()
    );
    assert!(
        std::env::split_paths(lines[1]).any(|path| path == uv_bin),
        "Prepared environment must include managed uv"
    );
    assert_eq!(
        &lines[2..],
        [
            "tool",
            "run",
            "--from",
            expected_spec,
            expected_executable,
            "-c",
            "print('argument with spaces')",
            "second argument"
        ]
    );
    assert!(!shims_dir.join("python.cmd").exists());
    assert!(!shims_dir.join("python").exists());
}

#[test]
fn test_uvx_explicit_python_and_version_override_need_no_entrypoint() {
    check_execution(
        "uvx:example-library@2.3.4::python",
        "example-library==2.3.4",
        "python",
    );
}

#[test]
fn test_uvx_different_cli_name_uses_registered_version() {
    check_execution(
        "uvx:example-library::different-command",
        "example-library==1.0.0",
        "different-command",
    );
}

#[test]
fn test_uvx_latest_does_not_generate_invalid_equality_pin() {
    check_execution(
        "uvx:example-library@latest::python",
        "example-library",
        "python",
    );
}

#[test]
fn test_uvx_registered_default_command_needs_no_physical_shim() {
    check_execution("global", "example-library==1.0.0", "example-library");
}

// A subprocess avoids mutating VX_HOME or PATH in the parallel test process.
#[test]
#[ignore = "fixture invoked in an isolated process by UVX execution tests"]
fn test_uvx_execution_child() {
    let registry = PathBuf::from(std::env::var_os("VX_UVX_TEST_REGISTRY").unwrap());
    let shims = PathBuf::from(std::env::var_os("VX_UVX_TEST_SHIMS").unwrap());
    let request = std::env::var("VX_UVX_TEST_REQUEST").unwrap();
    let executor = ShimExecutor::new(registry, shims);
    let args = ["-c", "print('argument with spaces')", "second argument"].map(str::to_string);
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let exit = if request == "global" {
        runtime
            .block_on(executor.try_execute("example-library", &args))
            .unwrap()
            .unwrap()
    } else {
        let request = PackageRequest::parse(&request).unwrap();
        runtime
            .block_on(executor.execute_request(&request, &args))
            .unwrap()
    };
    assert_eq!(
        exit, 23,
        "UVX must preserve the executed command's exit code"
    );
}
