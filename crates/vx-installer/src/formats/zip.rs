//! ZIP archive format handler

use super::FormatHandler;
use crate::{Error, Result, progress::ProgressContext};
use std::path::{Path, PathBuf};

/// Handler for ZIP archive format
pub struct ZipHandler;

impl ZipHandler {
    /// Create a new ZIP handler
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl FormatHandler for ZipHandler {
    fn name(&self) -> &str {
        "zip"
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            filename.ends_with(".zip")
        } else {
            false
        }
    }

    async fn extract(
        &self,
        source_path: &Path,
        target_dir: &Path,
        progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        // Ensure target directory exists
        std::fs::create_dir_all(target_dir)?;

        // Open the ZIP file
        let file = std::fs::File::open(source_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| {
            Error::extraction_failed(source_path, format!("Failed to open ZIP archive: {}", e))
        })?;

        let total_files = archive.len();
        progress
            .start(
                &format!("Extracting {} files", total_files),
                Some(total_files as u64),
            )
            .await?;

        let mut extracted_files = Vec::new();

        // Extract each file
        for i in 0..total_files {
            let mut file = archive.by_index(i).map_err(|e| {
                Error::extraction_failed(
                    source_path,
                    format!("Failed to access file at index {}: {}", i, e),
                )
            })?;

            let file_path = match file.enclosed_name() {
                Some(path) => target_dir.join(path),
                None => {
                    // Skip files with invalid names
                    progress.increment(1).await?;
                    continue;
                }
            };

            // On Windows, check path length and use extended-length path if needed
            #[cfg(windows)]
            let file_path = {
                use vx_paths::windows::{PathLengthStatus, check_path_length, to_long_path};

                match check_path_length(&file_path) {
                    PathLengthStatus::TooLong { length, .. } => {
                        tracing::warn!(
                            "Path length ({}) exceeds Windows MAX_PATH limit, using extended path: {}",
                            length,
                            file_path.display()
                        );
                        to_long_path(&file_path)
                    }
                    PathLengthStatus::Warning { length, .. } => {
                        tracing::debug!(
                            "Path length ({}) approaching Windows limit: {}",
                            length,
                            file_path.display()
                        );
                        file_path
                    }
                    PathLengthStatus::Safe => file_path,
                }
            };

            // Create parent directories
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Extract the file
            if file.is_dir() {
                // Create directory
                std::fs::create_dir_all(&file_path)?;
            } else {
                // Extract file
                let mut output_file = std::fs::File::create(&file_path)?;
                std::io::copy(&mut file, &mut output_file)?;

                // Make executable if needed
                #[cfg(unix)]
                {
                    if file.unix_mode().unwrap_or(0) & 0o111 != 0 {
                        self.make_executable(&file_path)?;
                    }
                }

                // Store the original path (without extended prefix) for return value
                #[cfg(windows)]
                {
                    use vx_paths::windows::from_long_path;
                    extracted_files.push(from_long_path(&file_path));
                }

                #[cfg(not(windows))]
                {
                    extracted_files.push(file_path);
                }
            }

            progress.increment(1).await?;
        }

        progress.finish("Extraction completed").await?;

        Ok(extracted_files)
    }
}

impl Default for ZipHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_zip_handler_can_handle() {
        let handler = ZipHandler::new();

        assert!(handler.can_handle(Path::new("test.zip")));
        assert!(!handler.can_handle(Path::new("test.tar.gz")));
        assert!(!handler.can_handle(Path::new("test.exe")));
    }

    #[tokio::test]
    async fn test_zip_handler_name() {
        let handler = ZipHandler::new();
        assert_eq!(handler.name(), "zip");
    }

    // Note: More comprehensive tests would require creating actual ZIP files
    // This is a basic structure test
}
