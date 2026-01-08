//! Binary file handler for direct executable installation

use super::FormatHandler;
use crate::{progress::ProgressContext, Result};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Handler for binary files (direct executables)
pub struct BinaryHandler;

impl BinaryHandler {
    /// Create a new binary handler
    pub fn new() -> Self {
        Self
    }

    /// Check if a file appears to be a binary executable based on file content (magic bytes)
    fn is_likely_binary(&self, file_path: &Path) -> bool {
        // First check file extension for quick detection
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();

            // Windows executables
            if cfg!(windows) && matches!(ext_lower.as_str(), "exe" | "msi" | "bat" | "cmd") {
                return true;
            }

            // Known binary extensions
            if matches!(ext_lower.as_str(), "bin" | "run" | "app") {
                return true;
            }
            
            // Known archive extensions - NOT binaries
            if matches!(ext_lower.as_str(), 
                "zip" | "tar" | "gz" | "xz" | "bz2" | "7z" | "rar" |
                "tgz" | "txz" | "tbz2" | "tbz"
            ) {
                return false;
            }
        }

        // Check magic bytes to detect executable format
        if let Ok(magic) = self.read_magic_bytes(file_path, 4) {
            // ELF binary (Linux/Unix)
            if magic.starts_with(&[0x7f, b'E', b'L', b'F']) {
                return true;
            }
            
            // Windows PE executable (MZ header)
            if magic.starts_with(&[b'M', b'Z']) {
                return true;
            }
            
            // Mach-O binary (macOS) - various magic numbers
            // 32-bit: 0xFEEDFACE, 64-bit: 0xFEEDFACF
            // Fat binary: 0xCAFEBABE
            if magic.len() >= 4 {
                let magic_u32_be = u32::from_be_bytes([magic[0], magic[1], magic[2], magic[3]]);
                let magic_u32_le = u32::from_le_bytes([magic[0], magic[1], magic[2], magic[3]]);
                
                if matches!(magic_u32_be, 0xFEEDFACE | 0xFEEDFACF | 0xCAFEBABE) ||
                   matches!(magic_u32_le, 0xFEEDFACE | 0xFEEDFACF) {
                    return true;
                }
            }
            
            // Shebang script (#!/...) - treat as executable
            if magic.starts_with(&[b'#', b'!']) {
                return true;
            }
        }

        // Fallback: no extension on Unix might indicate a binary
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            if !filename.contains('.') && !cfg!(windows) {
                // Try to read magic bytes if we haven't already
                if let Ok(magic) = self.read_magic_bytes(file_path, 4) {
                    // If it has valid magic bytes for any executable format, it's a binary
                    if !magic.is_empty() && magic[0] != 0 {
                        return true;
                    }
                }
            }
        }

        false
    }
    
    /// Read the first N bytes of a file (magic bytes)
    fn read_magic_bytes(&self, file_path: &Path, n: usize) -> std::io::Result<Vec<u8>> {
        let mut file = std::fs::File::open(file_path)?;
        let mut buffer = vec![0u8; n];
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    /// Determine the target executable name for a tool
    fn get_target_name(&self, tool_name: &str, source_path: &Path) -> String {
        // If the source already has the correct name, use it
        if let Some(filename) = source_path.file_name().and_then(|n| n.to_str()) {
            // Remove extension for comparison
            let name_without_ext =
                if let Some(stem) = source_path.file_stem().and_then(|s| s.to_str()) {
                    stem
                } else {
                    filename
                };

            if name_without_ext.starts_with(tool_name) {
                // For Windows, ensure .exe extension
                if cfg!(windows) && !filename.ends_with(".exe") {
                    return format!("{}.exe", filename);
                }
                return filename.to_string();
            }
        }

        // Otherwise, use the standard executable name
        self.get_executable_name(tool_name)
    }
}

#[async_trait::async_trait]
impl FormatHandler for BinaryHandler {
    fn name(&self) -> &str {
        "binary"
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        // This handler is a fallback for files that don't match other formats
        // It should be checked last in the handler chain
        self.is_likely_binary(file_path)
    }

    async fn extract(
        &self,
        source_path: &Path,
        target_dir: &Path,
        progress: &ProgressContext,
    ) -> Result<Vec<PathBuf>> {
        // For binary files, "extraction" means copying to the bin directory
        let bin_dir = target_dir.join("bin");
        std::fs::create_dir_all(&bin_dir)?;

        progress.start("Installing binary", Some(1)).await?;

        // Determine the target filename
        let tool_name = target_dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("tool");

        let target_name = self.get_target_name(tool_name, source_path);
        let target_path = bin_dir.join(target_name);

        // Copy the binary
        std::fs::copy(source_path, &target_path)?;

        // Make it executable
        self.make_executable(&target_path)?;

        progress.increment(1).await?;
        progress.finish("Binary installation completed").await?;

        Ok(vec![target_path])
    }
}

