//! UVX cache preparation accepts libraries and reports actual resolution failures.

use std::path::{Path, PathBuf};

use vx_ecosystem_pm::{EcosystemInstaller, InstallOptions, installers::UvxInstaller};

fn fake_uv(directory: &Path, exit_code: i32) -> (PathBuf, PathBuf) {
    let capture = directory.join("arguments.txt");
    #[cfg(windows)]
    {
        let script = directory.join("capture.ps1");
        std::fs::write(
            &script,
            format!(
                "[IO.File]::WriteAllLines('{}', $args)\nexit {exit_code}\n",
                capture.display().to_string().replace('\'', "''")
            ),
        )
        .unwrap();
        let powershell = PathBuf::from(std::env::var("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let uv = directory.join("uv.cmd");
        std::fs::write(&uv, format!("@echo off\r\n\"{}\" -NoProfile -ExecutionPolicy Bypass -File \"{}\" %*\r\nexit /b %errorlevel%\r\n", powershell.display(), script.display())).unwrap();
        (uv, capture)
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let uv = directory.join("uv");
        std::fs::write(
            &uv,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit {exit_code}\n",
                capture.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(&uv, std::fs::Permissions::from_mode(0o755)).unwrap();
        (uv, capture)
    }
}

#[tokio::test]
async fn test_uvx_install_warms_isolated_environment_without_console_entrypoints() {
    let directory = tempfile::tempdir().unwrap();
    let (uv, capture) = fake_uv(directory.path(), 0);
    let install_dir = directory.path().join("installed");
    let result = UvxInstaller::with_uv_path(uv)
        .install(
            &install_dir,
            "example-library",
            "2.3.4",
            &InstallOptions::new().force(true),
        )
        .await
        .unwrap();
    let output = std::fs::read_to_string(capture).unwrap();
    assert_eq!(
        output.lines().collect::<Vec<_>>(),
        [
            "tool",
            "run",
            "--refresh",
            "--from",
            "example-library==2.3.4",
            "python",
            "-c",
            "pass"
        ]
    );
    assert_eq!(result.executables, ["example-library"]);
    assert!(!result.bin_dir.join("python.cmd").exists());
    assert!(!result.bin_dir.join("python").exists());
}

#[tokio::test]
async fn test_uvx_install_failure_does_not_claim_success_or_write_shim() {
    let directory = tempfile::tempdir().unwrap();
    let (uv, _) = fake_uv(directory.path(), 17);
    let install_dir = directory.path().join("installed");
    let result = UvxInstaller::with_uv_path(uv)
        .install(
            &install_dir,
            "example-library",
            "2.3.4",
            &InstallOptions::new(),
        )
        .await;
    assert!(result.is_err());
    assert!(!install_dir.join("bin/example-library.cmd").exists());
    assert!(!install_dir.join("bin/example-library").exists());
}
