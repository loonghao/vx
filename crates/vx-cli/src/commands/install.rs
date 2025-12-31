//! Install command implementation

use crate::ui::{ProgressSpinner, UI};
use anyhow::Result;
use std::env;
use vx_paths::project::{find_vx_config, LOCK_FILE_NAME};
use vx_resolver::{LockFile, LockedTool};
use vx_runtime::{ProviderRegistry, RuntimeContext};

pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    tool_name: &str,
    version: Option<&str>,
    force: bool,
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
    let spinner = ProgressSpinner::new(&format!(
        "Resolving version {} for {}...",
        requested_version, tool_name
    ));
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

    // Install the version with progress spinner
    let spinner =
        ProgressSpinner::new_install(&format!("Installing {} {}...", tool_name, target_version));
    let install_result = runtime.install(&target_version, context).await;

    match install_result {
        Ok(result) => {
            spinner.finish_with_message(&format!(
                "✓ Successfully installed {} {}",
                tool_name, target_version
            ));

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
            UI::hint(&format!(
                "Use 'vx {} --version' to verify installation",
                tool_name
            ));
        }
        Err(e) => {
            spinner.finish_with_error(&format!(
                "Failed to install {} {}: {}",
                tool_name, target_version, e
            ));
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
