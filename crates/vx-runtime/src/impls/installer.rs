//! Real installer implementation

use super::http_client::RealHttpClient;
use crate::traits::Installer;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

/// Real installer for downloading and extracting archives
pub struct RealInstaller {
    http: RealHttpClient,
}

impl RealInstaller {
    /// Create a new real installer
    pub fn new() -> Self {
        Self {
            http: RealHttpClient::new(),
        }
    }

    /// Create a new installer with download caching enabled
    pub fn with_download_cache(cache_dir: std::path::PathBuf) -> Self {
        Self {
            http: RealHttpClient::new().with_download_cache(cache_dir),
        }
    }

    /// Get filename from server response headers
    ///
    /// This method sends a HEAD request to get the actual filename from:
    /// 1. Content-Disposition header
    /// 2. Final redirected URL
    async fn get_filename_from_server(&self, url: &str) -> Option<String> {
        // Send HEAD request following redirects
        let response = self.http.client.head(url).send().await.ok()?;

        // Try Content-Disposition header first
        if let Some(content_disposition) = response.headers().get("content-disposition") {
            if let Ok(value) = content_disposition.to_str() {
                // Parse filename from Content-Disposition: attachment; filename=xxx.zip
                if let Some(filename) = Self::parse_content_disposition(value) {
                    tracing::debug!("Got filename from Content-Disposition: {}", filename);
                    return Some(filename);
                }
            }
        }

        // Try to get filename from final URL (after redirects)
        let final_url = response.url().as_str();
        if final_url != url {
            let filename = final_url
                .split('/')
                .next_back()
                .unwrap_or("download")
                .split('?')
                .next()
                .unwrap_or("download");
            // Only use if it has a valid extension
            if filename.contains('.') && !filename.starts_with('.') {
                tracing::debug!("Got filename from final URL: {}", filename);
                return Some(filename.to_string());
            }
        }

        None
    }

    /// Parse filename from Content-Disposition header value
    fn parse_content_disposition(value: &str) -> Option<String> {
        // Handle: attachment; filename=xxx.zip
        // Handle: attachment; filename="xxx.zip"
        // Handle: attachment; filename*=UTF-8''xxx.zip

        for part in value.split(';') {
            let part = part.trim();

            // Standard filename parameter
            if let Some(filename) = part.strip_prefix("filename=") {
                let filename = filename.trim_matches('"').trim_matches('\'');
                if !filename.is_empty() {
                    return Some(filename.to_string());
                }
            }

            // RFC 5987 encoded filename (filename*=)
            if let Some(encoded) = part.strip_prefix("filename*=") {
                // Format: charset'language'encoded_filename
                // e.g., UTF-8''OpenJDK25U-jdk_x64_windows_hotspot_25.0.1_8.zip
                if let Some(pos) = encoded.rfind("''") {
                    let filename = &encoded[pos + 2..];
                    // URL decode the filename
                    if let Ok(decoded) = urlencoding::decode(filename) {
                        if !decoded.is_empty() {
                            return Some(decoded.into_owned());
                        }
                    }
                }
            }
        }

        None
    }
}

impl Default for RealInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Installer for RealInstaller {
    async fn extract(&self, archive: &Path, dest: &Path) -> Result<()> {
        std::fs::create_dir_all(dest)?;

        let archive_str = archive.to_string_lossy();

        // First try to determine format by extension
        let format = if archive_str.ends_with(".tar.gz") || archive_str.ends_with(".tgz") {
            Some("tar.gz")
        } else if archive_str.ends_with(".tar.xz") {
            Some("tar.xz")
        } else if archive_str.ends_with(".tar.zst") || archive_str.ends_with(".tzst") {
            Some("tar.zst")
        } else if archive_str.ends_with(".zip") {
            Some("zip")
        } else if archive_str.ends_with(".7z") {
            Some("7z")
        } else {
            // Try to detect by magic bytes
            let file = std::fs::File::open(archive)?;
            use std::io::Read;
            let mut magic = [0u8; 6];
            if (&file).take(6).read(&mut magic).is_ok() {
                if magic[0] == 0x50 && magic[1] == 0x4B {
                    // ZIP magic: PK\x03\x04
                    Some("zip")
                } else if magic[0] == 0x1f && magic[1] == 0x8b {
                    // GZIP magic: \x1f\x8b
                    Some("tar.gz")
                } else if magic[0] == 0xFD
                    && magic[1] == 0x37
                    && magic[2] == 0x7A
                    && magic[3] == 0x58
                {
                    // XZ magic: \xFD7zXZ
                    Some("tar.xz")
                } else if magic[0] == 0x28
                    && magic[1] == 0xB5
                    && magic[2] == 0x2F
                    && magic[3] == 0xFD
                {
                    // Zstd magic: \x28\xB5\x2F\xFD
                    Some("tar.zst")
                } else if magic[0] == 0x37
                    && magic[1] == 0x7A
                    && magic[2] == 0xBC
                    && magic[3] == 0xAF
                    && magic[4] == 0x27
                    && magic[5] == 0x1C
                {
                    // 7z magic: 7z\xBC\xAF\x27\x1C
                    Some("7z")
                } else {
                    None
                }
            } else {
                None
            }
        };

        match format {
            Some("tar.gz") => {
                let file = std::fs::File::open(archive)?;
                let decoder = flate2::read::GzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            Some("tar.xz") => {
                let file = std::fs::File::open(archive)?;
                let decoder = xz2::read::XzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            Some("tar.zst") => {
                let file = std::fs::File::open(archive)?;
                let decoder = zstd::stream::read::Decoder::new(std::io::BufReader::new(file))?;
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            Some("zip") => {
                let file = std::fs::File::open(archive)?;
                let mut archive = zip::ZipArchive::new(file)?;
                archive.extract(dest)?;
            }
            Some("7z") => {
                sevenz_rust::decompress_file(archive, dest)
                    .map_err(|e| anyhow::anyhow!("Failed to extract 7z archive: {}", e))?;
            }
            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported archive format: {}",
                    archive_str
                ));
            }
        }

        Ok(())
    }

