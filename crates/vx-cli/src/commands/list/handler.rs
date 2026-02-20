//! List command handler
//!
//! RFC-0037: installed version queries now delegate to ProviderHandle when available.

use super::Args;
use crate::cli::OutputFormat;
use crate::commands::CommandContext;
use crate::output::{ListOutput, OutputRenderer, RuntimeEntry, VersionEntry, VersionsOutput};
use crate::registry::get_runtime_platform_label;
use crate::system_tools::{discover_system_tools, group_by_category};
use crate::ui::UI;
use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use vx_paths::{PathManager, PathResolver};
use vx_runtime::{Platform, ProviderRegistry, Runtime, RuntimeContext};
use vx_starlark::handle::global_registry;

/// Check if a runtime supports the given platform
fn is_platform_supported(runtime: &Arc<dyn Runtime>, platform: &Platform) -> bool {
    runtime.as_ref().is_platform_supported(platform)
}

/// Handle list command with Args
pub async fn handle(ctx: &CommandContext, args: &Args) -> Result<()> {
    handle_list(
        ctx.registry(),
        ctx.runtime_context(),
        args.tool.as_deref(),
        args.status,
        args.all,
        args.system,
        ctx.output_format(),
    )
    .await
}

/// Legacy handle function for backwards compatibility
pub async fn handle_list(
    registry: &ProviderRegistry,
    _context: &RuntimeContext,
    tool: Option<&str>,
    show_status: bool,
    show_all: bool,
    show_system: bool,
    format: OutputFormat,
) -> Result<()> {
    // Create path manager and resolver
    let path_manager = PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let resolver = PathResolver::new(path_manager);

    if show_system {
        // Show system tools
        list_system_tools(registry, show_all, format).await?;
        return Ok(());
    }

    match tool {
        Some(tool_name) => {
            // List versions for a specific tool (always show regardless of platform)
            list_tool_versions(registry, &resolver, tool_name, show_status, format).await?;
        }
        None => {
            // List all tools with optional platform filtering
            list_all_tools(registry, &resolver, show_status, show_all, format).await?;
        }
    }
    Ok(())
}

/// List system tools discovered from PATH and known locations
async fn list_system_tools(
    registry: &ProviderRegistry,
    show_all: bool,
    format: OutputFormat,
) -> Result<()> {
    let current_platform = Platform::current();
    let discovery = discover_system_tools(registry);

    let renderer = OutputRenderer::new(format);

    if renderer.is_json() {
        // JSON output
        let mut runtimes = Vec::new();

        // Group available tools by category (unused but kept for future use)
        let _grouped = group_by_category(&discovery.available);

        for tool in &discovery.available {
            runtimes.push(RuntimeEntry {
                name: tool.name.clone(),
                versions: tool.version.clone().map(|v| vec![v]).unwrap_or_default(),
                installed: true,
                description: tool.description.clone(),
                platform_supported: true,
                ecosystem: Some(tool.category.clone()),
                platform_label: None,
            });
        }

        let output = ListOutput {
            runtimes,
            total: discovery.available.len(),
            installed_count: discovery.available.len(),
            platform: current_platform.as_str().to_string(),
        };

        renderer.render(&output)?;
    } else {
        // Text output
        UI::info(&format!("🔧 System Tools ({})", current_platform.as_str()));
        println!();

        // Group available tools by category
        let grouped = group_by_category(&discovery.available);

        // Define category order
        let category_order = [
            "build",
            "compiler",
            "vcs",
            "container",
            "cloud",
            "network",
            "security",
            "package",
            "system",
            "archive",
            "filesystem",
            "mlops",
            "other",
        ];

        for category in category_order {
            if let Some(tools) = grouped.get(category)
                && !tools.is_empty()
            {
                println!("  {}:", capitalize_category(category));
                for tool in tools {
                    let path_str = tool
                        .path
                        .as_ref()
                        .map(|p| format!(" @ {}", p.display()))
                        .unwrap_or_default();
                    let version_str = tool
                        .version
                        .as_ref()
                        .map(|v| format!(" ({})", v))
                        .unwrap_or_default();
                    println!(
                        "    ✅ {}{} - {}{}",
                        tool.name, version_str, tool.description, path_str
                    );
                }
            }
        }

        if discovery.available.is_empty() {
            UI::hint("  No system tools discovered");
        }

        // Show unavailable tools if --all is specified
        if show_all && !discovery.unavailable.is_empty() {
            println!();
            UI::info("⚠️  Unavailable on this platform:");
            for tool in &discovery.unavailable {
                let platform_str = tool
                    .platform
                    .as_ref()
                    .map(|p| format!(" ({} only)", p))
                    .unwrap_or_default();
                println!(
                    "    ❌ {} - {}{}",
                    tool.name, tool.description, platform_str
                );
            }
        }

        // Summary
        println!();
        UI::info(&format!(
            "📊 Summary: {} system tools available",
            discovery.available.len()
        ));

        if !show_all && !discovery.unavailable.is_empty() {
            UI::hint(&format!(
                "   {} tools unavailable on {}. Use --all to show all.",
                discovery.unavailable.len(),
                current_platform.as_str()
            ));
        }
    }

    Ok(())
}

