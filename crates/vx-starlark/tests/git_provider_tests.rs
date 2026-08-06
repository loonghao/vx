//! Tests for the `git` provider.

use vx_starlark::{ProviderContext, StarlarkEngine};

#[rstest::rstest]
#[case("x64", "Git-2.54.0-64-bit.tar.bz2")]
#[case("arm64", "Git-2.54.0-arm64.tar.bz2")]
fn windows_download_includes_bash(#[case] arch: &str, #[case] asset: &str) {
    let star_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("vx-providers/git/provider.star");
    let content = std::fs::read_to_string(&star_path).unwrap();
    let mut ctx = ProviderContext::new("git", std::env::temp_dir().join("vx-test"));
    ctx.platform.os = "windows".to_string();
    ctx.platform.arch = arch.to_string();

    let url = StarlarkEngine::new()
        .call_function(
            &star_path,
            &content,
            "download_url",
            &ctx,
            &[serde_json::json!("2.54.0.windows.1")],
        )
        .unwrap();

    assert!(url.as_str().unwrap().ends_with(asset));
}

#[cfg(windows)]
#[test]
#[ignore = "requires network access and a built vx binary"]
fn windows_shebang_hook_resolves_external_path_tool() {
    use std::process::{Command, Output};

    fn run_vx(
        vx: &std::path::Path,
        cwd: &std::path::Path,
        vx_home: &std::path::Path,
        path: &std::ffi::OsStr,
        args: &[&str],
    ) -> Output {
        Command::new(vx)
            .args(args)
            .current_dir(cwd)
            .env("VX_HOME", vx_home)
            .env("PATH", path)
            .output()
            .unwrap()
    }

    fn assert_success(output: &Output) {
        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let vx = std::env::var_os("VX_BINARY")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| workspace.join("target").join("debug").join("vx.exe"));
    assert!(vx.is_file(), "build vx or set VX_BINARY={}", vx.display());

    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let tool_dir = temp.path().join("external-bin");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&tool_dir).unwrap();
    std::fs::copy(
        std::env::current_exe().unwrap(),
        tool_dir.join("hook-path-tool.exe"),
    )
    .unwrap();

    let mut path_entries = vec![tool_dir];
    path_entries.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let path = std::env::join_paths(path_entries).unwrap();
    let vx_home = temp.path().join("vx-home");

    assert_success(&run_vx(
        &vx,
        &repo,
        &vx_home,
        &path,
        &["git@latest", "init", "--quiet"],
    ));

    let hook = repo.join(".git").join("hooks").join("pre-push");
    std::fs::write(
        hook,
        "#!/usr/bin/env bash\nset -eu\ncommand -v hook-path-tool >/dev/null\n",
    )
    .unwrap();

    assert_success(&run_vx(
        &vx,
        &repo,
        &vx_home,
        &path,
        &[
            "git@latest",
            "-c",
            "core.hooksPath=.git/hooks",
            "hook",
            "run",
            "pre-push",
        ],
    ));
}
