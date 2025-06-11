//! UV tool implementations - Python package management tools

use vx_core::{Result, Tool, ToolInfo};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Base trait for UV tools that provides common functionality
trait UvTool {
    fn tool_name() -> &'static str;
    fn tool_description() -> &'static str;
    fn tool_homepage() -> Option<&'static str>;
}

/// Macro to generate UV tool implementations
macro_rules! uv_tool {
    ($name:ident, $cmd:literal, $desc:literal, $homepage:expr) => {
        #[derive(Debug, Clone)]
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }

        impl UvTool for $name {
            fn tool_name() -> &'static str {
                $cmd
            }

            fn tool_description() -> &'static str {
                $desc
            }

            fn tool_homepage() -> Option<&'static str> {
                $homepage
            }
        }

        impl Tool for $name {
            fn name(&self) -> &str {
                Self::tool_name()
            }

            fn description(&self) -> &str {
                Self::tool_description()
            }

            fn homepage(&self) -> Option<&str> {
                Self::tool_homepage()
            }

            fn is_installed(&self) -> Result<bool> {
                Ok(which::which(Self::tool_name()).is_ok())
            }

            fn get_version(&self) -> Result<Option<String>> {
                let output = Command::new(Self::tool_name()).arg("--version").output()
                    .map_err(|e| vx_core::VxError::Other { message: e.to_string() })?;

                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    // Parse "uv 0.5.26 (5ef3d5139 2025-01-30)" -> "0.5.26"
                    if let Some(version_part) = version_str.split_whitespace().nth(1) {
                        return Ok(Some(version_part.to_string()));
                    }
                }
                Ok(None)
            }

            fn get_executable_path(&self, _version: &str, install_dir: &Path) -> PathBuf {
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", Self::tool_name())
                } else {
                    Self::tool_name().to_string()
                };

                install_dir.join(exe_name)
            }

            fn execute(&self, args: &[String]) -> Result<i32> {
                let status = Command::new(Self::tool_name()).args(args).status()
                    .map_err(|e| vx_core::VxError::Other { message: e.to_string() })?;
                Ok(status.code().unwrap_or(1))
            }

            fn supports_auto_install(&self) -> bool {
                true
            }

            fn get_info(&self) -> ToolInfo {
                ToolInfo {
                    name: self.name().to_string(),
                    version: self.get_version().unwrap_or_default().unwrap_or_else(|| "unknown".to_string()),
                    description: self.description().to_string(),
                    homepage: self.homepage().map(|s| s.to_string()),
                    repository: Some("https://github.com/astral-sh/uv".to_string()),
                    license: Some("MIT OR Apache-2.0".to_string()),
                    keywords: vec![
                        "python".to_string(),
                        "package-manager".to_string(),
                        "uv".to_string(),
                    ],
                    categories: vec!["development-tools".to_string()],
                    supported_platforms: vec![
                        "windows".to_string(),
                        "macos".to_string(),
                        "linux".to_string(),
                    ],
                    dependencies: vec!["python".to_string()],
                }
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

// Define UV tools using the macro
uv_tool!(UvCommand, "uv", "An extremely fast Python package installer and resolver", Some("https://docs.astral.sh/uv/"));
uv_tool!(UvxTool, "uvx", "Python application runner", Some("https://docs.astral.sh/uv/"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_tool_creation() {
        let tool = UvCommand::new();
        assert_eq!(tool.name(), "uv");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }

    #[test]
    fn test_uvx_tool_creation() {
        let tool = UvxTool::new();
        assert_eq!(tool.name(), "uvx");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }

    #[test]
    fn test_uv_tool_info() {
        let tool = UvCommand::new();
        let info = tool.get_info();

        assert_eq!(info.name, "uv");
        assert!(info.description.contains("Python"));
        assert!(info.keywords.contains(&"python".to_string()));
        assert!(info.keywords.contains(&"package-manager".to_string()));
    }

    #[test]
    fn test_executable_path() {
        let tool = UvCommand::new();
        let install_dir = Path::new("/test/dir");
        let exe_path = tool.get_executable_path("0.1.0", install_dir);

        if cfg!(windows) {
            assert!(exe_path.to_string_lossy().ends_with("uv.exe"));
        } else {
            assert!(exe_path.to_string_lossy().ends_with("uv"));
        }
    }
}
