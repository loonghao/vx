//! Installation hook diagnostics stay out of the caller's structured stdout.

use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use anyhow::{Result, bail};
use async_trait::async_trait;
use rstest::rstest;
use serde_json::{Value, json};

use vx_runtime::{
    HttpClient, Installer, ManifestDrivenRuntime, ProviderSource, RealFileSystem, RealPathProvider,
    Runtime, RuntimeContext,
};

const PARENT_GUARD: &str = "VX_POST_INSTALL_OUTPUT_PARENT";
const CHILD_GUARD: &str = "VX_POST_INSTALL_OUTPUT_CHILD";
const ROOT_ENV: &str = "VX_POST_INSTALL_OUTPUT_ROOT";
const EXIT_ENV: &str = "VX_POST_INSTALL_OUTPUT_EXIT";
const FAILURE_ENV: &str = "VX_POST_INSTALL_OUTPUT_FAILURE";
const CHILD_TEST: &str = "test_post_install_child_process";
const PARENT_TEST: &str = "test_post_install_parent_process";
const STDOUT_MARKER: &str = "hook stdout diagnostic";
const STDERR_MARKER: &str = "hook stderr diagnostic";
const RESULT_PREFIX: &str = "POST_INSTALL_RESULT:";

#[derive(Debug)]
struct UnusedServices;

#[async_trait]
impl HttpClient for UnusedServices {
    async fn get(&self, _url: &str) -> Result<String> {
        bail!("post_install must not use HTTP")
    }

    async fn get_json_value(&self, _url: &str) -> Result<Value> {
        bail!("post_install must not use HTTP")
    }

    async fn download(&self, _url: &str, _dest: &Path) -> Result<()> {
        bail!("post_install must not download")
    }

    async fn download_with_progress(
        &self,
        _url: &str,
        _dest: &Path,
        _on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> Result<()> {
        bail!("post_install must not download")
    }
}

#[async_trait]
impl Installer for UnusedServices {
    async fn extract(&self, _archive: &Path, _dest: &Path) -> Result<()> {
        bail!("post_install must not extract")
    }

    async fn download_and_extract(&self, _url: &str, _dest: &Path) -> Result<()> {
        bail!("post_install must not download")
    }
}

// Native test subprocesses avoid shells, ambient runtimes and global env changes.
#[test]
#[ignore = "native fixture launched by the output contract test"]
fn test_post_install_child_process() {
    if std::env::var_os(CHILD_GUARD).is_none() {
        return;
    }
    writeln!(std::io::stdout(), "{STDOUT_MARKER}").unwrap();
    writeln!(std::io::stderr(), "{STDERR_MARKER}").unwrap();
    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
    let exit = std::env::var(EXIT_ENV).unwrap().parse().unwrap();
    std::process::exit(exit);
}

#[tokio::test]
#[ignore = "native fixture launched by the output contract test"]
async fn test_post_install_parent_process() {
    if std::env::var_os(PARENT_GUARD).is_none() {
        return;
    }
    let root = std::env::var_os(ROOT_ENV).unwrap();
    let ctx = RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(Path::new(&root))),
        Arc::new(UnusedServices),
        Arc::new(RealFileSystem::new()),
        Arc::new(UnusedServices),
    );
    let install_dir = ctx.paths.version_store_dir("hook-output", "1.0");
    std::fs::create_dir_all(&install_dir).unwrap();
    let action = json!({
        "type": "run_command",
        "command": std::env::current_exe().unwrap(),
        "args": ["--exact", CHILD_TEST, "--ignored", "--nocapture"],
        "env": {
            (CHILD_GUARD): "1",
            (EXIT_ENV): std::env::var(EXIT_ENV).unwrap(),
        },
        "on_failure": std::env::var(FAILURE_ENV).unwrap(),
    });
    let runtime = ManifestDrivenRuntime::new("hook-output", "hook-output", ProviderSource::BuiltIn)
        .with_post_extract(Arc::new(move |_, _| {
            let action = action.clone();
            Box::pin(async move { Ok(vec![action]) })
        }));

    let result = runtime.post_install("1.0", &ctx).await;
    let result = json!({
        "succeeded": result.is_ok(),
        "error": result.err().map(|error| error.to_string()),
    });
    writeln!(std::io::stdout(), "{RESULT_PREFIX}{result}").unwrap();
}

#[rstest]
#[case("0", "error", true)]
#[case("7", "error", false)]
#[case("7", "ignore", true)]
fn test_post_install_streams_diagnostics_to_stderr_and_preserves_failure_policy(
    #[case] exit_code: &str,
    #[case] on_failure: &str,
    #[case] expected_success: bool,
) {
    let root = tempfile::tempdir().unwrap();
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", PARENT_TEST, "--ignored", "--nocapture"])
        .env(PARENT_GUARD, "1")
        .env(ROOT_ENV, root.path())
        .env(EXIT_ENV, exit_code)
        .env(FAILURE_ENV, on_failure)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stdout.contains(STDOUT_MARKER), "{stdout}");
    assert!(!stdout.contains(STDERR_MARKER), "{stdout}");
    assert!(stderr.contains(STDOUT_MARKER), "{stderr}");
    assert!(stderr.contains(STDERR_MARKER), "{stderr}");
    let result: Value = serde_json::from_str(
        stdout
            .lines()
            .find_map(|line| line.split_once(RESULT_PREFIX).map(|(_, result)| result))
            .expect("parent structured result"),
    )
    .unwrap();
    assert_eq!(result["succeeded"], expected_success, "{result}");
    if !expected_success {
        assert!(result["error"].as_str().unwrap().contains("exited with"));
    }
}
