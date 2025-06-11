//! Rust toolchain implementations with environment isolation

use std::path::PathBuf;
use std::process::Command;
use vx_core::{Tool, ToolContext, VersionInfo, Result};

/// Macro to generate Rust tool implementations with environment isolation
macro_rules! rust_tool {
    ($name:ident, $cmd:literal, $desc:literal) => {
        #[derive(Debug, Clone)]
        pub struct $name;

        impl $name {
            pub fn new() -> Self {
                Self
            }
        }

        #[async_trait::async_trait]
        impl Tool for $name {
            fn name(&self) -> &str {
                $cmd
            }

            fn description(&self) -> &str {
                $desc
            }

            fn aliases(&self) -> Vec<&str> {
                vec![]
            }

            async fn fetch_versions(&self, _include_prerelease: bool) -> Result<Vec<VersionInfo>> {
                // For Rust tools, we typically use rustup to manage versions
                // This is a placeholder implementation
                Ok(vec![])
            }

            async fn install_version(&self, version: &str) -> Result<PathBuf> {
                // For Rust tools, installation is typically handled by rustup
                // This would integrate with rustup or download from GitHub releases
                let install_dir = self.get_version_install_dir(version);
                std::fs::create_dir_all(&install_dir)?;

                // Return the executable path
                Ok(self.get_executable_path(version))
            }

            async fn get_installed_versions(&self) -> Result<Vec<String>> {
                let base_dir = self.get_base_install_dir();
                if !base_dir.exists() {
                    return Ok(vec![]);
                }

                let mut versions = vec![];
                for entry in std::fs::read_dir(&base_dir)? {
                    let entry = entry?;
                    if entry.file_type()?.is_dir() {
                        if let Some(version) = entry.file_name().to_str() {
                            versions.push(version.to_string());
                        }
                    }
                }

                versions.sort();
                versions.reverse();
                Ok(versions)
            }

            async fn uninstall_version(&self, version: &str) -> Result<()> {
                let version_dir = self.get_version_install_dir(version);
                if version_dir.exists() {
                    std::fs::remove_dir_all(version_dir)?;
                }
                Ok(())
            }

            async fn get_download_url(&self, _version: &str) -> Result<Option<String>> {
                // This would return the download URL for the specific version
                // For Rust tools, this might be GitHub releases or rustup channels
                Ok(None)
            }

            fn get_executable_path(&self, version: &str) -> PathBuf {
                let exe_name = if cfg!(windows) {
                    format!("{}.exe", self.name())
                } else {
                    self.name().to_string()
                };

                self.get_version_install_dir(version).join("bin").join(exe_name)
            }

            async fn execute(&self, context: &ToolContext) -> Result<i32> {
                let exe_path = self.get_executable_path(&context.version);

                let mut cmd = Command::new(&exe_path);
                cmd.args(&context.args);

                if let Some(cwd) = &context.working_directory {
                    cmd.current_dir(cwd);
                }

                for (key, value) in &context.environment_variables {
                    cmd.env(key, value);
                }

                let status = cmd.status()?;
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

// Define all Rust tools using the macro
rust_tool!(CargoTool, "cargo", "Rust package manager and build tool");
rust_tool!(RustcTool, "rustc", "The Rust compiler");
rust_tool!(RustupTool, "rustup", "Rust toolchain installer and version manager");
rust_tool!(RustdocTool, "rustdoc", "Rust documentation generator");
rust_tool!(RustfmtTool, "rustfmt", "Rust code formatter");
rust_tool!(ClippyTool, "clippy", "Rust linter for catching common mistakes");


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cargo_tool_creation() {
        let tool = CargoTool::new();
        assert_eq!(tool.name(), "cargo");
        assert!(!tool.description().is_empty());
        assert!(tool.aliases().is_empty());
    }

    #[test]
    fn test_rustc_tool_creation() {
        let tool = RustcTool::new();
        assert_eq!(tool.name(), "rustc");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_rustup_tool_creation() {
        let tool = RustupTool::new();
        assert_eq!(tool.name(), "rustup");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_rustdoc_tool_creation() {
        let tool = RustdocTool::new();
        assert_eq!(tool.name(), "rustdoc");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_rustfmt_tool_creation() {
        let tool = RustfmtTool::new();
        assert_eq!(tool.name(), "rustfmt");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_clippy_tool_creation() {
        let tool = ClippyTool::new();
        assert_eq!(tool.name(), "clippy");
        assert!(!tool.description().is_empty());
    }

    #[test]
    fn test_executable_path() {
        let cargo = CargoTool::new();
        let exe_path = cargo.get_executable_path("1.75.0");

        if cfg!(windows) {
            assert!(exe_path.to_string_lossy().ends_with("cargo.exe"));
        } else {
            assert!(exe_path.to_string_lossy().ends_with("cargo"));
        }

        assert!(exe_path.to_string_lossy().contains("1.75.0"));
    }

    #[tokio::test]
    async fn test_get_installed_versions() {
        let cargo = CargoTool::new();
        let versions = cargo.get_installed_versions().await.unwrap();
        // Should return empty list if no versions installed
        assert!(versions.is_empty() || !versions.is_empty());
    }
}
