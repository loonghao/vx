//! Go tool implementation

use std::collections::HashMap;
use vx_core::{
    GitHubVersionParser, GoUrlBuilder, GoVersionParser, HttpUtils, Result, ToolContext,
    ToolExecutionResult, VersionInfo, VxTool,
};

/// Go tool implementation
#[derive(Debug, Clone)]
pub struct GoTool {
    url_builder: GoUrlBuilder,
    version_parser: GitHubVersionParser,
}

impl GoTool {
    pub fn new() -> Self {
        Self {
            url_builder: GoUrlBuilder::new(),
            version_parser: GitHubVersionParser::new("golang", "go"),
        }
    }
}

impl Default for GoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl VxTool for GoTool {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language"
    }

    fn aliases(&self) -> Vec<&str> {
        vec!["golang"]
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // For Go, fetch from GitHub releases
        let json = HttpUtils::fetch_json(GoUrlBuilder::versions_url()).await?;
        GoVersionParser::parse_versions(&json, include_prerelease)
    }

    async fn install_version(&self, version: &str, force: bool) -> Result<()> {
        if !force && self.is_version_installed(version).await? {
            return Err(vx_core::VxError::VersionAlreadyInstalled {
                tool_name: self.name().to_string(),
                version: version.to_string(),
            });
        }

        let install_dir = self.get_version_install_dir(version);
        let _exe_path = self.default_install_workflow(version, &install_dir).await?;

        // Verify installation
        if !self.is_version_installed(version).await? {
            return Err(vx_core::VxError::InstallationFailed {
                tool_name: self.name().to_string(),
                version: version.to_string(),
                message: "Installation verification failed".to_string(),
            });
        }

        Ok(())
    }

    async fn execute(&self, args: &[String], context: &ToolContext) -> Result<ToolExecutionResult> {
        self.default_execute_workflow(args, context).await
    }

    async fn get_download_url(&self, version: &str) -> Result<Option<String>> {
        Ok(GoUrlBuilder::download_url(version))
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://golang.org/".to_string());
        meta.insert("ecosystem".to_string(), "go".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/golang/go".to_string(),
        );
        meta.insert("license".to_string(), "BSD-3-Clause".to_string());
        meta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_tool_basic() {
        let tool = GoTool::default();

        assert_eq!(tool.name(), "go");
        assert_eq!(tool.description(), "Go programming language");
        assert!(tool.aliases().contains(&"golang"));
    }

    #[test]
    fn test_go_tool_metadata() {
        let tool = GoTool::new();
        let metadata = tool.metadata();

        assert!(metadata.contains_key("homepage"));
        assert!(metadata.contains_key("ecosystem"));
        assert_eq!(metadata.get("ecosystem"), Some(&"go".to_string()));
    }
}
