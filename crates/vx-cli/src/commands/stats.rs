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

async fn show_all_stats(_registry: &PluginRegistry) -> Result<()> {
    let spinner = UI::new_spinner("Collecting package statistics...");
    UI::warning("Package statistics not yet implemented in new architecture");

    // TODO: Replace with vx-core executor
    // let mut executor = crate::executor::Executor::new()?;
    // let stats = executor.get_stats()?;
    spinner.finish_and_clear();

    // UI::show_stats(
    //     stats.total_packages,
    //     stats.total_versions,
    //     stats.total_size,
    //     &stats
    //         .last_updated
    //         .format("%Y-%m-%d %H:%M:%S UTC")
    //         .to_string(),
    // );

    // List installed packages
    // if let Ok(packages) = executor.list_installed_packages() {
    //     if !packages.is_empty() {
    //         if detailed {
    //             println!();
    //             UI::header("Installed Packages");
    //             for package in &packages {
    //                 println!(
    //                     "  {} {} - {}",
    //                     package.name, package.version, &package.metadata.description
    //                 );
    //             }
    //         } else {
    //             // Create a simple list without active status for now
    //             let package_list: Vec<(String, String, bool)> = packages
    //                 .iter()
    //                 .map(|package| {
    //                     // For now, mark all as inactive to avoid borrowing issues
    //                     // TODO: Improve this to show actual active status
    //                     (package.name.clone(), package.version.clone(), false)
    //                 })
    //                 .collect();

    //             println!();
    //             UI::show_package_list(&package_list);
    //         }
    //     }
    // }

    Ok(())
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
