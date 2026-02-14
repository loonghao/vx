//! Tests for LockFile::prune functionality
//!
//! Ensures that tools removed from vx.toml are cleaned up from vx.lock,
//! while auto-resolved dependencies are preserved.

use std::collections::HashSet;
use vx_resolver::{Ecosystem, LockFile, LockedTool};

fn make_tool(version: &str, source: &str, resolved_from: &str) -> LockedTool {
    LockedTool::new(version, source).with_resolved_from(resolved_from)
}

#[test]
fn test_prune_removes_stale_tools() {
    let mut lock = LockFile::new();
    lock.lock_tool("python", make_tool("3.11.13", "python", "3.11"));
    lock.lock_tool("node", make_tool("20.0.0", "nodejs.org", "20"));
    lock.lock_tool("bun", make_tool("1.3.8", "bun", "latest"));
    lock.lock_tool("magick", make_tool("7.1.2-13", "generic", "latest"));

    // Only python and node are in config + resolved
    let keep = HashSet::from(["python".to_string(), "node".to_string()]);
    let removed = lock.prune(&keep);

    assert_eq!(lock.tools.len(), 2);
    assert!(lock.is_locked("python"));
    assert!(lock.is_locked("node"));
    assert!(!lock.is_locked("bun"));
    assert!(!lock.is_locked("magick"));
    assert_eq!(removed.len(), 2);
    assert!(removed.contains(&"bun".to_string()));
    assert!(removed.contains(&"magick".to_string()));
}

#[test]
fn test_prune_preserves_resolved_dependencies() {
    let mut lock = LockFile::new();
    lock.lock_tool("npm", make_tool("10.0.0", "nodejs.org", "latest"));
    lock.lock_tool("node", make_tool("20.0.0", "nodejs.org", "20"));
    lock.lock_tool("bun", make_tool("1.3.8", "bun", "latest"));
    lock.add_dependency("npm".to_string(), vec!["node".to_string()]);

    // Config has npm only, but node was resolved as dependency
    let keep = HashSet::from(["npm".to_string(), "node".to_string()]);
    let removed = lock.prune(&keep);

    assert_eq!(lock.tools.len(), 2);
    assert!(lock.is_locked("npm"));
    assert!(lock.is_locked("node"));
    assert!(!lock.is_locked("bun"));
    assert_eq!(removed.len(), 1);
    assert!(removed.contains(&"bun".to_string()));
}

#[test]
fn test_prune_cleans_up_dependencies() {
    let mut lock = LockFile::new();
    lock.lock_tool("npm", make_tool("10.0.0", "nodejs.org", "latest"));
    lock.lock_tool("node", make_tool("20.0.0", "nodejs.org", "20"));
    lock.lock_tool("bun", make_tool("1.3.8", "bun", "latest"));
    lock.add_dependency("npm".to_string(), vec!["node".to_string()]);
    lock.add_dependency(
        "bun".to_string(),
        vec!["node".to_string(), "npm".to_string()],
    );

    // Keep npm and node, remove bun
    let keep = HashSet::from(["npm".to_string(), "node".to_string()]);
    lock.prune(&keep);

    // bun's dependency entry should be removed
    assert!(lock.get_dependencies("bun").is_none());
    // npm's dependency entry should still exist
    assert_eq!(
        lock.get_dependencies("npm"),
        Some(&vec!["node".to_string()])
    );
}

#[test]
fn test_prune_removes_references_to_pruned_tools_in_deps() {
    let mut lock = LockFile::new();
    lock.lock_tool("a", make_tool("1.0.0", "src", "1.0"));
    lock.lock_tool("b", make_tool("2.0.0", "src", "2.0"));
    lock.lock_tool("c", make_tool("3.0.0", "src", "3.0"));
    // a depends on both b and c
    lock.add_dependency("a".to_string(), vec!["b".to_string(), "c".to_string()]);

    // Keep a and b, remove c
    let keep = HashSet::from(["a".to_string(), "b".to_string()]);
    lock.prune(&keep);

    // a's deps should now only reference b (c was pruned)
    let deps = lock.get_dependencies("a").unwrap();
    assert_eq!(deps, &vec!["b".to_string()]);
    assert!(!deps.contains(&"c".to_string()));
}

#[test]
fn test_prune_removes_empty_dependency_entries() {
    let mut lock = LockFile::new();
    lock.lock_tool("a", make_tool("1.0.0", "src", "1.0"));
    lock.lock_tool("b", make_tool("2.0.0", "src", "2.0"));
    // a depends only on b
    lock.add_dependency("a".to_string(), vec!["b".to_string()]);

    // Keep a, remove b
    let keep = HashSet::from(["a".to_string()]);
    lock.prune(&keep);

    // a's dependency entry should be removed (b was pruned, leaving empty list)
    assert!(lock.get_dependencies("a").is_none());
}

#[test]
fn test_prune_noop_when_all_tools_kept() {
    let mut lock = LockFile::new();
    lock.lock_tool(
        "python",
        make_tool("3.11.13", "python", "3.11").with_ecosystem(Ecosystem::Python),
    );
    lock.lock_tool("rust", make_tool("1.93.1", "rust", "1.93.1"));

    let keep = HashSet::from(["python".to_string(), "rust".to_string()]);
    let removed = lock.prune(&keep);

    assert_eq!(lock.tools.len(), 2);
    assert!(removed.is_empty());
}

#[test]
fn test_prune_all_tools() {
    let mut lock = LockFile::new();
    lock.lock_tool("bun", make_tool("1.3.8", "bun", "latest"));
    lock.lock_tool("magick", make_tool("7.1.2-13", "generic", "latest"));

    let keep = HashSet::new();
    let removed = lock.prune(&keep);

    assert!(lock.tools.is_empty());
    assert_eq!(removed.len(), 2);
}

#[test]
fn test_prune_empty_lock() {
    let mut lock = LockFile::new();
    let keep = HashSet::from(["python".to_string()]);
    let removed = lock.prune(&keep);

    assert!(lock.tools.is_empty());
    assert!(removed.is_empty());
}
