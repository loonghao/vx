//! Bun global installation keeps package files and binaries inside the selected environment.

use std::path::{Path, PathBuf};

use rstest::rstest;
use vx_ecosystem_pm::{EcosystemInstaller, InstallOptions, installers::BunInstaller};

fn fake_bun(directory: &Path) -> (PathBuf, PathBuf) {
    let capture = directory.join("invocation.txt");
    #[cfg(windows)]
    {
        let script = directory.join("capture.ps1");
        std::fs::write(
            &script,
            format!(
                "[IO.File]::WriteAllLines('{}', @($env:BUN_INSTALL_GLOBAL_DIR, $env:BUN_INSTALL_BIN) + $args)\nexit 0\n",
                capture.display().to_string().replace('\'', "''")
            ),
        )
        .unwrap();
        let powershell = PathBuf::from(std::env::var("SystemRoot").unwrap())
            .join("System32/WindowsPowerShell/v1.0/powershell.exe");
        let bun = directory.join("bun.cmd");
        std::fs::write(
            &bun,
            format!(
                "@echo off\r\n\"{}\" -NoProfile -ExecutionPolicy Bypass -File \"{}\" %*\r\nexit /b %errorlevel%\r\n",
                powershell.display(), script.display()
            ),
        )
        .unwrap();
        (bun, capture)
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let bun = directory.join("bun");
        std::fs::write(
            &bun,
            format!(
                "#!/bin/sh\nprintf '%s\\n' \"$BUN_INSTALL_GLOBAL_DIR\" \"$BUN_INSTALL_BIN\" \"$@\" > '{}'\n",
                capture.display().to_string().replace('\'', "'\"'\"'")
            ),
        )
        .unwrap();
        std::fs::set_permissions(&bun, std::fs::Permissions::from_mode(0o755)).unwrap();
        (bun, capture)
    }
}

#[rstest]
#[case("latest", false, "@openai/codex")]
#[case("1.2.3", true, "@openai/codex@1.2.3")]
#[tokio::test]
async fn test_bun_install_uses_isolated_environment_without_global_dir_argument(
    #[case] version: &str,
    #[case] force: bool,
    #[case] expected_package: &str,
) {
    let directory = tempfile::tempdir().unwrap();
    let (bun, capture) = fake_bun(directory.path());
    let install_dir = directory.path().join("package environment with spaces");
    let result = BunInstaller::with_bun_path(bun)
        .install(
            &install_dir,
            "@openai/codex",
            version,
            &InstallOptions::new()
                .force(force)
                .extra_args(vec!["--ignore-scripts".to_string()]),
        )
        .await
        .unwrap();

    let output = std::fs::read_to_string(capture).unwrap();
    let lines: Vec<_> = output.lines().collect();
    assert_eq!(lines[0], install_dir.to_string_lossy());
    assert_eq!(lines[1], install_dir.join("bin").to_string_lossy());
    let mut expected_args = vec!["install", "--global"];
    if force {
        expected_args.push("--force");
    }
    expected_args.extend(["--ignore-scripts", expected_package]);
    assert_eq!(&lines[2..], expected_args);
    assert_eq!(result.install_dir, install_dir);
    assert_eq!(result.bin_dir, install_dir.join("bin"));
}
