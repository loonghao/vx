// Stats and cleanup command implementation

use crate::ui::UI;
use anyhow::Result;
use vx_runtime::ProviderRegistry;

pub async fn handle(registry: &ProviderRegistry) -> Result<()> {
    show_all_stats(registry).await
}

pub async fn handle_cleanup(cache: bool, orphaned: bool, dry_run: bool) -> Result<()> {
    if dry_run {
        UI::header("Cleanup Preview (Dry Run)");
    } else {
        UI::header("Cleaning up...");
    }

    if cache || !orphaned {
        if dry_run {
            UI::info("Would clean cache directories");
        } else {
            UI::step("Cleaning cache...");
            UI::warning("Cache cleanup not yet implemented in new architecture");
        }
    }

    if orphaned || !cache {
        if dry_run {
            UI::info("Would clean orphaned packages");
        } else {
            UI::step("Cleaning orphaned packages...");
            UI::warning("Orphaned package cleanup not yet implemented in new architecture");
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
    Ok(())
}

async fn show_all_stats(_registry: &ProviderRegistry) -> Result<()> {
    let spinner = UI::new_spinner("Collecting package statistics...");
    UI::warning("Package statistics not yet implemented in new architecture");
    spinner.finish_and_clear();
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
