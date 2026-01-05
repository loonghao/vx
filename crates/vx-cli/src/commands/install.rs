//! Install command implementation

use crate::ui::{ProgressSpinner, UI};
use anyhow::Result;
use std::env;
use vx_paths::project::{find_vx_config, LOCK_FILE_NAME};
use vx_resolver::{LockFile, LockedTool};
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// Parse tool specification in format "tool" or "tool@version"
fn parse_tool_spec(spec: &str) -> (&str, Option<&str>) {
    if let Some((tool, version)) = spec.split_once('@') {
        (tool, Some(version))
    } else {
        (spec, None)
    }
}

pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tools: &[String],
    force: bool,
) -> Result<()> {
    let mut success_count = 0;
    let mut fail_count = 0;
    let total = tools.len();
    let is_multi = total > 1;

    for (idx, tool_spec) in tools.iter().enumerate() {
        let (tool_name, version) = parse_tool_spec(tool_spec);

        if is_multi {
            UI::section(&format!("[{}/{}] {}", idx + 1, total, tool_spec));
        }

        match install_single(registry, context, tool_name, version, force, is_multi).await {
            Ok(()) => success_count += 1,
            Err(e) => {
                UI::error(&format!("Failed to install {}: {}", tool_spec, e));
                fail_count += 1;
            }
        }
    }

    // Summary for multiple tools
    if is_multi {
        println!();
        if fail_count == 0 {
            UI::success(&format!("Successfully installed {} tool(s)", success_count));
        } else {
            UI::warn(&format!(
                "Installed {} tool(s), {} failed",
                success_count, fail_count
            ));
        }
    }

    if fail_count > 0 {
        Err(anyhow::anyhow!("{} tool(s) failed to install", fail_count))
    } else {
        Ok(())
    }
}

async fn install_single(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
    is_multi: bool,
) -> Result<()> {
    // Get the runtime from registry
    let runtime = match registry.get_runtime(tool_name) {
        Some(r) => r,
        None => {
            // Show friendly error with suggestions
            let available_tools = registry.runtime_names();
            UI::tool_not_found(tool_name, &available_tools);
            return Err(anyhow::anyhow!("Tool not found: {}", tool_name));
        }
    };

    // Determine version to install
    let requested_version = version.unwrap_or("latest");

    // Resolve version (handles "latest", partial versions like "3.11", etc.)
    let resolve_msg = if is_multi {
        format!("Resolving {}...", requested_version)
    } else {
        format!(
            "Resolving version {} for {}...",
            requested_version, tool_name
        )
    };
    let spinner = ProgressSpinner::new(&resolve_msg);
    let target_version = runtime.resolve_version(requested_version, context).await?;
    spinner.finish_and_clear();

    if requested_version != target_version {
        UI::detail(&format!(
            "Resolved {} → {}",
            requested_version, target_version
        ));
    }

    // Check if already installed
    if !force && runtime.is_installed(&target_version, context).await? {
        UI::success(&format!(
            "{} {} is already installed",
            tool_name, target_version
        ));
        UI::hint("Use --force to reinstall");
        return Ok(());
    }

    // Run pre-install hook
    runtime.pre_install(&target_version, context).await?;

    // Install the version
    let install_result = if is_multi {
        // In multi-tool mode, use simpler output without spinner
        // to avoid visual clutter
        runtime.install(&target_version, context).await
    } else {
        // In single-tool mode, show spinner
        let spinner = ProgressSpinner::new_install(&format!(
            "Installing {} {}...",
            tool_name, target_version
        ));
        let result = runtime.install(&target_version, context).await;
        match &result {
            Ok(_) => spinner.finish_with_message(&format!(
                "✓ Successfully installed {} {}",
                tool_name, target_version
            )),
            Err(e) => spinner.finish_with_error(&format!(
                "Failed to install {} {}: {}",
                tool_name, target_version, e
            )),
        }
        result
    };

    match install_result {
        Ok(result) => {
            if is_multi {
                UI::success(&format!("Installed {} {}", tool_name, target_version));
            }

            // Run post-install hook
            runtime.post_install(&target_version, context).await?;

            // Show installation path
            UI::detail(&format!("Installed to: {}", result.install_path.display()));

            // Update lock file if it exists
            update_lockfile_if_exists(
                tool_name,
                &target_version,
                requested_version,
                runtime.ecosystem(),
            );

            // Show usage hint
            if !is_multi {
                UI::hint(&format!(
                    "Use 'vx {} --version' to verify installation",
                    tool_name
                ));
            }
        }
        Err(e) => {
            if is_multi {
                UI::error(&format!(
                    "Failed to install {} {}: {}",
                    tool_name, target_version, e
                ));
            }
            return Err(e);
        }
    }

    Ok(())
}

/// Update lock file if it exists in the current project
fn update_lockfile_if_exists(
    tool_name: &str,
    version: &str,
    resolved_from: &str,
    ecosystem: vx_runtime::Ecosystem,
) {
    // Try to find project root with vx.toml
    let current_dir = match env::current_dir() {
        Ok(d) => d,
        Err(_) => return,
    };

    let config_path = match find_vx_config(&current_dir) {
        Ok(p) => p,
        Err(_) => return, // No project config, skip lock file update
    };

    let project_root = match config_path.parent() {
        Some(p) => p,
        None => return,
    };

    let lock_path = project_root.join(LOCK_FILE_NAME);

    // Only update if lock file already exists
    if !lock_path.exists() {
        return;
    }

    // Load existing lock file
    let mut lockfile = match LockFile::load(&lock_path) {
        Ok(lf) => lf,
        Err(_) => return,
    };

    // Convert ecosystem
    let resolver_ecosystem = match ecosystem {
        vx_runtime::Ecosystem::NodeJs => vx_resolver::Ecosystem::Node,
        vx_runtime::Ecosystem::Python => vx_resolver::Ecosystem::Python,
        vx_runtime::Ecosystem::Rust => vx_resolver::Ecosystem::Rust,
        vx_runtime::Ecosystem::Go => vx_resolver::Ecosystem::Go,
        _ => vx_resolver::Ecosystem::Generic,
    };

    // Create locked tool entry
    let locked_tool = LockedTool::new(version, "vx install")
        .with_resolved_from(resolved_from)
        .with_ecosystem(resolver_ecosystem);

    // Update lock file
    lockfile.lock_tool(tool_name, locked_tool);

    // Save lock file
    if let Err(e) = lockfile.save(&lock_path) {
        UI::warn(&format!("Failed to update {}: {}", LOCK_FILE_NAME, e));
    } else {
        UI::detail(&format!(
            "Updated {} with {} = {}",
            LOCK_FILE_NAME, tool_name, version
        ));
    }
}
