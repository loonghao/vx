// Where command implementation
// Shows the installation path of tools

use crate::ui::UI;
use anyhow::Result;
use std::path::PathBuf;

pub async fn handle(tool: String, all: bool) -> Result<()> {
    if all {
        show_all_installations(&tool).await
    } else {
        show_active_installation(&tool).await
    }
}

async fn show_active_installation(tool: &str) -> Result<()> {
    UI::header(&format!("Active installation of {}", tool));

    let mut found_vx_managed = false;

    // First check for vx-managed versions (environment isolation priority)
    let package_manager = crate::package_manager::PackageManager::new()?;
    let versions = package_manager.list_versions(tool);

    if !versions.is_empty() {
        // Check for active vx-managed version
        if let Some(active_version) = package_manager.get_active_version(tool) {
            found_vx_managed = true;
            println!("  vx-managed: {}", active_version.executable_path.display());
            println!("  Version: {}", active_version.version);
            println!("  Managed by: vx");

            // Try to get detailed version info
            if let Ok(output) = std::process::Command::new(&active_version.executable_path)
                .arg("--version")
                .output()
            {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    let first_line = version_output.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() && first_line != active_version.version {
                        println!("  Full version: {}", first_line);
                    }
                }
            }
        } else {
            // Has vx-managed versions but none active
            println!("  vx-managed versions available but none active:");
            for version in &versions {
                if let Ok(path) = package_manager.get_version_path(tool, version) {
                    if let Ok(exe_path) = find_executable(&path, tool) {
                        println!("    {} -> {}", version.version, exe_path.display());
                    }
                }
            }
            UI::hint(&format!(
                "Use 'vx switch {}@<version>' to activate a version",
                tool
            ));
        }
    }

    // Show system PATH info only if no vx-managed version is active
    if !found_vx_managed {
        if let Ok(system_path) = which::which(tool) {
            if is_vx_managed(&system_path) {
                println!("  vx-managed: {}", system_path.display());
                println!("  Managed by: vx");
            } else {
                println!("  System PATH: {}", system_path.display());
                println!("  Managed by: system");

                // Try to get version
                if let Ok(output) = std::process::Command::new(tool).arg("--version").output() {
                    if output.status.success() {
                        let version_output = String::from_utf8_lossy(&output.stdout);
                        let first_line = version_output.lines().next().unwrap_or("").trim();
                        if !first_line.is_empty() {
                            println!("  Version: {}", first_line);
                        }
                    }
                }

                UI::hint(&format!(
                    "Use 'vx install {}' to install a vx-managed version",
                    tool
                ));
            }
        } else if versions.is_empty() {
            UI::warning(&format!("Tool '{}' not found", tool));
            UI::hint(&format!("Install with: vx install {}", tool));
        }
    }

    Ok(())
}

async fn show_all_installations(tool: &str) -> Result<()> {
    UI::header(&format!("All installations of {}", tool));

    let mut found_any = false;

    // Check system PATH
    if let Ok(system_path) = which::which(tool) {
        found_any = true;
        println!("📍 System PATH:");
        println!("  {}", system_path.display());

        if is_vx_managed(&system_path) {
            println!("  (vx-managed)");
        } else {
            println!("  (system-managed)");
        }
        println!();
    }

    // Check vx-managed installations
    let package_manager = crate::package_manager::PackageManager::new()?;
    let versions = package_manager.list_versions(tool);

    if !versions.is_empty() {
        found_any = true;
        println!("📦 vx-managed installations:");

        for version in versions {
            if let Ok(path) = package_manager.get_version_path(tool, version) {
                match find_executable(&path, tool) {
                    Ok(exe_path) => {
                        println!("  {} -> {}", version, exe_path.display());

                        // Show additional info
                        if let Ok(metadata) = std::fs::metadata(&exe_path) {
                            let size = metadata.len();
                            let modified = metadata
                                .modified()
                                .map(|t| format!("{:?}", t))
                                .unwrap_or_else(|_| "unknown".to_string());
                            println!("    Size: {} bytes, Modified: {}", size, modified);
                        }
                    }
                    Err(_) => {
                        println!("  {} -> {} (executable not found)", version, path.display());
                    }
                }
            }
        }
        println!();
    }

    // Check common installation locations
    check_common_locations(tool, &mut found_any);

    if !found_any {
        UI::warning(&format!("No installations of '{}' found", tool));
        UI::hint(&format!("Install with: vx install {}", tool));
    }

    Ok(())
}

