//! 7z archive format handler
//!
//! Provides support for extracting .7z archives using the sevenz-rust library.

use crate::{Error, Result, progress::ProgressContext};
use std::path::{Path, PathBuf};

/// Handler for 7z archive format
pub struct SevenZipHandler;

impl SevenZipHandler {
    /// Create a new 7z handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for SevenZipHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl super::FormatHandler for SevenZipHandler {
    fn name(&self) -> &str {
        "7z"
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        file_path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("7z"))
    }

    async fn extract(
        &self,
        source_path: &Path,
        target_dir: &Path,
        progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        progress.start("Extracting 7z archive...", None).await?;

        // Create target directory if it doesn't exist
        std::fs::create_dir_all(target_dir)?;

        // Use sevenz-rust to decompress
        let source = source_path.to_path_buf();
        let target = target_dir.to_path_buf();
        let source_for_error = source_path.to_path_buf();

        // Run extraction in blocking task since sevenz-rust is sync
        let extracted_files = tokio::task::spawn_blocking(move || {
            sevenz_rust::decompress_file(&source, &target).map_err(|e| {
                Error::extraction_failed(&source, format!("7z extraction failed: {}", e))
            })?;

            // Collect all extracted files
            let mut files = Vec::new();
            collect_files(&target, &mut files)?;
            Ok::<_, Error>(files)
        })
        .await
        .map_err(|e| {
            Error::extraction_failed(
                &source_for_error,
                format!("7z extraction task failed: {}", e),
            )
        })??;

        progress
            .finish(&format!("Extracted {} files", extracted_files.len()))
            .await?;

        Ok(extracted_files)
    }
}

/// Recursively collect all files in a directory
fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, files)?;
            } else {
                files.push(path);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::FormatHandler;

    #[test]
    fn test_can_handle_7z() {
        let handler = SevenZipHandler::new();
        assert!(handler.can_handle(Path::new("archive.7z")));
        assert!(handler.can_handle(Path::new("archive.7Z")));
        assert!(!handler.can_handle(Path::new("archive.zip")));
        assert!(!handler.can_handle(Path::new("archive.tar.gz")));
    }

    #[test]
    fn test_handler_name() {
        let handler = SevenZipHandler::new();
        assert_eq!(handler.name(), "7z");
    }
}
