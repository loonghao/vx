//! TAR archive format handler (including compressed variants)

use super::FormatHandler;
use crate::{progress::ProgressContext, Error, Result};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Handler for TAR archive formats (tar, tar.gz, tar.xz, tar.bz2)
pub struct TarHandler;

impl TarHandler {
    /// Create a new TAR handler
    pub fn new() -> Self {
        Self
    }

    /// Detect the compression type from filename
    fn detect_compression(&self, file_path: &Path) -> CompressionType {
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
                CompressionType::Gzip
            } else if filename.ends_with(".tar.xz") || filename.ends_with(".txz") {
                CompressionType::Xz
            } else if filename.ends_with(".tar.bz2") || filename.ends_with(".tbz2") {
                CompressionType::Bzip2
            } else if filename.ends_with(".tar") {
                CompressionType::None
            } else {
                CompressionType::Unknown
            }
        } else {
            CompressionType::Unknown
        }
    }
}

/// Compression types supported for TAR archives
#[derive(Debug, Clone, Copy)]
enum CompressionType {
    None,
    Gzip,
    Xz,
    Bzip2,
    Unknown,
}

#[async_trait::async_trait]
impl FormatHandler for TarHandler {
    fn name(&self) -> &str {
        "tar"
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            filename.ends_with(".tar")
                || filename.ends_with(".tar.gz")
                || filename.ends_with(".tgz")
                || filename.ends_with(".tar.xz")
                || filename.ends_with(".txz")
                || filename.ends_with(".tar.bz2")
                || filename.ends_with(".tbz2")
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

        let compression = self.detect_compression(source_path);

        progress.start("Extracting TAR archive", None).await?;

        let file = std::fs::File::open(source_path)?;
        let mut extracted_files = Vec::new();

        match compression {
            CompressionType::None => {
                self.extract_tar(file, target_dir, &mut extracted_files)
                    .await?;
            }
            CompressionType::Gzip => {
                let decoder = flate2::read::GzDecoder::new(file);
                self.extract_tar(decoder, target_dir, &mut extracted_files)
                    .await?;
            }
            CompressionType::Xz => {
                // Note: xz support would require additional dependency
                return Err(Error::unsupported_format("tar.xz"));
            }
            CompressionType::Bzip2 => {
                // Note: bzip2 support would require additional dependency
                return Err(Error::unsupported_format("tar.bz2"));
            }
            CompressionType::Unknown => {
                return Err(Error::unsupported_format("unknown tar format"));
            }
        }

        progress.finish("TAR extraction completed").await?;

        Ok(extracted_files)
    }
}

impl TarHandler {
    /// Extract TAR archive from a reader
    async fn extract_tar<R: Read>(
        &self,
        reader: R,
        target_dir: &Path,
        extracted_files: &mut Vec<PathBuf>,
    ) -> Result<()> {
        let mut archive = tar::Archive::new(reader);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;
            let target_path = target_dir.join(&path);

            // Create parent directories
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Extract the entry
            if entry.header().entry_type().is_dir() {
                std::fs::create_dir_all(&target_path)?;
            } else {
                entry.unpack(&target_path)?;

                // Make executable if needed
                #[cfg(unix)]
                {
                    let mode = entry.header().mode()?;
                    if mode & 0o111 != 0 {
                        self.make_executable(&target_path)?;
                    }
                }

                extracted_files.push(target_path);
            }
        }

        Ok(())
    }
}

impl Default for TarHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tar_handler_can_handle() {
        let handler = TarHandler::new();

        assert!(handler.can_handle(Path::new("test.tar")));
        assert!(handler.can_handle(Path::new("test.tar.gz")));
        assert!(handler.can_handle(Path::new("test.tgz")));
        assert!(handler.can_handle(Path::new("test.tar.xz")));
        assert!(handler.can_handle(Path::new("test.tar.bz2")));
        assert!(!handler.can_handle(Path::new("test.zip")));
        assert!(!handler.can_handle(Path::new("test.exe")));
    }

    #[tokio::test]
    async fn test_tar_handler_name() {
        let handler = TarHandler::new();
        assert_eq!(handler.name(), "tar");
    }

    #[test]
    fn test_compression_detection() {
        let handler = TarHandler::new();

        assert!(matches!(
            handler.detect_compression(Path::new("test.tar")),
            CompressionType::None
        ));
        assert!(matches!(
            handler.detect_compression(Path::new("test.tar.gz")),
            CompressionType::Gzip
        ));
        assert!(matches!(
            handler.detect_compression(Path::new("test.tgz")),
            CompressionType::Gzip
        ));
        assert!(matches!(
            handler.detect_compression(Path::new("test.tar.xz")),
            CompressionType::Xz
        ));
    }
}