fn check_common_locations(tool: &str, found_any: &mut bool) {
    let common_paths = get_common_tool_paths(tool);
    let mut found_common = Vec::new();

    for path in common_paths {
        if path.exists() {
            found_common.push(path);
        }
    }

    if !found_common.is_empty() {
        *found_any = true;
        println!("🔍 Common installation locations:");

        for path in found_common {
            println!("  {}", path.display());

            // Try to get version
            if let Ok(output) = std::process::Command::new(&path).arg("--version").output() {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    let first_line = version_output.lines().next().unwrap_or("").trim();
                    if !first_line.is_empty() {
                        println!("    Version: {}", first_line);
                    }
                }
            }
        }
        println!();
    }
}

fn get_common_tool_paths(tool: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    // Windows common paths
    if cfg!(windows) {
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            paths.push(
                PathBuf::from(program_files)
                    .join(tool)
                    .join(format!("{}.exe", tool)),
            );
        }
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            paths.push(
                PathBuf::from(program_files_x86)
                    .join(tool)
                    .join(format!("{}.exe", tool)),
            );
        }
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            paths.push(
                PathBuf::from(local_app_data)
                    .join("Programs")
                    .join(tool)
                    .join(format!("{}.exe", tool)),
            );
        }
    }

    // Unix common paths
    if cfg!(unix) {
        paths.extend_from_slice(&[
            PathBuf::from("/usr/bin").join(tool),
            PathBuf::from("/usr/local/bin").join(tool),
            PathBuf::from("/opt").join(tool).join("bin").join(tool),
        ]);

        if let Some(home) = std::env::var_os("HOME") {
            paths.extend_from_slice(&[
                PathBuf::from(home.clone())
                    .join(".local")
                    .join("bin")
                    .join(tool),
                PathBuf::from(home).join("bin").join(tool),
            ]);
        }
    }

    // Tool-specific paths
    match tool {
        "node" => {
            if cfg!(windows) {
                if let Some(app_data) = std::env::var_os("APPDATA") {
                    paths.push(PathBuf::from(app_data).join("npm").join("node.exe"));
                }
            }
            paths.push(PathBuf::from("/usr/local/nodejs/bin/node"));
        }
        "go" => {
            paths.push(PathBuf::from("/usr/local/go/bin/go"));
            if let Some(home) = std::env::var_os("HOME") {
                paths.push(PathBuf::from(home).join("go").join("bin").join("go"));
            }
        }
        "rust" | "cargo" => {
            if let Some(home) = std::env::var_os("HOME") {
                paths.push(PathBuf::from(home).join(".cargo").join("bin").join(tool));
            }
        }
        _ => {}
    }

    paths
}

fn is_vx_managed(path: &std::path::Path) -> bool {
    // Check if the path is within vx's managed directories
    if let Some(cache_dir) = dirs::cache_dir() {
        let vx_dir = cache_dir.join("vx");
        path.starts_with(vx_dir)
    } else {
        false
    }
}

fn find_executable(dir: &std::path::Path, tool_name: &str) -> Result<PathBuf> {
    let exe_name = if cfg!(windows) {
        format!("{}.exe", tool_name)
    } else {
        tool_name.to_string()
    };

    // Search in the directory and subdirectories
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.file_name() == std::ffi::OsStr::new(&exe_name) {
            return Ok(entry.path().to_path_buf());
        }
    }

    Err(anyhow::anyhow!(
        "Executable {} not found in {}",
        exe_name,
        dir.display()
    ))
}
