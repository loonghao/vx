// UV tool implementation - simplified from the original plugin
use crate::tool::Tool;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// UV tool implementation
pub struct UvTool;

impl UvTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for UvTool {
    fn name(&self) -> &str {
        "uv"
    }

    fn description(&self) -> &str {
        "An extremely fast Python package installer and resolver"
    }

    fn homepage(&self) -> Option<&str> {
        Some("https://docs.astral.sh/uv/")
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("uv").is_ok())
    }

    fn get_version(&self) -> Result<Option<String>> {
        let output = Command::new("uv").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "uv 0.5.26 (5ef3d5139 2025-01-30)" -> "0.5.26"
                if let Some(version_part) = version_str.split_whitespace().nth(1) {
                    return Ok(Some(version_part.to_string()));
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn get_executable_path(&self, _version: &str, install_dir: &Path) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("uv.exe")
        } else {
            install_dir.join("uv")
        }
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        let status = Command::new("uv").args(args).status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn supports_auto_install(&self) -> bool {
        true
    }
}

impl Default for UvTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_tool_creation() {
        let tool = UvTool::new();
        assert_eq!(tool.name(), "uv");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }

    #[test]
    fn test_executable_path() {
        let tool = UvTool::new();
        let install_dir = Path::new("/test/dir");
        let exe_path = tool.get_executable_path("0.1.0", install_dir);

        if cfg!(windows) {
            assert!(exe_path.to_string_lossy().ends_with("uv.exe"));
        } else {
            assert!(exe_path.to_string_lossy().ends_with("uv"));
        }
    }
}
