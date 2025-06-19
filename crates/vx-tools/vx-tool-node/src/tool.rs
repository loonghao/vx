//! Node.js tool implementations - JavaScript runtime and package management tools

use std::collections::HashMap;
use vx_core::{
    HttpUtils, NodeUrlBuilder, NodeVersionParser, Result, ToolContext, ToolExecutionResult,
    VersionInfo, VxEnvironment, VxError, VxTool,
};
// use vx_core::{UrlBuilder, VersionParser};

/// Macro to generate Node.js tool implementations using VxTool trait
macro_rules! node_vx_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name {
            _url_builder: NodeUrlBuilder,
            _version_parser: NodeVersionParser,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    _url_builder: NodeUrlBuilder::new(),
                    _version_parser: NodeVersionParser::new(),
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
                // For Node.js, fetch from official API
                let json = HttpUtils::fetch_json(NodeUrlBuilder::versions_url()).await?;
                NodeVersionParser::parse_versions(&json, include_prerelease)
            }

            async fn install_version(&self, version: &str, force: bool) -> Result<()> {
                if !force && self.is_version_installed(version).await? {
                    return Err(VxError::VersionAlreadyInstalled {
                        tool_name: self.name().to_string(),
                        version: version.to_string(),
                    });
                }

                let install_dir = self.get_version_install_dir(version);
                let _exe_path = self.default_install_workflow(version, &install_dir).await?;

                // Verify installation
                if !self.is_version_installed(version).await? {
                    return Err(VxError::InstallationFailed {
                        tool_name: self.name().to_string(),
                        version: version.to_string(),
                        message: "Installation verification failed".to_string(),
                    });
                }

                Ok(())
            }

            async fn is_version_installed(&self, version: &str) -> Result<bool> {
                let env = VxEnvironment::new().expect("Failed to create VX environment");

                // For npm and npx, check if Node.js is installed (they come bundled)
                if self.name() == "npm" || self.name() == "npx" {
                    return Ok(env.is_version_installed("node", version));
                }

                Ok(env.is_version_installed(self.name(), version))
            }

            async fn execute(
                &self,
                args: &[String],
                context: &ToolContext,
            ) -> Result<ToolExecutionResult> {
                // For npm and npx, find executable in Node.js installation
                if (self.name() == "npm" || self.name() == "npx") && !context.use_system_path {
                    let active_version = self.get_active_version().await?;
                    let env = VxEnvironment::new().expect("Failed to create VX environment");
                    let node_install_dir = env.get_version_install_dir("node", &active_version);
                    let exe_path = env.find_executable_in_dir(&node_install_dir, self.name())?;

                    // Execute the tool
                    let mut cmd = std::process::Command::new(&exe_path);
                    cmd.args(args);

                    if let Some(cwd) = &context.working_directory {
                        cmd.current_dir(cwd);
                    }

                    for (key, value) in &context.environment_variables {
                        cmd.env(key, value);
                    }

                    let status = cmd.status().map_err(|e| VxError::Other {
                        message: format!("Failed to execute {}: {}", self.name(), e),
                    })?;

                    return Ok(ToolExecutionResult {
                        exit_code: status.code().unwrap_or(1),
                        stdout: None,
                        stderr: None,
                    });
                }

                // For node or system path execution, use default workflow
                self.default_execute_workflow(args, context).await
            }

            async fn get_active_version(&self) -> Result<String> {
                let env = VxEnvironment::new().expect("Failed to create VX environment");

                // For npm and npx, use Node.js version
                if self.name() == "npm" || self.name() == "npx" {
                    if let Some(active_version) = env.get_active_version("node")? {
                        return Ok(active_version);
                    }

                    let installed_versions = env.list_installed_versions("node")?;
                    return installed_versions.first().cloned().ok_or_else(|| {
                        VxError::ToolNotInstalled {
                            tool_name: "node".to_string(),
                        }
                    });
                }

                // For node, use default implementation
                if let Some(active_version) = env.get_active_version(self.name())? {
                    return Ok(active_version);
                }

                let installed_versions = env.list_installed_versions(self.name())?;
                installed_versions
                    .first()
                    .cloned()
                    .ok_or_else(|| VxError::ToolNotInstalled {
                        tool_name: self.name().to_string(),
                    })
            }

            async fn get_installed_versions(&self) -> Result<Vec<String>> {
                let env = VxEnvironment::new().expect("Failed to create VX environment");

                // For npm and npx, use Node.js versions
                if self.name() == "npm" || self.name() == "npx" {
                    return env.list_installed_versions("node");
                }

                env.list_installed_versions(self.name())
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
}
