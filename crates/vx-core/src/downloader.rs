//! Download and extraction utilities for VX tool manager

use crate::{Result, VxEnvironment, VxError};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Tool downloader and installer
pub struct ToolDownloader {
    env: VxEnvironment,
}

impl ToolDownloader {
    /// Create a new tool downloader
    pub fn new() -> Result<Self> {
        let env = VxEnvironment::new()?;
        Ok(Self { env })
    }

    /// Download and install a tool from URL
    pub async fn download_and_install(
        &self,
        tool_name: &str,
        version: &str,
        download_url: &str,
    ) -> Result<PathBuf> {
        // Create cache directory for downloads
        let cache_dir = self.env.get_tool_cache_dir(tool_name);
        std::fs::create_dir_all(&cache_dir)?;

        // Download file
        let filename = self.extract_filename_from_url(download_url);
        let download_path = cache_dir.join(&filename);

        println!("📥 Downloading {} from {}", filename, download_url);
        self.download_file(download_url, &download_path).await?;

        // Create installation directory
        let install_dir = self.env.get_version_install_dir(tool_name, version);
        std::fs::create_dir_all(&install_dir)?;

        // Extract or copy file
        let executable_path = if self.is_archive(&filename) {
            println!("📦 Extracting archive...");
            self.extract_archive(&download_path, &install_dir, tool_name)
                .await?
        } else {
            println!("📋 Installing binary...");
            self.install_binary(&download_path, &install_dir, tool_name)
                .await?
        };

        // Make executable on Unix systems
        #[cfg(unix)]
        self.make_executable(&executable_path)?;

        // Clean up download
        let _ = std::fs::remove_file(&download_path);

        println!("✅ Installation completed: {}", executable_path.display());
        Ok(executable_path)
    }

    /// Download a file from URL
    async fn download_file(&self, url: &str, output_path: &Path) -> Result<()> {
        let client = crate::http::get_http_client();
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| VxError::DownloadFailed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

        if !response.status().is_success() {
            return Err(VxError::DownloadFailed {
                url: url.to_string(),
                reason: format!("HTTP {}", response.status()),
            });
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| VxError::DownloadFailed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

        let mut file = std::fs::File::create(output_path)?;
        file.write_all(&bytes)?;
        file.flush()?;

        Ok(())
    }

    /// Extract filename from URL
    fn extract_filename_from_url(&self, url: &str) -> String {
        url.split('/')
            .next_back()
            .unwrap_or("download")
            .split('?')
            .next()
            .unwrap_or("download")
            .to_string()
    }

    /// Check if file is an archive
    fn is_archive(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        lower.ends_with(".tar.gz")
            || lower.ends_with(".tgz")
            || lower.ends_with(".tar.xz")
            || lower.ends_with(".tar.bz2")
            || lower.ends_with(".zip")
            || lower.ends_with(".7z")
    }

    /// Extract archive to installation directory
    async fn extract_archive(
        &self,
        archive_path: &Path,
        install_dir: &Path,
        tool_name: &str,
    ) -> Result<PathBuf> {
        let filename = archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("archive");

        if filename.ends_with(".zip") {
            self.extract_zip(archive_path, install_dir, tool_name).await
        } else if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            self.extract_tar_gz(archive_path, install_dir, tool_name)
                .await
        } else {
            // For other formats, try to use system tools or fall back to binary install
            self.install_binary(archive_path, install_dir, tool_name)
                .await
        }
    }

    /// Extract ZIP archive
    async fn extract_zip(
        &self,
        archive_path: &Path,
        install_dir: &Path,
        tool_name: &str,
    ) -> Result<PathBuf> {
        

        let file = std::fs::File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| VxError::InstallationFailed {
            tool_name: tool_name.to_string(),
            version: "unknown".to_string(),
            message: format!("Failed to open ZIP archive: {}", e),
        })?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| VxError::InstallationFailed {
                    tool_name: tool_name.to_string(),
                    version: "unknown".to_string(),
                    message: format!("Failed to read ZIP entry: {}", e),
                })?;

            let outpath = match file.enclosed_name() {
                Some(path) => install_dir.join(path),
                None => continue,
            };

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() {
                    if !p.exists() {
                        std::fs::create_dir_all(p)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        // Find the executable
        self.find_executable_in_dir(install_dir, tool_name)
    }

    /// Extract tar.gz archive
    async fn extract_tar_gz(
        &self,
        archive_path: &Path,
        install_dir: &Path,
        tool_name: &str,
    ) -> Result<PathBuf> {
        

        let file = std::fs::File::open(archive_path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(decoder);

        archive
            .unpack(install_dir)
            .map_err(|e| VxError::InstallationFailed {
                tool_name: tool_name.to_string(),
                version: "unknown".to_string(),
                message: format!("Failed to extract tar.gz: {}", e),
            })?;

        // Find the executable
        self.find_executable_in_dir(install_dir, tool_name)
    }

    /// Install binary file
    async fn install_binary(
        &self,
        binary_path: &Path,
        install_dir: &Path,
        tool_name: &str,
    ) -> Result<PathBuf> {
        let bin_dir = install_dir.join("bin");
        std::fs::create_dir_all(&bin_dir)?;

        let exe_name = if cfg!(windows) {
            format!("{}.exe", tool_name)
        } else {
            tool_name.to_string()
        };

        let target_path = bin_dir.join(&exe_name);
        std::fs::copy(binary_path, &target_path)?;

        Ok(target_path)
    }

    /// Find executable in directory
    fn find_executable_in_dir(&self, dir: &Path, tool_name: &str) -> Result<PathBuf> {
        let env = &self.env;
        env.find_executable_in_dir(dir, tool_name)
    }

    /// Make file executable on Unix systems
    #[cfg(unix)]
    fn make_executable(&self, path: &Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let metadata = std::fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions)?;

        Ok(())
    }

    /// Make file executable on Windows (no-op)
    #[cfg(not(unix))]
    fn make_executable(&self, _path: &Path) -> Result<()> {
        Ok(())
    }
}

impl Default for ToolDownloader {
    fn default() -> Self {
        Self::new().expect("Failed to create tool downloader")
    }
}
