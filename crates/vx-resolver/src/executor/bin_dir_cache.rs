//! Bin directory cache and discovery
//!
//! This module manages the process-level bin directory cache backed by disk
//! persistence, and provides functions to locate executable bin directories
//! within the vx store.

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::trace;
use vx_cache::BinDirCache;

/// Track which tools have been warned about missing versions.
/// This prevents duplicate warnings when building PATH.
pub(crate) static WARNED_TOOLS: Mutex<Option<HashSet<String>>> = Mutex::new(None);

/// Process-level bin directory cache, backed by disk persistence.
///
/// On first access, loads from `~/.vx/cache/bin-dirs.bin`. New entries are
/// accumulated in memory and flushed to disk via `save_bin_dir_cache()`.
/// This avoids the cold-start penalty of walkdir traversals when the
/// process-level cache would otherwise be empty.
static BIN_DIR_CACHE: Mutex<Option<BinDirCache>> = Mutex::new(None);

/// Initialize the bin directory cache from disk (if not already loaded).
///
/// Called by the Executor during construction to pre-warm the cache.
pub(crate) fn init_bin_dir_cache(cache_dir: &std::path::Path) {
    let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
    if cache.is_none() {
        *cache = Some(BinDirCache::load(cache_dir));
    }
}

/// Save the bin directory cache to disk.
///
/// Called by the Executor after command execution to persist new entries.
pub(crate) fn save_bin_dir_cache(cache_dir: &std::path::Path) {
    let cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
    if let Some(ref c) = *cache
        && let Err(e) = c.save(cache_dir)
    {
        tracing::debug!("Failed to save bin dir cache: {}", e);
    }
}

/// Invalidate the bin directory cache for a specific runtime.
///
/// Call this after installing or uninstalling a runtime.
pub fn invalidate_bin_dir_cache(runtime_store_prefix: &str) {
    let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
    if let Some(ref mut c) = *cache {
        c.invalidate_runtime(runtime_store_prefix);
    }
}

/// Clear the entire bin directory cache.
pub fn clear_bin_dir_cache() {
    let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
    *cache = None;
}

