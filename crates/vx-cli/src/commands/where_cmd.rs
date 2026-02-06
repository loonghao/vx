//! Which command implementation - Find vx-managed tools

use crate::registry::get_embedded_manifests;
use crate::suggestions;
use crate::ui::UI;
use anyhow::Result;
use colored::Colorize;
use vx_manifest::ManifestLoader;
use vx_paths::{PackageRegistry, PathManager, PathResolver, VxPaths};
use vx_resolver::RuntimeIndex;
use vx_runtime::ProviderRegistry;

pub async fn handle(
    registry: &ProviderRegistry,
    tool: &str,
    version: Option<&str>,
    all: bool,
    use_system_path: bool,
) -> Result<()> {
    UI::debug(&format!("Looking for tool: {}@{:?}", tool, version));

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

    // Resolve canonical runtime name and executable (handles aliases like imagemagick -> magick)
    // Try to find the runtime in the registry - this handles alias resolution
    let (canonical_name, exe_name) = if let Some(runtime) = registry.get_runtime(tool) {
        // Found runtime (possibly via alias), use its canonical name
        let canonical = runtime.name().to_string();
        // The canonical name is also the executable name
        let exe = canonical.clone();
        UI::debug(&format!(
            "Resolved '{}' to canonical='{}', exe='{}'",
            tool, canonical, exe
        ));
        (canonical, exe)
    } else {
        // Not found in registry, use the tool name directly
        (tool.to_string(), tool.to_string())
    };

    // Try RuntimeIndex first for fast lookup
    let mut runtime_index = RuntimeIndex::new().ok();

    // Build index if it doesn't exist or is invalid
    if let Some(ref mut index) = runtime_index {
        if !index.is_valid() {
            UI::debug("Runtime index missing or expired, building...");
            // Load manifests from embedded data
            let mut loader = ManifestLoader::new();
            let manifests_data: Vec<(&str, &str)> = get_embedded_manifests().to_vec();
            if loader.load_embedded(manifests_data).is_ok() {
                let manifests: Vec<_> = loader.all().cloned().collect();
                if let Err(e) = index.build_and_save(&manifests) {
                    UI::debug(&format!("Failed to build runtime index: {}", e));
                } else {
                    UI::debug(&format!(
                        "Built runtime index with {} manifests",
                        manifests.len()
                    ));
                }
            }
        }
    }

    let index_lookup = runtime_index.as_mut().and_then(|index| {
        if let Some(ver) = version {
            // Specific version requested
            index
                .get(&canonical_name)
                .and_then(|entry| entry.get_executable_path(&VxPaths::new().ok()?.store_dir, ver))
        } else if all {
            // All versions - we'll use PathResolver for this
            None
        } else {
            // Latest version
            index.get_executable_path(&canonical_name)
        }
    });

    let locations = if let Some(path) = index_lookup {
        // Fast path: found in RuntimeIndex
        UI::debug(&format!("Found in runtime index: {}", path.display()));
        vec![path]
    } else {
        // Slow path: use PathResolver to scan file system
        UI::debug("Not in runtime index, scanning file system...");

        // Create path manager and resolver
        let path_manager = PathManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
        let resolver = PathResolver::new(path_manager);

        if let Some(ver) = version {
            // Specific version requested (e.g., yarn@4)
            match resolver.find_tool_version_with_executable(&canonical_name, ver, &exe_name) {
                Some(location) => vec![location.path],
                None => vec![],
            }
        } else if all {
            // Find all versions
            resolver.find_tool_executables_with_exe(&canonical_name, &exe_name)?
        } else {
            // Find only the latest version
            match resolver.find_latest_executable_with_exe(&canonical_name, &exe_name)? {
                Some(path) => vec![path],
                None => vec![],
            }
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
