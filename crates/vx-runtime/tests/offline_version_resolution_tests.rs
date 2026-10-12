//! Offline reuse contract for `resolve_version`.
//!
//! An exact version that already exists in the local store must resolve without
//! fetching the remote index, so installed runtimes stay usable with no network.
//! These tests assert that property directly by making every network call fail.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{Result, bail};
use async_trait::async_trait;
use rstest::rstest;
use tempfile::TempDir;
use vx_runtime::{
    Ecosystem, HttpClient, Installer, ManifestDrivenRuntime, ProviderSource, RealFileSystem,
    RealPathProvider, Runtime, RuntimeContext, VersionInfo,
};

/// Fails every request, so a resolved version proves no index was fetched.
#[derive(Debug)]
struct OfflineHttpClient {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl HttpClient for OfflineHttpClient {
    async fn get(&self, _url: &str) -> Result<String> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        bail!("network is unavailable")
    }

    async fn get_json_value(&self, _url: &str) -> Result<serde_json::Value> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        bail!("network is unavailable")
    }

    async fn download(&self, _url: &str, _dest: &Path) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        bail!("network is unavailable")
    }

    async fn download_with_progress(
        &self,
        _url: &str,
        _dest: &Path,
        _on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        bail!("network is unavailable")
    }
}

#[derive(Debug)]
struct NoopInstaller;

#[async_trait]
impl Installer for NoopInstaller {
    async fn extract(&self, _archive: &Path, _dest: &Path) -> Result<()> {
        bail!("not used in offline_version_resolution_tests")
    }

    async fn download_and_extract(&self, _url: &str, _dest: &Path) -> Result<()> {
        bail!("not used in offline_version_resolution_tests")
    }
}

struct OfflineFixture {
    _temp: TempDir,
    ctx: RuntimeContext,
    calls: Arc<AtomicUsize>,
}

fn offline_fixture() -> OfflineFixture {
    let temp = tempfile::tempdir().expect("temp dir should be created");
    let calls = Arc::new(AtomicUsize::new(0));
    let ctx = RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(temp.path())),
        Arc::new(OfflineHttpClient {
            calls: Arc::clone(&calls),
        }),
        Arc::new(RealFileSystem::new()),
        Arc::new(NoopInstaller),
    );
    OfflineFixture {
        _temp: temp,
        ctx,
        calls,
    }
}

/// A runtime whose only version source is the network, which is offline here.
fn offline_runtime(store_name: &str) -> ManifestDrivenRuntime {
    ManifestDrivenRuntime::new(store_name, "tool", ProviderSource::BuiltIn)
        .with_ecosystem(Ecosystem::DevTools)
        .with_fetch_versions(|| {
            Box::pin(async { Err(anyhow::anyhow!("remote index is unreachable")) })
        })
}

/// An installed version resolves to itself without touching the network.
#[rstest]
#[case("1.2.3")]
#[case("1.2.3-rc.1")]
#[case("v1.2.3")]
#[tokio::test]
async fn installed_exact_version_resolves_offline(#[case] version: &str) {
    let fixture = offline_fixture();
    let runtime = offline_runtime("offline-tool");
    let store_dir = fixture.ctx.paths.version_store_dir("offline-tool", version);
    std::fs::create_dir_all(&store_dir).expect("store dir should be created");

    let resolved = Runtime::resolve_version(&runtime, version, &fixture.ctx)
        .await
        .expect("an installed exact version should resolve without a remote index");

    assert_eq!(resolved, version);
    assert_eq!(
        fixture.calls.load(Ordering::SeqCst),
        0,
        "no network call should be made for an installed exact version"
    );
}

/// A version that is not installed still needs the index, and fails offline.
#[tokio::test]
async fn missing_exact_version_still_requires_the_remote_index() {
    let fixture = offline_fixture();
    let runtime = offline_runtime("offline-tool");

    let err = Runtime::resolve_version(&runtime, "9.9.9", &fixture.ctx)
        .await
        .expect_err("a version that is not installed cannot resolve offline");

    assert!(
        err.to_string().contains("remote index is unreachable"),
        "the failure should come from the version fetch, got: {err}"
    );
}

/// Version specs that are not exact pins must always consult the index.
#[rstest]
#[case("latest")]
#[case("1.2.*")]
#[case("^1.2.0")]
#[case("~1.2.0")]
#[case(">=1.2.0")]
#[case("<2.0.0")]
#[case("1.2.3, <1.3.0")]
#[case("=1.2.3")]
#[tokio::test]
async fn non_exact_specs_always_fetch_the_remote_index(#[case] spec: &str) {
    let fixture = offline_fixture();
    let runtime = offline_runtime("offline-tool");
    // Install the pinned form so an incorrect short-circuit would succeed
    // offline and hide the bug the spec is protecting against.
    let store_dir = fixture.ctx.paths.version_store_dir("offline-tool", "1.2.3");
    std::fs::create_dir_all(&store_dir).expect("store dir should be created");

    let err = Runtime::resolve_version(&runtime, spec, &fixture.ctx)
        .await
        .expect_err("a non-exact spec should require the remote index");

    assert!(
        err.to_string().contains("remote index is unreachable"),
        "spec {spec} should have reached the version fetch, got: {err}"
    );
}

/// The short-circuit must not fire on a path-like or empty spec.
#[rstest]
#[case("")]
#[case(".")]
#[case("..")]
#[case("1.2.3/../../etc")]
#[case("./1.2.3")]
#[tokio::test]
async fn path_like_specs_are_not_treated_as_exact_local_versions(#[case] spec: &str) {
    let fixture = offline_fixture();
    let runtime = offline_runtime("offline-tool");

    let err = Runtime::resolve_version(&runtime, spec, &fixture.ctx)
        .await
        .expect_err("a path-like spec should not be treated as an installed version");

    assert!(
        err.to_string().contains("remote index is unreachable"),
        "spec {spec:?} should have reached the version fetch, got: {err}"
    );
}

/// An exact pin resolves offline even when the runtime advertises remote versions.
#[tokio::test]
async fn installed_version_wins_over_the_remote_index() {
    let temp = tempfile::tempdir().expect("temp dir should be created");
    let calls = Arc::new(AtomicUsize::new(0));
    let ctx = RuntimeContext::new(
        Arc::new(RealPathProvider::with_base_dir(temp.path())),
        Arc::new(OfflineHttpClient {
            calls: Arc::clone(&calls),
        }),
        Arc::new(RealFileSystem::new()),
        Arc::new(NoopInstaller),
    );
    let runtime = ManifestDrivenRuntime::new("offline-tool", "tool", ProviderSource::BuiltIn)
        .with_ecosystem(Ecosystem::DevTools)
        .with_fetch_versions(|| Box::pin(async { Ok(vec![VersionInfo::new("2.0.0")]) }));
    let store_dir = ctx.paths.version_store_dir("offline-tool", "1.2.3");
    std::fs::create_dir_all(&store_dir).expect("store dir should be created");

    let resolved = Runtime::resolve_version(&runtime, "1.2.3", &ctx)
        .await
        .expect("the installed version should resolve");

    assert_eq!(resolved, "1.2.3");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "the local store should be preferred over the remote index"
    );
}
