//! Extension management commands

use crate::ui::UI;
use anyhow::Result;
use std::path::PathBuf;
use vx_extension::ExtensionManager;

/// Handle `vx ext list` command
pub async fn handle_list(verbose: bool) -> Result<()> {
    let manager = ExtensionManager::new()?;
    let extensions = manager.list_extensions().await?;

    if extensions.is_empty() {
        UI::info("No extensions installed.");
        UI::info("");
        UI::info("To install an extension:");
        UI::info("  vx ext install github:user/vx-ext-name");
        UI::info("");
        UI::info("To link a local extension for development:");
        UI::info("  vx ext dev /path/to/extension");
        return Ok(());
    }

    UI::header("Installed Extensions");
    println!();

    if verbose {
        for ext in &extensions {
            println!("{}", ExtensionManager::format_extension_info(ext));
            println!("---");
        }
    } else {
        println!(
            "{:<20} {:<10} {:<8} {:<10} {:<10} DESCRIPTION",
            "NAME", "VERSION", "TYPE", "SOURCE", "COMMANDS"
        );
        println!("{}", "-".repeat(80));

        for ext in &extensions {
            println!("{}", ExtensionManager::format_extension_summary(ext));
        }
    }

    println!();
    UI::info(&format!("Total: {} extension(s)", extensions.len()));

    Ok(())
}

/// Handle `vx ext info` command
pub async fn handle_info(name: &str) -> Result<()> {
    let manager = ExtensionManager::new()?;

    match manager.find_extension(name).await? {
        Some(ext) => {
            println!("{}", ExtensionManager::format_extension_info(&ext));
        }
        None => {
            UI::error(&format!("Extension '{}' not found", name));
            return Err(anyhow::anyhow!("Extension not found"));
        }
    }

    Ok(())
}

/// Handle `vx ext dev` command
pub async fn handle_dev(path: &str, unlink: bool) -> Result<()> {
    let manager = ExtensionManager::new()?;

    if unlink {
        // Treat path as extension name when unlinking
        manager.unlink_dev_extension(path).await?;
        UI::success(&format!("Unlinked development extension '{}'", path));
    } else {
        let source_path = PathBuf::from(path);

        if !source_path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path));
        }

        let source_path = source_path.canonicalize()?;
        manager.link_dev_extension(source_path.clone()).await?;

        UI::success(&format!(
            "Linked development extension from {}",
            source_path.display()
        ));
        UI::info("The extension is now available via `vx x <name>`");
    }

    Ok(())
}

/// Handle `vx ext install` command (placeholder for future)
pub async fn handle_install(source: &str) -> Result<()> {
    UI::warning("Remote extension installation is not yet implemented.");
    UI::info(&format!("Source: {}", source));
    UI::info("");
    UI::info("For now, you can:");
    UI::info("  1. Clone the extension repository manually");
    UI::info("  2. Link it using: vx ext dev /path/to/extension");

    Ok(())
}

/// Handle `vx ext uninstall` command
pub async fn handle_uninstall(name: &str) -> Result<()> {
    let manager = ExtensionManager::new()?;

    // Check if extension exists
    match manager.find_extension(name).await? {
        Some(ext) => {
            // For now, only support unlinking dev extensions
            if ext.source == vx_extension::ExtensionSource::Dev {
                manager.unlink_dev_extension(name).await?;
                UI::success(&format!("Uninstalled extension '{}'", name));
            } else {
                // For user extensions, remove the directory
                let ext_dir = manager
                    .list_extensions()
                    .await?
                    .into_iter()
                    .find(|e| e.name == name)
                    .map(|e| e.path);

                if let Some(path) = ext_dir {
                    std::fs::remove_dir_all(&path)?;
                    UI::success(&format!(
                        "Removed extension '{}' from {}",
                        name,
                        path.display()
                    ));
                } else {
                    return Err(anyhow::anyhow!("Could not find extension path"));
                }
            }
        }
        None => {
            UI::error(&format!("Extension '{}' not found", name));
            return Err(anyhow::anyhow!("Extension not found"));
        }
    }

    Ok(())
}

/// Handle `vx x <extension> [args...]` command
pub async fn handle_execute(extension_name: &str, args: &[String]) -> Result<()> {
    let manager = ExtensionManager::with_project_dir(std::env::current_dir()?)?;

    let exit_code = manager.execute(extension_name, args).await?;

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}
