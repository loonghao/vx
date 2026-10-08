//! Manifest installations and completeness readers share the physical store lock.

use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::task::Poll;
use std::time::Duration;

use anyhow::{Result, bail};
use async_trait::async_trait;
use futures_util::poll;
use rstest::rstest;
use tokio::sync::{Notify, Semaphore};
use vx_runtime::{
    HttpClient, Installer, ManifestDrivenRuntime, ProviderSource, RealFileSystem, RealPathProvider,
    Runtime, RuntimeContext,
};

const VERSION: &str = "1.0.0";
const WAIT: Duration = Duration::from_secs(10);

#[derive(Debug)]
struct NoopHttpClient;

#[async_trait]
impl HttpClient for NoopHttpClient {
    async fn get(&self, _url: &str) -> Result<String> {
        bail!("HTTP is not used by the controlled installer")
    }

    async fn get_json_value(&self, _url: &str) -> Result<serde_json::Value> {
        bail!("HTTP is not used by the controlled installer")
    }

    async fn download(&self, _url: &str, _dest: &Path) -> Result<()> {
        bail!("HTTP is not used by the controlled installer")
    }

    async fn download_with_progress(
        &self,
        _url: &str,
        _dest: &Path,
        _on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> Result<()> {
        bail!("HTTP is not used by the controlled installer")
    }
}

#[derive(Debug)]
struct ControlledInstaller {
    calls: AtomicUsize,
    entered: Notify,
    release: Semaphore,
    early_executable: bool,
}

impl ControlledInstaller {
    fn new(early_executable: bool) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            entered: Notify::new(),
            release: Semaphore::new(0),
            early_executable,
        }
    }
}

#[async_trait]
impl Installer for ControlledInstaller {
    async fn extract(&self, _archive: &Path, _dest: &Path) -> Result<()> {
        bail!("only download_and_extract is used")
    }

    async fn download_and_extract(&self, _url: &str, dest: &Path) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        std::fs::create_dir_all(dest.join("bin"))?;
        if self.early_executable {
            std::fs::write(dest.join("bin").join(executable()), b"sdk executable")?;
        }
        self.entered.notify_one();
        self.release.acquire().await?.forget();

        // Model an SDK whose launcher precedes its compiler and other resources.
        std::fs::create_dir_all(dest.join("bin"))?;
        std::fs::write(dest.join("bin").join(executable()), b"sdk executable")?;
        std::fs::create_dir_all(dest.join("pkg/tool"))?;
        std::fs::write(dest.join("pkg/tool/compiler"), b"complete compiler")?;
        Ok(())
    }
}

fn context(base: &Path, installer: Arc<ControlledInstaller>) -> RuntimeContext {
    RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(base)),
        Arc::new(NoopHttpClient),
        Arc::new(RealFileSystem::new()),
        installer,
    )
}

fn executable() -> &'static str {
    if cfg!(windows) { "sdk.exe" } else { "sdk" }
}

fn layout(inline_url: bool) -> serde_json::Value {
    let mut layout = serde_json::json!({
        "type": "archive",
        "executable_paths": [format!("bin/{}", executable())],
    });
    if inline_url {
        layout["url"] = "https://example.invalid/sdk.zip".into();
    }
    layout
}

fn runtime(inline_url: bool) -> ManifestDrivenRuntime {
    ManifestDrivenRuntime::new("sdk", "sdk", ProviderSource::BuiltIn)
        .with_download_url(|_, _| {
            Box::pin(async { Ok(Some("https://example.invalid/sdk.zip".to_string())) })
        })
        .with_install_layout(move |_| Box::pin(async move { Ok(Some(layout(inline_url))) }))
}

#[rstest]
#[case::layout_url_before_executable(true, false)]
#[case::layout_url_after_executable(true, true)]
#[case::download_url_before_executable(false, false)]
#[case::download_url_after_executable(false, true)]
#[tokio::test]
async fn test_manifest_concurrent_writers_wait_for_the_completed_sdk(
    #[case] inline_url: bool,
    #[case] early_executable: bool,
) {
    let temp = tempfile::tempdir().unwrap();
    let installer = Arc::new(ControlledInstaller::new(early_executable));
    let ctx = context(temp.path(), installer.clone());
    let runtime = Arc::new(runtime(inline_url));
    let first = {
        let runtime = runtime.clone();
        let ctx = ctx.clone();
        tokio::spawn(async move { runtime.install(VERSION, &ctx).await })
    };
    tokio::time::timeout(WAIT, installer.entered.notified())
        .await
        .expect("first writer must reach the controlled extraction checkpoint");

    let mut second = Box::pin(runtime.install(VERSION, &ctx));
    let second_poll = poll!(second.as_mut());
    let second_waited = second_poll.is_pending();
    let calls_while_first_is_paused = installer.calls.load(Ordering::SeqCst);

    installer.release.add_permits(2);
    let first = tokio::time::timeout(WAIT, first)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let second = match second_poll {
        Poll::Ready(result) => result.unwrap(),
        Poll::Pending => tokio::time::timeout(WAIT, second).await.unwrap().unwrap(),
    };

    assert!(second_waited, "a cached launcher is not a completed SDK");
    assert_eq!(calls_while_first_is_paused, 1, "writers must not overlap");
    assert_eq!(installer.calls.load(Ordering::SeqCst), 1);
    assert_eq!(first.executable_path, second.executable_path);
    assert!(first.install_path.join("pkg/tool/compiler").is_file());
    assert!(
        !first
            .install_path
            .with_file_name("1.0.0.install.lock")
            .exists()
    );
}

