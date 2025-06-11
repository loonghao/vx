// Rust tool implementation
use crate::tool::Tool;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Rust tool implementation
pub struct RustTool;

impl RustTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for RustTool {
    fn name(&self) -> &str {
        "rust"
    }

    fn description(&self) -> &str {
        "A language empowering everyone to build reliable and efficient software"
    }

    fn homepage(&self) -> Option<&str> {
        Some("https://www.rust-lang.org")
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("rustc").is_ok())
    }

    fn get_version(&self) -> Result<Option<String>> {
        let output = Command::new("rustc").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "rustc 1.75.0 (82e1608df 2023-12-21)" -> "1.75.0"
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
            install_dir.join("bin").join("rustc.exe")
        } else {
            install_dir.join("bin").join("rustc")
        }
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        // For Rust, we typically want to execute cargo, not rustc directly
        let command = if args.is_empty() || args[0] == "rustc" {
            "rustc"
        } else {
            "cargo"
        };

        let actual_args = if command == "cargo" && !args.is_empty() && args[0] != "cargo" {
            args
        } else if command == "cargo" && !args.is_empty() && args[0] == "cargo" {
            &args[1..]
        } else {
            args
        };

        let status = Command::new(command).args(actual_args).status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn supports_auto_install(&self) -> bool {
        true
    }
}

impl Default for RustTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_tool_creation() {
        let tool = RustTool::new();
        assert_eq!(tool.name(), "rust");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }
}
