// Go tool implementation
use crate::tool::Tool;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Go tool implementation
pub struct GoTool;

impl GoTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for GoTool {
    fn name(&self) -> &str {
        "go"
    }

    fn description(&self) -> &str {
        "The Go programming language"
    }

    fn homepage(&self) -> Option<&str> {
        Some("https://golang.org")
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("go").is_ok())
    }

    fn get_version(&self) -> Result<Option<String>> {
        let output = Command::new("go").arg("version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "go version go1.21.6 linux/amd64" -> "1.21.6"
                if let Some(version_part) = version_str.split_whitespace().nth(2) {
                    let version = version_part.trim_start_matches("go");
                    return Ok(Some(version.to_string()));
                }
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn get_executable_path(&self, _version: &str, install_dir: &Path) -> PathBuf {
        if cfg!(windows) {
            install_dir.join("bin").join("go.exe")
        } else {
            install_dir.join("bin").join("go")
        }
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        let status = Command::new("go").args(args).status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn supports_auto_install(&self) -> bool {
        true
    }
}

impl Default for GoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_tool_creation() {
        let tool = GoTool::new();
        assert_eq!(tool.name(), "go");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }
}
