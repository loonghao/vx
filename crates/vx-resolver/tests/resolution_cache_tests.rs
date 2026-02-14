use rstest::rstest;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use vx_resolver::{
    RESOLUTION_CACHE_SCHEMA_VERSION, ResolutionCache, ResolutionCacheKey, ResolvedGraph,
};
use vx_runtime::CacheMode;

fn make_key(cwd: PathBuf) -> ResolutionCacheKey {
    ResolutionCacheKey {
        schema_version: RESOLUTION_CACHE_SCHEMA_VERSION,
        vx_version: "0.0.0-test".to_string(),
        os: "test-os".to_string(),
        arch: "test-arch".to_string(),
        cwd,
        runtime: "npm".to_string(),
        version: None,
        args_digest: "args".to_string(),
        config_digest: None,
        lock_digest: None,
        prefer_vx_managed: true,
        fallback_to_system: true,
    }
}

fn make_graph() -> ResolvedGraph {
    ResolvedGraph {
        runtime: "npm".to_string(),
        // Keep this relative so tests don't depend on filesystem executability.
        executable: PathBuf::from("npm"),
        command_prefix: vec![],
        missing_dependencies: vec![],
        install_order: vec![],
        runtime_needs_install: false,
        incompatible_dependencies: vec![],
        unsupported_platform_runtimes: vec![],
    }
}

#[rstest]
fn test_resolution_cache_set_get() {
    let dir = TempDir::new().unwrap();
    let cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Normal)
        .with_ttl(Duration::from_secs(60));

    let key = make_key(dir.path().to_path_buf());
    let value = make_graph();

    cache.set(&key, &value).unwrap();
    let got = cache.get(&key).expect("expected cache hit");
    assert_eq!(got.runtime, value.runtime);
    assert_eq!(got.executable, value.executable);
}

#[rstest]
fn test_resolution_cache_ttl_expired_is_miss_in_normal_mode() {
    let dir = TempDir::new().unwrap();
    let cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Normal)
        .with_ttl(Duration::from_secs(0));

    let key = make_key(dir.path().to_path_buf());
    let value = make_graph();

    cache.set(&key, &value).unwrap();
    assert!(cache.get(&key).is_none());
}

#[rstest]
fn test_resolution_cache_ttl_expired_is_allowed_in_offline_mode() {
    let dir = TempDir::new().unwrap();
    let cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Offline)
        .with_ttl(Duration::from_secs(0));

    let key = make_key(dir.path().to_path_buf());
    let value = make_graph();

    cache.set(&key, &value).unwrap();
    assert!(cache.get(&key).is_some());
}

#[rstest]
fn test_resolution_cache_refresh_mode_ignores_cache() {
    let dir = TempDir::new().unwrap();
    let cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Normal)
        .with_ttl(Duration::from_secs(60));

    let key = make_key(dir.path().to_path_buf());
    let value = make_graph();

    cache.set(&key, &value).unwrap();

    let refresh_cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::Refresh)
        .with_ttl(Duration::from_secs(60));

    assert!(refresh_cache.get(&key).is_none());
}

#[rstest]
fn test_resolution_cache_no_cache_mode_does_not_write() {
    let dir = TempDir::new().unwrap();
    let cache = ResolutionCache::new(dir.path().to_path_buf())
        .with_mode(CacheMode::NoCache)
        .with_ttl(Duration::from_secs(60));

    let key = make_key(dir.path().to_path_buf());
    let value = make_graph();

    cache.set(&key, &value).unwrap();
    assert!(cache.get(&key).is_none());
}