/// Find the bin directory for a runtime within a store directory.
///
/// This method searches for the executable within the store directory
/// structure and returns the directory containing it. This handles various archive
/// structures like:
/// - `<version>/<platform>/bin/node.exe` (standard)
/// - `<version>/<platform>/node-v25.6.0-win-x64/node.exe` (Node.js style)
/// - `<version>/<platform>/yarn-v1.22.19/bin/yarn.cmd` (Yarn style)
///
/// Results are cached in a process-level `BIN_DIR_CACHE` to avoid repeated
/// walkdir traversals on subsequent calls.
///
/// ## Performance optimization
///
/// Uses a two-phase search:
/// 1. **Quick check**: Common locations (root, bin/, one-level subdirs)
/// 2. **Walkdir fallback**: Only if quick check fails, with directory filtering
pub fn find_bin_dir(store_dir: &std::path::Path, runtime_name: &str) -> Option<PathBuf> {
    let cache_key = store_dir.to_string_lossy().to_string();

    // Check process-level cache first (backed by disk persistence)
    {
        let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
        if let Some(ref mut c) = *cache
            && let Some(cached) = c.get(&cache_key)
        {
            trace!(
                "BIN_DIR_CACHE hit for {} in {}",
                runtime_name,
                store_dir.display()
            );
            return Some(cached);
        }
    }

    let platform = vx_paths::manager::CurrentPlatform::current();

    // Build the list of possible executable names
    let exe_names: Vec<String> = if cfg!(windows) {
        vec![
            format!("{}.exe", runtime_name),
            format!("{}.cmd", runtime_name),
            runtime_name.to_string(),
        ]
    } else {
        vec![runtime_name.to_string()]
    };

    // Platform-specific directory
    let platform_dir = store_dir.join(platform.as_str());
    let search_dir = if platform_dir.exists() {
        &platform_dir
    } else if store_dir.exists() {
        store_dir
    } else {
        return None;
    };

    // Phase 1: Quick check common locations first (avoids walkdir entirely
    // for most runtimes like uv, go, bun, pnpm where exe is in root or bin/)
    if let Some(result) = quick_find_bin_dir(search_dir, &exe_names) {
        let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
        let c = cache.get_or_insert_with(BinDirCache::new);
        c.put(cache_key, result.clone());
        return Some(result);
    }

    // Phase 2: Walkdir with directory filtering
    for entry in walkdir::WalkDir::new(search_dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            // Skip known non-target directories
            if e.file_type().is_dir()
                && let Some(name) = e.file_name().to_str()
            {
                return !matches!(
                    name,
                    "node_modules"
                        | ".git"
                        | "__pycache__"
                        | "site-packages"
                        | "lib"
                        | "share"
                        | "include"
                        | "man"
                        | "doc"
                        | "docs"
                );
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file()
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
            && exe_names.iter().any(|exe_name| name == exe_name)
            && let Some(parent) = path.parent()
        {
            trace!(
                "Found executable for {} at {}, using bin dir: {}",
                runtime_name,
                path.display(),
                parent.display()
            );
            let result = parent.to_path_buf();
            let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
            let c = cache.get_or_insert_with(BinDirCache::new);
            c.put(cache_key, result.clone());
            return Some(result);
        }
    }

    // Fallback: check standard locations
    let bin_dir = platform_dir.join("bin");
    if bin_dir.exists() {
        let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
        let c = cache.get_or_insert_with(BinDirCache::new);
        c.put(cache_key, bin_dir.clone());
        return Some(bin_dir);
    }

    let bin_dir = store_dir.join("bin");
    if bin_dir.exists() {
        let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
        let c = cache.get_or_insert_with(BinDirCache::new);
        c.put(cache_key, bin_dir.clone());
        return Some(bin_dir);
    }

    // Last resort: return platform dir if it exists
    if platform_dir.exists() {
        let mut cache = BIN_DIR_CACHE.lock().expect("BIN_DIR_CACHE mutex poisoned");
        let c = cache.get_or_insert_with(BinDirCache::new);
        c.put(cache_key, platform_dir.clone());
        return Some(platform_dir);
    }

    None
}

/// Quick check common locations for bin directory (avoids walkdir).
///
/// Most runtimes follow predictable patterns:
/// - Root dir contains executable (uv, pnpm, kubectl)
/// - bin/ subdirectory (go, java)
/// - Single subdirectory contains executable (node-v25.6.0-win-x64/node.exe)
/// - Single subdirectory + bin/ (yarn-v1.22.19/bin/yarn)
fn quick_find_bin_dir(search_dir: &std::path::Path, exe_names: &[String]) -> Option<PathBuf> {
    // Check root directory
    for name in exe_names {
        if search_dir.join(name).is_file() {
            return Some(search_dir.to_path_buf());
        }
    }

    // Check bin/ subdirectory
    let bin_dir = search_dir.join("bin");
    if bin_dir.is_dir() {
        for name in exe_names {
            if bin_dir.join(name).is_file() {
                return Some(bin_dir);
            }
        }
    }

    // Check one level of subdirectories
    if let Ok(entries) = std::fs::read_dir(search_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let sub_path = entry.path();
            if !sub_path.is_dir() {
                continue;
            }
            if let Some(dir_name) = sub_path.file_name().and_then(|n| n.to_str())
                && matches!(
                    dir_name,
                    "bin" | "node_modules" | "lib" | "share" | "include" | "man" | "doc"
                )
            {
                continue;
            }
            // Check files in subdirectory
            for name in exe_names {
                if sub_path.join(name).is_file() {
                    return Some(sub_path);
                }
            }
            // Check subdirectory/bin/
            let sub_bin = sub_path.join("bin");
            if sub_bin.is_dir() {
                for name in exe_names {
                    if sub_bin.join(name).is_file() {
                        return Some(sub_bin);
                    }
                }
            }
        }
    }

    None
}

/// Record a "version not installed" warning for a tool (prevents duplicates).
///
/// Returns `true` if this is the first warning for the given tool name.
pub(crate) fn record_warned_tool(tool_name: &str) -> bool {
    let mut warned = WARNED_TOOLS.lock().expect("WARNED_TOOLS mutex poisoned");
    let warned_set = warned.get_or_insert_with(HashSet::new);
    warned_set.insert(tool_name.to_string())
}
