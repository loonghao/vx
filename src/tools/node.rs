// Node.js tool implementation
use crate::tool::Tool;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Node.js tool implementation
pub struct NodeTool;

impl NodeTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for NodeTool {
    fn name(&self) -> &str {
        "node"
    }

    fn description(&self) -> &str {
        "Node.js JavaScript runtime and npm package manager"
    }

    fn homepage(&self) -> Option<&str> {
        Some("https://nodejs.org")
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("node").is_ok())
    }

    fn get_version(&self) -> Result<Option<String>> {
        let output = Command::new("node").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "v18.19.0" -> "18.19.0"
                let version = version_str.trim().trim_start_matches('v');
                Ok(Some(version.to_string()))
            }
            _ => Ok(None),
        }
    }

    fn get_executable_path(&self, _version: &str, install_dir: &Path) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("node.exe")
        } else {
            install_dir.join("bin").join("node")
        }
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        let status = Command::new("node").args(args).status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn supports_auto_install(&self) -> bool {
        true
    }
}

impl Default for NodeTool {
    fn default() -> Self {
        Self::new()
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
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }
}
