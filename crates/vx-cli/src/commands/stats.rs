// Stats and cleanup command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_plugin::PluginRegistry;

pub async fn handle(registry: &PluginRegistry) -> Result<()> {
    show_all_stats(registry).await
}

pub async fn handle_cleanup(cache: bool, orphaned: bool, dry_run: bool) -> Result<()> {
    if dry_run {
        UI::header("Cleanup Preview (Dry Run)");
    } else {
        UI::header("Cleaning up...");
    }

    // TODO: Replace with vx-core executor
    // let mut executor = crate::executor::Executor::new()?;

    if cache || !orphaned {
        if dry_run {
            UI::info("Would clean cache directories");
        } else {
            UI::step("Cleaning cache...");
            UI::warning("Cache cleanup not yet implemented in new architecture");
            // TODO: Implement cache cleanup
        }
    }

    if orphaned || !cache {
        if dry_run {
            UI::info("Would clean orphaned packages");
        } else {
            UI::step("Cleaning orphaned packages...");
            UI::warning("Orphaned package cleanup not yet implemented in new architecture");
            // executor.cleanup()?;
        }
    }

    if !dry_run {
        UI::success("Cleanup completed");
    }

    Ok(())
}

#[allow(dead_code)]
async fn show_tool_stats(tool_name: &str, _detailed: bool) -> Result<()> {
    UI::header(&format!("Statistics for {tool_name}"));
    UI::warning("Tool stats not yet implemented in new architecture");

    // TODO: Replace with vx-core package manager
    // let package_manager = crate::package_manager::PackageManager::new()?;
    // let versions = package_manager.list_versions(tool_name);

    // if versions.is_empty() {
    //     UI::warning(&format!("Tool '{tool_name}' is not installed"));
    //     return Ok(());
    // }

    // let mut total_size = 0u64;
    // let version_count = versions.len();

    // if detailed {
    //     println!("Installed versions:");
    //     for version in &versions {
    //         if let Ok(path) = package_manager.get_version_path(tool_name, version) {
    //             let size = calculate_directory_size(&path).unwrap_or(0);
    //             total_size += size;
    //             println!("  {} - {} ({})", version, format_size(size), path.display());
    //         }
    //     }
    //     println!();
    // } else {
    //     for version in &versions {
    //         if let Ok(path) = package_manager.get_version_path(tool_name, version) {
    //             total_size += calculate_directory_size(&path).unwrap_or(0);
    //         }
    //     }
    // }

    // println!("Total versions: {version_count}");
    // println!("Total size: {}", format_size(total_size));

    Ok(())
}

async fn show_all_stats(registry: &PluginRegistry) -> Result<()> {
    let spinner = UI::new_spinner("Collecting package statistics...");

    // Collect statistics from the file system
    let stats = collect_stats().await?;

    spinner.finish_and_clear();

    // Display statistics
    UI::header("VX Statistics");
    println!();

    println!("📦 Installed Tools: {}", stats.total_tools);
    println!("🔢 Total Versions: {}", stats.total_versions);
    println!("💾 Total Size: {}", format_size(stats.total_size));
    println!("📁 Cache Size: {}", format_size(stats.cache_size));
    println!();

    if !stats.tools.is_empty() {
        UI::header("Tool Details");
        for tool_stat in &stats.tools {
            println!("  📦 {}", tool_stat.name);
            println!("     Versions: {}", tool_stat.version_count);
            println!("     Size: {}", format_size(tool_stat.total_size));
            if !tool_stat.versions.is_empty() {
                println!("     Installed: {}", tool_stat.versions.join(", "));
            }
            println!();
        }
    }

    // Show registry information
    println!("🔌 Available Tools: {}", registry.list_tools().len());
    for tool_name in registry.list_tools() {
        println!("  - {}", tool_name);
    }

    Ok(())
}

/// Statistics for a single tool
#[derive(Debug)]
struct ToolStats {
    name: String,
    version_count: usize,
    total_size: u64,
    versions: Vec<String>,
}

/// Overall VX statistics
#[derive(Debug)]
struct VxStats {
    total_tools: usize,
    total_versions: usize,
    total_size: u64,
    cache_size: u64,
    tools: Vec<ToolStats>,
}

/// Collect statistics from the file system
async fn collect_stats() -> Result<VxStats> {
    let paths = vx_paths::VxPaths::default();
    let mut stats = VxStats {
        total_tools: 0,
        total_versions: 0,
        total_size: 0,
        cache_size: 0,
        tools: Vec::new(),
    };

    // Calculate cache size
    if paths.cache_dir.exists() {
        stats.cache_size = calculate_directory_size(&paths.cache_dir)?;
    }

    // Scan tools directory
    if paths.tools_dir.exists() {
        for tool_entry in std::fs::read_dir(&paths.tools_dir)? {
            let tool_entry = tool_entry?;
            let tool_path = tool_entry.path();

            if tool_path.is_dir() {
                let tool_name = tool_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let mut tool_stats = ToolStats {
                    name: tool_name,
                    version_count: 0,
                    total_size: 0,
                    versions: Vec::new(),
                };

                // Scan version directories
                for version_entry in std::fs::read_dir(&tool_path)? {
                    let version_entry = version_entry?;
                    let version_path = version_entry.path();

                    if version_path.is_dir() {
                        let version_name = version_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let version_size = calculate_directory_size(&version_path)?;

                        tool_stats.versions.push(version_name);
                        tool_stats.total_size += version_size;
                        tool_stats.version_count += 1;
                        stats.total_versions += 1;
                    }
                }

                if tool_stats.version_count > 0 {
                    stats.total_size += tool_stats.total_size;
                    stats.tools.push(tool_stats);
                    stats.total_tools += 1;
                }
            }
        }
    }

    Ok(stats)
}

#[allow(dead_code)]
fn calculate_directory_size(path: &std::path::Path) -> Result<u64> {
    if path.is_file() {
        Ok(path.metadata()?.len())
    } else if path.is_dir() {
        let mut size = 0;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                size += entry.metadata()?.len();
            }
        }
        Ok(size)
    } else {
        Ok(0)
    }
}

#[allow(dead_code)]
fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
