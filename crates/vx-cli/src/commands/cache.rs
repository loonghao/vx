//! Cache management command implementation
//!
//! This module consolidates all cache-related operations:
//! - `info`: Show cache statistics and disk usage
//! - `list`: List cached items
//! - `prune`: Safely remove expired/orphaned cache entries
//! - `purge`: Forcefully remove all cache data
//! - `dir`: Show cache directory path
//!
//! ## Design Philosophy
//!
//! - `prune` = Safe, intelligent cleanup (only removes expired/orphaned items)
//! - `purge` = Destructive, complete removal (removes everything)
//!
//! This avoids the confusing `clear` vs `clean` naming.

use super::common::format_size;
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
        CacheCommand::Info => handle_info().await,
        CacheCommand::List { verbose } => handle_list(verbose).await,
        CacheCommand::Prune {
            dry_run,
            versions,
            downloads,
            resolutions,
            orphaned,
            older_than,
            verbose,
        } => {
            handle_prune(
                dry_run,
                versions,
                downloads,
                resolutions,
                orphaned,
                older_than,
                verbose,
            )
            .await
        }
        CacheCommand::Purge {
            versions,
            downloads,
            resolutions,
            tool,
            yes,
        } => handle_purge(versions, downloads, resolutions, tool, yes).await,
        CacheCommand::Dir => handle_dir().await,
    }
}

/// Show cache statistics (formerly `stats`)
async fn handle_info() -> Result<()> {
    let paths = VxPaths::new()?;
    // VersionCache::new expects the base cache dir and appends "versions_v2" internally
    let version_cache = VersionCache::new(paths.cache_dir.clone());

    UI::header("Cache Information");

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

    // Download cache stats
    let download_cache = DownloadCache::new(paths.cache_dir.clone());
    let download_stats = download_cache.stats();
    println!();
    UI::info("Download Cache:");
    println!("  Cached files: {}", download_stats.file_count);
    println!("  Total size:   {}", download_stats.formatted_size());

    // Store directory stats
    if paths.store_dir.exists() {
        let store_size = calculate_dir_size(&paths.store_dir);
        println!();
        UI::info("Tool Store:");
        println!("  Location: {}", paths.store_dir.display());
        println!("  Total size: {}", format_size(store_size));
    }

    println!();
    UI::hint("Run 'vx cache prune' to remove expired entries");
    UI::hint("Run 'vx cache purge' to remove all cache (destructive)");

    Ok(())
}

/// List cached items
async fn handle_list(verbose: bool) -> Result<()> {
    let paths = VxPaths::new()?;
    // VersionCache::new expects the base cache dir and appends "versions_v2" internally
    let version_cache = VersionCache::new(paths.cache_dir.clone());
    let cache_dir = paths.cache_dir.join("versions_v2");

    UI::header("Cached Version Lists");

    if !cache_dir.exists() {
        UI::info("No version cache found");
        return Ok(());
    }

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        let mut found = false;
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            // Check for data files (.data or .jsonval) or metadata files (.meta)
            let is_cache_file = path
                .extension()
                .is_some_and(|ext| ext == "data" || ext == "jsonval" || ext == "meta");

            if is_cache_file {
                // Only show each tool once (use .meta files)
                if path.extension().is_some_and(|ext| ext == "meta") {
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
        }

        if !found {
            UI::info("No cached version lists");
        }
    }

    Ok(())
}

