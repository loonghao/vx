//! Node.js tool implementations - JavaScript runtime and package management tools

use crate::config::NodeUrlBuilder;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use vx_plugin::{ToolContext, ToolExecutionResult, VersionInfo, VxTool};
use vx_tool_standard::StandardUrlBuilder;
use vx_version::{NodeVersionFetcher, VersionFetcher};
// use vx_core::{UrlBuilder, VersionParser};

/// Macro to generate Node.js tool implementations using VxTool trait
macro_rules! node_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            version_fetcher: NodeVersionFetcher,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    version_fetcher: NodeVersionFetcher::new(),
                }
            }
        }

        #[async_trait::async_trait]
        impl VxTool for $name {
            fn name(&self) -> &str {
                $cmd
            }

            fn description(&self) -> &str {
                $desc
            }

            fn aliases(&self) -> Vec<&str> {
                match $cmd {
                    "node" => vec!["nodejs"],
                    "npm" => vec![],
                    "npx" => vec![],
                    _ => vec![],
                }
            }

            async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
                match $cmd {
                    "node" => {
                        // For Node.js, fetch from official API
                        self.version_fetcher
                            .fetch_versions(include_prerelease)
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to fetch versions: {}", e))
                    }
                    "npm" | "npx" => {
                        // npm and npx use Node.js versions since they're bundled
                        let node_fetcher = NodeVersionFetcher::new();
                        node_fetcher
                            .fetch_versions(include_prerelease)
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to fetch Node.js versions: {}", e))
                    }
                    _ => Err(anyhow::anyhow!("Unknown tool: {}", $cmd)),
                }
            }

            async fn install_version(&self, version: &str, force: bool) -> Result<()> {
                match $cmd {
                    "node" => {
                        // Resolve "latest" to actual version first
                        let actual_version = if version == "latest" {
                            let versions = self.fetch_versions(false).await?;
                            if let Some(latest_version) = versions.first() {
                                latest_version.version.clone()
                            } else {
                                return Err(anyhow::anyhow!(
                                    "No versions found for {}",
                                    self.name()
                                ));
                            }
                        } else {
                            version.to_string()
                        };

                        // Install Node.js directly
                        if !force && self.is_version_installed(&actual_version).await? {
                            return Err(anyhow::anyhow!(
                                "Version {} of {} is already installed",
                                actual_version,
                                self.name()
                            ));
                        }

                        let install_dir = self.get_version_install_dir(&actual_version);

                        // Use real installation with vx-installer (with resolved version)
                        let mut config =
                            crate::config::create_install_config(&actual_version, install_dir)
                                .await?;
                        config.force = force; // Set the force flag
                        let installer = vx_installer::Installer::new().await?;

                        let _exe_path = installer.install(&config).await.map_err(|e| {
                            anyhow::anyhow!("Failed to install Node.js {}: {}", actual_version, e)
                        })?;

                        // Verify installation
                        if !self.is_version_installed(&actual_version).await? {
                            return Err(anyhow::anyhow!(
                                "Installation verification failed for {} version {}",
                                self.name(),
                                actual_version
                            ));
                        }

                        Ok(())
                    }
                    "npm" | "npx" => {
                        // npm and npx are bundled with Node.js, so install Node.js instead
                        let node_tool = NodeTool::new();
                        node_tool.install_version(version, force).await?;

                        // Verify that the bundled tool is available
                        if !self.is_version_installed(version).await? {
                            return Err(anyhow::anyhow!(
                                "{} is not available in Node.js {} installation",
                                self.name(),
                                version
                            ));
                        }

                        Ok(())
                    }
                    _ => Err(anyhow::anyhow!("Unknown tool: {}", $cmd)),
                }
            }

            async fn is_version_installed(&self, version: &str) -> Result<bool> {
                match $cmd {
                    "node" => {
                        // Check if Node.js version directory exists and contains executable
                        let install_dir = self.get_version_install_dir(version);
                        if !install_dir.exists() {
                            return Ok(false);
                        }

                        // Check if executable exists in the install directory
                        match self.get_executable_path(&install_dir).await {
                            Ok(exe_path) => Ok(exe_path.exists()),
                            Err(_) => Ok(false),
                        }
                    }
                    "npm" | "npx" => {
                        // npm and npx are bundled with Node.js, so check Node.js installation
                        let node_tool = NodeTool::new();
                        let node_installed = node_tool.is_version_installed(version).await?;

                        if !node_installed {
                            return Ok(false);
                        }

                        // Also verify that the specific tool executable exists
                        let node_install_dir = node_tool.get_version_install_dir(version);
                        Ok(self.verify_installation(&node_install_dir).await)
                    }
                    _ => Ok(false),
                }
            }

            async fn execute(
                &self,
                args: &[String],
                context: &ToolContext,
            ) -> Result<ToolExecutionResult> {
                // Use vx-managed installation instead of system PATH
                let executable = self.ensure_available(context).await?;

                let mut cmd = std::process::Command::new(&executable);
                cmd.args(args);

                if let Some(cwd) = &context.working_directory {
                    cmd.current_dir(cwd);
                }

                for (key, value) in &context.environment_variables {
                    cmd.env(key, value);
                }

                let status = cmd
                    .status()
                    .map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", $cmd, e))?;

                Ok(ToolExecutionResult {
                    exit_code: status.code().unwrap_or(1),
                    stdout: None,
                    stderr: None,
                })
            }

            async fn get_active_version(&self) -> Result<String> {
                // Get the latest installed version
                let installed_versions = self.get_installed_versions().await?;
                if let Some(latest_version) = installed_versions.first() {
                    Ok(latest_version.clone())
                } else {
                    Err(anyhow::anyhow!("No {} versions installed", self.name()))
                }
            }

            async fn get_installed_versions(&self) -> Result<Vec<String>> {
                match $cmd {
                    "node" => {
                        // Check for installed versions in vx-managed directories using async I/O
                        let path_manager = vx_paths::PathManager::new().unwrap_or_default();
                        let tool_dir = path_manager.tools_dir().join($cmd);

                        if !tokio::fs::try_exists(&tool_dir).await.unwrap_or(false) {
                            return Ok(vec![]);
                        }

                        let mut versions = Vec::new();
                        let mut entries = tokio::fs::read_dir(&tool_dir).await?;

                        // Collect all directory entries first
                        let mut version_dirs = Vec::new();
                        while let Some(entry) = entries.next_entry().await? {
                            if entry.file_type().await?.is_dir() {
                                if let Some(version) = entry.file_name().to_str() {
                                    version_dirs.push((version.to_string(), entry.path()));
                                }
                            }
                        }

                        // Verify installations concurrently
                        let verification_futures =
                            version_dirs.iter().map(|(version, install_dir)| {
                                let version = version.clone();
                                let install_dir = install_dir.clone();
                                async move {
                                    if self.verify_installation(&install_dir).await {
                                        Some(version)
                                    } else {
                                        None
                                    }
                                }
                            });

                        let verification_results =
                            futures::future::join_all(verification_futures).await;
                        versions.extend(verification_results.into_iter().flatten());

                        // Sort versions (latest first)
                        versions.sort_by(|a, b| b.cmp(a));
                        Ok(versions)
                    }
                    "npm" | "npx" => {
                        // npm and npx use Node.js versions, so return Node.js installed versions
                        // but only those where this specific tool is available
                        let node_tool = NodeTool::new();
                        let node_versions = node_tool.get_installed_versions().await?;

                        // Check availability concurrently
                        let availability_futures = node_versions.iter().map(|version| {
                            let version = version.clone();
                            async move {
                                if self.is_version_installed(&version).await.unwrap_or(false) {
                                    Some(version)
                                } else {
                                    None
                                }
                            }
                        });

                        let availability_results =
                            futures::future::join_all(availability_futures).await;
                        let available_versions: Vec<String> =
                            availability_results.into_iter().flatten().collect();

                        Ok(available_versions)
                    }
                    _ => Ok(vec![]),
                }
            }

            async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
                Ok(NodeUrlBuilder::download_url(version))
            }

            fn metadata(&self) -> HashMap<String, String> {
                let mut meta = HashMap::new();
                meta.insert("homepage".to_string(), $homepage.unwrap_or("").to_string());
                meta.insert("ecosystem".to_string(), "javascript".to_string());
                meta
            }

            fn get_dependencies(&self) -> Vec<vx_plugin::ToolDependency> {
                // Define dependencies based on tool type
                match $cmd {
                    "node" => Vec::new(), // Node.js has no dependencies
                    "npm" => vec![vx_plugin::ToolDependency::required(
                        "node",
                        "npm is bundled with Node.js",
                    )],
                    "npx" => vec![vx_plugin::ToolDependency::required(
                        "node",
                        "npx is bundled with Node.js",
                    )],
                    _ => Vec::new(),
                }
            }
        }

        impl $name {
            /// Verify if a tool installation is valid using PATHEXT-aware search
            async fn verify_installation(&self, install_dir: &std::path::Path) -> bool {
                use vx_paths::find_executable_with_extensions;

                // Check common locations for the executable
                let possible_base_paths = vec![
                    install_dir.join($cmd),                 // Direct in install dir
                    install_dir.join("bin").join($cmd),     // Unix-style bin directory
                    install_dir.join("Scripts").join($cmd), // Windows Scripts directory (npm/node)
                ];

                for base_path in possible_base_paths {
                    if find_executable_with_extensions(&base_path, $cmd).is_some() {
                        return true;
                    }
                }

                false
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

// Define Node.js tools using the VxTool macro
node_vx_tool!(
    NodeTool,
    "node",
    "Node.js JavaScript runtime",
    Some("https://nodejs.org/")
);
node_vx_tool!(
    NpmTool,
    "npm",
    "Node.js package manager",
    Some("https://www.npmjs.com/")
);
node_vx_tool!(
    NpxTool,
    "npx",
    "Node.js package runner",
    Some("https://www.npmjs.com/package/npx")
);

// Add ExecutableTool-like methods for Node tools
impl NodeTool {
    /// Get version install directory
    fn get_version_install_dir(&self, version: &str) -> std::path::PathBuf {
        let paths = vx_paths::VxPaths::default();
        paths.tools_dir.join(self.name()).join(version)
    }

    /// Get executable path for a given install directory
    async fn get_executable_path(&self, install_dir: &Path) -> Result<std::path::PathBuf> {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", self.name())
        } else {
            self.name().to_string()
        };
        let exe_path = install_dir.join(&exe_name);

        if exe_path.exists() {
            Ok(exe_path)
        } else {
            Err(anyhow::anyhow!(
                "{} executable not found at {}",
                self.name(),
                exe_path.display()
            ))
        }
    }

    /// Ensure tool is available and return executable path
    async fn ensure_available(&self, _context: &ToolContext) -> Result<String> {
        // Try to get active version
        match self.get_active_version().await {
            Ok(version) => {
                let install_dir = self.get_version_install_dir(&version);
                match self.get_executable_path(&install_dir).await {
                    Ok(path) => Ok(path.to_string_lossy().to_string()),
                    Err(_) => {
                        // Install latest if not found
                        self.install_version("latest", false).await?;
                        let latest_version = self.get_active_version().await?;
                        let latest_dir = self.get_version_install_dir(&latest_version);
                        let exe_path = self.get_executable_path(&latest_dir).await?;
                        Ok(exe_path.to_string_lossy().to_string())
                    }
                }
            }
            Err(_) => {
                // No versions installed, install latest
                self.install_version("latest", false).await?;
                let latest_version = self.get_active_version().await?;
                let latest_dir = self.get_version_install_dir(&latest_version);
                let exe_path = self.get_executable_path(&latest_dir).await?;
                Ok(exe_path.to_string_lossy().to_string())
            }
        }
    }
}

// Add similar methods for NpmTool and NpxTool
impl NpmTool {
    /// Get version install directory (npm uses node's installation)
    fn get_version_install_dir(&self, version: &str) -> std::path::PathBuf {
        let paths = vx_paths::VxPaths::default();
        paths.tools_dir.join("node").join(version)
    }

    /// Get executable path for npm (in node's installation)
    async fn get_executable_path(&self, install_dir: &Path) -> Result<std::path::PathBuf> {
        let exe_name = if cfg!(windows) { "npm.cmd" } else { "npm" };
        let exe_path = install_dir.join(exe_name);

        if exe_path.exists() {
            Ok(exe_path)
        } else {
            Err(anyhow::anyhow!(
                "npm executable not found at {}",
                exe_path.display()
            ))
        }
    }

    /// Ensure npm is available (depends on node)
    async fn ensure_available(&self, context: &ToolContext) -> Result<String> {
        // npm depends on node, so ensure node is available first
        let node_tool = NodeTool::new();
        let _node_path = node_tool.ensure_available(context).await?;

        // Get node's active version for npm
        let node_version = node_tool.get_active_version().await?;
        let install_dir = self.get_version_install_dir(&node_version);
        let exe_path = self.get_executable_path(&install_dir).await?;
        Ok(exe_path.to_string_lossy().to_string())
    }
}

impl NpxTool {
    /// Get version install directory (npx uses node's installation)
    fn get_version_install_dir(&self, version: &str) -> std::path::PathBuf {
        let paths = vx_paths::VxPaths::default();
        paths.tools_dir.join("node").join(version)
    }

    /// Get executable path for npx (in node's installation)
    async fn get_executable_path(&self, install_dir: &Path) -> Result<std::path::PathBuf> {
        let exe_name = if cfg!(windows) { "npx.cmd" } else { "npx" };
        let exe_path = install_dir.join(exe_name);

        if exe_path.exists() {
            Ok(exe_path)
        } else {
            Err(anyhow::anyhow!(
                "npx executable not found at {}",
                exe_path.display()
            ))
        }
    }

    /// Ensure npx is available (depends on node)
    async fn ensure_available(&self, context: &ToolContext) -> Result<String> {
        // npx depends on node, so ensure node is available first
        let node_tool = NodeTool::new();
        let _node_path = node_tool.ensure_available(context).await?;

        // Get node's active version for npx
        let node_version = node_tool.get_active_version().await?;
        let install_dir = self.get_version_install_dir(&node_version);
        let exe_path = self.get_executable_path(&install_dir).await?;
        Ok(exe_path.to_string_lossy().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_tool_creation() {
        let tool = NodeTool::new();
        assert_eq!(tool.name(), "node");
        assert!(!tool.description().is_empty());
        assert!(tool.aliases().contains(&"nodejs"));
    }

    #[test]
    fn test_npm_tool_creation() {
        let tool = NpmTool::new();
        assert_eq!(tool.name(), "npm");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_npx_tool_creation() {
        let tool = NpxTool::new();
        assert_eq!(tool.name(), "npx");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_node_tool_metadata() {
        let tool = NodeTool::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"javascript".to_string()));
    }

    #[test]
    fn test_tool_dependencies() {
        let node_tool = NodeTool::new();
        let npm_tool = NpmTool::new();
        let npx_tool = NpxTool::new();

        // Node.js should have no dependencies
        assert_eq!(node_tool.get_dependencies().len(), 0);

        // npm should depend on node
        let npm_deps = npm_tool.get_dependencies();
        assert_eq!(npm_deps.len(), 1);
        assert_eq!(npm_deps[0].tool_name, "node");
        assert!(npm_deps[0].required);

        // npx should depend on node
        let npx_deps = npx_tool.get_dependencies();
        assert_eq!(npx_deps.len(), 1);
        assert_eq!(npx_deps[0].tool_name, "node");
        assert!(npx_deps[0].required);
    }
}
