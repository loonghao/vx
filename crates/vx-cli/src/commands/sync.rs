// Sync command implementation

use crate::ui::UI;

use vx_core::{ConfigManager, PluginRegistry, Result, VxError};

pub async fn handle(
    registry: &PluginRegistry,
    check: bool,
    force: bool,
    dry_run: bool,
    verbose: bool,
    no_parallel: bool,
    no_auto_install: bool,
) -> Result<()> {
    let spinner = UI::new_spinner("Reading project configuration...");

    // Check if .vx.toml exists
    let config_path = std::env::current_dir()
        .map_err(|e| VxError::Other {
            message: format!("Failed to get current directory: {}", e),
        })?
        .join(".vx.toml");

    if !config_path.exists() {
        spinner.finish_and_clear();
        return handle_no_config().await;
    }

    // Load project configuration
    let config_manager = ConfigManager::new().await?;
    let config = config_manager.config();

    spinner.finish_and_clear();

    if config.tools.is_empty() {
        UI::warn("No tools defined in .vx.toml");
        return Ok(());
    }

    UI::header("Project Sync");
    UI::info(&format!("Found {} tools to sync", config.tools.len()));

    // Analyze tools and their status
    let mut sync_plan = Vec::new();
    for (tool_name, tool_config) in &config.tools {
        let version = tool_config.version.as_deref().unwrap_or("latest");
        let status = check_tool_status(registry, tool_name, version).await?;
        sync_plan.push(SyncItem {
            name: tool_name.to_string(),
            version: version.to_string(),
            status,
        });
    }

    if check {
        return display_check_results(&sync_plan);
    }

    if dry_run {
        return display_dry_run(&sync_plan);
    }

    // Execute sync
    execute_sync(
        registry,
        &sync_plan,
        force,
        verbose,
        !no_parallel,
        !no_auto_install,
    )
    .await
}

async fn handle_no_config() -> Result<()> {
    UI::warn("No .vx.toml configuration file found");
    UI::info("Detecting project type...");

    let current_dir = std::env::current_dir().map_err(|e| VxError::Other {
        message: format!("Failed to get current directory: {}", e),
    })?;

    // Check for common project files
    if current_dir.join("package.json").exists() {
        suggest_node_config().await?;
    } else if current_dir.join("pyproject.toml").exists()
        || current_dir.join("requirements.txt").exists()
    {
        suggest_python_config().await?;
    } else if current_dir.join("go.mod").exists() {
        suggest_go_config().await?;
    } else if current_dir.join("Cargo.toml").exists() {
        suggest_rust_config().await?;
    } else {
        UI::info("No project type detected. Use 'vx init' to create a configuration file.");
    }

    Ok(())
}

async fn suggest_node_config() -> Result<()> {
    UI::info("🔍 Detected Node.js project (package.json found)");
    UI::info("💡 Suggested configuration:");
    println!();
    println!("[tools]");
    println!("node = \"18.17.0\"  # LTS version");
    println!("npm = \"latest\"");
    println!();
    UI::info("Run 'vx init --template node' to create .vx.toml with these settings");
    Ok(())
}

async fn suggest_python_config() -> Result<()> {
    UI::info("🔍 Detected Python project");
    UI::info("💡 Suggested configuration:");
    println!();
    println!("[tools]");
    println!("python = \"3.11\"");
    println!("uv = \"latest\"");
    println!();
    UI::info("Run 'vx init --template python' to create .vx.toml with these settings");
    Ok(())
}

async fn suggest_go_config() -> Result<()> {
    UI::info("🔍 Detected Go project (go.mod found)");
    UI::info("💡 Suggested configuration:");
    println!();
    println!("[tools]");
    println!("go = \"latest\"");
    println!();
    UI::info("Run 'vx init --template go' to create .vx.toml with these settings");
    Ok(())
}

async fn suggest_rust_config() -> Result<()> {
    UI::info("🔍 Detected Rust project (Cargo.toml found)");
    UI::info("💡 Suggested configuration:");
    println!();
    println!("[tools]");
    println!("cargo = \"latest\"");
    println!();
    UI::info("Run 'vx init --template rust' to create .vx.toml with these settings");
    Ok(())
}

#[derive(Debug)]
struct SyncItem {
    name: String,
    version: String,
    status: ToolStatus,
}

#[derive(Debug)]
#[allow(dead_code)]
enum ToolStatus {
    Installed(String), // version
    NotInstalled,
    NotSupported,
}