/// Prune expired and orphaned cache entries (safe cleanup)
async fn handle_prune(
    dry_run: bool,
    versions_only: bool,
    downloads_only: bool,
    resolutions_only: bool,
    orphaned_only: bool,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<()> {
    let paths = VxPaths::new()?;

    if dry_run {
        UI::header("Prune Preview (Dry Run)");
    } else {
        UI::header("Pruning Cache");
    }

    // Determine what to prune
    // If no selector flag is provided, prune all categories
    let any_selector = versions_only || downloads_only || resolutions_only || orphaned_only;
    let prune_versions = if any_selector { versions_only } else { true };
    let prune_downloads = if any_selector { downloads_only } else { true };
    let prune_resolutions = if any_selector { resolutions_only } else { true };
    let prune_orphaned = if any_selector { orphaned_only } else { true };

    let mut total_pruned = 0;

    // Prune version cache (expired entries only)
    if prune_versions {
        // VersionCache::new expects the base cache dir and appends "versions_v2" internally
        let version_cache = VersionCache::new(paths.cache_dir.clone());

        if let Ok(stats) = version_cache.stats() {
            if verbose || dry_run {
                UI::info(&format!(
                    "Version cache: {} expired of {} entries",
                    stats.expired_entries, stats.total_entries
                ));
            }
        }

        if dry_run {
            if let Ok(stats) = version_cache.stats() {
                if stats.expired_entries > 0 {
                    UI::hint(&format!(
                        "  Would prune {} expired entries",
                        stats.expired_entries
                    ));
                }
            }
        } else {
            let pruned = version_cache.prune()?;
            if pruned > 0 {
                UI::success(&format!("Pruned {} expired version cache entries", pruned));
                total_pruned += pruned;
            } else if verbose {
                UI::info("No expired version cache entries to prune");
            }
        }
    }

    // Prune resolution cache (expired entries only)
    if prune_resolutions {
        let cache_dir = paths.cache_dir.join(RESOLUTION_CACHE_DIR_NAME);
        let resolution_cache = ResolutionCache::new(cache_dir);

        let stats = resolution_cache.stats()?;
        if verbose || dry_run {
            UI::info(&format!(
                "Resolution cache: {} expired of {} entries",
                stats.expired_entries, stats.total_entries
            ));
        }

        if dry_run {
            if stats.expired_entries > 0 {
                UI::hint(&format!(
                    "  Would prune {} expired entries",
                    stats.expired_entries
                ));
            }
        } else {
            let pruned = resolution_cache.prune_expired()?;
            if pruned > 0 {
                UI::success(&format!(
                    "Pruned {} expired resolution cache entries",
                    pruned
                ));
                total_pruned += pruned;
            } else if verbose {
                UI::info("No expired resolution cache entries to prune");
            }
        }
    }

    // Prune download cache (old files based on older_than)
    if prune_downloads {
        let download_cache = DownloadCache::new(paths.cache_dir.clone());
        let stats = download_cache.stats();

        if verbose || dry_run {
            UI::info(&format!(
                "Download cache: {} files ({})",
                stats.file_count,
                stats.formatted_size()
            ));
        }

        if let Some(days) = older_than {
            if dry_run {
                UI::hint(&format!("  Would prune files older than {} days", days));
            } else {
                // Prune old download files
                let pruned = prune_old_downloads(&paths.cache_dir, days)?;
                if pruned > 0 {
                    UI::success(&format!("Pruned {} old download cache files", pruned));
                    total_pruned += pruned;
                } else if verbose {
                    UI::info("No old download cache files to prune");
                }
            }
        } else if verbose {
            UI::hint("Use --older-than to prune old download files");
        }
    }

    // Prune orphaned tool versions
    if prune_orphaned {
        if dry_run {
            UI::info("Orphaned tool versions:");
            UI::hint("  (orphaned version detection not yet implemented)");
        } else if verbose {
            UI::info("Orphaned version cleanup not yet fully implemented");
            UI::hint("Use 'vx uninstall <tool>@<version>' to remove specific versions");
        }
    }

    if !dry_run {
        if total_pruned > 0 {
            UI::success(&format!("Prune completed: {} items removed", total_pruned));
        } else {
            UI::info("Nothing to prune");
        }
    }

    Ok(())
}

/// Purge all cache data (destructive)
async fn handle_purge(
    versions_only: bool,
    downloads_only: bool,
    resolutions_only: bool,
    tool: Option<String>,
    yes: bool,
) -> Result<()> {
    let paths = VxPaths::new()?;

    // If specific tool is specified, only purge that tool's cache
    if let Some(tool_name) = tool {
        // Clear both .data and .jsonval files
        let version_cache = VersionCache::new(paths.cache_dir.clone());
        if version_cache.clear(&tool_name).is_ok() {
            UI::success(&format!("Purged cache for '{}'", tool_name));
        } else {
            UI::info(&format!("No cache found for '{}'", tool_name));
        }
        return Ok(());
    }

    // Determine what to purge
    let any_selector = versions_only || downloads_only || resolutions_only;
    let purge_versions = if any_selector { versions_only } else { true };
    let purge_downloads = if any_selector { downloads_only } else { true };
    let purge_resolutions = if any_selector { resolutions_only } else { true };

    // Confirmation
    if !yes {
        UI::warning("This will permanently remove all cache data!");
        let mut targets = vec![];
        if purge_versions {
            targets.push("version cache");
        }
        if purge_downloads {
            targets.push("download cache");
        }
        if purge_resolutions {
            targets.push("resolution cache");
        }
        UI::info(&format!("Targets: {}", targets.join(", ")));

        if !confirm_action()? {
            UI::info("Cancelled");
            return Ok(());
        }
    }

    UI::header("Purging Cache");

    // Purge version cache
    if purge_versions {
        // VersionCache::new expects the base cache dir and appends "versions_v2" internally
        let version_cache = VersionCache::new(paths.cache_dir.clone());
        version_cache.clear_all()?;
        UI::success("Version cache purged");
    }

    // Purge download cache
    if purge_downloads {
        let download_cache = DownloadCache::new(paths.cache_dir.clone());
        let stats_before = download_cache.stats();
        if stats_before.file_count > 0 {
            match download_cache.clear() {
                Ok(bytes_freed) => {
                    UI::success(&format!(
                        "Download cache purged: {} files ({})",
                        stats_before.file_count,
                        format_size(bytes_freed)
                    ));
                }
                Err(e) => {
                    UI::warning(&format!("Failed to purge download cache: {}", e));
                }
            }
        } else {
            UI::info("Download cache: (already empty)");
        }
    }

    // Purge resolution cache
    if purge_resolutions {
        let cache_dir = paths.cache_dir.join(RESOLUTION_CACHE_DIR_NAME);
        let resolution_cache = ResolutionCache::new(cache_dir);
        let removed = resolution_cache.clear_all()?;
        if removed > 0 {
            UI::success(&format!("Resolution cache purged: {} entries", removed));
        } else {
            UI::info("Resolution cache: (already empty)");
        }
    }

    // Always clear exec path cache on full purge
    if !any_selector {
        let _ = vx_cache::ExecPathCache::remove_file(&paths.cache_dir);
        vx_resolver::clear_bin_dir_cache();
        UI::success("Exec path cache cleared");
    }

    UI::success("Purge completed");
    Ok(())
}

/// Show cache directory path
async fn handle_dir() -> Result<()> {
    let paths = VxPaths::new()?;
    println!("{}", paths.cache_dir.display());
    Ok(())
}

/// Prune old download files
fn prune_old_downloads(cache_dir: &std::path::Path, days: u32) -> Result<usize> {
    let mut count = 0;
    let threshold = std::time::Duration::from_secs(days as u64 * 24 * 60 * 60);

    if let Ok(entries) = std::fs::read_dir(cache_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    if let Ok(modified) = meta.modified() {
                        let age = modified.elapsed().unwrap_or_default();
                        if age > threshold {
                            std::fs::remove_file(&path)?;
                            count += 1;
                        }
                    }
                }
            }
        }
    }

    Ok(count)
}

/// Ask for user confirmation
fn confirm_action() -> Result<bool> {
    use std::io::{self, Write};

    print!("Continue? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes"))
}

/// Calculate directory size recursively
fn calculate_dir_size(path: &std::path::Path) -> u64 {
    if !path.exists() {
        return 0;
    }

    let mut size = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() {
                    size += meta.len();
                }
            } else if path.is_dir() {
                size += calculate_dir_size(&path);
            }
        }
    }
    size
}
