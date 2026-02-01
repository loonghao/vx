//! Which command implementation - Find vx-managed tools

use crate::suggestions;
use crate::ui::UI;
use anyhow::Result;
use colored::Colorize;
use vx_paths::{PackageRegistry, PathManager, PathResolver, VxPaths};
use vx_runtime::ProviderRegistry;

pub async fn handle(
    registry: &ProviderRegistry,
    tool: &str,
    all: bool,
    use_system_path: bool,
) -> Result<()> {
    UI::debug(&format!("Looking for tool: {}", tool));

    // If --use-system-path is specified, only check system PATH
    if use_system_path {
        match which::which(tool) {
            Ok(path) => {
                println!("{}", path.display());
                return Ok(());
            }
            Err(_) => {
                UI::error(&format!("Tool '{}' not found in system PATH", tool));
                std::process::exit(1);
            }
        }
    }

    // Resolve canonical runtime name and executable (handles aliases like cl -> msvc)
    let runtime = registry.get_runtime(tool);
    let (canonical_name, exe_name) = runtime
        .as_ref()
        .map(|rt| (rt.name().to_string(), rt.executable_name().to_string()))
        .unwrap_or_else(|| (tool.to_string(), tool.to_string()));

    // Create path manager and resolver
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let resolver = PathResolver::new(path_manager);

    let locations = if all {
        // Find all versions
        resolver.find_tool_executables_with_exe(&canonical_name, &exe_name)?
    } else {
        // Find only the latest version
        match resolver.find_latest_executable_with_exe(&canonical_name, &exe_name)? {
            Some(path) => vec![path],
            None => vec![],
        }
    };

    if locations.is_empty() {
        // Not found in vx-managed runtimes, check global packages (RFC 0025)
        if let Some(exe_path) = find_in_global_packages(tool)? {
            println!("{} (global package)", exe_path.display());
            return Ok(());
        }

        // Also try with canonical executable name for aliases
        if exe_name != tool {
            if let Some(exe_path) = find_in_global_packages(&exe_name)? {
                println!("{} (global package)", exe_path.display());
                return Ok(());
            }
        }

        // Not found in global packages, check system PATH as fallback
        // Use executable name for system PATH search (handles aliases like imagemagick -> magick)
        match which::which(&exe_name) {
            Ok(path) => {
                // Found in system PATH
                println!("{} (system)", path.display());
                return Ok(());
            }
            Err(_) => {
                // Not found anywhere - show friendly error with suggestions
                let available_tools = registry.runtime_names();
                let tool_suggestions = suggestions::get_tool_suggestions(tool, &available_tools);

                // Use eprintln for all output to ensure consistent ordering
                eprintln!(
                    "{} {}",
                    "✗".red(),
                    format!(
                        "Tool '{}' not found in vx-managed installations or system PATH",
                        tool
                    )
                    .red()
                );

                if !tool_suggestions.is_empty() {
                    eprintln!();
                    for suggestion in &tool_suggestions {
                        if suggestion.is_alias {
                            eprintln!(
                                "{} Did you mean: {} ({})",
                                "💡".cyan(),
                                suggestion.suggested_tool.cyan().bold(),
                                suggestion.description.dimmed()
                            );
                        } else {
                            eprintln!(
                                "{} Did you mean: {}",
                                "💡".cyan(),
                                suggestion.suggested_tool.cyan().bold()
                            );
                        }
                    }
                }

                eprintln!();
                eprintln!(
                    "{} {}",
                    "💡".cyan(),
                    "Use 'vx list' to see installed tools".dimmed()
                );
                eprintln!(
                    "{} {}",
                    "💡".cyan(),
                    format!("Use 'vx install {}' to install this tool", tool).dimmed()
                );
                std::process::exit(1);
            }
        }
    }

    for location in locations {
        println!("{}", location.display());
    }

    Ok(())
}

/// Find an executable in globally installed packages (RFC 0025)
///
/// This function looks up the package registry to find if a tool
/// was installed via `vx global install`.
fn find_in_global_packages(exe_name: &str) -> Result<Option<std::path::PathBuf>> {
    let paths = VxPaths::new()?;
    let registry_path = paths.packages_registry_file();

    UI::debug(&format!(
        "Looking for '{}' in global packages, registry: {}",
        exe_name,
        registry_path.display()
    ));

    // Load the package registry
    let registry = match PackageRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(e) => {
            UI::debug(&format!("Failed to load registry: {}", e));
            return Ok(None);
        }
    };

    UI::debug(&format!("Registry has {} packages", registry.len()));

    // Find package by executable name
    if let Some(package) = registry.find_by_executable(exe_name) {
        UI::debug(&format!(
            "Found package '{}' for executable '{}', install_dir: {}",
            package.name,
            exe_name,
            package.install_dir.display()
        ));

        // Try various path patterns based on how npm packages install executables
        let candidates = vec![
            // npm pattern: install_dir/exe.cmd (Windows), install_dir/exe (Unix)
            #[cfg(windows)]
            package.install_dir.join(format!("{}.cmd", exe_name)),
            #[cfg(windows)]
            package.install_dir.join(format!("{}.ps1", exe_name)),
            #[cfg(windows)]
            package.install_dir.join(format!("{}.exe", exe_name)),
            package.install_dir.join(exe_name),
            // bin subdirectory pattern
            package
                .install_dir
                .join("bin")
                .join(format!("{}.cmd", exe_name)),
            package
                .install_dir
                .join("bin")
                .join(format!("{}.ps1", exe_name)),
            package
                .install_dir
                .join("bin")
                .join(format!("{}.exe", exe_name)),
            package.install_dir.join("bin").join(exe_name),
            // global_package_bin_dir pattern
            paths
                .global_package_bin_dir(&package.ecosystem, &package.name, &package.version)
                .join(format!("{}.cmd", exe_name)),
            paths
                .global_package_bin_dir(&package.ecosystem, &package.name, &package.version)
                .join(exe_name),
        ];

        for candidate in &candidates {
            UI::debug(&format!(
                "  Checking: {} (exists: {})",
                candidate.display(),
                candidate.exists()
            ));
            if candidate.exists() {
                return Ok(Some(candidate.clone()));
            }
        }
    } else {
        UI::debug(&format!("No package found for executable '{}'", exe_name));
    }

    Ok(None)
}
