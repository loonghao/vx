//! Node.js tool implementations - JavaScript runtime and package management tools

use std::path::PathBuf;
use std::process::Command;
use vx_core::{Tool, ToolContext, VersionInfo, Result};

/// Macro to generate Node.js tool implementations with environment isolation
macro_rules! node_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }

        impl Tool for $name {
            fn name(&self) -> &str {
                $cmd
            }

            fn description(&self) -> &str {
                $desc
            }

            fn homepage(&self) -> Option<&str> {
                $homepage
            }

            fn is_installed(&self) -> Result<bool> {
                Ok(which::which(self.name()).is_ok())
            }

            fn get_version(&self) -> Result<Option<String>> {
                let output = Command::new(self.name()).arg("--version").output();

                match output {
                    Ok(output) if output.status.success() => {
                        let version_str = String::from_utf8_lossy(&output.stdout);
                        // Parse version from output (format varies by tool)
                        if let Some(version_part) = version_str.split_whitespace().nth(0) {
                            // Remove 'v' prefix if present
                            let version = version_part.trim_start_matches('v');
                            return Ok(Some(version.to_string()));
                        }
                        Ok(None)
                    }
                    _ => Ok(None),
                }
            }

            fn get_executable_path(&self, version: &str, install_dir: &Path) -> PathBuf {
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", self.name())
                } else {
                    self.name().to_string()
                };

                install_dir.join("bin").join(exe_name)
            }

            fn execute(&self, args: &[String]) -> Result<i32> {
                let status = Command::new(self.name()).args(args).status()?;
                Ok(status.code().unwrap_or(1))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

// Define Node.js tools using the macro
node_tool!(NodeTool, "node", "Node.js JavaScript runtime", Some("https://nodejs.org/"));
node_tool!(NpmTool, "npm", "Node.js package manager", Some("https://www.npmjs.com/"));
node_tool!(NpxTool, "npx", "Node.js package runner", Some("https://www.npmjs.com/package/npx"));

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

    #[test]
    fn test_npm_tool_creation() {
        let tool = NpmTool::new();
        assert_eq!(tool.name(), "npm");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }

    #[test]
    fn test_npx_tool_creation() {
        let tool = NpxTool::new();
        assert_eq!(tool.name(), "npx");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }

    #[test]
    fn test_node_tool_info() {
        let tool = NodeTool::new();
        let info = tool.get_info();

        assert_eq!(info.name, "node");
        assert!(info.description.contains("JavaScript"));
    }

    #[test]
    fn test_executable_path() {
        let tool = NodeTool::new();
        let install_dir = std::path::Path::new("/test/dir");
        let exe_path = tool.get_executable_path("18.0.0", install_dir);

        if cfg!(windows) {
            assert!(exe_path.to_string_lossy().ends_with("node.exe"));
        } else {
            assert!(exe_path.to_string_lossy().ends_with("node"));
        }
    }
}
