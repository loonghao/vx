use std::path::PathBuf;
use tempfile::TempDir;
use vx_cache::ExecPathCache;

#[test]
fn test_put_and_get() {
    let mut cache = ExecPathCache::new();
    let dir = PathBuf::from("/fake/store/node/20.0.0/windows-x64");
    let exe_path = dir.join("node.exe");

    cache.put(&dir, "node", exe_path.clone());
    // We can't validate on-disk existence in unit tests with fake paths,
    // so test the raw entry
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_invalidate_runtime() {
    let mut cache = ExecPathCache::new();
    let base = PathBuf::from("/store/node");

    let dir1 = base.join("20.0.0/windows-x64");
    let dir2 = base.join("18.0.0/windows-x64");
    let other = PathBuf::from("/store/go/1.21.0/windows-x64");

    cache.put(&dir1, "node", dir1.join("node.exe"));
    cache.put(&dir2, "node", dir2.join("node.exe"));
    cache.put(&other, "go", other.join("go.exe"));

    assert_eq!(cache.len(), 3);

    cache.invalidate_runtime(&base);
    assert_eq!(cache.len(), 1); // Only go remains
}

#[test]
fn test_save_and_load() {
    let temp = TempDir::new().unwrap();
    let cache_dir = temp.path();

    // Create a real file so get() validation passes
    let exe_dir = temp.path().join("store/node/20.0.0");
    std::fs::create_dir_all(&exe_dir).unwrap();
    let exe_path = exe_dir.join("node.exe");
    std::fs::write(&exe_path, b"fake").unwrap();

    let mut cache = ExecPathCache::new();
    cache.put(&exe_dir, "node", exe_path.clone());
    cache.save(cache_dir).unwrap();

    // Load from disk
    let mut loaded = ExecPathCache::load(cache_dir);
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded.get(&exe_dir, "node"), Some(exe_path));
}

#[test]
fn test_load_missing_file() {
    let temp = TempDir::new().unwrap();
    let cache = ExecPathCache::load(temp.path());
    assert!(cache.is_empty());
}

#[test]
fn test_clear() {
    let mut cache = ExecPathCache::new();
    cache.put(&PathBuf::from("/a"), "x", PathBuf::from("/a/x.exe"));
    cache.put(&PathBuf::from("/b"), "y", PathBuf::from("/b/y.exe"));
    assert_eq!(cache.len(), 2);
    cache.clear();
    assert!(cache.is_empty());
}