/// Capitalize category name for display
fn capitalize_category(category: &str) -> String {
    match category {
        "build" => "Build Tools",
        "compiler" => "Compilers",
        "vcs" => "Version Control",
        "container" => "Container & Orchestration",
        "cloud" => "Cloud CLI",
        "network" => "Network Tools",
        "security" => "Security",
        "package" => "Package Managers",
        "system" => "System Tools",
        "archive" => "Archive Tools",
        "filesystem" => "Filesystem Tools",
        "mlops" => "ML/AI Tools",
        "other" => "Other",
        _ => category,
    }
    .to_string()
}

async fn list_tool_versions(
    registry: &ProviderRegistry,
    resolver: &PathResolver,
    tool_name: &str,
    show_status: bool,
    format: OutputFormat,
) -> Result<()> {
    // Check if tool is supported
    let runtime = registry.get_runtime(tool_name);
    if runtime.is_none() {
        // Show friendly error with suggestions
        let available_tools = registry.runtime_names();
        UI::tool_not_found(tool_name, &available_tools);
        return Ok(());
    }

    let runtime = runtime.unwrap();
    let current_platform = Platform::current();
    let platform_supported = is_platform_supported(&runtime, &current_platform);

    // Use canonical runtime name for store lookup and executable name for file search
    let canonical_name = runtime.name();
    let exe_name = runtime.executable_name();

    let renderer = OutputRenderer::new(format);

    // Check if this tool is bundled with another tool
    let bundled_with = runtime.metadata().get("bundled_with").cloned();

    // --- RFC-0037: Try ProviderHandle first for installed versions ---
    let handle_installed: Option<Vec<String>> = {
        let reg = global_registry().await;
        reg.get(canonical_name)
            .map(|handle| handle.installed_versions())
    };

    // Get installed versions - prefer ProviderHandle, fall back to PathResolver
    let installed_executables: Vec<std::path::PathBuf> =
        if let Some(versions) = handle_installed.filter(|v| !v.is_empty()) {
            UI::debug(&format!(
                "ProviderHandle::installed_versions({}) => {:?}",
                canonical_name, versions
            ));
            // Convert version strings to paths for backward-compatible display
            versions
                .iter()
                .filter_map(|v| {
                    // Build the expected executable path from the store
                    let paths = vx_paths::VxPaths::new().ok()?;
                    let version_dir = paths.store_dir.join(canonical_name).join(v);
                    // Try common locations
                    let exe_with_ext = vx_paths::with_executable_extension(exe_name);
                    let candidates = vec![
                        version_dir.join(&exe_with_ext),
                        version_dir.join(exe_name),
                        version_dir.join("bin").join(&exe_with_ext),
                        version_dir.join("bin").join(exe_name),
                    ];
                    candidates
                        .into_iter()
                        .find(|p| p.exists())
                        .or_else(|| Some(version_dir.join(&exe_with_ext)))
                })
                .collect()
        } else {
            resolver.find_tool_executables_with_exe(canonical_name, exe_name)?
        };

    // If this tool is bundled with another and has no direct installations,
    // check the parent tool's installations
    if installed_executables.is_empty()
        && let Some(parent_tool) = &bundled_with
    {
        let parent_executables = resolver.find_tool_executables(parent_tool)?;
        if !parent_executables.is_empty() {
            // Tool is available via parent
            let mut versions = Vec::new();
            for exe_path in &parent_executables {
                let version = extract_version_from_path(exe_path);
                versions.push(version);
            }

            if renderer.is_json() {
                let output = VersionsOutput {
                    tool: tool_name.to_string(),
                    versions: versions
                        .iter()
                        .map(|v| VersionEntry {
                            version: v.clone(),
                            installed: true,
                            lts: false,
                            lts_name: None,
                            date: None,
                            prerelease: false,
                            download_url: None,
                        })
                        .collect(),
                    total: versions.len(),
                    latest: versions.first().cloned(),
                    lts: None,
                };
                renderer.render(&output)?;
            } else {
                // Text output
                if platform_supported {
                    UI::info(&format!("📦 {}", tool_name));
                } else {
                    UI::info(&format!(
                        "📦 {} ⚠️  (not supported on {})",
                        tool_name,
                        current_platform.as_str()
                    ));
                }

                for version in &versions {
                    let status_icon = if show_status { "✅" } else { "  " };
                    println!(
                        "  {} {} (bundled with {})",
                        status_icon, version, parent_tool
                    );
                }

                if show_status {
                    UI::success(&format!(
                        "Total: {} version(s) available (bundled with {})",
                        versions.len(),
                        parent_tool
                    ));
                }
            }
            return Ok(());
        }
    }

    if installed_executables.is_empty() {
        if renderer.is_json() {
            let output = VersionsOutput {
                tool: tool_name.to_string(),
                versions: vec![],
                total: 0,
                latest: None,
                lts: None,
            };
            renderer.render(&output)?;
        } else {
            // Text output
            if platform_supported {
                UI::info(&format!("📦 {}", tool_name));
            } else {
                UI::info(&format!(
                    "📦 {} ⚠️  (not supported on {})",
                    tool_name,
                    current_platform.as_str()
                ));
            }
            UI::hint("  No versions installed");
            if show_status {
                if let Some(parent_tool) = bundled_with {
                    UI::hint(&format!(
                        "  This tool is bundled with '{}'. Install {} to get {}.",
                        parent_tool, parent_tool, tool_name
                    ));
                } else if platform_supported {
                    UI::hint(&format!(
                        "  Use 'vx install {}' to install this tool",
                        tool_name
                    ));
                } else {
                    UI::hint(&format!(
                        "  This tool is not available on {}",
                        current_platform.as_str()
                    ));
                }
            }
        }
        return Ok(());
    }

    // Collect versions
    let versions: Vec<(String, std::path::PathBuf)> = installed_executables
        .iter()
        .map(|exe_path| (extract_version_from_path(exe_path), exe_path.clone()))
        .collect();

    if renderer.is_json() {
        let output = VersionsOutput {
            tool: tool_name.to_string(),
            versions: versions
                .iter()
                .map(|(v, path)| VersionEntry {
                    version: v.clone(),
                    installed: true,
                    lts: false,
                    lts_name: None,
                    date: None,
                    prerelease: false,
                    download_url: Some(path.display().to_string()),
                })
                .collect(),
            total: versions.len(),
            latest: versions.first().map(|(v, _)| v.clone()),
            lts: None,
        };
        renderer.render(&output)?;
    } else {
        // Text output
        if platform_supported {
            UI::info(&format!("📦 {}", tool_name));
        } else {
            UI::info(&format!(
                "📦 {} ⚠️  (not supported on {})",
                tool_name,
                current_platform.as_str()
            ));
        }

        for (version, exe_path) in &versions {
            let status_icon = if show_status { "✅" } else { "  " };
            println!("  {} {}", status_icon, version);

            if show_status {
                println!("     📁 {}", exe_path.display());
            }
        }

        if show_status {
            UI::success(&format!("Total: {} version(s) installed", versions.len()));
        }
    }

    Ok(())
}

