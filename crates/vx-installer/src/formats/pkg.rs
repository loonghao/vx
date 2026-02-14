//! macOS Package (.pkg) handler
//!
//! This handler uses `pkgutil --expand-full` to extract macOS .pkg packages to a specified
//! directory. macOS .pkg files are XAR archives containing one or more component packages,
//! each with a Payload (cpio+gzip compressed files).
//!
//! The extraction process:
//! 1. `pkgutil --expand-full` expands the .pkg into a directory structure
//! 2. Each component package gets a subdirectory with its Payload fully extracted
//! 3. We search the expanded Payload directories for executables
//!
//! This handler is macOS-only. On other platforms, it acts as a stub that always
//! returns `can_handle() == false`.

use super::FormatHandler;
use crate::{Error, Result, progress::ProgressContext};
use std::path::{Path, PathBuf};

// ============================================================================
// macOS implementation
// ============================================================================

#[cfg(target_os = "macos")]
use std::process::Command;

/// Handler for macOS .pkg files (flat packages)
#[cfg(target_os = "macos")]
pub struct PkgHandler;

#[cfg(target_os = "macos")]
impl PkgHandler {
    /// Create a new PKG handler
    pub fn new() -> Self {
        Self
    }

    /// Extract files from .pkg using `pkgutil --expand-full`
    ///
    /// `pkgutil --expand-full` produces a directory structure like:
    /// ```text
    /// output_dir/
    /// ├── Distribution          (XML manifest)
    /// ├── Resources/            (localization, scripts)
    /// └── component.pkg/
    ///     ├── PackageInfo
    ///     ├── Bom
    ///     ├── Scripts/
    ///     └── Payload/          (fully expanded files)
    ///         └── usr/
    ///             └── local/
    ///                 └── bin/
    ///                     └── mytool
    /// ```
    async fn extract_pkg(&self, source_path: &Path, target_dir: &Path) -> Result<()> {
        std::fs::create_dir_all(target_dir)?;

        // pkgutil --expand-full expands the .pkg and fully extracts Payload
        let output = Command::new("pkgutil")
            .arg("--expand-full")
            .arg(source_path)
            .arg(target_dir)
            .output()
            .map_err(Error::Io)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::ExtractionFailed {
                archive_path: source_path.to_path_buf(),
                reason: format!("pkgutil --expand-full failed: {}", stderr),
            });
        }

        Ok(())
    }

    /// Recursively find all executable files in the extracted directory
    ///
    /// After `pkgutil --expand-full`, executables are typically nested under
    /// `<component>.pkg/Payload/...`. This method searches recursively through
    /// all Payload directories.
    fn find_all_executables(&self, dir: &Path) -> Vec<PathBuf> {
        let mut executables = Vec::new();

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && self.is_executable_file(&path) {
                    executables.push(path);
                } else if path.is_dir() {
                    executables.extend(self.find_all_executables(&path));
                }
            }
        }

        executables
    }

    /// Check if a file is executable (Unix permissions)
    fn is_executable_file(&self, path: &Path) -> bool {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = std::fs::metadata(path) {
            let permissions = metadata.permissions();
            permissions.mode() & 0o111 != 0
        } else {
            false
        }
    }
}

#[cfg(target_os = "macos")]
#[async_trait::async_trait]
impl FormatHandler for PkgHandler {
    fn name(&self) -> &str {
        "pkg"
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("pkg"))
            .unwrap_or(false)
    }

    async fn extract(
        &self,
        source_path: &Path,
        target_dir: &Path,
        progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        progress
            .start("Extracting macOS package (.pkg)", Some(1))
            .await?;

        // Use a temporary expand directory, then promote files
        let expand_dir = target_dir.join(".pkg_expand");
        self.extract_pkg(source_path, &expand_dir).await?;

        progress.increment(1).await?;

        // Find all Payload directories and move their contents to target_dir
        Self::promote_payload_contents(&expand_dir, target_dir)?;

        // Clean up the expand directory
        let _ = std::fs::remove_dir_all(&expand_dir);

        // Find all executables in the final directory
        let executables = self.find_all_executables(target_dir);

        if executables.is_empty() {
            return Err(Error::ExecutableNotFound {
                tool_name: "unknown".to_string(),
                search_path: target_dir.to_path_buf(),
            });
        }

        progress
            .finish("macOS package extraction completed")
            .await?;

        Ok(executables)
    }
}

