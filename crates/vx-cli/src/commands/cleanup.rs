// Cleanup command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_paths::VxPaths;
use vx_runtime::VersionCache;

pub async fn handle(
    dry_run: bool,
    cache_only: bool,
    orphaned_only: bool,
    force: bool,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<()> {
    let paths = VxPaths::new()?;

    if cache_only {
        // Clean version cache
        let cache_dir = paths.cache_dir.join("versions");
        let version_cache = VersionCache::new(cache_dir.clone());

        // Show cache statistics
        if let Ok(stats) = version_cache.stats() {
            if verbose || dry_run {
                UI::info(&format!(
                    "Version cache: {} entries ({} valid, {} expired), {}",
                    stats.total_entries,
                    stats.valid_entries,
                    stats.expired_entries,
                    stats.formatted_size()
                ));
            }
        }

        if dry_run {
            UI::info("Would clean version cache:");
            if cache_dir.exists() {
                let mut found = false;
                if let Ok(entries) = std::fs::read_dir(&cache_dir) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.extension().is_some_and(|ext| ext == "json") {
                            found = true;
                            if verbose {
                                UI::hint(&format!("  - {}", path.display()));
                            }
                        }
                    }
                }
                if !found {
                    UI::hint("  (no version cache found)");
                }
            } else {
                UI::hint("  (no version cache found)");
            }
        } else {
            // Prune expired entries first if not force
            if !force {
                let pruned = version_cache.prune()?;
                if pruned > 0 {
                    UI::success(&format!("Pruned {} expired cache entries", pruned));
                }
            } else {
                version_cache.clear_all()?;
                UI::success("Version cache cleared");
            }
        }

        // Clean download cache
        let download_cache = &paths.cache_dir;
        if dry_run {
            UI::info("Would clean download cache:");
            if download_cache.exists() {
                let mut count = 0;
                let mut size: u64 = 0;
                if let Ok(entries) = std::fs::read_dir(download_cache) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            count += 1;
                            if let Ok(meta) = path.metadata() {
                                size += meta.len();
                            }
                            if verbose {
                                UI::hint(&format!("  - {}", path.display()));
                            }
                        }
                    }
                }
                let size_str = if size < 1024 {
                    format!("{} B", size)
                } else if size < 1024 * 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                };
                UI::hint(&format!("  {} files ({})", count, size_str));
            }
        } else if force {
            // Only clean files in cache dir, not subdirectories
            if download_cache.exists() {
                let mut count = 0;
                if let Ok(entries) = std::fs::read_dir(download_cache) {
                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            std::fs::remove_file(&path)?;
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    UI::success(&format!("Cleaned {} cached files", count));
                }
            }
        }

        return Ok(());
    }

    UI::warning("Cleanup command not yet fully implemented in new architecture");
    UI::hint(&format!(
        "Would cleanup with options: dry_run={}, cache_only={}, orphaned_only={}, force={}, older_than={:?}, verbose={}",
        dry_run, cache_only, orphaned_only, force, older_than, verbose
    ));
    Ok(())
}
