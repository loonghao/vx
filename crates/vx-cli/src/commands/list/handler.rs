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
use std::sync::Arc;
use vx_paths::PathResolver;
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
    // Dummy resolver kept for function signature compatibility
    let path_manager = vx_paths::PathManager::new()
        .map_err(|e| anyhow::anyhow!("Failed to initialize path manager: {}", e))?;
    let resolver = PathResolver::new(path_manager);

    if show_system {
        list_system_tools(registry, show_all, format).await?;
        return Ok(());
    }

    match tool {
        Some(tool_name) => {
            list_tool_versions(registry, &resolver, tool_name, show_status, format).await?;
        }
        None => {
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

        // Sort alphabetically (a-z) for consistent output
        runtimes.sort_by(|a, b| a.name.cmp(&b.name));

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
    _resolver: &PathResolver,
    tool_name: &str,
    show_status: bool,
    format: OutputFormat,
) -> Result<()> {
    // Check if tool is supported
    let runtime = registry.get_runtime(tool_name);
    let Some(runtime) = runtime else {
        let available_tools = registry.runtime_names();
        UI::tool_not_found(tool_name, &available_tools);
        return Ok(());
    };

    let current_platform = Platform::current();
    let platform_supported = is_platform_supported(&runtime, &current_platform);
    let canonical_name = runtime.name();
    let bundled_with = runtime.metadata().get("bundled_with").cloned();
    let renderer = OutputRenderer::new(format);

    // ── RFC-0037: ProviderHandle is the single source of truth ───────────
    let reg = global_registry().await;

    // Resolve the handle: try canonical name first, then bundled parent
    let (versions, source_label): (Vec<String>, Option<String>) =
        if let Some(handle) = reg.get(canonical_name) {
            let v = handle.installed_versions();
            (v, None)
        } else if let Some(parent) = &bundled_with
            && let Some(parent_handle) = reg.get(parent)
        {
            // Tool is bundled with parent — show parent's installed versions
            let v = parent_handle.installed_versions();
            (v, Some(parent.clone()))
        } else {
            (vec![], None)
        };

    // Get executable paths for each version (for --status display)
    let version_paths: Vec<(String, Option<std::path::PathBuf>)> = if show_status {
        versions
            .iter()
            .map(|v| {
                let path = reg.get(canonical_name).and_then(|h| h.get_execute_path(v));
                (v.clone(), path)
            })
            .collect()
    } else {
        versions.iter().map(|v| (v.clone(), None)).collect()
    };

    drop(reg);

    if version_paths.is_empty() {
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
                if let Some(parent_tool) = &bundled_with {
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

    if renderer.is_json() {
        let output = VersionsOutput {
            tool: tool_name.to_string(),
            versions: version_paths
                .iter()
                .map(|(v, path)| VersionEntry {
                    version: v.clone(),
                    installed: true,
                    lts: false,
                    lts_name: None,
                    date: None,
                    prerelease: false,
                    download_url: path.as_ref().map(|p| p.display().to_string()),
                })
                .collect(),
            total: version_paths.len(),
            latest: version_paths.first().map(|(v, _)| v.clone()),
            lts: None,
        };
        renderer.render(&output)?;
    } else {
        if platform_supported {
            UI::info(&format!("📦 {}", tool_name));
        } else {
            UI::info(&format!(
                "📦 {} ⚠️  (not supported on {})",
                tool_name,
                current_platform.as_str()
            ));
        }

        for (version, exe_path) in &version_paths {
            let status_icon = if show_status { "✅" } else { "  " };
            if let Some(label) = &source_label {
                println!("  {} {} (bundled with {})", status_icon, version, label);
            } else {
                println!("  {} {}", status_icon, version);
            }
            if show_status && let Some(path) = exe_path {
                println!("     📁 {}", path.display());
            }
        }

        if show_status {
            UI::success(&format!(
                "Total: {} version(s) installed",
                version_paths.len()
            ));
        }
    }

    Ok(())
}

async fn list_all_tools(
    registry: &ProviderRegistry,
    _resolver: &PathResolver,
    _show_status: bool,
    show_all: bool,
    format: OutputFormat,
) -> Result<()> {
    let current_platform = Platform::current();
    let supported_tools = registry.runtime_names();

    // ── RFC-0037: ProviderHandle is the single source of truth ───────────
    let reg = global_registry().await;

    let mut installed_count = 0;
    let mut runtimes = Vec::new();

    for tool_name in &supported_tools {
        let platform_supported = if let Some(ref runtime) = registry.get_runtime(tool_name) {
            is_platform_supported(runtime, &current_platform)
        } else {
            true
        };

        // Skip unsupported platforms unless --all
        if !platform_supported && !show_all {
            continue;
        }

        let runtime = match registry.get_runtime(tool_name) {
            Some(r) => r,
            None => continue,
        };

        let canonical_name = runtime.name();
        let bundled_with = runtime.metadata().get("bundled_with").cloned();

        // Get installed versions from ProviderHandle
        let versions: Vec<String> = if let Some(handle) = reg.get(canonical_name) {
            handle.installed_versions()
        } else if let Some(parent) = &bundled_with
            && let Some(parent_handle) = reg.get(parent)
        {
            // Bundled tool: show parent's versions
            parent_handle.installed_versions()
        } else {
            vec![]
        };

        let is_available = !versions.is_empty();
        if is_available {
            installed_count += 1;
        }

        let platform_label = get_runtime_platform_label(tool_name);
        let ecosystem = runtime.metadata().get("ecosystem").cloned();

        runtimes.push(RuntimeEntry {
            name: tool_name.clone(),
            versions,
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

    drop(reg);

    // Sort tools alphabetically (a-z) for consistent, predictable output
    runtimes.sort_by(|a, b| a.name.cmp(&b.name));

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
