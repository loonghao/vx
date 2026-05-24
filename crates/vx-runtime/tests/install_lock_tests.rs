//! Regression tests for concurrent runtime installation locking.

use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use anyhow::{Result, bail};
use async_trait::async_trait;
use vx_runtime::runtime::install_impl::{InstallParams, default_install_inner};
use vx_runtime::{HttpClient, Installer, RealFileSystem, RealPathProvider, RuntimeContext};

#[derive(Debug)]
struct NoopHttpClient;

#[async_trait]
impl HttpClient for NoopHttpClient {
    async fn get(&self, _url: &str) -> Result<String> {
        bail!("not used")
    }

    async fn get_json_value(&self, _url: &str) -> Result<serde_json::Value> {
        bail!("not used")
    }

    async fn download(&self, _url: &str, _dest: &Path) -> Result<()> {
        bail!("not used")
    }

    async fn download_with_progress(
        &self,
        _url: &str,
        _dest: &Path,
        _on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> Result<()> {
        bail!("not used")
    }
}

#[derive(Debug)]
struct SlowInstaller {
    calls: AtomicUsize,
}

impl SlowInstaller {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Installer for SlowInstaller {
    async fn extract(&self, _archive: &Path, _dest: &Path) -> Result<()> {
        bail!("not used")
    }

    async fn download_and_extract(&self, _url: &str, dest: &Path) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);

        let bin_dir = dest.join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        tokio::time::sleep(Duration::from_millis(200)).await;

        let exe_path = bin_dir.join(exe_file_name());
        std::fs::write(&exe_path, b"#!/bin/sh\nexit 0\n")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&exe_path)?.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&exe_path, permissions)?;
        }

        Ok(())
    }
}

#[tokio::test]
async fn concurrent_installs_wait_for_the_first_completed_install() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let installer = Arc::new(SlowInstaller::new());
    let ctx = RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(temp_dir.path())),
        Arc::new(NoopHttpClient),
        Arc::new(RealFileSystem::new()),
        installer.clone(),
    );

    let ctx_a = ctx.clone();
    let first = tokio::spawn(async move {
        default_install_inner(make_params(), "1.0.0", &ctx_a, |_, _| Ok(())).await
    });

    let ctx_b = ctx.clone();
    let second = tokio::spawn(async move {
        default_install_inner(make_params(), "1.0.0", &ctx_b, |_, _| Ok(())).await
    });

    let (first, second) = tokio::join!(first, second);
    let first = first
        .expect("first task should not panic")
        .expect("first install should succeed");
    let second = second
        .expect("second task should not panic")
        .expect("second install should succeed");

    assert_eq!(first.executable_path, second.executable_path);
    assert_eq!(
        installer.calls.load(Ordering::SeqCst),
        1,
        "only the lock holder should download and extract"
    );

    let lock_path = ctx
        .paths
        .version_store_dir("race-tool", "1.0.0")
        .with_file_name("1.0.0.install.lock");
    assert!(!lock_path.exists(), "install lock should be removed");
}

fn make_params() -> InstallParams<'static> {
    InstallParams {
        name: "race-tool",
        store_name: "race-tool",
        exe_relative: format!("bin/{}", exe_file_name()),
        exe_name: "race-tool",
        exe_extensions: exe_extensions(),
        layout_metadata: Default::default(),
        download_urls: vec!["https://example.invalid/race-tool.tar.gz".to_string()],
        normalize_config: None,
    }
}

fn exe_file_name() -> &'static str {
    if cfg!(windows) {
        "race-tool.exe"
    } else {
        "race-tool"
    }
}

fn exe_extensions() -> &'static [&'static str] {
    if cfg!(windows) { &[".exe"] } else { &[] }
}
