//! Download utilities for vx-installer

use crate::{cdn::CdnOptimizer, progress::ProgressContext, Error, Result, USER_AGENT};
use futures_util::StreamExt;
use sha2::Digest;
use std::path::{Path, PathBuf};

/// HTTP downloader for fetching files from URLs
pub struct Downloader {
    client: reqwest::Client,
    cdn_optimizer: CdnOptimizer,
}

impl Downloader {
    /// Create a new downloader with default configuration
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(300)) // 5 minutes
            .build()?;

        Ok(Self {
            client,
            cdn_optimizer: CdnOptimizer::default(),
        })
    }

    /// Create a downloader with CDN acceleration enabled
    pub fn with_cdn(cdn_enabled: bool) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self {
            client,
            cdn_optimizer: CdnOptimizer::new(cdn_enabled),
        })
    }

    /// Create a downloader with custom client configuration
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            cdn_optimizer: CdnOptimizer::default(),
        }
    }

    /// Create a downloader with custom client and CDN optimizer
    pub fn with_client_and_cdn(client: reqwest::Client, cdn_optimizer: CdnOptimizer) -> Self {
        Self {
            client,
            cdn_optimizer,
        }
    }

    /// Enable or disable CDN acceleration
    pub fn set_cdn_enabled(&mut self, enabled: bool) {
        self.cdn_optimizer = CdnOptimizer::new(enabled);
    }

    /// Check if CDN acceleration is enabled
    pub fn is_cdn_enabled(&self) -> bool {
        self.cdn_optimizer.is_enabled()
    }

    /// Download a file from URL to the specified path
    ///
    /// If CDN acceleration is enabled, the URL will be optimized before downloading.
    pub async fn download(
        &self,
        url: &str,
        output_path: &Path,
        progress: &ProgressContext,
    ) -> Result<()> {
        // Optimize URL with CDN if enabled
        let download_url = self.cdn_optimizer.optimize_url(url).await?;

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Start the download request
        let response = self
            .client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| Error::download_failed(&download_url, e.to_string()))?;

        // Check response status
        if !response.status().is_success() {
            return Err(Error::download_failed(
                &download_url,
                format!("HTTP {}", response.status()),
            ));
        }

        // Get content length for progress tracking
        let total_size = response.content_length();

        // Extract filename for progress display (use original URL for display)
        let filename = self.extract_filename_from_url(url);
        let message = format!("Downloading {}", filename);

        progress.start(&message, total_size).await?;

        // Create the output file
        let mut file = std::fs::File::create(output_path)?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        // Download with progress tracking
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| Error::download_failed(url, e.to_string()))?;

            std::io::Write::write_all(&mut file, &chunk)?;
            downloaded += chunk.len() as u64;

            progress.update(downloaded, None).await?;
        }

        // Ensure all data is written
        std::io::Write::flush(&mut file)?;

        progress.finish("Download completed").await?;

        Ok(())
    }

    /// Download a file to a temporary location and return the path
    pub async fn download_temp(&self, url: &str, progress: &ProgressContext) -> Result<PathBuf> {
        let filename = self.extract_filename_from_url(url);
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path().join(filename);

        self.download(url, &temp_path, progress).await?;

        // Convert to a persistent path (caller is responsible for cleanup)
        let persistent_path = temp_path.clone();
        std::mem::forget(temp_dir); // Prevent automatic cleanup

        Ok(persistent_path)
    }

    /// Download and verify checksum
    pub async fn download_with_checksum(
        &self,
        url: &str,
        output_path: &Path,
        expected_checksum: &str,
        progress: &ProgressContext,
    ) -> Result<()> {
        // Download the file
        self.download(url, output_path, progress).await?;

        // Verify checksum
        let actual_checksum = self.calculate_sha256(output_path)?;
        if actual_checksum != expected_checksum {
            return Err(Error::ChecksumMismatch {
                file_path: output_path.to_path_buf(),
                expected: expected_checksum.to_string(),
                actual: actual_checksum,
            });
        }

        Ok(())
    }

    /// Get the size of a remote file without downloading it
    pub async fn get_file_size(&self, url: &str) -> Result<Option<u64>> {
        let response = self
            .client
            .head(url)
            .send()
            .await
            .map_err(|e| Error::download_failed(url, e.to_string()))?;

        if !response.status().is_success() {
            return Err(Error::download_failed(
                url,
                format!("HTTP {}", response.status()),
            ));
        }

        Ok(response.content_length())
    }

    /// Check if a URL is accessible
    pub async fn check_url(&self, url: &str) -> Result<bool> {
        match self.client.head(url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Extract filename from URL
    fn extract_filename_from_url(&self, url: &str) -> String {
        let filename = url
            .split('/')
            .next_back()
            .unwrap_or("download")
            .split('?')
            .next()
            .unwrap_or("download");

        if filename.is_empty() {
            "download".to_string()
        } else {
            filename.to_string()
        }
    }

    /// Calculate SHA256 checksum of a file
    fn calculate_sha256(&self, file_path: &Path) -> Result<String> {
        use std::io::Read;

        let mut file = std::fs::File::open(file_path)?;
        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new().expect("Failed to create default downloader")
    }
}

/// Configuration for download operations
#[derive(Debug, Clone)]
pub struct DownloadConfig {
    /// URL to download from
    pub url: String,
    /// Output file path
    pub output_path: PathBuf,
    /// Expected checksum (optional)
    pub checksum: Option<String>,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Timeout for the download operation
    pub timeout: std::time::Duration,
    /// Whether to overwrite existing files
    pub overwrite: bool,
}

impl DownloadConfig {
    /// Create a new download configuration
    pub fn new(url: impl Into<String>, output_path: impl Into<PathBuf>) -> Self {
        Self {
            url: url.into(),
            output_path: output_path.into(),
            checksum: None,
            max_retries: 3,
            timeout: std::time::Duration::from_secs(300),
            overwrite: false,
        }
    }

    /// Set the expected checksum
    pub fn with_checksum(mut self, checksum: impl Into<String>) -> Self {
        self.checksum = Some(checksum.into());
        self
    }

    /// Set the maximum number of retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the timeout
    pub fn with_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set whether to overwrite existing files
    pub fn with_overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename_from_url() {
        let downloader = Downloader::default();

        assert_eq!(
            downloader.extract_filename_from_url("https://example.com/file.zip"),
            "file.zip"
        );
        assert_eq!(
            downloader.extract_filename_from_url("https://example.com/file.zip?version=1.0"),
            "file.zip"
        );
        assert_eq!(
            downloader.extract_filename_from_url("https://example.com/"),
            "download"
        );
    }

    #[test]
    fn test_download_config() {
        let config = DownloadConfig::new("https://example.com/file.zip", "/tmp/file.zip")
            .with_checksum("abc123")
            .with_max_retries(5)
            .with_overwrite(true);

        assert_eq!(config.url, "https://example.com/file.zip");
        assert_eq!(config.output_path, PathBuf::from("/tmp/file.zip"));
        assert_eq!(config.checksum, Some("abc123".to_string()));
        assert_eq!(config.max_retries, 5);
        assert!(config.overwrite);
    }
}