    async fn download_and_extract(&self, url: &str, dest: &Path) -> Result<()> {
        use crate::traits::HttpClient;

        // Create temp file for download
        let temp_dir = tempfile::tempdir()?;

        // Extract archive name from URL, handling URL fragments (e.g., #.zip hint)
        let url_without_fragment = url.split('#').next().unwrap_or(url);

        // Try to get actual filename from server (handles redirects and Content-Disposition)
        let archive_name = self
            .get_filename_from_server(url_without_fragment)
            .await
            .unwrap_or_else(|| {
                url_without_fragment
                    .split('/')
                    .next_back()
                    .unwrap_or("archive")
                    .split('?')
                    .next()
                    .unwrap_or("archive")
                    .to_string()
            });

        // Check for extension hint in URL fragment
        let extension_hint = url.split('#').nth(1);

        let temp_path = temp_dir.path().join(&archive_name);

        // Download with caching support (if cache is enabled)
        let from_cache = self
            .http
            .download_cached(url_without_fragment, &temp_path)
            .await?;
        if from_cache {
            tracing::debug!(url = url_without_fragment, "Using cached download");
        }

        // Check if it's an archive or a single executable
        // First check the URL/filename, then check extension hint, then check file magic bytes
        let archive_str = archive_name.to_lowercase();
        let mut is_archive = archive_str.ends_with(".tar.gz")
            || archive_str.ends_with(".tgz")
            || archive_str.ends_with(".tar.xz")
            || archive_str.ends_with(".tar.zst")
            || archive_str.ends_with(".tzst")
            || archive_str.ends_with(".zip")
            || archive_str.ends_with(".7z");

        // Check extension hint from URL fragment
        if !is_archive {
            if let Some(hint) = extension_hint {
                is_archive = hint.ends_with(".tar.gz")
                    || hint.ends_with(".tgz")
                    || hint.ends_with(".tar.xz")
                    || hint.ends_with(".zip")
                    || hint.ends_with(".7z");
            }
        }

        // Check file magic bytes if still uncertain
        if !is_archive {
            if let Ok(mut file) = std::fs::File::open(&temp_path) {
                use std::io::Read;
                let mut magic = [0u8; 6];
                if file.read_exact(&mut magic).is_ok() {
                    // ZIP magic: PK\x03\x04
                    // GZIP magic: \x1f\x8b
                    // 7z magic: 7z\xBC\xAF\x27\x1C (first 6 bytes: 37 7A BC AF 27 1C)
                    is_archive = (magic[0] == 0x50 && magic[1] == 0x4B)  // ZIP
                        || (magic[0] == 0x1f && magic[1] == 0x8b) // GZIP (tar.gz)
                        || (magic[0] == 0x37 && magic[1] == 0x7A && magic[2] == 0xBC
                            && magic[3] == 0xAF && magic[4] == 0x27 && magic[5] == 0x1C);
                    // 7z
                }
            }
        }

        if is_archive {
            // Extract archive
            self.extract(&temp_path, dest).await?;
        } else {
            // Single executable file - place under bin/
            let bin_dir = dest.join("bin");
            std::fs::create_dir_all(&bin_dir)?;

            // Preserve original filename (e.g., kubectl.exe, bun)
            let exe_name = archive_name.to_string();
            let dest_path = bin_dir.join(&exe_name);
            std::fs::copy(&temp_path, &dest_path)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dest_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest_path, perms)?;
            }
        }

        Ok(())
    }
}
