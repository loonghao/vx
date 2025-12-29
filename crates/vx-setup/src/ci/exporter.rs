//! Path exporter for CI environments

use super::CiProvider;
use crate::error::{SetupError, SetupResult};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Result of path export operation
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Number of paths exported
    pub paths_count: usize,
    /// Target file (if any)
    pub target_file: Option<String>,
    /// CI provider used
    pub provider: CiProvider,
    /// Human-readable message
    pub message: String,
    /// Shell commands for manual export (if no target file)
    pub shell_commands: Option<String>,
}

/// Path exporter for CI environments
pub struct PathExporter {
    /// CI provider
    provider: CiProvider,
    /// Custom path file (overrides provider detection)
    custom_path_file: Option<String>,
}

impl PathExporter {
    /// Create a new path exporter
    pub fn new(provider: CiProvider) -> Self {
        Self {
            provider,
            custom_path_file: None,
        }
    }

    /// Create a path exporter with auto-detected provider
    pub fn auto() -> Self {
        Self::new(CiProvider::detect())
    }

    /// Set custom path file
    pub fn with_custom_path_file(mut self, file: impl Into<String>) -> Self {
        self.custom_path_file = Some(file.into());
        self
    }

    /// Get the CI provider
    pub fn provider(&self) -> CiProvider {
        self.provider
    }

    /// Check if running in CI
    pub fn is_ci(&self) -> bool {
        self.provider.is_ci()
    }

    /// Export paths to CI environment
    ///
    /// # Arguments
    /// * `paths` - List of paths to export
    ///
    /// # Returns
    /// * `ExportResult` with details about the export operation
    pub fn export(&self, paths: &[PathBuf]) -> SetupResult<ExportResult> {
        if paths.is_empty() {
            return Ok(ExportResult {
                paths_count: 0,
                target_file: None,
                provider: self.provider,
                message: "No paths to export".to_string(),
                shell_commands: None,
            });
        }

        // Get target file (custom or from provider)
        let path_file = self
            .custom_path_file
            .clone()
            .or_else(|| self.provider.path_export_file());

        match path_file {
            Some(file) => self.export_to_file(paths, &file),
            None => self.generate_shell_commands(paths),
        }
    }

    /// Export paths to a file
    fn export_to_file(&self, paths: &[PathBuf], file: &str) -> SetupResult<ExportResult> {
        let mut content = String::new();
        for path in paths {
            content.push_str(&path.display().to_string());
            content.push('\n');
        }

        fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(file)
            .map_err(|e| SetupError::PathExportFailed(format!("Failed to open {}: {}", file, e)))?
            .write_all(content.as_bytes())
            .map_err(|e| {
                SetupError::PathExportFailed(format!("Failed to write to {}: {}", file, e))
            })?;

        Ok(ExportResult {
            paths_count: paths.len(),
            target_file: Some(file.to_string()),
            provider: self.provider,
            message: format!(
                "Exported {} paths to {} ({})",
                paths.len(),
                file,
                self.provider
            ),
            shell_commands: None,
        })
    }

    /// Generate shell commands for manual export
    fn generate_shell_commands(&self, paths: &[PathBuf]) -> SetupResult<ExportResult> {
        let mut commands = String::new();
        commands.push_str(&format!("# {} detected\n", self.provider));
        commands.push_str("# Add these paths to your PATH:\n");

        for path in paths {
            commands.push_str(&format!("export PATH=\"{}:$PATH\"\n", path.display()));
        }

        Ok(ExportResult {
            paths_count: paths.len(),
            target_file: None,
            provider: self.provider,
            message: format!(
                "{} detected - {} paths ready for manual export",
                self.provider,
                paths.len()
            ),
            shell_commands: Some(commands),
        })
    }
}

impl Default for PathExporter {
    fn default() -> Self {
        Self::auto()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_export_empty_paths() {
        let exporter = PathExporter::new(CiProvider::None);
        let result = exporter.export(&[]).unwrap();
        assert_eq!(result.paths_count, 0);
        assert!(result.message.contains("No paths"));
    }

    #[test]
    fn test_export_to_file() {
        let temp_dir = tempdir().unwrap();
        let path_file = temp_dir.path().join("github_path");

        let exporter = PathExporter::new(CiProvider::GitHub)
            .with_custom_path_file(path_file.to_str().unwrap());

        let paths = vec![
            PathBuf::from("/usr/local/bin"),
            PathBuf::from("/home/user/.vx/bin"),
        ];

        let result = exporter.export(&paths).unwrap();
        assert_eq!(result.paths_count, 2);
        assert!(result.target_file.is_some());

        // Verify file contents
        let content = fs::read_to_string(&path_file).unwrap();
        assert!(content.contains("/usr/local/bin"));
        assert!(content.contains("/home/user/.vx/bin"));
    }

    #[test]
    fn test_generate_shell_commands() {
        let exporter = PathExporter::new(CiProvider::None);
        let paths = vec![PathBuf::from("/usr/local/bin")];

        let result = exporter.export(&paths).unwrap();
        assert!(result.shell_commands.is_some());
        let commands = result.shell_commands.unwrap();
        assert!(commands.contains("export PATH="));
        assert!(commands.contains("/usr/local/bin"));
    }
}
