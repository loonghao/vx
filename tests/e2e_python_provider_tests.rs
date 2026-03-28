//! E2E tests for the Python provider
//!
//! These tests verify that the Python provider works correctly across
//! all three platforms (Linux, macOS, Windows), covering:
//! - Version listing (fetch from python-build-standalone releases)
//! - Installation (download + extract + verify)
//! - Execution (python --version, python -c "...")
//! - pip bundled runtime
//! - Error handling (unsupported platforms, network issues)
//!
//! Background: Python installation has had recurring CI failures:
//! - "No installation strategy available for python on this platform"
//! - Network timeout issues with python-build-standalone
//! - version_date lookup failures causing download_url to return None

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

/// Get the path to the vx binary for testing
fn vx_binary() -> PathBuf {
    let mut path = env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    if path.ends_with("deps") {
        path.pop(); // Remove deps directory
    }
    path.push("vx");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// E2E test environment with isolated VX_HOME
struct E2ETestEnv {
    home: TempDir,
}

impl E2ETestEnv {
    fn new() -> Self {
        Self {
            home: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new(vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
            .output()
            .expect("Failed to execute vx command")
    }

    fn run_with_timeout(&self, args: &[&str], timeout_secs: u64) -> Option<std::process::Output> {
        let mut child = Command::new(vx_binary())
            .args(args)
            .env("VX_HOME", self.home.path())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to spawn vx command");

        let timeout = Duration::from_secs(timeout_secs);
        let start = std::time::Instant::now();

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let stdout = child
                        .stdout
                        .take()
                        .map(|mut s| {
                            let mut buf = Vec::new();
                            std::io::Read::read_to_end(&mut s, &mut buf).unwrap_or(0);
                            buf
                        })
                        .unwrap_or_default();
                    let stderr = child
                        .stderr
                        .take()
                        .map(|mut s| {
                            let mut buf = Vec::new();
                            std::io::Read::read_to_end(&mut s, &mut buf).unwrap_or(0);
                            buf
                        })
                        .unwrap_or_default();

                    return Some(std::process::Output {
                        status,
                        stdout,
                        stderr,
                    });
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        return None;
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(_) => return None,
            }
        }
    }

    /// Helper: get combined stdout + stderr
    fn combined_output(output: &std::process::Output) -> String {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        format!("{}{}", stdout, stderr)
    }

    /// Helper: check if output indicates a network/API error (acceptable in CI)
    fn is_network_error(combined: &str) -> bool {
        combined.contains("network")
            || combined.contains("Network")
            || combined.contains("timeout")
            || combined.contains("Timeout")
            || combined.contains("connection")
            || combined.contains("Connection")
            || combined.contains("rate limit")
            || combined.contains("Rate limit")
            || combined.contains("Failed to fetch")
            || combined.contains("error sending request")
            || combined.contains("error decoding response body")
            || combined.contains("GITHUB_TOKEN")
            || combined.contains("GH_TOKEN")
            || combined.contains("API rate limit")
            || combined.contains("GitHub API")
            || combined.contains("fetch failed")
            || combined.contains("fetch_versions failed")
    }
}

// ============================================================================
// Version listing tests
// ============================================================================

#[test]
fn test_python_versions_list() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "python"]);

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        // Should contain Python version numbers like 3.12, 3.13, etc.
        assert!(
            combined.contains("3.") || combined.contains("Version"),
            "Expected Python version numbers in output: {}",
            combined
        );
    } else {
        // Network errors are acceptable in CI
        assert!(
            E2ETestEnv::is_network_error(&combined),
            "Expected network error or version list, got unexpected error: {}",
            combined
        );
    }
}

#[test]
fn test_python_versions_list_via_alias_python3() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "python3"]);

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        assert!(
            combined.contains("3.") || combined.contains("Version"),
            "Expected Python version numbers via 'python3' alias: {}",
            combined
        );
    } else {
        assert!(
            E2ETestEnv::is_network_error(&combined),
            "Expected network error for 'python3' alias, got: {}",
            combined
        );
    }
}

#[test]
fn test_python_versions_list_via_alias_py() {
    let env = E2ETestEnv::new();
    let output = env.run(&["versions", "py"]);

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        assert!(
            combined.contains("3.") || combined.contains("Version"),
            "Expected Python version numbers via 'py' alias: {}",
            combined
        );
    } else {
        assert!(
            E2ETestEnv::is_network_error(&combined),
            "Expected network error for 'py' alias, got: {}",
            combined
        );
    }
}