#[cfg(target_os = "macos")]
impl PkgHandler {
    /// Promote files from Payload directories to the target directory.
    ///
    /// After `pkgutil --expand-full`, files are nested like:
    ///   `expand_dir/<component>.pkg/Payload/<actual files>`
    ///
    /// This method moves the Payload contents up to `target_dir` so the
    /// directory structure matches what other handlers produce.
    fn promote_payload_contents(expand_dir: &Path, target_dir: &Path) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(expand_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Check for Payload subdirectory (component packages)
                    let payload_dir = path.join("Payload");
                    if payload_dir.exists() && payload_dir.is_dir() {
                        Self::copy_dir_contents(&payload_dir, target_dir)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Recursively copy contents of src_dir into dst_dir
    fn copy_dir_contents(src_dir: &Path, dst_dir: &Path) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir(src_dir) {
            for entry in entries.flatten() {
                let src_path = entry.path();
                let file_name = match src_path.file_name() {
                    Some(name) => name.to_owned(),
                    None => continue,
                };
                let dst_path = dst_dir.join(&file_name);

                if src_path.is_dir() {
                    std::fs::create_dir_all(&dst_path)?;
                    Self::copy_dir_contents(&src_path, &dst_path)?;
                } else {
                    // Ensure parent exists
                    if let Some(parent) = dst_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::rename(&src_path, &dst_path).or_else(|_| {
                        // rename may fail across filesystems, fall back to copy
                        std::fs::copy(&src_path, &dst_path).map(|_| ())
                    })?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
impl Default for PkgHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Non-macOS stub implementation
// ============================================================================

/// Stub handler for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub struct PkgHandler;

#[cfg(not(target_os = "macos"))]
impl PkgHandler {
    /// Create a new PKG handler (stub on non-macOS)
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(target_os = "macos"))]
#[async_trait::async_trait]
impl FormatHandler for PkgHandler {
    fn name(&self) -> &str {
        "pkg"
    }

    fn can_handle(&self, _file_path: &Path) -> bool {
        false // PKG is macOS-only
    }

    async fn extract(
        &self,
        _source_path: &Path,
        _target_dir: &Path,
        _progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        Err(Error::UnsupportedFormat {
            format: "pkg (macOS-only)".to_string(),
        })
    }
}

#[cfg(not(target_os = "macos"))]
impl Default for PkgHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests (platform-aware)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkg_handler_name() {
        let handler = PkgHandler::new();
        assert_eq!(handler.name(), "pkg");
    }

    // On macOS, can_handle should return true for .pkg files
    #[cfg(target_os = "macos")]
    #[test]
    fn test_can_handle_pkg_on_macos() {
        let handler = PkgHandler::new();
        assert!(handler.can_handle(Path::new("package.pkg")));
        assert!(handler.can_handle(Path::new("PACKAGE.PKG")));
        assert!(handler.can_handle(Path::new("/tmp/tool-v1.0.pkg")));
        assert!(!handler.can_handle(Path::new("package.zip")));
        assert!(!handler.can_handle(Path::new("package.tar.gz")));
        assert!(!handler.can_handle(Path::new("package.dmg")));
    }

    // On non-macOS, can_handle should always return false
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn test_can_handle_pkg_on_non_macos() {
        let handler = PkgHandler::new();
        assert!(!handler.can_handle(Path::new("package.pkg")));
        assert!(!handler.can_handle(Path::new("PACKAGE.PKG")));
        assert!(!handler.can_handle(Path::new("package.zip")));
    }

    #[cfg(not(target_os = "macos"))]
    #[tokio::test]
    async fn test_extract_returns_unsupported_on_non_macos() {
        let handler = PkgHandler::new();
        let progress = ProgressContext::disabled();
        let result = handler
            .extract(Path::new("test.pkg"), Path::new("/tmp/out"), &progress)
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("pkg (macOS-only)"),
            "Expected macOS-only error, got: {}",
            err
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_is_executable_file() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let exec_path = temp_dir.path().join("test_exec");
        std::fs::write(&exec_path, "#!/bin/sh\necho hello").unwrap();

        // Set executable permission
        let mut perms = std::fs::metadata(&exec_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&exec_path, perms).unwrap();

        let handler = PkgHandler::new();
        assert!(handler.is_executable_file(&exec_path));

        // Non-executable file
        let non_exec_path = temp_dir.path().join("test_non_exec");
        std::fs::write(&non_exec_path, "just text").unwrap();
        let mut perms = std::fs::metadata(&non_exec_path).unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(&non_exec_path, perms).unwrap();

        assert!(!handler.is_executable_file(&non_exec_path));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_promote_payload_contents() {
        let temp_dir = tempfile::tempdir().unwrap();
        let expand_dir = temp_dir.path().join("expand");
        let target_dir = temp_dir.path().join("target");

        // Simulate pkgutil --expand-full output structure
        let component_dir = expand_dir.join("myapp.pkg");
        let payload_dir = component_dir.join("Payload");
        let bin_dir = payload_dir.join("usr").join("local").join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("mytool"), "binary content").unwrap();

        std::fs::create_dir_all(&target_dir).unwrap();

        PkgHandler::promote_payload_contents(&expand_dir, &target_dir).unwrap();

        // Verify the file was promoted correctly
        let promoted_path = target_dir
            .join("usr")
            .join("local")
            .join("bin")
            .join("mytool");
        assert!(
            promoted_path.exists(),
            "File should be promoted from Payload to target_dir"
        );

        let content = std::fs::read_to_string(&promoted_path).unwrap();
        assert_eq!(content, "binary content");
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_promote_payload_multiple_components() {
        let temp_dir = tempfile::tempdir().unwrap();
        let expand_dir = temp_dir.path().join("expand");
        let target_dir = temp_dir.path().join("target");

        // Simulate multiple component packages
        let comp1_payload = expand_dir.join("comp1.pkg").join("Payload");
        let comp2_payload = expand_dir.join("comp2.pkg").join("Payload");

        std::fs::create_dir_all(comp1_payload.join("bin")).unwrap();
        std::fs::create_dir_all(comp2_payload.join("lib")).unwrap();

        std::fs::write(comp1_payload.join("bin").join("tool1"), "tool1").unwrap();
        std::fs::write(comp2_payload.join("lib").join("libfoo.dylib"), "lib").unwrap();

        std::fs::create_dir_all(&target_dir).unwrap();

        PkgHandler::promote_payload_contents(&expand_dir, &target_dir).unwrap();

        assert!(target_dir.join("bin").join("tool1").exists());
        assert!(target_dir.join("lib").join("libfoo.dylib").exists());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_find_all_executables() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::tempdir().unwrap();
        let bin_dir = temp_dir.path().join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();

        // Create executable
        let exec_path = bin_dir.join("mytool");
        std::fs::write(&exec_path, "#!/bin/sh").unwrap();
        let mut perms = std::fs::metadata(&exec_path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&exec_path, perms).unwrap();

        // Create non-executable
        let txt_path = bin_dir.join("readme.txt");
        std::fs::write(&txt_path, "readme").unwrap();

        let handler = PkgHandler::new();
        let executables = handler.find_all_executables(temp_dir.path());

        assert_eq!(executables.len(), 1);
        assert!(executables[0].ends_with("mytool"));
    }
}
