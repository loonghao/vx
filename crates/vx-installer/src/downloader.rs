//! Download utilities for vx-installer

use crate::{cdn::CdnOptimizer, progress::ProgressContext, Error, Result, USER_AGENT};
use backon::{ExponentialBuilder, Retryable};
use futures_util::StreamExt;
use sha2::Digest;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::{debug, warn};

/// HTTP downloader for fetching files from URLs
pub struct Downloader {
    client: reqwest::Client,
    cdn_optimizer: CdnOptimizer,
    /// Maximum number of retry attempts for failed downloads
    max_retries: usize,
    /// Minimum delay between retry attempts
    min_delay: Duration,
    /// Maximum delay between retry attempts
    max_delay: Duration,
}

impl Downloader {
    /// Default maximum retry attempts (increased for CI environments with transient network issues)
    const DEFAULT_MAX_RETRIES: usize = 5;
    /// Default minimum retry delay (2 seconds for better recovery from DNS issues)
    const DEFAULT_MIN_DELAY: Duration = Duration::from_secs(2);
    /// Default maximum retry delay (60 seconds)
    const DEFAULT_MAX_DELAY: Duration = Duration::from_secs(60);

    /// Create a new downloader with default configuration
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(std::time::Duration::from_secs(300)) // 5 minutes
            .build()?;

