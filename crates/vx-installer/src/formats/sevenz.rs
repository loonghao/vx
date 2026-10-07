//! 7z archive format handler
//!
//! Provides support for extracting .7z archives and 7z SFX executables using the sevenz-rust library.
//!
//! ## SFX Support
//!
//! Many tools distribute as Self-Extracting Archives (SFX) with a `.exe` extension.
//! For example, 7-Zip itself ships as `7z2500-x64.exe` which is a PE executable
//! with an embedded 7z archive. The handler finds the 7z signature
//! (`37 7A BC AF 27 1C`) and supplies an archive starting at that offset to
//! `sevenz-rust`, whose seek offsets are relative to the archive header.
//!
//! This handler detects SFX files by reading the file magic bytes rather than
//! relying solely on the file extension.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Component, Path, PathBuf};

use crate::{Error, Result, progress::ProgressContext};

/// 7z archive magic bytes: `7z\xBC\xAF\x27\x1C`
const SEVENZ_MAGIC: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];
const MAX_SIGNATURE_SCAN: u64 = 4 * 1024 * 1024;

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
    /// For SFX `.exe` files the signature appears after the PE stub.
    fn has_sevenz_signature(file_path: &Path) -> bool {
        let Ok(mut file) = File::open(file_path) else {
            return false;
        };
        signature_offset(&mut file).ok().flatten().is_some()
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
            let archive = open_archive(&source).map_err(|error| {
                Error::extraction_failed(&source, format!("Failed to open 7z archive: {error}"))
            })?;
            let mut files = Vec::new();
            sevenz_rust::decompress_with_extract_fn(archive, &target, |entry, reader, _| {
                let path =
                    safe_entry_path(&target, entry.name()).map_err(sevenz_rust::Error::io)?;
                sevenz_rust::default_entry_extract_fn(entry, reader, &path)?;
                if !entry.is_directory() {
                    files.push(path);
                }
                Ok(true)
            })
            .map_err(|error| {
                Error::extraction_failed(&source, format!("7z extraction failed: {error}"))
            })?;
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

/// Find a signature within the same bounded prefix used for SFX detection.
fn signature_offset(file: &mut File) -> io::Result<Option<u64>> {
    file.rewind()?;
    let mut prefix = Vec::new();
    file.take(MAX_SIGNATURE_SCAN).read_to_end(&mut prefix)?;
    Ok(prefix
        .windows(SEVENZ_MAGIC.len())
        .position(|bytes| bytes == SEVENZ_MAGIC)
        .map(|offset| offset as u64))
}

/// Remove the SFX stub without buffering the full download in memory.
fn open_archive(source: &Path) -> io::Result<File> {
    let mut file = File::open(source)?;
    let offset = signature_offset(&mut file)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "7z signature not found within the first 4 MiB",
        )
    })?;
    file.seek(SeekFrom::Start(offset))?;
    if offset == 0 {
        return Ok(file);
    }

    // sevenz-rust seeks from byte zero; an unnamed temporary file keeps those
    // offsets correct and is removed automatically on success or failure.
    let mut archive = tempfile::tempfile()?;
    io::copy(&mut file, &mut archive)?;
    archive.rewind()?;
    Ok(archive)
}

/// Reject archive paths that could leave the extraction directory.
fn safe_entry_path(target: &Path, name: &str) -> io::Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    let relative = Path::new(&normalized);
    if normalized.contains(':')
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Unsafe path in 7z archive",
        ));
    }
    let path = target.join(relative);
    for ancestor in path.ancestors().take_while(|path| path.starts_with(target)) {
        match ancestor.symlink_metadata() {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Symlink in 7z extraction path",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
    }
    Ok(path)
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
