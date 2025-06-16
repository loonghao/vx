// Cleanup command implementation

use crate::ui::UI;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use vx_core::{Result, VxEnvironment, VxError};

pub async fn handle(
    dry_run: bool,
    cache_only: bool,
    orphaned_only: bool,
    force: bool,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<()> {
    let spinner = UI::new_spinner("Scanning cleanup targets...");

    let env = VxEnvironment::new()?;
    let vx_home = VxEnvironment::get_vx_home()?;

    let mut cleanup_plan = CleanupPlan::new();

    // Scan for different types of cleanup targets
    if !orphaned_only {
        scan_cache_files(&mut cleanup_plan, &vx_home, older_than, verbose).await?;
        scan_temp_files(&mut cleanup_plan, &vx_home, older_than, verbose).await?;
    }

    if !cache_only {
        scan_orphaned_tools(&mut cleanup_plan, &env, older_than, verbose).await?;
    }

    spinner.finish_and_clear();

    if cleanup_plan.is_empty() {
        UI::success("✅ No cleanup needed - everything is clean!");
        return Ok(());
    }

    // Display cleanup plan
    display_cleanup_plan(&cleanup_plan, dry_run);

    if dry_run {
        UI::info("Run 'vx cleanup' to execute cleanup operations");
        return Ok(());
    }

    // Confirm cleanup unless forced
    if !force && !confirm_cleanup(&cleanup_plan)? {
        UI::info("Cleanup cancelled");
        return Ok(());
    }

    // Execute cleanup
    execute_cleanup(&cleanup_plan, verbose).await
}

#[derive(Debug, Default)]
struct CleanupPlan {
    cache_files: Vec<CleanupItem>,
    orphaned_tools: Vec<CleanupItem>,
    temp_files: Vec<CleanupItem>,
}

#[derive(Debug)]
struct CleanupItem {
    path: PathBuf,
    size: u64,
    description: String,
}

impl CleanupPlan {
    fn new() -> Self {
        Self::default()
    }

    fn is_empty(&self) -> bool {
        self.cache_files.is_empty() && self.orphaned_tools.is_empty() && self.temp_files.is_empty()
    }

    fn total_size(&self) -> u64 {
        self.cache_files.iter().map(|i| i.size).sum::<u64>()
            + self.orphaned_tools.iter().map(|i| i.size).sum::<u64>()
            + self.temp_files.iter().map(|i| i.size).sum::<u64>()
    }

    fn total_items(&self) -> usize {
        self.cache_files.len() + self.orphaned_tools.len() + self.temp_files.len()
    }
}

async fn scan_cache_files(
    plan: &mut CleanupPlan,
    vx_home: &Path,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<()> {
    let cache_dir = vx_home.join("cache");
    if !cache_dir.exists() {
        return Ok(());
    }

    if verbose {
        UI::info("Scanning cache files...");
    }

    // Scan downloads cache
    let downloads_dir = cache_dir.join("downloads");
    if downloads_dir.exists() {
        scan_directory(
            &mut plan.cache_files,
            &downloads_dir,
            "Download cache",
            older_than,
        )?;
    }

    // Scan versions cache
    let versions_dir = cache_dir.join("versions");
    if versions_dir.exists() {
        scan_directory(
            &mut plan.cache_files,
            &versions_dir,
            "Version cache",
            older_than,
        )?;
    }

    Ok(())
}

async fn scan_temp_files(
    plan: &mut CleanupPlan,
    vx_home: &Path,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<()> {
    let temp_dir = vx_home.join("tmp");
    if !temp_dir.exists() {
        return Ok(());
    }

    if verbose {
        UI::info("Scanning temporary files...");
    }

    scan_directory(
        &mut plan.temp_files,
        &temp_dir,
        "Temporary files",
        older_than,
    )?;
    Ok(())
}

async fn scan_orphaned_tools(
    plan: &mut CleanupPlan,
    env: &VxEnvironment,
    older_than: Option<u32>,
    verbose: bool,
) -> Result<()> {
    if verbose {
        UI::info("Scanning for orphaned tool versions...");
    }

    let tools_dir = env.get_base_install_dir();
    if !tools_dir.exists() {
        return Ok(());
    }

    // For now, just scan for old tool versions
    // TODO: Implement proper orphan detection by checking virtual environment references
    for entry in fs::read_dir(&tools_dir).map_err(|e| VxError::Other {
        message: format!("Failed to read tools directory: {}", e),
    })? {
        let entry = entry.map_err(|e| VxError::Other {
            message: format!("Failed to read directory entry: {}", e),
        })?;

        if entry
            .file_type()
            .map_err(|e| VxError::Other {
                message: format!("Failed to get file type: {}", e),
            })?
            .is_dir()
        {
            let tool_dir = entry.path();
            scan_tool_versions(plan, &tool_dir, older_than)?;
        }
    }

    Ok(())
}

fn scan_tool_versions(
    plan: &mut CleanupPlan,
    tool_dir: &Path,
    older_than: Option<u32>,
) -> Result<()> {
    let tool_name = tool_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    for entry in fs::read_dir(tool_dir).map_err(|e| VxError::Other {
        message: format!("Failed to read tool directory: {}", e),
    })? {
        let entry = entry.map_err(|e| VxError::Other {
            message: format!("Failed to read directory entry: {}", e),
        })?;

        if entry
            .file_type()
            .map_err(|e| VxError::Other {
                message: format!("Failed to get file type: {}", e),
            })?
            .is_dir()
        {
            let version_dir = entry.path();
            let version = version_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            if should_cleanup_path(&version_dir, older_than)? {
                let size = calculate_directory_size(&version_dir)?;
                plan.orphaned_tools.push(CleanupItem {
                    path: version_dir.clone(),
                    size,
                    description: format!("{}@{} (potentially orphaned)", tool_name, version),
                });
            }
        }
    }

    Ok(())
}

fn scan_directory(
    items: &mut Vec<CleanupItem>,
    dir: &Path,
    description: &str,
    older_than: Option<u32>,
) -> Result<()> {
    if should_cleanup_path(dir, older_than)? {
        let size = calculate_directory_size(dir)?;
        items.push(CleanupItem {
            path: dir.to_path_buf(),
            size,
            description: description.to_string(),
        });
    }

    Ok(())
}

fn should_cleanup_path(path: &Path, older_than: Option<u32>) -> Result<bool> {
    if let Some(days) = older_than {
        let metadata = fs::metadata(path).map_err(|e| VxError::Other {
            message: format!("Failed to get metadata for {}: {}", path.display(), e),
        })?;

        let modified = metadata.modified().map_err(|e| VxError::Other {
            message: format!("Failed to get modification time: {}", e),
        })?;

        let now = SystemTime::now();
        let age = now.duration_since(modified).map_err(|e| VxError::Other {
            message: format!("Failed to calculate file age: {}", e),
        })?;

        let max_age = std::time::Duration::from_secs(days as u64 * 24 * 60 * 60);
        Ok(age > max_age)
    } else {
        Ok(true)
    }
}

fn calculate_directory_size(dir: &Path) -> Result<u64> {
    let mut total_size = 0;

    if dir.is_file() {
        return Ok(fs::metadata(dir)
            .map_err(|e| VxError::Other {
                message: format!("Failed to get file metadata: {}", e),
            })?
            .len());
    }

    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry.map_err(|e| VxError::Other {
            message: format!("Failed to walk directory: {}", e),
        })?;

        if entry.file_type().is_file() {
            total_size += entry
                .metadata()
                .map_err(|e| VxError::Other {
                    message: format!("Failed to get metadata: {}", e),
                })?
                .len();
        }
    }

    Ok(total_size)
}

fn display_cleanup_plan(plan: &CleanupPlan, dry_run: bool) {
    if dry_run {
        UI::info("🔍 Cleanup plan preview:");
    } else {
        UI::info(&format!(
            "Will clean {} items ({}):",
            plan.total_items(),
            format_size(plan.total_size())
        ));
    }

    println!();

    if !plan.cache_files.is_empty() {
        println!("Cache files:");
        for item in &plan.cache_files {
            println!("  📁 {} ({})", item.description, format_size(item.size));
            if dry_run {
                println!("     {}", item.path.display());
            }
        }
        println!();
    }

    if !plan.orphaned_tools.is_empty() {
        println!("Orphaned tool versions:");
        for item in &plan.orphaned_tools {
            println!("  🗑️  {} ({})", item.description, format_size(item.size));
            if dry_run {
                println!("     {}", item.path.display());
            }
        }
        println!();
    }

    if !plan.temp_files.is_empty() {
        println!("Temporary files:");
        for item in &plan.temp_files {
            println!("  🗂️  {} ({})", item.description, format_size(item.size));
            if dry_run {
                println!("     {}", item.path.display());
            }
        }
        println!();
    }

    println!(
        "Total space to be freed: {}",
        format_size(plan.total_size())
    );
}

fn confirm_cleanup(plan: &CleanupPlan) -> Result<bool> {
    let items_text = if plan.total_items() == 1 {
        "1 item".to_string()
    } else {
        format!("{} items", plan.total_items())
    };

    print!(
        "Confirm cleanup of {} ({})? [y/N]: ",
        format_size(plan.total_size()),
        items_text
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    Ok(input.trim().to_lowercase().starts_with('y'))
}

async fn execute_cleanup(plan: &CleanupPlan, verbose: bool) -> Result<()> {
    UI::info("🧹 Executing cleanup...");

    let mut cleaned_size = 0u64;
    let mut cleaned_items = 0usize;

    // Clean cache files
    for item in &plan.cache_files {
        if verbose {
            UI::info(&format!("Cleaning {}", item.description));
        }

        if remove_path(&item.path)? {
            cleaned_size += item.size;
            cleaned_items += 1;
        }
    }

    // Clean orphaned tools
    for item in &plan.orphaned_tools {
        if verbose {
            UI::info(&format!("Cleaning {}", item.description));
        }

        if remove_path(&item.path)? {
            cleaned_size += item.size;
            cleaned_items += 1;
        }
    }

    // Clean temp files
    for item in &plan.temp_files {
        if verbose {
            UI::info(&format!("Cleaning {}", item.description));
        }

        if remove_path(&item.path)? {
            cleaned_size += item.size;
            cleaned_items += 1;
        }
    }

    UI::success(&format!(
        "✅ Cleanup completed! Freed {} from {} items",
        format_size(cleaned_size),
        cleaned_items
    ));

    Ok(())
}

fn remove_path(path: &Path) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|e| VxError::Other {
            message: format!("Failed to remove directory {}: {}", path.display(), e),
        })?;
    } else {
        fs::remove_file(path).map_err(|e| VxError::Other {
            message: format!("Failed to remove file {}: {}", path.display(), e),
        })?;
    }

    Ok(true)
}

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
