//! Missing package targets must never fall back to a command shim that re-enters vx.

use std::path::Path;
use std::time::Duration;

use vx_paths::global_packages::{GlobalPackage, PackageRegistry};
use vx_paths::shims::get_shim_path;
use vx_shim::{PackageRequest, ShimError, ShimExecutor};

fn write_poison_shim(shims: &Path, marker: &Path) {
    std::fs::create_dir_all(shims).unwrap();
    let path = get_shim_path(shims, "codex");
    #[cfg(windows)]
    std::fs::write(
        path,
        format!(
            "@echo off\r\necho invoked>\"{}\"\r\nexit /b 99\r\n",
            marker.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        std::fs::write(
            &path,
            format!(
                "#!/bin/sh\nprintf invoked > '{}'\nexit 99\n",
                marker.display()
            ),
        )
        .unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

async fn check_missing_executable(by_name: bool, with_shim: bool) {
    let temp = tempfile::tempdir().unwrap();
    let registry_path = temp.path().join("packages.json");
    let shims = temp.path().join("shims");
    let marker = temp.path().join("shim-was-executed");
    if with_shim {
        write_poison_shim(&shims, &marker);
    }
    let mut registry = PackageRegistry::new();
    registry.register(
        GlobalPackage::new("@openai/codex", "1.2.3", "npm", temp.path().join("missing"))
            .with_executable("codex"),
    );
    registry.save(&registry_path).unwrap();
    let executor = ShimExecutor::new(registry_path, shims);
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        if by_name {
            executor.try_execute("codex", &[]).await.map(|_| ())
        } else {
            let request = PackageRequest::parse("npm:@openai/codex::codex").unwrap();
            executor.execute_request(&request, &[]).await.map(|_| ())
        }
    })
    .await
    .expect("missing executable lookup must terminate");

    let error = result.expect_err("missing package executable must fail");
    assert!(matches!(&error, ShimError::ExecutionFailed(_)));
    assert!(error.to_string().contains("Executable 'codex' is missing"));
    assert!(
        error
            .to_string()
            .contains("vx pkg install npm:@openai/codex --force")
    );
    assert!(!marker.exists(), "a global command shim must never execute");
}

#[tokio::test]
async fn test_explicit_package_request_does_not_reenter_command_shim() {
    check_missing_executable(false, true).await;
}

#[tokio::test]
async fn test_registered_executable_does_not_reenter_command_shim() {
    check_missing_executable(true, true).await;
}

#[tokio::test]
async fn test_missing_executable_without_shim_reports_reinstall_command() {
    check_missing_executable(false, false).await;
}
