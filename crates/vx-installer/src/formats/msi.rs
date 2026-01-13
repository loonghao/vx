//! MSI (Windows Installer) handler for Windows packages
//!
//! This handler uses msiexec to install MSI packages silently to a specified directory.
//! It extracts files from the MSI to our managed directory structure.

use super::FormatHandler;
use crate::{progress::ProgressContext, Error, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Handler for MSI files (Windows Installer packages)
#[cfg(windows)]
pub struct MsiHandler;

#[cfg(windows)]
impl MsiHandler {
    /// Create a new MSI handler
    pub fn new() -> Self {
        Self
    }

    /// Extract files from MSI using msiexec
    async fn extract_msi(&self, source_path: &Path, target_dir: &Path) -> Result<()> {
        // Create target directory
        std::fs::create_dir_all(target_dir)?;

        // Use msiexec in administrative installation mode (/a)
        // This extracts files without actually installing to the system
        let output = Command::new("msiexec")
            .arg("/a") // Administrative installation
            .arg(source_path)
            .arg("/qn") // Quiet mode, no UI
            .arg("/norestart") // Don't restart
            .arg(format!("TARGETDIR={}", target_dir.display()))
            .output()
            .map_err(Error::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ExtractionFailed {
                archive_path: source_path.to_path_buf(),
                reason: format!("msiexec failed: {}", stderr),
            });
        }

        Ok(())
    }

    /// Find all executable files in the extracted directory
    fn find_all_executables(&self, dir: &Path) -> Vec<PathBuf> {
        let mut executables = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && self.is_executable(&path) {
                    executables.push(path);
                } else if path.is_dir() {
                    // Recursively search subdirectories
                    executables.extend(self.find_all_executables(&path));
                }
            }
        }

        executables
    }

    /// Check if a file is an executable
    fn is_executable(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| {
                let e_lower = e.to_lowercase();
                matches!(e_lower.as_str(), "exe" | "cmd" | "bat" | "com")
            })
            .unwrap_or(false)
    }
}

#[cfg(windows)]
#[async_trait::async_trait]
impl FormatHandler for MsiHandler {
    fn name(&self) -> &str {
        "msi"
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("msi"))
            .unwrap_or(false)
    }

    async fn extract(
        &self,
        source_path: &Path,
        target_dir: &Path,
        progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        progress
            .start("Extracting MSI package", Some(1))
            .await?;

        // Extract MSI to target directory
        self.extract_msi(source_path, target_dir).await?;

        progress.increment(1).await?;

        // Find all executables in the extracted directory
        let executables = self.find_all_executables(target_dir);

        if executables.is_empty() {
            return Err(Error::ExecutableNotFound {
                tool_name: "unknown".to_string(),
                search_path: target_dir.to_path_buf(),
            });
        }

        progress
            .finish("MSI package extraction completed")
            .await?;

        Ok(executables)
    }
}

#[cfg(windows)]
impl Default for MsiHandler {
    fn default() -> Self {
        Self::new()
    }
}

// Non-Windows stub implementation
#[cfg(not(windows))]
pub struct MsiHandler;

#[cfg(not(windows))]
impl MsiHandler {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(windows))]
#[async_trait::async_trait]
impl FormatHandler for MsiHandler {
    fn name(&self) -> &str {
        "msi"
    }

    fn can_handle(&self, _file_path: &Path) -> bool {
        false // MSI is Windows-only
    }

    async fn extract(
        &self,
        _source_path: &Path,
        _target_dir: &Path,
        _progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        Err(Error::UnsupportedFormat {
            format: "msi".to_string(),
            message: "MSI format is only supported on Windows".to_string(),
        })
    }
}

#[cfg(not(windows))]
impl Default for MsiHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    #[test]
    fn test_msi_handler_name() {
        let handler = MsiHandler::new();
        assert_eq!(handler.name(), "msi");
    }

    #[test]
    fn test_can_handle_msi() {
        let handler = MsiHandler::new();
        assert!(handler.can_handle(Path::new("package.msi")));
        assert!(handler.can_handle(Path::new("PACKAGE.MSI")));
        assert!(!handler.can_handle(Path::new("package.exe")));
        assert!(!handler.can_handle(Path::new("package.zip")));
    }
}
