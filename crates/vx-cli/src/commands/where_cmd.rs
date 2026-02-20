//! Which command implementation - Find vx-managed tools (RFC 0031, RFC 0035: unified structured output)

use crate::cli::OutputFormat;
use crate::output::{OutputRenderer, ToolPathEntry, ToolSource, WhichOutput};
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
    format: OutputFormat,
) -> Result<()> {
    UI::debug(&format!("Looking for tool: {}@{:?}", tool, version));

    // Parse runtime::executable syntax (e.g., msvc::cl)
    let (runtime_part, exe_override) = if let Some((rt, exe)) = tool.split_once("::") {
        (rt, Some(exe))
    } else {
        (tool, None)
    };

    // If --use-system-path is specified, only check system PATH
    if use_system_path {
        let search_name = exe_override.unwrap_or(runtime_part);
        match which::which(search_name) {
            Ok(path) => {
                let output = WhichOutput {
                    tool: tool.to_string(),
                    version: None,
                    path: Some(path.display().to_string()),
                    // Use Vx source to avoid showing "(system)" suffix —
                    // the user explicitly requested system path lookup,
                    // so the result is the expected output without extra annotation.
                    source: ToolSource::Vx,
                    all_paths: vec![],
                };
                let renderer = OutputRenderer::new(format);
                renderer.render(&output)?;
                return Ok(());
            }
            Err(_) => {
                let output = WhichOutput {
                    tool: tool.to_string(),
                    version: None,
                    path: None,
                    source: ToolSource::NotFound,
                    all_paths: vec![],
                };
                let renderer = OutputRenderer::new(format);
                renderer.render(&output)?;
                std::process::exit(1);
            }
        }
    }

    // Resolve canonical runtime name and executable (handles aliases like imagemagick -> magick)
    let (canonical_name, exe_name) = if let Some(exe) = exe_override {
        let rt_name = if let Some(runtime) = registry.get_runtime(runtime_part) {
            runtime.name().to_string()
        } else {
            runtime_part.to_string()
        };
        UI::debug(&format!(
            "Parsed '{}' as runtime='{}', exe='{}'",
            tool, rt_name, exe
        ));
        (rt_name, exe.to_string())
    } else if let Some(runtime) = registry.get_runtime(runtime_part) {
        let canonical = runtime.name().to_string();
        // Use executable_name() which may differ from the runtime name
        // e.g. runtime name "7zip" has executable "7z"
        let exe = runtime.executable_name().to_string();
        UI::debug(&format!(
            "Resolved '{}' to canonical='{}', exe='{}'",
            tool, canonical, exe
        ));
        (canonical, exe)
    } else {
        (runtime_part.to_string(), runtime_part.to_string())
    };

    // Try RuntimeIndex first for fast lookup
    let mut runtime_index = RuntimeIndex::new().ok();

    // Build index if it doesn't exist or is invalid
    if let Some(ref mut index) = runtime_index
        && !index.is_valid()
    {
        UI::debug("Runtime index missing or expired, building...");
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

    let index_lookup = runtime_index.as_mut().and_then(|index| {
        if let Some(ver) = version {
            index
                .get(&canonical_name)
                .and_then(|entry| entry.get_executable_path(&VxPaths::new().ok()?.store_dir, ver))
        } else if all {
            None
        } else {
            index.get_executable_path(&canonical_name)
        }
    });

    let locations = if let Some(path) = index_lookup {
        UI::debug(&format!("Found in runtime index: {}", path.display()));
        vec![(path, ToolSource::Vx)]
    } else {
        UI::debug("Not in runtime index, scanning file system...");

        let path_manager = PathManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
        let resolver = PathResolver::new(path_manager);

        if let Some(ver) = version {
            match resolver.find_tool_version_with_executable(&canonical_name, ver, &exe_name) {
                Some(location) => vec![(location.path, ToolSource::Vx)],
                None => vec![],
            }
        } else if all {
            resolver
                .find_tool_executables_with_exe(&canonical_name, &exe_name)?
                .into_iter()
                .map(|p| (p, ToolSource::Vx))
                .collect()
        } else {
            match resolver.find_latest_executable_with_exe(&canonical_name, &exe_name)? {
                Some(path) => vec![(path, ToolSource::Vx)],
                None => vec![],
            }
        }
    };

    // Check global packages and system PATH if not found in vx
    let (final_path, final_source, all_paths) = if locations.is_empty() {
        // Check global packages (RFC 0025)
        if let Some(exe_path) = find_in_global_packages(tool)? {
            if all {
                (
                    None,
                    ToolSource::NotFound,
                    vec![ToolPathEntry {
                        path: exe_path.display().to_string(),
                        version: None,
                        source: ToolSource::GlobalPackage,
                    }],
                )
            } else {
                (
                    Some(exe_path.display().to_string()),
                    ToolSource::GlobalPackage,
                    vec![],
                )
            }
        } else if exe_name != tool
            && let Some(exe_path) = find_in_global_packages(&exe_name)?
        {
            if all {
                (
                    None,
                    ToolSource::NotFound,
                    vec![ToolPathEntry {
                        path: exe_path.display().to_string(),
                        version: None,
                        source: ToolSource::GlobalPackage,
                    }],
                )
            } else {
                (
                    Some(exe_path.display().to_string()),
                    ToolSource::GlobalPackage,
                    vec![],
                )
            }
        } else {
            // Check system PATH
            match which::which(&exe_name) {
                Ok(path) => {
                    if all {
                        (
                            None,
                            ToolSource::NotFound,
                            vec![ToolPathEntry {
                                path: path.display().to_string(),
                                version: None,
                                source: ToolSource::System,
                            }],
                        )
                    } else {
                        (Some(path.display().to_string()), ToolSource::System, vec![])
                    }
                }
                Err(_) => {
                    // Try detection system_paths
                    if let Some(path) =
                        find_via_detection_paths(&canonical_name, &exe_name, registry)?
                    {
                        if all {
                            (
                                None,
                                ToolSource::NotFound,
                                vec![ToolPathEntry {
                                    path: path.display().to_string(),
                                    version: None,
                                    source: ToolSource::Detected,
                                }],
                            )
                        } else {
                            (
                                Some(path.display().to_string()),
                                ToolSource::Detected,
                                vec![],
                            )
                        }
                    } else {
                        (None, ToolSource::NotFound, vec![])
                    }
                }
            }
        }
    } else if all {
        let all_paths: Vec<ToolPathEntry> = locations
            .iter()
            .map(|(path, source)| ToolPathEntry {
                path: path.display().to_string(),
                version: extract_version_from_path(path),
                source: *source,
            })
            .collect();
        (None, ToolSource::NotFound, all_paths)
    } else {
        let (path, source) = &locations[0];
        (Some(path.display().to_string()), *source, vec![])
    };

    let renderer = OutputRenderer::new(format);

    // Handle not found case
    if final_path.is_none() && all_paths.is_empty() {
        let output = WhichOutput {
            tool: tool.to_string(),
            version: None,
            path: None,
            source: ToolSource::NotFound,
            all_paths: vec![],
        };

        if renderer.is_json() {
            renderer.render(&output)?;
        } else {
            // Show friendly error with suggestions
            let available_tools = registry.runtime_names();
            let tool_suggestions = suggestions::get_tool_suggestions(tool, &available_tools);

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
        }
        std::process::exit(1);
    }

    let output = WhichOutput {
        tool: tool.to_string(),
        version: final_path
            .as_ref()
            .and_then(|p| extract_version_from_path_str(p)),
        path: final_path,
        source: final_source,
        all_paths,
    };

    renderer.render(&output)?;

    Ok(())
}

/// Extract version from path string
fn extract_version_from_path_str(path_str: &str) -> Option<String> {
    let path = std::path::Path::new(path_str);
    extract_version_from_path(path)
}

/// Extract version from executable path
fn extract_version_from_path(path: &std::path::Path) -> Option<String> {
    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str())
            && name.chars().any(|c| c.is_ascii_digit())
            && (name.contains('.') || name.chars().all(|c| c.is_ascii_digit()))
            && !name.contains('-')
        {
            return Some(name.to_string());
        }
    }
    None
}

