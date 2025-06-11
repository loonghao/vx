//! Go tool implementation

use vx_core::{Tool, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Go tool implementation
#[derive(Debug, Clone)]
pub struct GoTool;

impl GoTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Tool for GoTool {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "Go programming language"
    }

    fn homepage(&self) -> Option<&str> {
        Some("https://golang.org/")
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("go").is_ok())
    }

    fn get_version(&self) -> Result<Option<String>> {
        let output = Command::new("go").arg("version").output()
            .map_err(|e| vx_core::VxError::Other { message: e.to_string() })?;

        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            // Parse "go version go1.21.6 linux/amd64" -> "1.21.6"
            if let Some(version_part) = version_str.split_whitespace().nth(2) {
                let version = version_part.trim_start_matches("go");
                return Ok(Some(version.to_string()));
            }
        }
        Ok(None)
    }

    fn get_executable_path(&self, _version: &str, install_dir: &Path) -> PathBuf {
        let exe_name = if cfg!(windows) {
            "go.exe"
        } else {
            "go"
        };

        install_dir.join("bin").join(exe_name)
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        let status = Command::new("go").args(args).status()
            .map_err(|e| vx_core::VxError::Other { message: e.to_string() })?;
        Ok(status.code().unwrap_or(1))
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
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }

    #[test]
    fn test_executable_path() {
        let tool = GoTool::new();
        let install_dir = std::path::Path::new("/test/dir");
        let exe_path = tool.get_executable_path("1.21.0", install_dir);

        if cfg!(windows) {
            assert!(exe_path.to_string_lossy().ends_with("go.exe"));
        } else {
            assert!(exe_path.to_string_lossy().ends_with("go"));
        }
    }
}
