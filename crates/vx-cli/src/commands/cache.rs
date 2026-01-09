//! Cache management command implementation

use crate::cli::CacheCommand;
use crate::ui::UI;
use anyhow::Result;
use vx_cache::DownloadCache;
use vx_paths::VxPaths;
use vx_resolver::{ResolutionCache, RESOLUTION_CACHE_DIR_NAME};
use vx_runtime::VersionCache;

/// Handle cache subcommands
pub async fn handle(command: CacheCommand) -> Result<()> {
    match command {
        CacheCommand::Clear {
            versions,
            downloads,
            resolutions,
            tool,
            force,
        } => handle_clear(versions, downloads, resolutions, tool, force).await,
        CacheCommand::Stats => handle_stats().await,
        CacheCommand::List { verbose } => handle_list(verbose).await,
    }
}

/// Clear cache
async fn handle_clear(
    versions_only: bool,
    downloads_only: bool,
    resolutions_only: bool,
    tool: Option<String>,
    force: bool,
) -> Result<()> {
    let paths = VxPaths::new()?;

    // If specific tool is specified, only clear that tool's cache
    if let Some(tool_name) = tool {
        let cache_file = paths
            .cache_dir
            .join("versions")
            .join(format!("{}.json", tool_name));
        if cache_file.exists() {
            std::fs::remove_file(&cache_file)?;
            UI::success(&format!("Cleared cache for '{}'", tool_name));
        } else {
            UI::info(&format!("No cache found for '{}'", tool_name));
        }
        return Ok(());
    }

    // Determine what to clear
    // - If no selector flag is provided, clear everything.
    // - If any selector flag is provided, clear only the selected categories.
    let any_selector = versions_only || downloads_only || resolutions_only;
    let clear_versions = if any_selector { versions_only } else { true };
    let clear_downloads = if any_selector { downloads_only } else { true };
    let clear_resolutions = if any_selector { resolutions_only } else { true };

    // Clear version cache
    if clear_versions {
        let cache_dir = paths.cache_dir.join("versions");
        let version_cache = VersionCache::new(cache_dir);

        if force {
            version_cache.clear_all()?;
            UI::success("Version cache cleared");
        } else {
            let pruned = version_cache.prune()?;
            if pruned > 0 {
                UI::success(&format!("Pruned {} expired cache entries", pruned));
            } else {
                UI::info("No expired cache entries to prune");
            }
            UI::hint("Use --force to clear all cache entries");
        }
    }

    // Clear download cache (new high-performance CAS cache)
    if clear_downloads {
        let download_cache = DownloadCache::new(paths.cache_dir.clone());
        let stats_before = download_cache.stats();
        if stats_before.file_count > 0 {
            match download_cache.clear() {
                Ok(bytes_freed) => {
                    UI::success(&format!(
                        "Cleared {} download cache files ({})",
                        stats_before.file_count,
                        format_size(bytes_freed)
                    ));
                }
                Err(e) => {
                    UI::warning(&format!("Failed to clear download cache: {}", e));
                }
            }
        } else {
            UI::info("Download cache: (empty)");
        }
    }

    // Clear resolution cache
    if clear_resolutions {
        let cache_dir = paths.cache_dir.join(RESOLUTION_CACHE_DIR_NAME);
        let resolution_cache = ResolutionCache::new(cache_dir);

        if force {
            let removed = resolution_cache.clear_all()?;
            if removed > 0 {
                UI::success(&format!("Resolution cache cleared ({} entries)", removed));
            } else {
                UI::info("Resolution cache: (empty)");
            }
        } else {
            let pruned = resolution_cache.prune_expired()?;
            if pruned > 0 {
                UI::success(&format!("Pruned {} expired resolution entries", pruned));
            } else {
                UI::info("No expired resolution entries to prune");
            }
            UI::hint("Use --force to clear all resolution cache entries");
        }
    }

    Ok(())
}

/// Show cache statistics
async fn handle_stats() -> Result<()> {
    let paths = VxPaths::new()?;
    let cache_dir = paths.cache_dir.join("versions");
    let version_cache = VersionCache::new(cache_dir);

    UI::header("Cache Statistics");

    // Version cache stats
    if let Ok(stats) = version_cache.stats() {
        println!();
        UI::info("Version Cache:");
        println!("  Total entries:   {}", stats.total_entries);
        println!("  Valid entries:   {}", stats.valid_entries);
        println!("  Expired entries: {}", stats.expired_entries);
        println!("  Total size:      {}", stats.formatted_size());
    } else {
        UI::info("Version Cache: (empty)");
    }

    // Resolution cache stats
    let resolution_dir = paths.cache_dir.join(RESOLUTION_CACHE_DIR_NAME);
    let resolution_cache = ResolutionCache::new(resolution_dir);
    let resolution_stats = resolution_cache.stats()?;
    println!();
    UI::info("Resolution Cache:");
    println!("  Total entries:   {}", resolution_stats.total_entries);
    println!("  Valid entries:   {}", resolution_stats.valid_entries);
    println!("  Expired entries: {}", resolution_stats.expired_entries);
    println!("  Total size:      {}", resolution_stats.formatted_size());

    // Download cache stats (new high-performance CAS cache)
    let download_cache = DownloadCache::new(paths.cache_dir.clone());
    let download_stats = download_cache.stats();
    println!();
    UI::info("Download Cache:");
    println!("  Cached files: {}", download_stats.file_count);
    println!("  Total size:   {}", download_stats.formatted_size());

    println!();
    UI::hint("Run 'vx cache clear' to prune expired entries");
    UI::hint("Run 'vx cache clear --force' to remove all cache");

    Ok(())
}

/// List cached items
async fn handle_list(verbose: bool) -> Result<()> {
    let paths = VxPaths::new()?;
    let cache_dir = paths.cache_dir.join("versions");

    UI::header("Cached Version Lists");

    if !cache_dir.exists() {
        UI::info("No version cache found");
        return Ok(());
    }

    let version_cache = VersionCache::new(cache_dir.clone());

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        let mut found = false;
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                found = true;
                let tool_name = path.file_stem().unwrap_or_default().to_string_lossy();

                if verbose {
                    // Show detailed info
                    if let Ok(meta) = path.metadata() {
                        let size = format_size(meta.len());
                        // Use get_entry to check if cache is valid
                        let is_valid = version_cache.get_entry(&tool_name).is_some();
                        let status = if is_valid { "valid" } else { "expired" };
                        println!("  {} ({}, {})", tool_name, size, status);
                    } else {
                        println!("  {}", tool_name);
                    }
                } else {
                    println!("  {}", tool_name);
                }
            }
        }

        if !found {
            UI::info("No cached version lists");
        }
    }

    Ok(())
}

/// Format byte size to human-readable string
fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