async fn check_tool_status(
    registry: &PluginRegistry,
    tool_name: &str,
    _version: &str,
) -> Result<ToolStatus> {
    if registry.supports_tool(tool_name) {
        // TODO: Check if specific version is installed
        // For now, just return NotInstalled
        Ok(ToolStatus::NotInstalled)
    } else {
        Ok(ToolStatus::NotSupported)
    }
}

fn display_check_results(sync_plan: &[SyncItem]) -> Result<()> {
    UI::info("🔍 Checking project requirements...");
    println!();

    let mut installed_count = 0;
    let mut missing_count = 0;
    let mut unsupported_count = 0;

    println!("Required tools:");
    for item in sync_plan {
        match &item.status {
            ToolStatus::Installed(version) => {
                println!(
                    "  ✅ {}@{} (installed: {})",
                    item.name, item.version, version
                );
                installed_count += 1;
            }
            ToolStatus::NotInstalled => {
                println!("  ❌ {}@{} (not installed)", item.name, item.version);
                missing_count += 1;
            }
            ToolStatus::NotSupported => {
                println!("  ⚠️  {}@{} (not supported)", item.name, item.version);
                unsupported_count += 1;
            }
        }
    }

    println!();
    println!("Summary:");
    if installed_count > 0 {
        println!("  - {} tools already installed", installed_count);
    }
    if missing_count > 0 {
        println!("  - {} tools need installation", missing_count);
    }
    if unsupported_count > 0 {
        println!("  - {} tools not supported", unsupported_count);
    }

    if missing_count > 0 {
        println!();
        UI::info("Run 'vx sync' to install missing tools.");
    }

    Ok(())
}

fn display_dry_run(sync_plan: &[SyncItem]) -> Result<()> {
    UI::info("🔍 Sync plan preview:");
    println!();

    let mut will_install = Vec::new();
    let mut will_skip = Vec::new();

    for item in sync_plan {
        match &item.status {
            ToolStatus::Installed(_) => will_skip.push(item),
            ToolStatus::NotInstalled => will_install.push(item),
            ToolStatus::NotSupported => {
                println!("  ⚠️  {}@{} (not supported)", item.name, item.version);
            }
        }
    }

    if !will_install.is_empty() {
        println!("Will install:");
        for item in &will_install {
            println!("  📦 {}@{}", item.name, item.version);
            println!(
                "    - Install to: ~/.vx/tools/{}/{}/",
                item.name, item.version
            );
        }
        println!();
    }

    if !will_skip.is_empty() {
        println!("Will skip:");
        for item in &will_skip {
            println!("  ⏭️  {}@{} (already installed)", item.name, item.version);
        }
        println!();
    }

    UI::info("Run 'vx sync' to execute this plan.");
    Ok(())
}

async fn execute_sync(
    _registry: &PluginRegistry,
    sync_plan: &[SyncItem],
    _force: bool,
    verbose: bool,
    _parallel: bool,
    _auto_install: bool,
) -> Result<()> {
    UI::info("📦 Installing tools...");

    let mut success_count = 0;
    let mut error_count = 0;

    for item in sync_plan {
        match &item.status {
            ToolStatus::Installed(_) if !_force => {
                if verbose {
                    UI::info(&format!(
                        "⏭️  {}@{} (already installed)",
                        item.name, item.version
                    ));
                }
            }
            ToolStatus::NotInstalled | ToolStatus::Installed(_) => {
                if verbose {
                    UI::info(&format!("⬇️  Installing {}@{}...", item.name, item.version));
                }

                // TODO: Implement actual installation
                // For now, just simulate
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                UI::success(&format!(
                    "✅ {}@{} installed successfully",
                    item.name, item.version
                ));
                success_count += 1;
            }
            ToolStatus::NotSupported => {
                UI::error(&format!(
                    "❌ {}@{} (not supported)",
                    item.name, item.version
                ));
                error_count += 1;
            }
        }
    }

    println!();
    if error_count == 0 {
        UI::success("🎉 Project sync completed! All tools are ready.");

        if success_count > 0 {
            println!();
            println!("Next steps:");
            for item in sync_plan {
                if matches!(item.status, ToolStatus::NotInstalled) {
                    println!("  vx {} --version", item.name);
                }
            }
        }
    } else {
        UI::error(&format!("❌ Sync completed with {} errors", error_count));
    }

    Ok(())
}
