//! Common traits for tool implementations
//!
//! This module provides reusable traits that eliminate code duplication
//! across different tool implementations.

use crate::{Result, ToolContext, ToolExecutionResult, VersionInfo};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Common installation logic for tools
///
/// This trait provides a standard implementation for tool installation
/// that can be reused across different tool types.
#[async_trait]
pub trait InstallableTool: Send + Sync {
    /// Get the tool name
    fn tool_name(&self) -> &str;

    /// Check if a specific version is already installed
    async fn is_version_installed(&self, version: &str) -> Result<bool>;

    /// Get the installation directory for a specific version
    fn get_version_install_dir(&self, version: &str) -> PathBuf;

    /// Get the executable path for an installed version
    async fn get_executable_path(&self, install_dir: &Path) -> Result<PathBuf>;

    /// Create installation configuration for the tool
    async fn create_install_config(
        &self,
        version: &str,
        install_dir: PathBuf,
    ) -> Result<vx_installer::InstallConfig>;

    /// Verify that the installation was successful
    async fn verify_installation(&self, version: &str) -> Result<bool> {
        self.is_version_installed(version).await
    }

    /// Standard installation implementation
    async fn install_version_impl(&self, version: &str, force: bool) -> Result<()> {
        // Resolve "latest" to actual version number if needed
        let actual_version = self.resolve_version(version).await?;

        // Check if already installed (unless force is true)
        if !force && self.is_version_installed(&actual_version).await? {
            return Err(anyhow::anyhow!(
                "Version {} of {} is already installed. Use --force to reinstall.",
                actual_version,
                self.tool_name()
            ));
        }

        let install_dir = self.get_version_install_dir(&actual_version);

        // Create installation configuration
        let mut config = self
            .create_install_config(&actual_version, install_dir)
            .await?;
        config.force = force;

        // Perform installation
        let installer = vx_installer::Installer::new().await?;
        let _exe_path = installer.install(&config).await.map_err(|e| {
            anyhow::anyhow!(
                "Failed to install {} {}: {}",
                self.tool_name(),
                actual_version,
                e
            )
        })?;

        // Verify installation
        if !self.verify_installation(&actual_version).await? {
            return Err(anyhow::anyhow!(
                "Installation verification failed for {} version {}",
                self.tool_name(),
                actual_version
            ));
        }

        Ok(())
    }

    /// Resolve version (e.g., "latest" to actual version)
    /// Default implementation returns the version as-is
    async fn resolve_version(&self, version: &str) -> Result<String> {
        Ok(version.to_string())
    }
}

/// Common execution logic for tools
///
/// This trait provides standard command execution patterns
/// that can be reused across different tool types.
#[async_trait]
pub trait ExecutableTool: Send + Sync {
    /// Get the tool name
    fn tool_name(&self) -> &str;

    /// Get the command name (might be different from tool name)
    fn command_name(&self) -> &str {
        self.tool_name()
    }

    /// Ensure the tool is available (install if needed)
    async fn ensure_available(&self, context: &ToolContext) -> Result<String>;

    /// Standard execution implementation
    async fn execute_impl(
        &self,
        args: &[String],
        context: &ToolContext,
    ) -> Result<ToolExecutionResult> {
        // Get the executable path
        let executable = if context.use_system_path {
            self.command_name().to_string()
        } else {
            self.ensure_available(context).await?
        };

        // Build and execute command
        let mut cmd = std::process::Command::new(&executable);
        cmd.args(args);

        // Set working directory if specified
        if let Some(cwd) = &context.working_directory {
            cmd.current_dir(cwd);
        }

        // Set environment variables
        for (key, value) in &context.environment_variables {
            cmd.env(key, value);
        }

        // Execute command
        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", self.command_name(), e))?;

        Ok(ToolExecutionResult {
            exit_code: status.code().unwrap_or(-1), // Use -1 for unknown exit codes
            stdout: None,
            stderr: None,
        })
    }
}

