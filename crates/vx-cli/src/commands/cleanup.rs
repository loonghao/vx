// Cleanup command implementation

use crate::ui::UI;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use vx_paths::VxPaths;

pub async fn handle(
    dry_run: bool,
    cache_only: bool,
    orphaned_only: bool,
    force: bool,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<()> {
    UI::info("🧹 Starting VX cleanup...");

    let paths = VxPaths::default();
    let mut total_freed = 0u64;
    let mut items_cleaned = 0usize;

    // Confirm before cleanup (unless force is true)
    if !force && !dry_run {
        let action = if cache_only {
            "clean cache files"
        } else if orphaned_only {
            "remove orphaned installations"
        } else {
            "perform full cleanup"
        };

        if !dialoguer::Confirm::new()
            .with_prompt(format!("Are you sure you want to {}?", action))
            .default(false)
            .interact()?
        {
            UI::info("Cleanup cancelled");
            return Ok(());
        }
    }

    // Clean cache if requested or doing full cleanup
    if cache_only || !orphaned_only {
        UI::info("🗂️  Cleaning cache...");
        let (cache_freed, cache_items) =
            cleanup_cache(&paths, dry_run, older_than, verbose).await?;
        total_freed += cache_freed;
        items_cleaned += cache_items;
    }

    // Clean orphaned installations if requested or doing full cleanup
    if orphaned_only || !cache_only {
        UI::info("🔍 Finding orphaned installations...");
        let (orphaned_freed, orphaned_items) = cleanup_orphaned(&paths, dry_run, verbose).await?;
        total_freed += orphaned_freed;
        items_cleaned += orphaned_items;
    }

    // Clean temporary files
    if !cache_only && !orphaned_only {
        UI::info("🗑️  Cleaning temporary files...");
        let (temp_freed, temp_items) =
            cleanup_temp_files(&paths, dry_run, older_than, verbose).await?;
        total_freed += temp_freed;
        items_cleaned += temp_items;
    }

    // Summary
    let action = if dry_run { "Would free" } else { "Freed" };
    let size_mb = total_freed as f64 / 1024.0 / 1024.0;

    if items_cleaned > 0 {
        UI::success(&format!(
            "✨ {} {:.2} MB by cleaning {} items",
            action, size_mb, items_cleaned
        ));
    } else {
        UI::info("✨ No cleanup needed - everything is already clean!");
    }

    Ok(())
}

/// Clean cache files
async fn cleanup_cache(
    paths: &VxPaths,
    dry_run: bool,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<(u64, usize)> {
    let cache_dir = &paths.cache_dir;
    if !cache_dir.exists() {
        if verbose {
            UI::info("Cache directory doesn't exist, skipping");
        }
        return Ok((0, 0));
    }

    let mut total_size = 0u64;
    let mut items_count = 0usize;

    // Calculate cutoff time if older_than is specified
    let cutoff_time =
        older_than.map(|days| SystemTime::now() - Duration::from_secs(days as u64 * 24 * 60 * 60));

    for entry in walkdir::WalkDir::new(cache_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let metadata = fs::metadata(path)?;
            let file_size = metadata.len();

            // Check if file is older than cutoff
            let should_delete = if let Some(cutoff) = cutoff_time {
                metadata.modified()? < cutoff
            } else {
                true
            };

            if should_delete {
                if verbose {
                    let size_kb = file_size as f64 / 1024.0;
                    UI::info(&format!(
                        "  {} {:.1} KB: {}",
                        if dry_run { "Would delete" } else { "Deleting" },
                        size_kb,
                        path.display()
                    ));
                }

                if !dry_run {
                    fs::remove_file(path)?;
                }

                total_size += file_size;
                items_count += 1;
            }
        }
    }

    Ok((total_size, items_count))
}

/// Clean orphaned tool installations
async fn cleanup_orphaned(paths: &VxPaths, dry_run: bool, verbose: bool) -> Result<(u64, usize)> {
    let tools_dir = &paths.tools_dir;
    if !tools_dir.exists() {
        if verbose {
            UI::info("Tools directory doesn't exist, skipping");
        }
        return Ok((0, 0));
    }

    let mut total_size = 0u64;
    let mut items_count = 0usize;

    // Scan for tool directories
    for tool_entry in fs::read_dir(tools_dir)? {
        let tool_entry = tool_entry?;
        let tool_path = tool_entry.path();

        if tool_path.is_dir() {
            let _tool_name = tool_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // Scan for version directories
            for version_entry in fs::read_dir(&tool_path)? {
                let version_entry = version_entry?;
                let version_path = version_entry.path();

                if version_path.is_dir() {
                    // Check if this version directory is orphaned
                    // (for now, we'll consider empty directories as orphaned)
                    let is_orphaned = is_directory_orphaned(&version_path)?;

                    if is_orphaned {
                        let dir_size = calculate_directory_size(&version_path)?;

                        if verbose {
                            let size_mb = dir_size as f64 / 1024.0 / 1024.0;
                            UI::info(&format!(
                                "  {} {:.1} MB: {} (orphaned)",
                                if dry_run { "Would remove" } else { "Removing" },
                                size_mb,
                                version_path.display()
                            ));
                        }

                        if !dry_run {
                            fs::remove_dir_all(&version_path)?;
                        }

                        total_size += dir_size;
                        items_count += 1;
                    }
                }
            }
        }
    }

    Ok((total_size, items_count))
}

/// Clean temporary files
async fn cleanup_temp_files(
    paths: &VxPaths,
    dry_run: bool,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<(u64, usize)> {
    let temp_dir = &paths.tmp_dir;
    if !temp_dir.exists() {
        if verbose {
            UI::info("Temp directory doesn't exist, skipping");
        }
        return Ok((0, 0));
    }

    let mut total_size = 0u64;
    let mut items_count = 0usize;

    // Calculate cutoff time (default to 7 days for temp files)
    let cutoff_days = older_than.unwrap_or(7);
    let cutoff_time = SystemTime::now() - Duration::from_secs(cutoff_days as u64 * 24 * 60 * 60);

    for entry in walkdir::WalkDir::new(temp_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let metadata = fs::metadata(path)?;
            let file_size = metadata.len();

            // Check if file is older than cutoff
            if metadata.modified()? < cutoff_time {
                if verbose {
                    let size_kb = file_size as f64 / 1024.0;
                    UI::info(&format!(
                        "  {} {:.1} KB: {}",
                        if dry_run { "Would delete" } else { "Deleting" },
                        size_kb,
                        path.display()
                    ));
                }

                if !dry_run {
                    fs::remove_file(path)?;
                }

                total_size += file_size;
                items_count += 1;
            }
        }
    }

    Ok((total_size, items_count))
}

/// Check if a directory is orphaned (empty or contains only empty subdirectories)
fn is_directory_orphaned(path: &PathBuf) -> Result<bool> {
    let entries: Vec<_> = fs::read_dir(path)?.collect::<Result<Vec<_>, _>>()?;

    if entries.is_empty() {
        return Ok(true);
    }

    // Check if all entries are empty directories
    for entry in entries {
        let entry_path = entry.path();
        if entry_path.is_file() {
            return Ok(false); // Has files, not orphaned
        }
        if entry_path.is_dir() && !is_directory_orphaned(&entry_path)? {
            return Ok(false); // Has non-empty subdirectories
        }
    }

    Ok(true) // All subdirectories are empty
}

/// Calculate the total size of a directory
fn calculate_directory_size(path: &PathBuf) -> Result<u64> {
    let mut total_size = 0u64;

    for entry in walkdir::WalkDir::new(path) {
        let entry = entry?;
        if entry.path().is_file() {
            total_size += fs::metadata(entry.path())?.len();
        }
    }

    Ok(total_size)
}
