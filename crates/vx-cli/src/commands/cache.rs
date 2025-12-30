//! Cache management command implementation

use crate::cli::CacheCommand;
use crate::ui::UI;
use anyhow::Result;
use vx_paths::VxPaths;
use vx_runtime::VersionCache;

/// Handle cache subcommands
pub async fn handle(command: CacheCommand) -> Result<()> {
    match command {
        CacheCommand::Clear {
            versions,
            downloads,
            tool,
            force,
        } => handle_clear(versions, downloads, tool, force).await,
        CacheCommand::Stats => handle_stats().await,
        CacheCommand::List { verbose } => handle_list(verbose).await,
    }
}

/// Clear cache
async fn handle_clear(
    versions_only: bool,
    downloads_only: bool,
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
    let clear_versions = !downloads_only;
    let clear_downloads = !versions_only;

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

    // Clear download cache
    if clear_downloads {
        let download_cache = &paths.cache_dir;
        if download_cache.exists() {
            let mut count = 0;
            if let Ok(entries) = std::fs::read_dir(download_cache) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    // Only remove files, not subdirectories like 'versions'
                    if path.is_file() {
                        std::fs::remove_file(&path)?;
                        count += 1;
                    }
                }
            }
            if count > 0 {
                UI::success(&format!("Cleared {} download cache files", count));
            }
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

    // Download cache stats
    let download_cache = &paths.cache_dir;
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
                }
            }
        }
        let size_str = format_size(size);
        println!();
        UI::info("Download Cache:");
        println!("  Files: {}", count);
        println!("  Size:  {}", size_str);
    }

    println!();
    UI::hint("Run 'vx cache clear' to remove expired entries");
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