impl Default for BinaryHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::ProgressContext;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_binary_handler_name() {
        let handler = BinaryHandler::new();
        assert_eq!(handler.name(), "binary");
    }

    #[test]
    fn test_is_likely_binary_by_extension() {
        let handler = BinaryHandler::new();

        // Windows executables (by extension)
        if cfg!(windows) {
            assert!(handler.is_likely_binary(Path::new("tool.exe")));
            assert!(handler.is_likely_binary(Path::new("installer.msi")));
            assert!(handler.is_likely_binary(Path::new("script.bat")));
        }

        // Binary extensions
        assert!(handler.is_likely_binary(Path::new("tool.bin")));
        assert!(handler.is_likely_binary(Path::new("app.run")));

        // Not binaries (by extension)
        assert!(!handler.is_likely_binary(Path::new("archive.zip")));
        assert!(!handler.is_likely_binary(Path::new("source.tar.gz")));
    }

    #[test]
    fn test_is_likely_binary_by_magic_bytes() {
        let handler = BinaryHandler::new();
        let temp_dir = TempDir::new().unwrap();

        // Create ELF binary
        let elf_file = temp_dir.path().join("elf_binary");
        std::fs::write(&elf_file, b"\x7fELF\x02\x01\x01\x00").unwrap();
        assert!(handler.is_likely_binary(&elf_file));

        // Create PE/Windows binary
        let pe_file = temp_dir.path().join("pe_binary");
        std::fs::write(&pe_file, b"MZ\x90\x00").unwrap();
        assert!(handler.is_likely_binary(&pe_file));

        // Create shebang script
        let script_file = temp_dir.path().join("script");
        std::fs::write(&script_file, b"#!/bin/bash\necho hello").unwrap();
        assert!(handler.is_likely_binary(&script_file));

        // Create Mach-O binary (64-bit)
        let macho_file = temp_dir.path().join("macho_binary");
        std::fs::write(&macho_file, &[0xCF, 0xFA, 0xED, 0xFE]).unwrap();
        assert!(handler.is_likely_binary(&macho_file));
    }

    #[test]
    fn test_get_target_name() {
        let handler = BinaryHandler::new();

        // Source already has correct name
        let expected = if cfg!(windows) {
            "node-v18.17.0.exe"
        } else {
            "node-v18.17.0"
        };
        assert_eq!(
            handler.get_target_name("node", Path::new("node-v18.17.0")),
            expected
        );

        // Source starts with tool name, should keep original name
        if cfg!(windows) {
            assert_eq!(
                handler.get_target_name("go", Path::new("golang.exe")),
                "golang.exe" // Should keep original name since "golang" starts with "go"
            );
        } else {
            assert_eq!(handler.get_target_name("go", Path::new("golang")), "golang");
        }

        // Source doesn't match, use standard name
        if cfg!(windows) {
            assert_eq!(
                handler.get_target_name("go", Path::new("python.exe")),
                "go.exe" // Should use standard name since "python" doesn't start with "go"
            );
        } else {
            assert_eq!(handler.get_target_name("go", Path::new("python")), "go");
        }
    }

    #[tokio::test]
    async fn test_binary_extraction() {
        let handler = BinaryHandler::new();
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let target_dir = temp_dir.path().join("target").join("tool").join("1.0.0");

        std::fs::create_dir_all(&source_dir).unwrap();

        // Create a mock binary file with shebang (will be detected as binary)
        let source_file = source_dir.join("tool");
        let mut file = std::fs::File::create(&source_file).unwrap();
        file.write_all(b"#!/bin/bash\necho 'Hello World'").unwrap();

        let progress = ProgressContext::disabled();

        let result = handler.extract(&source_file, &target_dir, &progress).await;
        assert!(result.is_ok());

        let extracted_files = result.unwrap();
        assert_eq!(extracted_files.len(), 1);

        let expected_path =
            target_dir
                .join("bin")
                .join(if cfg!(windows) { "tool.exe" } else { "tool" });
        assert_eq!(extracted_files[0], expected_path);
        assert!(expected_path.exists());
    }
}
