//! Regression tests for the Rust provider's executable resolution.
//!
//! The Rust provider installs in two stages: `install_layout` drops the
//! `rustup-init` bootstrapper into `bin/`, and `post_extract` runs it, which is
//! what actually creates `cargo/bin/rustup`. Dispatch must target the latter —
//! `rustup-init` rejects `component`/`target`/`toolchain` subcommands — on a cold
//! store exactly as it does on a warm one.
//!
//! See <https://github.com/loonghao/vx/issues/1152>.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use vx_runtime::{
    RealFileSystem, RealPathProvider, Runtime, RuntimeContext,
    testing::{MockHttpClient, MockInstaller},
};
use vx_starlark::build_runtimes;

const VERSION: &str = "1.93.1";

fn provider_star() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/ parent")
        .join("vx-providers/rust/provider.star");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
}

fn exe_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

/// A `RuntimeContext` backed by the real filesystem, rooted at `root`.
fn real_context(root: &Path) -> RuntimeContext {
    RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(root)),
        Arc::new(MockHttpClient::new()),
        Arc::new(RealFileSystem),
        Arc::new(MockInstaller::new()),
    )
}

/// Materialise a store directory that looks like `post_extract` already ran:
/// both the bootstrapper (`bin/rustup-init`) and the real manager
/// (`cargo/bin/rustup`) are present.
fn write_warm_store(ctx: &RuntimeContext) -> (PathBuf, PathBuf) {
    let install_dir = ctx.paths.version_store_dir("rust", VERSION);

    let bin_dir = install_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    let init_path = bin_dir.join(exe_name("rustup-init"));
    std::fs::write(&init_path, b"bootstrapper").expect("write rustup-init");

    let cargo_bin = install_dir.join("cargo").join("bin");
    std::fs::create_dir_all(&cargo_bin).expect("create cargo/bin dir");
    let rustup_path = cargo_bin.join(exe_name("rustup"));
    std::fs::write(&rustup_path, b"rustup").expect("write rustup");

    (init_path, rustup_path)
}

fn rust_runtime() -> Arc<dyn Runtime> {
    build_runtimes("rust".to_string(), provider_star(), None::<String>)
        .into_iter()
        .find(|rt| rt.name() == "rust")
        .expect("rust provider must define a 'rust' runtime")
}

#[tokio::test]
async fn execute_path_targets_rustup_not_rustup_init() {
    let root = tempfile::tempdir().expect("tempdir");
    let ctx = real_context(root.path());
    let (init_path, rustup_path) = write_warm_store(&ctx);

    let resolved = rust_runtime()
        .get_executable_path_for_version(VERSION, &ctx)
        .await
        .expect("resolution should succeed");

    assert_eq!(
        resolved.as_deref(),
        Some(rustup_path.as_path()),
        "dispatch must target cargo/bin/rustup, not the bin/rustup-init bootstrapper"
    );
    assert_ne!(resolved.as_deref(), Some(init_path.as_path()));
}

#[tokio::test]
async fn execute_path_matches_the_warm_store_scan() {
    let root = tempfile::tempdir().expect("tempdir");
    let ctx = real_context(root.path());
    let (_, rustup_path) = write_warm_store(&ctx);

    // This is the lookup the resolver performs for an already-installed runtime.
    let manager = vx_paths::PathManager::with_base_dir(root.path()).expect("path manager");
    let scanned = vx_paths::PathResolver::new(manager)
        .find_tool_with_executable("rust", "rustup")
        .expect("store scan should succeed")
        .expect("rustup should be found in the store");

    let resolved = rust_runtime()
        .get_executable_path_for_version(VERSION, &ctx)
        .await
        .expect("resolution should succeed");

    assert_eq!(
        resolved.as_deref(),
        Some(scanned.path.as_path()),
        "cold and warm dispatch must resolve to the same binary"
    );
    assert_eq!(scanned.path, rustup_path);
}

#[tokio::test]
async fn execute_path_falls_back_to_layout_when_toolchain_is_missing() {
    let root = tempfile::tempdir().expect("tempdir");
    let ctx = real_context(root.path());
    let install_dir = ctx.paths.version_store_dir("rust", VERSION);

    // Only the bootstrapper exists: `post_extract` has not run yet.
    let bin_dir = install_dir.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("create bin dir");
    let init_path = bin_dir.join(exe_name("rustup-init"));
    std::fs::write(&init_path, b"bootstrapper").expect("write rustup-init");

    let resolved = rust_runtime()
        .get_executable_path_for_version(VERSION, &ctx)
        .await
        .expect("resolution should succeed");

    assert_eq!(
        resolved.as_deref(),
        Some(init_path.as_path()),
        "a declared path that does not exist must not replace the layout-derived path"
    );
}