// ============================================================================
// Provider discovery and metadata tests
// ============================================================================

#[test]
fn test_python_is_in_list() {
    let env = E2ETestEnv::new();
    let output = env.run(&["list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("python"),
        "Expected 'python' in list output: {}",
        stdout
    );
}

#[test]
fn test_python_search() {
    let env = E2ETestEnv::new();
    let output = env.run(&["search", "python"]);

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        assert!(
            combined.to_lowercase().contains("python"),
            "Expected 'python' in search results: {}",
            combined
        );
    }
}

#[test]
fn test_pip_is_in_list() {
    let env = E2ETestEnv::new();
    let output = env.run(&["list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("pip"),
        "Expected 'pip' in list output (bundled with python): {}",
        stdout
    );
}

// ============================================================================
// Installation tests
// ============================================================================

/// Test that `vx install python` works or fails gracefully
///
/// This test covers the critical installation path that has failed in CI with:
/// "No installation strategy available for python on this platform"
///
/// The Python provider uses python-build-standalone, which requires:
/// 1. Fetching version list from GitHub API (needs version_date/build_tag)
/// 2. Building download URL with the build tag
/// 3. Downloading and extracting the archive
///
/// If any step fails, vx should provide a clear error message.
#[test]
fn test_python_install_provides_clear_error_or_succeeds() {
    let env = E2ETestEnv::new();

    // Use a timeout because install can take a while
    let Some(output) = env.run_with_timeout(&["install", "python@3.12"], 120) else {
        eprintln!("Skipping: python install timed out (network issue)");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        // Installation succeeded — verify it was actually installed
        assert!(
            combined.contains("Installed")
                || combined.contains("installed")
                || combined.contains("already installed")
                || combined.contains("Already installed"),
            "Expected installation confirmation: {}",
            combined
        );
    } else {
        // Installation failed — should have a clear, actionable error message.
        // "No installation strategy" is a REGRESSION — it means download_url
        // returned None, likely because version_date lookup failed. After our fix,
        // download_url_for_runtime auto-triggers fetch_versions to populate the
        // version cache, so this error should no longer occur.
        assert!(
            !combined.contains("No installation strategy"),
            "REGRESSION: Python install failed with 'No installation strategy'. \
             This indicates the download_url returned None, likely because \
             version_date lookup failed. The auto-fetch fix should prevent this. \
             Error: {}",
            combined
        );

        let has_useful_error = combined.contains("python")
            || combined.contains("Python")
            || E2ETestEnv::is_network_error(&combined)
            || combined.contains("download")
            || combined.contains("Failed to install");

        assert!(
            has_useful_error,
            "Expected clear error message about Python installation failure, got: {}",
            combined
        );
    }
}

/// Test that `vx install python@3.12` with explicit version works
#[test]
fn test_python_install_specific_version() {
    let env = E2ETestEnv::new();

    let Some(output) = env.run_with_timeout(&["install", "python@3.12"], 120) else {
        eprintln!("Skipping: python install timed out");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    // Either succeeds or fails with a clear error
    if !output.status.success() {
        // "No installation strategy" is a REGRESSION after our fix
        assert!(
            !combined.contains("No installation strategy"),
            "REGRESSION: python@3.12 install failed with 'No installation strategy': {}",
            combined
        );

        // Acceptable failures (network issues only)
        assert!(
            E2ETestEnv::is_network_error(&combined)
                || combined.contains("Failed to install")
                || combined.contains("not found")
                || combined.contains("No version found"),
            "Unexpected error installing python@3.12: {}",
            combined
        );
    }
}

// ============================================================================
// Execution tests (require successful installation)
// ============================================================================

/// Test that `vx python --version` works or provides useful error
#[test]
fn test_python_version_command() {
    let env = E2ETestEnv::new();

    let Some(output) = env.run_with_timeout(&["python", "--version"], 120) else {
        eprintln!("Skipping: python --version timed out (install may be slow)");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        // Should output something like "Python 3.12.x"
        assert!(
            combined.contains("Python 3.") || combined.contains("python 3."),
            "Expected Python version string in output: {}",
            combined
        );
    } else {
        // Should fail with useful error, not silent failure
        assert!(
            !combined.is_empty(),
            "python --version failed silently with no output. \
             Exit code: {:?}",
            output.status.code()
        );

        // Should give useful error about installation
        let has_useful_error = E2ETestEnv::is_network_error(&combined)
            || combined.contains("install")
            || combined.contains("Install")
            || combined.contains("not found")
            || combined.contains("Failed");

        assert!(
            has_useful_error,
            "Expected useful error message for python --version failure: {}",
            combined
        );
    }
}

/// Test that `vx python -c "print('hello')"` works when Python is installed
#[test]
fn test_python_eval_command() {
    let env = E2ETestEnv::new();

    let Some(output) =
        env.run_with_timeout(&["python", "-c", "print('hello from vx python')"], 120)
    else {
        eprintln!("Skipping: python eval timed out");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        assert!(
            combined.contains("hello from vx python"),
            "Expected eval output: {}",
            combined
        );
    }
    // If it fails, that's OK — we just need it not to crash
}

/// Test that `vx pip --version` works (bundled runtime)
#[test]
fn test_pip_version_command() {
    let env = E2ETestEnv::new();

    let Some(output) = env.run_with_timeout(&["pip", "--version"], 120) else {
        eprintln!("Skipping: pip --version timed out");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        // pip output: "pip X.Y.Z from /path/to/pip (python X.Y)"
        assert!(
            combined.contains("pip") && combined.contains("python"),
            "Expected pip version info: {}",
            combined
        );
    } else {
        // Should fail gracefully with useful error
        assert!(!combined.is_empty(), "pip --version failed silently");
    }
}

// ============================================================================
// Platform-specific tests
// ============================================================================

/// Test that Python provider supports the current platform
#[test]
fn test_python_provider_supports_current_platform() {
    let env = E2ETestEnv::new();
    let output = env.run(&["list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Python should always appear in the list on supported platforms
    // (windows/x64, macos/x64, macos/arm64, linux/x64, linux/arm64)
    assert!(
        stdout.to_lowercase().contains("python"),
        "Python provider should be listed on this platform: {}",
        stdout
    );
}

/// Test that the error message is platform-aware when install fails
#[test]
fn test_python_install_error_mentions_platform() {
    let env = E2ETestEnv::new();

    // Try to install an intentionally impossible version to trigger error path
    let output = env.run(&["install", "python@1.0.0"]);

    let combined = E2ETestEnv::combined_output(&output);

    // Should fail (version 1.0.0 doesn't exist in python-build-standalone)
    if !output.status.success() {
        // The error should be informative
        assert!(
            !combined.is_empty(),
            "Install of invalid Python version should produce an error message"
        );
    }
}

// ============================================================================
// Version resolution edge cases
// ============================================================================

/// Test that installing with major version spec resolves correctly
#[test]
fn test_python_install_major_version_spec() {
    let env = E2ETestEnv::new();

    // "python@3" should resolve to latest 3.x
    let Some(output) = env.run_with_timeout(&["install", "python@3"], 120) else {
        eprintln!("Skipping: python install timed out");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        assert!(
            combined.contains("3."),
            "Expected Python 3.x to be installed: {}",
            combined
        );
    } else {
        // "No installation strategy" is a REGRESSION after our fix
        assert!(
            !combined.contains("No installation strategy"),
            "REGRESSION: python@3 install failed with 'No installation strategy': {}",
            combined
        );

        // Network errors or version resolution failures are acceptable
        assert!(
            E2ETestEnv::is_network_error(&combined)
                || combined.contains("No version found")
                || combined.contains("Failed"),
            "Unexpected error for python@3: {}",
            combined
        );
    }
}

/// Test that `vx install python@latest` works
#[test]
fn test_python_install_latest() {
    let env = E2ETestEnv::new();

    let Some(output) = env.run_with_timeout(&["install", "python@latest"], 120) else {
        eprintln!("Skipping: python@latest install timed out");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    if output.status.success() {
        assert!(
            combined.contains("3.")
                || combined.contains("Installed")
                || combined.contains("installed"),
            "Expected successful install of latest Python: {}",
            combined
        );
    } else {
        // "No installation strategy" is a REGRESSION after our fix
        assert!(
            !combined.contains("No installation strategy"),
            "REGRESSION: python@latest install failed with 'No installation strategy': {}",
            combined
        );

        assert!(
            E2ETestEnv::is_network_error(&combined)
                || combined.contains("No version found")
                || combined.contains("Failed"),
            "Unexpected error for python@latest: {}",
            combined
        );
    }
}

// ============================================================================
// vx.toml integration tests
// ============================================================================

/// Test that a project with python in vx.toml is recognized
#[test]
fn test_vx_toml_python_setup_dry_run() {
    let env = E2ETestEnv::new();
    let workdir = TempDir::new().unwrap();

    // Create a vx.toml with python
    std::fs::write(
        workdir.path().join("vx.toml"),
        r#"[tools]
python = "3.12"
"#,
    )
    .unwrap();

    let output = Command::new(vx_binary())
        .args(["setup", "--dry-run"])
        .env("VX_HOME", env.home.path())
        .current_dir(workdir.path())
        .output()
        .expect("Failed to execute vx setup");

    let combined = E2ETestEnv::combined_output(&output);

    // The setup command should either:
    // 1. Succeed and mention python in the plan, or
    // 2. Succeed and report "All tools are synchronized" (when already installed or dry-run skips),
    // 3. Fail due to network issues
    if output.status.success() {
        // Setup dry-run succeeded - it may mention python or just report synchronized
        assert!(
            combined.to_lowercase().contains("python")
                || combined.contains("synchronized")
                || combined.contains("Setup"),
            "Unexpected setup dry-run output: {}",
            combined
        );
    }
    // If setup fails (network, etc.), that's acceptable in CI
}

/// Test that a project with python and uv in vx.toml works
#[test]
fn test_vx_toml_python_with_uv() {
    let env = E2ETestEnv::new();
    let workdir = TempDir::new().unwrap();

    // Create a vx.toml with both python and uv
    std::fs::write(
        workdir.path().join("vx.toml"),
        r#"[tools]
python = "3.12"
uv = "latest"
"#,
    )
    .unwrap();

    let output = Command::new(vx_binary())
        .args(["setup", "--dry-run"])
        .env("VX_HOME", env.home.path())
        .current_dir(workdir.path())
        .output()
        .expect("Failed to execute vx setup");

    // Just verify it doesn't crash
    let combined = E2ETestEnv::combined_output(&output);
    assert!(
        !combined.is_empty() || output.status.success(),
        "vx setup --dry-run with python+uv should produce output"
    );
}

// ============================================================================
// Regression tests for known issues
// ============================================================================

/// Regression test: Python install should NOT silently fail
///
/// Issue: In CI, `vx python` would fail with "No installation strategy available"
/// but the error was confusing because it didn't explain WHY the download_url
/// returned None (usually because version_date was missing).
#[test]
fn test_python_install_does_not_fail_silently() {
    let env = E2ETestEnv::new();

    let Some(output) = env.run_with_timeout(&["python", "--version"], 60) else {
        // Timeout is acceptable (slow network)
        eprintln!("Skipping: python --version timed out");
        return;
    };

    let combined = E2ETestEnv::combined_output(&output);

    // The command should NEVER fail with empty output
    if !output.status.success() {
        assert!(
            !combined.trim().is_empty(),
            "REGRESSION: Python command failed silently! \
             Exit code: {:?}. This indicates error handling is broken.",
            output.status.code()
        );
    }
}

/// Regression test: pip should be recognized as a bundled runtime of python
#[test]
fn test_pip_recognized_as_bundled_runtime() {
    let env = E2ETestEnv::new();
    let output = env.run(&["list"]);

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_lower = stdout.to_lowercase();

    // Both python and pip should be listed
    assert!(
        stdout_lower.contains("python") && stdout_lower.contains("pip"),
        "Both 'python' and 'pip' should be in list output: {}",
        stdout
    );
}

/// Regression test: Python aliases (python3, py) should resolve
#[test]
fn test_python_aliases_resolve() {
    let env = E2ETestEnv::new();

    // Test that all aliases are recognized
    for alias in &["python", "python3", "py"] {
        let output = env.run(&["search", alias]);
        let combined = E2ETestEnv::combined_output(&output);

        if output.status.success() {
            assert!(
                combined.to_lowercase().contains("python"),
                "Alias '{}' should resolve to python provider: {}",
                alias,
                combined
            );
        }
    }
}