/// Find an executable in globally installed packages (RFC 0025)
fn find_in_global_packages(exe_name: &str) -> Result<Option<std::path::PathBuf>> {
    let paths = VxPaths::new()?;
    let registry_path = paths.packages_registry_file();

    UI::debug(&format!(
        "Looking for '{}' in global packages, registry: {}",
        exe_name,
        registry_path.display()
    ));

    let registry = match PackageRegistry::load(&registry_path) {
        Ok(r) => r,
        Err(e) => {
            UI::debug(&format!("Failed to load registry: {}", e));
            return Ok(None);
        }
    };

    UI::debug(&format!("Registry has {} packages", registry.len()));

    if let Some(package) = registry.find_by_executable(exe_name) {
        UI::debug(&format!(
            "Found package '{}' for executable '{}', install_dir: {}",
            package.name,
            exe_name,
            package.install_dir.display()
        ));

        let candidates = vec![
            #[cfg(windows)]
            package.install_dir.join(format!("{}.cmd", exe_name)),
            #[cfg(windows)]
            package.install_dir.join(format!("{}.ps1", exe_name)),
            #[cfg(windows)]
            package.install_dir.join(format!("{}.exe", exe_name)),
            package.install_dir.join(exe_name),
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

/// Find an executable via detection system_paths (glob patterns from provider.toml)
fn find_via_detection_paths(
    runtime_name: &str,
    _exe_name: &str,
    _registry: &ProviderRegistry,
) -> Result<Option<std::path::PathBuf>> {
    let mut loader = ManifestLoader::new();
    let manifests_data: Vec<(&str, &str)> = get_embedded_manifests().to_vec();
    if loader.load_embedded(manifests_data).is_err() {
        return Ok(None);
    }

    for manifest in loader.all() {
        for runtime_def in &manifest.runtimes {
            let matches = runtime_def.name == runtime_name
                || runtime_def
                    .aliases
                    .iter()
                    .any(|alias| alias == runtime_name);

            if matches && let Some(ref detection) = runtime_def.detection {
                for pattern in &detection.system_paths {
                    if let Ok(paths) = glob::glob(pattern) {
                        let mut matches: Vec<std::path::PathBuf> = paths
                            .filter_map(|p| p.ok())
                            .filter(|p| p.exists())
                            .collect();
                        matches.sort_by(|a, b| b.cmp(a));

                        if let Some(path) = matches.into_iter().next() {
                            return Ok(Some(path));
                        }
                    }
                }
            }
        }
    }

    Ok(None)
}
