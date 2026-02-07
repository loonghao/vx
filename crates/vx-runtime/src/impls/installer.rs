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

    /// Extract filename from response headers and final URL.
    ///
    /// Tries in order:
    /// 1. Content-Disposition header
    /// 2. Final URL path (after redirects)
    fn extract_filename_from_response(response: &reqwest::Response, original_url: &str) -> Option<String> {
        // Try Content-Disposition header first
        if let Some(content_disposition) = response.headers().get("content-disposition") {
            if let Ok(value) = content_disposition.to_str() {
                if let Some(filename) = Self::parse_content_disposition(value) {
                    tracing::debug!("Got filename from Content-Disposition: {}", filename);
                    return Some(filename);
                }
            }
        }

        // Try to get filename from final URL (after redirects)
        let final_url = response.url().as_str();
        if final_url != original_url {
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

    /// Download a file and return the detected filename from the server response.
    ///
    /// This avoids a separate HEAD request by extracting the filename from
    /// the GET response's Content-Disposition header or final redirected URL.
    /// For APIs like Adoptium that use redirect chains (307 → 302 → CDN),
    /// this saves ~3-5 seconds by eliminating the redundant HEAD round-trip.
    async fn download_and_detect_filename(
        &self,
        url: &str,
        dest: &Path,
    ) -> Result<Option<String>> {
        use futures_util::StreamExt;
        use indicatif::{ProgressBar, ProgressStyle};
        use tokio::io::AsyncWriteExt;

        // Check download cache first
        if let Some(cache) = &self.http.download_cache {
            let lookup = cache.lookup(url);
            match lookup {
                vx_cache::CacheLookupResult::Hit { path, metadata } => {
                    let filename = RealHttpClient::extract_display_name_from_url(url);
                    let size_mb = metadata.size as f64 / 1_000_000.0;
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::with_template(&format!(
                            "{filename} (cached) {{spinner:.green}} {size_mb:.1} MB"
                        ))
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                    );
                    pb.enable_steady_tick(std::time::Duration::from_millis(50));
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(&path, dest)?;
                    pb.finish_and_clear();
                    tracing::debug!(url = url, cached_path = ?path, "Served from download cache");
                    // Return cached filename from metadata
                    let cached_filename = if !metadata.filename.is_empty()
                        && metadata.filename.contains('.')
                        && !metadata.filename.starts_with('.')
                    {
                        Some(metadata.filename.clone())
                    } else {
                        None
                    };
                    return Ok(cached_filename);
                }
                vx_cache::CacheLookupResult::NeedsRevalidation { path, metadata } => {
                    let filename = RealHttpClient::extract_display_name_from_url(url);
                    let size_mb = metadata.size as f64 / 1_000_000.0;
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::with_template(&format!(
                            "{filename} (cached) {{spinner:.green}} {size_mb:.1} MB"
                        ))
                        .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                    );
                    pb.enable_steady_tick(std::time::Duration::from_millis(50));
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::copy(&path, dest)?;
                    pb.finish_and_clear();
                    tracing::debug!(url = url, cached_path = ?path, "Served from download cache (with ETag)");
                    return Ok(None);
                }
                vx_cache::CacheLookupResult::Miss => {}
            }
        }

        // Optimize URL with CDN if enabled
        let download_url = self.http.optimize_url(url).await;
        let using_cdn = download_url.as_str() != url;
        if using_cdn {
            tracing::info!(
                original = url,
                optimized = %download_url,
                "Using CDN accelerated URL"
            );
        }

        let response = self.http.client.get(download_url.as_str()).send().await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed: HTTP {} for {}",
                response.status(),
                if using_cdn { download_url.as_str() } else { url }
            ));
        }

        // Extract filename from response BEFORE consuming the body
        let detected_filename = Self::extract_filename_from_response(&response, url);
        let total_size = response.content_length().unwrap_or(0);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        let filename_display = RealHttpClient::extract_display_name_from_url(url);
        let cdn_suffix = if using_cdn { " [CDN]" } else { "" };

        let progress_bar = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "{filename_display}{cdn_suffix} (download) {{wide_bar:.cyan/blue}} {{bytes}}/{{total_bytes}}"
                ))
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("━━╺"),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "{{spinner:.green}} {filename_display}{cdn_suffix} (download) {{bytes}}"
                ))
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            pb
        };

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            progress_bar.inc(chunk.len() as u64);
        }

        progress_bar.finish_and_clear();
        file.flush().await?;

        // Store in download cache if enabled
        if let Some(cache) = &self.http.download_cache {
            if let Err(e) = cache.store(url, dest, None, None, None) {
                tracing::warn!(url = url, error = %e, "Failed to cache download");
            }
        }

        Ok(detected_filename)
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
        // Create temp file for download
        let temp_dir = tempfile::tempdir()?;

        // Extract archive name from URL, handling URL fragments (e.g., #.zip hint)
        let url_without_fragment = url.split('#').next().unwrap_or(url);

        // Download and detect filename in a single GET request (no separate HEAD).
        // This saves ~3-5 seconds for APIs that use redirect chains (e.g., Adoptium:
        // api.adoptium.net → github.com → objects.githubusercontent.com).
        let temp_download_path = temp_dir.path().join("download_temp");
        let detected_filename = self
            .download_and_detect_filename(url_without_fragment, &temp_download_path)
            .await?;

        let archive_name = detected_filename.unwrap_or_else(|| {
            url_without_fragment
                .split('/')
                .next_back()
                .unwrap_or("archive")
                .split('?')
                .next()
                .unwrap_or("archive")
                .to_string()
        });

        // Rename temp file to actual filename so extraction can detect format
        let temp_path = temp_dir.path().join(&archive_name);
        if temp_download_path != temp_path {
            std::fs::rename(&temp_download_path, &temp_path)?;
        }

        // Check for extension hint in URL fragment
        let extension_hint = url.split('#').nth(1);

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
