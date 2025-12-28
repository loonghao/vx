//! Extension manager - high-level API for managing extensions

use crate::error::{ExtensionError, ExtensionResult};
use crate::{Extension, ExtensionDiscovery, ExtensionExecutor};
use std::path::PathBuf;
use tracing::info;

/// Extension manager - main entry point for extension operations
pub struct ExtensionManager {
    discovery: ExtensionDiscovery,
    executor: ExtensionExecutor,
}

impl ExtensionManager {
    /// Create a new extension manager
    pub fn new() -> ExtensionResult<Self> {
        Ok(Self {
            discovery: ExtensionDiscovery::new()?,
            executor: ExtensionExecutor::new(),
        })
    }

    /// Create an extension manager with a project directory
    pub fn with_project_dir(project_dir: PathBuf) -> ExtensionResult<Self> {
        Ok(Self {
            discovery: ExtensionDiscovery::new()?.with_project_dir(project_dir),
            executor: ExtensionExecutor::new(),
        })
    }

    /// List all discovered extensions
    pub async fn list_extensions(&self) -> ExtensionResult<Vec<Extension>> {
        self.discovery.discover_all().await
    }

    /// Find an extension by name
    pub async fn find_extension(&self, name: &str) -> ExtensionResult<Option<Extension>> {
        self.discovery.find_extension(name).await
    }

    /// Execute an extension command
    pub async fn execute(&self, extension_name: &str, args: &[String]) -> ExtensionResult<i32> {
        // Find the extension with detailed error
        let extension = self
            .discovery
            .find_extension_or_error(extension_name)
            .await?;

        // Parse subcommand from args
        let (subcommand, remaining_args) = if args.is_empty() {
            (None, args)
        } else {
            // Check if first arg is a known subcommand
            let first_arg = &args[0];
            if extension.config.commands.contains_key(first_arg) {
                (Some(first_arg.as_str()), &args[1..])
            } else {
                (None, args)
            }
        };

        info!(
            "Running extension '{}' ({}) from {}",
            extension.name, extension.config.extension.extension_type, extension.source
        );

        self.executor
            .execute(&extension, subcommand, remaining_args)
            .await
    }

    /// Link a local development extension
    pub async fn link_dev_extension(&self, source_path: PathBuf) -> ExtensionResult<()> {
        let dev_dir = self.discovery.dev_extensions_dir();

        // Ensure dev directory exists
        std::fs::create_dir_all(dev_dir).map_err(|e| {
            ExtensionError::io(
                "Failed to create dev extensions directory",
                Some(dev_dir.to_path_buf()),
                e,
            )
        })?;

        // Load the extension config to get the name
        let config_path = source_path.join("vx-extension.toml");
        if !config_path.exists() {
            return Err(ExtensionError::config_not_found(&source_path));
        }

        let config = crate::ExtensionConfig::from_file(&config_path)?;
        let name = &config.extension.name;

        // Create symlink
        let link_path = dev_dir.join(name);

        if link_path.exists() {
            // Remove existing link
            if link_path.is_symlink() {
                #[cfg(unix)]
                std::fs::remove_file(&link_path).map_err(|e| {
                    ExtensionError::io(
                        "Failed to remove existing symlink",
                        Some(link_path.clone()),
                        e,
                    )
                })?;
                #[cfg(windows)]
                std::fs::remove_dir(&link_path).map_err(|e| {
                    ExtensionError::io(
                        "Failed to remove existing symlink",
                        Some(link_path.clone()),
                        e,
                    )
                })?;
            } else {
                return Err(ExtensionError::link_failed(
                    &source_path,
                    &link_path,
                    "Target path already exists and is not a symlink",
                ));
            }
        }

        // Create the symlink
        #[cfg(unix)]
        std::os::unix::fs::symlink(&source_path, &link_path).map_err(|e| {
            ExtensionError::link_failed(
                &source_path,
                &link_path,
                format!("Failed to create symlink: {}", e),
            )
        })?;

        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(&source_path, &link_path).map_err(|e| {
            ExtensionError::link_failed(
                &source_path,
                &link_path,
                format!("Failed to create symlink: {}", e),
            )
        })?;

        info!(
            "Linked development extension '{}' from {:?}",
            name, source_path
        );

        Ok(())
    }

    /// Unlink a development extension
    pub async fn unlink_dev_extension(&self, name: &str) -> ExtensionResult<()> {
        let link_path = self.discovery.dev_extensions_dir().join(name);

        if !link_path.exists() {
            // Collect available dev extensions for error message
            let available = self
                .list_extensions()
                .await?
                .into_iter()
                .filter(|e| e.source == crate::ExtensionSource::Dev)
                .map(|e| e.name)
                .collect();

            return Err(ExtensionError::extension_not_found(
                name,
                available,
                vec![self.discovery.dev_extensions_dir().to_path_buf()],
            ));
        }

        if !link_path.is_symlink() {
            return Err(ExtensionError::NotADevLink {
                name: name.to_string(),
                path: link_path,
            });
        }

        #[cfg(unix)]
        std::fs::remove_file(&link_path).map_err(|e| {
            ExtensionError::io("Failed to remove symlink", Some(link_path.clone()), e)
        })?;

        #[cfg(windows)]
        std::fs::remove_dir(&link_path).map_err(|e| {
            ExtensionError::io("Failed to remove symlink", Some(link_path.clone()), e)
        })?;

        info!("Unlinked development extension '{}'", name);

        Ok(())
    }

    /// Get extension info for display
    pub fn format_extension_info(extension: &Extension) -> String {
        let config = &extension.config;
        let mut info = String::new();

        info.push_str(&format!("Name: {}\n", extension.name));
        info.push_str(&format!("Version: {}\n", config.extension.version));
        info.push_str(&format!("Type: {}\n", config.extension.extension_type));
        info.push_str(&format!("Source: {}\n", extension.source));
        info.push_str(&format!("Path: {}\n", extension.path.display()));

        if !config.extension.description.is_empty() {
            info.push_str(&format!("Description: {}\n", config.extension.description));
        }

        if let Some(runtime) = config.runtime.runtime_name() {
            info.push_str(&format!("Runtime: {}", runtime));
            if let Some(constraint) = config.runtime.version_constraint() {
                info.push_str(&format!(" {}", constraint));
            }
            info.push('\n');
        }

        if !config.commands.is_empty() {
            info.push_str("\nCommands:\n");
            for (name, cmd) in &config.commands {
                info.push_str(&format!("  {} - {}\n", name, cmd.description));
            }
        }

        info
    }

    /// Get a summary line for an extension
    pub fn format_extension_summary(extension: &Extension) -> String {
        let config = &extension.config;
        let runtime = config.runtime.runtime_name().unwrap_or("unknown");
        let cmd_count = config.commands.len();

        format!(
            "{:<20} {:<10} {:<8} {:<10} {:>3} cmd(s)  {}",
            extension.name,
            config.extension.version,
            config.extension.extension_type,
            extension.source,
            cmd_count,
            if config.extension.description.is_empty() {
                format!("[{}]", runtime)
            } else {
                config.extension.description.clone()
            }
        )
    }
}

impl Default for ExtensionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create extension manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extension_manager_creation() {
        let manager = ExtensionManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_list_extensions_empty() {
        // This test may find actual extensions on the system
        // Just verify it doesn't crash
        let manager = ExtensionManager::new().unwrap();
        let result = manager.list_extensions().await;
        assert!(result.is_ok());
    }
}
