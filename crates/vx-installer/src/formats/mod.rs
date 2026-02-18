//! Archive format handling for vx-installer
//!
//! This module provides a unified interface for handling different archive formats
//! and installation methods. It abstracts the complexity of different compression
//! formats and provides a consistent API for extraction and installation.

use crate::{Error, Result, progress::ProgressContext};
use std::path::{Path, PathBuf};

pub mod binary;
pub mod msi;
pub mod pkg;
#[cfg(feature = "extended-formats")]
pub mod sevenz;
pub mod tar;
pub mod zip;

/// Trait for handling different archive formats and installation methods
#[async_trait::async_trait]
pub trait FormatHandler: Send + Sync {
    /// Get the name of this format handler
    fn name(&self) -> &str;

    /// Check if this handler can process the given file
    fn can_handle(&self, file_path: &Path) -> bool;

    /// Extract or install the file to the target directory
    async fn extract(
        &self,
        source_path: &Path,
        target_dir: &Path,
        progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>>;

    /// Get the expected executable name for a tool
    fn get_executable_name(&self, tool_name: &str) -> String {
        if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        }
    }

    /// Find executable files in the extracted directory
    fn find_executables(&self, dir: &Path, tool_name: &str) -> Result<Vec<PathBuf>> {
        let exe_name = self.get_executable_name(tool_name);
        let mut executables = Vec::new();

        // Search for the executable in common locations
        let search_paths = vec![
            dir.to_path_buf(),
            dir.join("bin"),
            dir.join("usr").join("bin"),
            dir.join("usr").join("local").join("bin"),
        ];

        for search_path in search_paths {
            if !search_path.exists() {
                continue;
            }

            // Direct match
            let exe_path = search_path.join(&exe_name);
            if exe_path.exists() && exe_path.is_file() {
                executables.push(exe_path);
                continue;
            }

            // Search in subdirectories
            if let Ok(entries) = std::fs::read_dir(&search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && let Some(filename) = path.file_name().and_then(|n| n.to_str())
                    {
                        // Exact match or partial match (for tools with version suffixes)
                        if filename == exe_name
                            || (filename.starts_with(tool_name) && self.is_executable(&path))
                        {
                            executables.push(path);
                        }
                    }
                }
            }
        }

        if executables.is_empty() {
            return Err(Error::executable_not_found(tool_name, dir));
        }

        Ok(executables)
    }

    /// Check if a file is executable
    fn is_executable(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let permissions = metadata.permissions();
                permissions.mode() & 0o111 != 0
            } else {
                false
            }
        }

        #[cfg(windows)]
        {
            // On Windows, check file extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "com")
            } else {
                false
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback: assume it's executable if it's a file
            path.is_file()
        }
    }

    /// Make a file executable on Unix systems
    #[cfg(unix)]
    fn make_executable(&self, path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = std::fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)?;

        Ok(())
    }

    /// Make a file executable (no-op on Windows)
    #[cfg(not(unix))]
    fn make_executable(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}

/// Archive extractor that delegates to specific format handlers
pub struct ArchiveExtractor {
    handlers: Vec<Box<dyn FormatHandler>>,
}

impl ArchiveExtractor {
    /// Create a new archive extractor with default handlers
    pub fn new() -> Self {
        #[cfg(feature = "extended-formats")]
        let handlers: Vec<Box<dyn FormatHandler>> = {
            let mut handlers = vec![
                Box::new(zip::ZipHandler::new()),
                Box::new(tar::TarHandler::new()),
                Box::new(msi::MsiHandler::new()),
                Box::new(pkg::PkgHandler::new()),
                Box::new(binary::BinaryHandler::new()), // Keep binary as fallback
            ];
            handlers.insert(2, Box::new(sevenz::SevenZipHandler::new()));
            handlers
        };

        #[cfg(not(feature = "extended-formats"))]
        let handlers: Vec<Box<dyn FormatHandler>> = vec![
            Box::new(zip::ZipHandler::new()),
            Box::new(tar::TarHandler::new()),
            Box::new(msi::MsiHandler::new()),
            Box::new(pkg::PkgHandler::new()),
            Box::new(binary::BinaryHandler::new()), // Keep binary as fallback
        ];

        Self { handlers }
    }

    /// Add a custom format handler
    pub fn with_handler(mut self, handler: Box<dyn FormatHandler>) -> Self {
        self.handlers.push(handler);
        self
    }

    /// Extract an archive using the appropriate handler
    pub async fn extract(
        &self,
        source_path: &Path,
        target_dir: &Path,
        progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        // Find a handler that can process this file
        for handler in &self.handlers {
            if handler.can_handle(source_path) {
                return handler.extract(source_path, target_dir, progress).await;
            }
        }

        Err(Error::unsupported_format(
            source_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown"),
        ))
    }

    /// Find the best executable from extracted files
    pub fn find_best_executable(
        &self,
        extracted_files: &[PathBuf],
        tool_name: &str,
    ) -> Result<PathBuf> {
        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };

        // First, look for exact matches
        for file in extracted_files {
            if let Some(filename) = file.file_name().and_then(|n| n.to_str())
                && filename == exe_name
            {
                return Ok(file.clone());
            }
        }

        // Then, look for partial matches
        for file in extracted_files {
            if let Some(filename) = file.file_name().and_then(|n| n.to_str())
                && filename.starts_with(tool_name)
                && self.is_executable_file(file)
            {
                return Ok(file.clone());
            }
        }

        // Finally, look for any executable in bin directories
        for file in extracted_files {
            if let Some(parent) = file.parent()
                && let Some(dir_name) = parent.file_name().and_then(|n| n.to_str())
                && dir_name == "bin"
                && self.is_executable_file(file)
            {
                return Ok(file.clone());
            }
        }

        Err(Error::executable_not_found(
            tool_name,
            extracted_files
                .first()
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new(".")),
        ))
    }

    /// Check if a file is executable
    fn is_executable_file(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let permissions = metadata.permissions();
                permissions.mode() & 0o111 != 0
            } else {
                false
            }
        }

        #[cfg(windows)]
        {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                matches!(ext.to_lowercase().as_str(), "exe" | "bat" | "cmd" | "com")
            } else {
                false
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            path.is_file()
        }
    }
}

impl Default for ArchiveExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility function to detect archive format from file extension
pub fn detect_format(file_path: &Path) -> Option<&str> {
    let filename = file_path.file_name()?.to_str()?;

    if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
        Some("tar.gz")
    } else if filename.ends_with(".tar.xz") || filename.ends_with(".txz") {
        Some("tar.xz")
    } else if filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") {
        Some("tar.bz2")
    } else if filename.ends_with(".tar.zst") || filename.ends_with(".tzst") {
        Some("tar.zst")
    } else if filename.ends_with(".zip") {
        Some("zip")
    } else if filename.ends_with(".msi") {
        Some("msi")
    } else if filename.ends_with(".pkg") {
        Some("pkg")
    } else if filename.ends_with(".7z") {
        Some("7z")
    } else {
        file_path.extension()?.to_str()
    }
}
