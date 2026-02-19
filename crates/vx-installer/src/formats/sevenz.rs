//! 7z archive format handler
//!
//! Provides support for extracting .7z archives and 7z SFX executables using the sevenz-rust library.
//!
//! ## SFX Support
//!
//! Many tools distribute as Self-Extracting Archives (SFX) with a `.exe` extension.
//! For example, 7-Zip itself ships as `7z2500-x64.exe` which is a PE executable
//! with an embedded 7z archive. `sevenz-rust` handles this transparently by scanning
//! for the 7z signature (`37 7A BC AF 27 1C`) within the file.
//!
//! This handler detects SFX files by reading the file magic bytes rather than
//! relying solely on the file extension.

use crate::{Error, Result, progress::ProgressContext};
use std::path::{Path, PathBuf};

/// 7z archive magic bytes: `7z\xBC\xAF\x27\x1C`
const SEVENZ_MAGIC: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

/// Handler for 7z archive format and 7z SFX executables
pub struct SevenZipHandler;

impl SevenZipHandler {
    /// Create a new 7z handler
    pub fn new() -> Self {
        Self
    }

    /// Check if a file contains a 7z archive signature.
    ///
    /// For plain `.7z` files the signature is at offset 0.
    /// For SFX `.exe` files the signature appears after the PE stub —
    /// `sevenz-rust` scans for it automatically, so we just need to confirm
    /// the signature exists somewhere in the first few MB of the file.
    fn has_sevenz_signature(file_path: &Path) -> bool {
        use std::io::Read;

        let Ok(mut file) = std::fs::File::open(file_path) else {
            return false;
        };

        // Read up to 4 MB to find the embedded 7z signature in SFX files.
        // Real 7z archives have the signature at byte 0; SFX stubs are typically
        // a few hundred KB at most.
        const MAX_SCAN: usize = 4 * 1024 * 1024;
        let mut buf = vec![
            0u8;
            MAX_SCAN.min(
                file_path
                    .metadata()
                    .map(|m| m.len() as usize)
                    .unwrap_or(MAX_SCAN),
            )
        ];

        let n = file.read(&mut buf).unwrap_or(0);
        let buf = &buf[..n];

        // Search for the 7z magic signature
        buf.windows(SEVENZ_MAGIC.len()).any(|w| w == SEVENZ_MAGIC)
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
        let ext = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            // Plain 7z archive — always handle
            "7z" => true,
            // SFX executable — verify by magic bytes
            "exe" => Self::has_sevenz_signature(file_path),
            _ => false,
        }
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

    #[test]
    fn test_can_handle_sfx_exe_with_magic() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a fake SFX: some PE stub bytes followed by the 7z magic
        let mut tmp = NamedTempFile::with_suffix(".exe").unwrap();
        // Simulate a small PE stub (just zeros) then the 7z signature
        tmp.write_all(&[0u8; 512]).unwrap();
        tmp.write_all(SEVENZ_MAGIC).unwrap();
        tmp.flush().unwrap();

        let handler = SevenZipHandler::new();
        assert!(
            handler.can_handle(tmp.path()),
            "SFX exe with embedded 7z signature should be handled"
        );
    }

    #[test]
    fn test_cannot_handle_plain_exe() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // A plain PE executable without any 7z signature
        let mut tmp = NamedTempFile::with_suffix(".exe").unwrap();
        tmp.write_all(b"MZ\x90\x00this is a plain exe without 7z magic")
            .unwrap();
        tmp.flush().unwrap();

        let handler = SevenZipHandler::new();
        assert!(
            !handler.can_handle(tmp.path()),
            "Plain exe without 7z signature should NOT be handled"
        );
    }

    #[test]
    fn test_has_sevenz_signature_at_offset_zero() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut tmp = NamedTempFile::with_suffix(".7z").unwrap();
        tmp.write_all(SEVENZ_MAGIC).unwrap();
        tmp.write_all(&[0u8; 64]).unwrap();
        tmp.flush().unwrap();

        assert!(SevenZipHandler::has_sevenz_signature(tmp.path()));
    }
}