#[rstest]
#[case::exact(VERSION, false, true)]
#[case::latest("latest", false, true)]
#[case::bundled_exact(VERSION, true, true)]
#[case::bundled_latest("latest", true, true)]
#[case::without_layout_exact(VERSION, false, false)]
#[case::without_layout_latest("latest", false, false)]
#[tokio::test]
async fn test_manifest_cached_reader_rejects_an_sdk_with_an_active_writer(
    #[case] requested_version: &str,
    #[case] bundled: bool,
    #[case] has_layout: bool,
) {
    let temp = tempfile::tempdir().unwrap();
    let installer = Arc::new(ControlledInstaller::new(true));
    let ctx = context(temp.path(), installer.clone());
    let writer = Arc::new(runtime(false));
    let first = {
        let writer = writer.clone();
        let ctx = ctx.clone();
        tokio::spawn(async move { writer.install(VERSION, &ctx).await })
    };
    tokio::time::timeout(WAIT, installer.entered.notified())
        .await
        .unwrap();

    let mut reader = if has_layout {
        runtime(false)
    } else {
        ManifestDrivenRuntime::new("sdk", "sdk", ProviderSource::BuiltIn)
            .with_download_url(|_, _| Box::pin(async { Ok(None) }))
    };
    if bundled {
        reader.name = "sdk-helper".to_string();
        reader = reader.with_executable("sdk").with_bundled_with("sdk");
    }
    let during_install =
        tokio::time::timeout(WAIT, reader.is_installed(requested_version, &ctx)).await;
    installer.release.add_permits(1);
    tokio::time::timeout(WAIT, first)
        .await
        .unwrap()
        .unwrap()
        .unwrap();

    assert!(
        !during_install
            .expect("a completeness reader must not wait for extraction")
            .unwrap(),
        "an active physical-store writer makes the SDK incomplete"
    );
    assert!(reader.is_installed(requested_version, &ctx).await.unwrap());
}

#[tokio::test]
async fn test_manifest_cached_reader_holds_the_lock_through_layout_validation() {
    let temp = tempfile::tempdir().unwrap();
    let installer = Arc::new(ControlledInstaller::new(false));
    let ctx = context(temp.path(), installer.clone());
    let install_dir = ctx.paths.version_store_dir("sdk", VERSION);
    std::fs::create_dir_all(install_dir.join("bin")).unwrap();
    std::fs::write(install_dir.join("bin").join(executable()), b"cached sdk").unwrap();

    let validating = Arc::new(Notify::new());
    let release = Arc::new(Semaphore::new(0));
    let calls = Arc::new(AtomicUsize::new(0));
    let reader = Arc::new(runtime(false).with_install_layout({
        let validating = validating.clone();
        let release = release.clone();
        let calls = calls.clone();
        move |_| {
            let validating = validating.clone();
            let release = release.clone();
            let first_call = calls.fetch_add(1, Ordering::SeqCst) == 0;
            Box::pin(async move {
                if first_call {
                    validating.notify_one();
                    release.acquire().await?.forget();
                }
                Ok(Some(layout(false)))
            })
        }
    }));
    let first = {
        let reader = reader.clone();
        let ctx = ctx.clone();
        tokio::spawn(async move { reader.is_installed(VERSION, &ctx).await })
    };
    tokio::time::timeout(WAIT, validating.notified())
        .await
        .unwrap();

    let writer = runtime(false);
    let mut second = Box::pin(writer.install(VERSION, &ctx));
    let second_poll = poll!(second.as_mut());
    let writer_waited = second_poll.is_pending();
    release.add_permits(1);
    assert!(
        tokio::time::timeout(WAIT, first)
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    );
    match second_poll {
        Poll::Ready(result) => result.unwrap(),
        Poll::Pending => tokio::time::timeout(WAIT, second).await.unwrap().unwrap(),
    };
    assert!(
        writer_waited,
        "cached validation must retain its lock guard"
    );
    assert_eq!(installer.calls.load(Ordering::SeqCst), 0);
}

#[rstest]
#[case::exact(false)]
#[case::latest(true)]
#[tokio::test]
async fn test_manifest_cached_reader_fails_closed_when_the_lock_cannot_be_created(
    #[case] latest: bool,
) {
    let temp = tempfile::tempdir().unwrap();
    let installer = Arc::new(ControlledInstaller::new(false));
    let ctx = context(temp.path(), installer);
    // The version directory fits the filesystem's 255-unit component limit,
    // while appending ".install.lock" exceeds it on Unix and Windows.
    let version = "v".repeat(245);
    let install_dir = ctx.paths.version_store_dir("sdk", &version);
    std::fs::create_dir_all(install_dir.join("bin")).unwrap();
    std::fs::write(install_dir.join("bin").join(executable()), b"cached sdk").unwrap();

    let runtime = runtime(false);
    let requested = if latest { "latest" } else { &version };
    assert!(
        !runtime.is_installed(requested, &ctx).await.unwrap(),
        "a failed lock acquisition must not validate an unprotected SDK"
    );
}
