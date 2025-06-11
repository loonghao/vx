// Rust toolchain implementations
use crate::tool::Tool;
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Cargo tool implementation (Rust package manager and build tool)
pub struct CargoTool;

impl CargoTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for CargoTool {
    fn name(&self) -> &str {
        "cargo"
    }

    fn description(&self) -> &str {
        "Rust package manager and build tool"
    }

    fn homepage(&self) -> Option<&str> {
        Some("https://doc.rust-lang.org/cargo/")
    }

    fn is_installed(&self) -> Result<bool> {
        Ok(which::which("cargo").is_ok())
    }

    fn get_version(&self) -> Result<Option<String>> {
        let output = Command::new("cargo").arg("--version").output();

        match output {
            Ok(output) if output.status.success() => {
                let version_str = String::from_utf8_lossy(&output.stdout);
                // Parse "cargo 1.75.0 (1d8b05cdd 2023-11-20)" -> "1.75.0"
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
            install_dir.join("bin").join("cargo.exe")
        } else {
            install_dir.join("bin").join("cargo")
        }
    }

    fn execute(&self, args: &[String]) -> Result<i32> {
        let status = Command::new("cargo").args(args).status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn supports_auto_install(&self) -> bool {
        true
    }
}

/// Rustc tool implementation (Rust compiler)
pub struct RustcTool;

impl RustcTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for RustcTool {
    fn name(&self) -> &str {
        "rustc"
    }

    fn description(&self) -> &str {
        "The Rust compiler"
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
        let status = Command::new("rustc").args(args).status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn supports_auto_install(&self) -> bool {
        true
    }
}

impl Default for CargoTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RustcTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_tool_creation() {
        let tool = CargoTool::new();
        assert_eq!(tool.name(), "cargo");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }

    #[test]
    fn test_rustc_tool_creation() {
        let tool = RustcTool::new();
        assert_eq!(tool.name(), "rustc");
        assert!(!tool.description().is_empty());
        assert!(tool.homepage().is_some());
        assert!(tool.supports_auto_install());
    }
}