/// Common version management for tools
///
/// This trait provides standard version-related operations
/// that can be reused across different tool types.
#[async_trait]
pub trait VersionedTool: Send + Sync {
    /// Get the tool name
    fn tool_name(&self) -> &str;

    /// Fetch available versions
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Get the latest version
    async fn get_latest_version(&self) -> Result<Option<VersionInfo>> {
        let versions = self.fetch_versions(false).await?;
        Ok(versions.into_iter().next())
    }

    /// Get the latest version including prereleases
    async fn get_latest_version_including_prerelease(&self) -> Result<Option<VersionInfo>> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions.into_iter().next())
    }

    /// Check if a specific version exists
    async fn version_exists(&self, version: &str) -> Result<bool> {
        let versions = self.fetch_versions(true).await?;
        Ok(versions.iter().any(|v| v.version == version))
    }

    /// Get installed versions
    async fn get_installed_versions(&self) -> Result<Vec<String>>;

    /// Get the currently active version
    async fn get_active_version(&self) -> Result<String>;
}

/// Composite trait for tools that support all common operations
///
/// This trait combines all the common traits for convenience.
/// Most tools should implement this trait.
#[async_trait]
pub trait StandardTool: InstallableTool + ExecutableTool + VersionedTool + Send + Sync {
    /// Get download URL for a specific version
    async fn get_download_url(&self, version: &str) -> Result<Option<String>>;

    /// Get tool dependencies
    async fn get_dependencies(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    /// Get tool metadata
    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }
}

/// Helper macro to implement common tool patterns
///
/// This macro reduces boilerplate code for simple tool implementations.
#[macro_export]
macro_rules! impl_standard_tool {
    ($tool_type:ty, $name:expr, $description:expr) => {
        #[async_trait::async_trait]
        impl $crate::VxTool for $tool_type {
            fn name(&self) -> &str {
                $name
            }

            fn description(&self) -> &str {
                $description
            }

            async fn install_version(&self, version: &str, force: bool) -> $crate::Result<()> {
                $crate::traits::InstallableTool::install_version_impl(self, version, force).await
            }

            async fn execute(
                &self,
                args: &[String],
                context: &$crate::ToolContext,
            ) -> $crate::Result<$crate::ToolExecutionResult> {
                $crate::traits::ExecutableTool::execute_impl(self, args, context).await
            }

            async fn fetch_versions(
                &self,
                include_prerelease: bool,
            ) -> $crate::Result<Vec<$crate::VersionInfo>> {
                $crate::traits::VersionedTool::fetch_versions(self, include_prerelease).await
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock tool for testing
    struct MockTool {
        name: String,
    }

    impl MockTool {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl InstallableTool for MockTool {
        fn tool_name(&self) -> &str {
            &self.name
        }

        async fn is_version_installed(&self, _version: &str) -> Result<bool> {
            Ok(false)
        }

        fn get_version_install_dir(&self, version: &str) -> PathBuf {
            PathBuf::from(format!("/tmp/{}/{}", self.name, version))
        }

        async fn get_executable_path(&self, install_dir: &Path) -> Result<PathBuf> {
            Ok(install_dir.join(&self.name))
        }

        async fn create_install_config(
            &self,
            _version: &str,
            _install_dir: PathBuf,
        ) -> Result<vx_installer::InstallConfig> {
            // Mock implementation
            Err(anyhow::anyhow!("Mock implementation"))
        }
    }

    #[tokio::test]
    async fn test_tool_name() {
        let tool = MockTool::new("test-tool");
        assert_eq!(tool.tool_name(), "test-tool");
    }

    #[tokio::test]
    async fn test_version_install_dir() {
        let tool = MockTool::new("test-tool");
        let dir = tool.get_version_install_dir("1.0.0");
        assert_eq!(dir, PathBuf::from("/tmp/test-tool/1.0.0"));
    }
}