/// Extract version from executable path
/// Paths are like: ~/.vx/store/uv/0.9.17/uv-platform/uv
/// or: ~/.vx/tools/node/18.17.0/node
fn extract_version_from_path(path: &std::path::Path) -> String {
    // Walk up the path to find a version-like component
    for ancestor in path.ancestors() {
        if let Some(name) = ancestor.file_name().and_then(|n| n.to_str()) {
            // Check if this looks like a version (contains digits and dots)
            if name.chars().any(|c| c.is_ascii_digit())
                && (name.contains('.') || name.chars().all(|c| c.is_ascii_digit()))
                && !name.contains('-')
            {
                return name.to_string();
            }
        }
    }
    "unknown".to_string()
}

async fn list_all_tools(
    registry: &ProviderRegistry,
    resolver: &PathResolver,
    _show_status: bool,
    show_all: bool,
    format: OutputFormat,
) -> Result<()> {
    let current_platform = Platform::current();

    // Get all supported tools from registry
    let supported_tools = registry.runtime_names();

    // Get all installed tools (from both store and tools directories)
    let installed_tools_with_versions = resolver.get_installed_tools_with_versions()?;
    let directly_installed: HashSet<_> = installed_tools_with_versions
        .iter()
        .map(|(name, _)| name.as_str())
        .collect();

    // Build a set of tools that are available (either directly installed or bundled with an installed tool)
    let mut available_tools: HashSet<String> =
        directly_installed.iter().map(|s| s.to_string()).collect();

    // Check for bundled tools - if a parent tool is installed, its bundled tools are also available
    for tool_name in &supported_tools {
        if let Some(runtime) = registry.get_runtime(tool_name)
            && let Some(parent_tool) = runtime.metadata().get("bundled_with")
            && directly_installed.contains(parent_tool.as_str())
        {
            available_tools.insert(tool_name.clone());
        }
    }

    let mut installed_count = 0;
    let mut runtimes = Vec::new();

    for tool_name in &supported_tools {
        // Check platform support
        let platform_supported = if let Some(ref runtime) = registry.get_runtime(tool_name) {
            is_platform_supported(runtime, &current_platform)
        } else {
            true
        };

        // If not supported and not showing all, skip
        if !platform_supported && !show_all {
            continue;
        }

        let is_available = available_tools.contains(tool_name);

        if let Some(runtime) = registry.get_runtime(tool_name) {
            // Get platform label from manifest
            let platform_label = get_runtime_platform_label(tool_name);

            // Get installed versions
            let versions = if is_available {
                let tool_name_str: &str = tool_name;
                let is_directly_installed = directly_installed.contains(tool_name_str);
                if is_directly_installed {
                    installed_tools_with_versions
                        .iter()
                        .find(|(name, _)| name == tool_name)
                        .map(|(_, vers)| vers.clone())
                        .unwrap_or_default()
                } else {
                    // Bundled tool - show parent's versions
                    runtime
                        .metadata()
                        .get("bundled_with")
                        .and_then(|parent_tool| {
                            installed_tools_with_versions
                                .iter()
                                .find(|(name, _)| name == parent_tool)
                                .map(|(_, vers)| vers.clone())
                        })
                        .unwrap_or_default()
                }
            } else {
                vec![]
            };

            if is_available {
                installed_count += 1;
            }

            // Get ecosystem
            let ecosystem = runtime.metadata().get("ecosystem").cloned();

            runtimes.push(RuntimeEntry {
                name: tool_name.clone(),
                versions: versions.clone(),
                installed: is_available,
                description: runtime.description().to_string(),
                platform_supported,
                ecosystem,
                platform_label: if !platform_supported || show_all {
                    platform_label
                } else {
                    None
                },
            });
        }
    }

    let renderer = OutputRenderer::new(format);

    let runtimes_count = runtimes.len();
    let output = ListOutput {
        runtimes,
        total: renderer.is_json() as usize * supported_tools.len()
            + !renderer.is_json() as usize * runtimes_count,
        installed_count,
        platform: current_platform.as_str().to_string(),
    };

    if renderer.is_json() {
        renderer.render(&output)?;
    } else {
        // Text mode: use existing UI for header, then render results
        if show_all {
            UI::info("📦 Available Tools (showing all, including unsupported)");
        } else {
            UI::info(&format!(
                "📦 Available Tools ({})",
                current_platform.as_str()
            ));
        }
        renderer.render(&output)?;
    }

    Ok(())
}