        Ok(Self {
            client,
            cdn_optimizer: CdnOptimizer::default(),
            max_retries: Self::DEFAULT_MAX_RETRIES,
            min_delay: Self::DEFAULT_MIN_DELAY,
            max_delay: Self::DEFAULT_MAX_DELAY,
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
            max_retries: Self::DEFAULT_MAX_RETRIES,
            min_delay: Self::DEFAULT_MIN_DELAY,
            max_delay: Self::DEFAULT_MAX_DELAY,
        })
    }

    /// Create a downloader with custom timeout
    ///
    /// # Arguments
    /// * `timeout` - Download timeout duration
    /// * `cdn_enabled` - Whether to enable CDN acceleration
    pub fn with_timeout(timeout: Duration, cdn_enabled: bool) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(timeout)
            .build()?;

        Ok(Self {
            client,
            cdn_optimizer: CdnOptimizer::new(cdn_enabled),
            max_retries: Self::DEFAULT_MAX_RETRIES,
            min_delay: Self::DEFAULT_MIN_DELAY,
            max_delay: Self::DEFAULT_MAX_DELAY,
        })
    }

    /// Create a downloader with custom client configuration
    pub fn with_client(client: reqwest::Client) -> Self {
        Self {
            client,
            cdn_optimizer: CdnOptimizer::default(),
            max_retries: Self::DEFAULT_MAX_RETRIES,
            min_delay: Self::DEFAULT_MIN_DELAY,
            max_delay: Self::DEFAULT_MAX_DELAY,
        }
    }

    /// Create a downloader with custom client and CDN optimizer
    pub fn with_client_and_cdn(client: reqwest::Client, cdn_optimizer: CdnOptimizer) -> Self {
        Self {
            client,
            cdn_optimizer,
            max_retries: Self::DEFAULT_MAX_RETRIES,
            min_delay: Self::DEFAULT_MIN_DELAY,
            max_delay: Self::DEFAULT_MAX_DELAY,
        }
    }

    /// Set the maximum number of retry attempts
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the minimum retry delay
    pub fn with_min_delay(mut self, delay: Duration) -> Self {
        self.min_delay = delay;
        self
    }

    /// Set the maximum retry delay
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Enable or disable CDN acceleration
    pub fn set_cdn_enabled(&mut self, enabled: bool) {
        self.cdn_optimizer = CdnOptimizer::new(enabled);
    }

    /// Check if CDN acceleration is enabled
    pub fn is_cdn_enabled(&self) -> bool {
        self.cdn_optimizer.is_enabled()
    }

    /// Build the retry strategy using backon
    fn build_retry_strategy(&self) -> ExponentialBuilder {
        ExponentialBuilder::default()
            .with_min_delay(self.min_delay)
            .with_max_delay(self.max_delay)
            .with_max_times(self.max_retries)
            .with_jitter()
    }

    /// Download a file from URL to the specified path
    ///
    /// If CDN acceleration is enabled, the URL will be optimized before downloading.
    /// Automatically retries on network failures with exponential backoff.
    pub async fn download(
        &self,
        url: &str,
        output_path: &Path,
        progress: &ProgressContext,
    ) -> Result<()> {
        let url = url.to_string();
        let output_path = output_path.to_path_buf();

        (|| async { self.download_once(&url, &output_path, progress).await })
            .retry(self.build_retry_strategy())
            .notify(|err: &Error, dur: Duration| {
                warn!("Download failed: {}, retrying in {:?}", err, dur);
            })
            .when(|e| e.is_recoverable())
            .await
    }

    /// Internal single download attempt without retry logic
    async fn download_once(
        &self,
        url: &str,
        output_path: &Path,
        progress: &ProgressContext,
    ) -> Result<()> {
        // Optimize URL with CDN if enabled
        let optimized = self.cdn_optimizer.optimize_url(url).await?;
        let urls = optimized.urls();

        debug!(
            "Attempting download from {} URL(s): primary={}, has_fallback={}",
            urls.len(),
            urls[0],
            optimized.has_fallback()
        );

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Try each URL in order (primary first, then fallback if available)
        let mut last_error = None;
        let mut attempt_errors: Vec<(String, String)> = Vec::new();

        for (index, download_url) in urls.iter().enumerate() {
            let is_fallback = index > 0;
            if is_fallback {
                warn!(
                    "Primary CDN URL failed, attempting fallback to original URL: {}",
                    download_url
                );
                progress
                    .start("Retrying with original URL...", None)
                    .await?;
            } else {
                progress.start("Connecting...", None).await?;
            }

            debug!("Downloading from: {}", download_url);

            // Start the download request
            let response = match self.client.get(*download_url).send().await {
                Ok(resp) => resp,
                Err(e) => {
                    let error = if e.is_timeout() {
                        Error::NetworkTimeout {
                            url: (*download_url).to_string(),
                        }
                    } else if e.is_connect() || e.is_request() {
                        Error::download_failed(*download_url, format!("Connection error: {}", e))
                    } else {
                        Error::download_failed(*download_url, e.to_string())
                    };

                    attempt_errors.push((download_url.to_string(), error.to_string()));

                    if is_fallback || !optimized.has_fallback() {
                        // Last URL or no fallback available, return error with attempts
                        let detail = format!(
                            "All download attempts failed: {}",
                            attempt_errors
                                .iter()
                                .map(|(u, e)| format!("{} => {}", u, e))
                                .collect::<Vec<_>>()
                                .join("; ")
                        );
                        return Err(Error::download_failed(url, detail));
                    }

                    // Store error and try next URL
                    warn!("Download from CDN failed: {}, will try fallback", error);
                    last_error = Some(error);
                    continue;
                }
            };

            // Check response status
            if !response.status().is_success() {
                let error =
                    Error::download_failed(*download_url, format!("HTTP {}", response.status()));

                attempt_errors.push((download_url.to_string(), error.to_string()));

                if is_fallback || !optimized.has_fallback() {
                    let _ = progress.error("HTTP error").await;
                    let detail = format!(
                        "All download attempts failed: {}",
                        attempt_errors
                            .iter()
                            .map(|(u, e)| format!("{} => {}", u, e))
                            .collect::<Vec<_>>()
                            .join("; ")
                    );
                    return Err(Error::download_failed(url, detail));
                }

                // Store error and try next URL
                warn!("CDN returned HTTP error: {}, will try fallback", error);
                last_error = Some(error);
                continue;
            }

            // Successfully got response, proceed with download
            if is_fallback {
                debug!("Fallback URL succeeded");
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

            // Download the file in chunks
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result
                    .map_err(|e| Error::download_failed(url, format!("Stream error: {}", e)))?;
                use std::io::Write;
                file.write_all(&chunk)?;
                downloaded += chunk.len() as u64;
                progress.update(downloaded, None).await?;
            }

            // Verify file was created and has content
            let metadata = std::fs::metadata(output_path)?;
            if metadata.len() == 0 {
                return Err(Error::download_failed(url, "Downloaded file is empty"));
            }

            progress.finish("Download complete").await?;
            return Ok(());
        }

        // All URLs failed
        if !attempt_errors.is_empty() {
            let detail = attempt_errors
                .iter()
                .map(|(u, e)| format!("{} => {}", u, e))
                .collect::<Vec<_>>()
                .join("; ");
            Err(Error::download_failed(
                url,
                format!("All download attempts failed: {}", detail),
            ))
        } else {
            Err(last_error
                .unwrap_or_else(|| Error::download_failed(url, "All download attempts failed")))
        }
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
        let url = url.to_string();

        (|| async {
            let response = self
                .client
                .head(&url)
                .send()
                .await
                .map_err(|e| Error::download_failed(&url, e.to_string()))?;

            if !response.status().is_success() {
                return Err(Error::download_failed(
                    &url,
                    format!("HTTP {}", response.status()),
                ));
            }

            Ok(response.content_length())
        })
        .retry(self.build_retry_strategy())
        .when(|e: &Error| e.is_recoverable())
        .await
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
    pub max_retries: usize,
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
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
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

    #[test]
    fn test_retry_strategy() {
        let downloader = Downloader::default()
            .with_max_retries(5)
            .with_min_delay(Duration::from_millis(100))
            .with_max_delay(Duration::from_secs(10));

        assert_eq!(downloader.max_retries, 5);
        assert_eq!(downloader.min_delay, Duration::from_millis(100));
        assert_eq!(downloader.max_delay, Duration::from_secs(10));
    }
}
