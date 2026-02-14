//! Real HTTP client implementation

use crate::region;
use crate::traits::HttpClient;
use anyhow::Result;
use async_trait::async_trait;
use backon::{ExponentialBuilder, Retryable};
use std::path::Path;
use std::time::Duration;

/// Determine whether CDN acceleration should be enabled.
///
/// CDN proxies (like gh-proxy.com) are mainly useful in China where GitHub
/// access is slow or unreliable. Outside China (e.g. GitHub CI), these proxies
/// can be unstable and cause download failures (HTTP 404).
///
/// Decision logic (in order):
/// 1. `VX_CDN=1` / `VX_CDN=true`  → force enable
/// 2. `VX_CDN=0` / `VX_CDN=false` → force disable
/// 3. `CI=true` or `GITHUB_ACTIONS=true` → disable (CI environments have direct GitHub access)
/// 4. Check system locale / timezone for China indicators → enable if detected
/// 5. Default → disable (safer default for international users)
fn should_enable_cdn() -> bool {
    // Use the shared region detection — CDN is enabled when in China
    region::detect_region() == region::Region::China
}

/// Real HTTP client using reqwest with optional CDN acceleration
pub struct RealHttpClient {
    pub(crate) client: reqwest::Client,
    /// Whether CDN acceleration is enabled (controlled by cdn-acceleration feature + region)
    cdn_enabled: bool,
    /// Download cache for avoiding re-downloads
    pub(crate) download_cache: Option<vx_cache::DownloadCache>,
}

impl RealHttpClient {
    /// Create a new real HTTP client with default timeouts
    ///
    /// CDN acceleration is automatically enabled only when:
    /// 1. The `cdn-acceleration` feature is compiled in, AND
    /// 2. The system environment indicates a China-based user (or `VX_CDN=1` is set)
    ///
    /// The client is configured with:
    /// - Connection pooling (idle connections kept alive for 90 seconds)
    /// - Up to 10 idle connections per host (reduces handshake overhead)
    /// - Compression enabled by default in reqwest
    /// - HTTP/2 adaptive (automatically used when server supports it)
    /// - Read timeout of 60 seconds (resets after each successful read, good for large files)
    /// - No total timeout (allows large file downloads to complete)
    pub fn new() -> Self {
        let cdn_enabled = cfg!(feature = "cdn-acceleration") && should_enable_cdn();
        Self {
            client: Self::build_client(),
            cdn_enabled,
            download_cache: None,
        }
    }

    /// Create a new HTTP client with explicit CDN setting and default timeouts
    pub fn with_cdn(cdn_enabled: bool) -> Self {
        Self {
            client: Self::build_client(),
            cdn_enabled: cdn_enabled && cfg!(feature = "cdn-acceleration"),
            download_cache: None,
        }
    }

    /// Create a new HTTP client with custom timeouts
    pub fn with_timeouts(
        cdn_enabled: bool,
        connect_timeout: Duration,
        read_timeout: Duration,
    ) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(format!("vx/{}", env!("CARGO_PKG_VERSION")))
                .connect_timeout(connect_timeout)
                .read_timeout(read_timeout)
                .pool_idle_timeout(Duration::from_secs(90))
                .pool_max_idle_per_host(10)
                .build()
                .expect("Failed to create HTTP client"),
            cdn_enabled: cdn_enabled && cfg!(feature = "cdn-acceleration"),
            download_cache: None,
        }
    }

    /// Build the default reqwest client
    fn build_client() -> reqwest::Client {
        reqwest::Client::builder()
            .user_agent(format!("vx/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(60))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client")
    }

    /// Enable download caching with the specified cache directory
    pub fn with_download_cache(mut self, cache_dir: std::path::PathBuf) -> Self {
        self.download_cache = Some(vx_cache::DownloadCache::new(cache_dir));
        self
    }

    /// Check if CDN acceleration is enabled
    pub fn is_cdn_enabled(&self) -> bool {
        self.cdn_enabled
    }

    /// Check if download caching is enabled
    pub fn is_cache_enabled(&self) -> bool {
        self.download_cache.is_some()
    }

    /// Optimize a download URL using CDN mirrors (if enabled)
    ///
    /// When CDN acceleration is enabled and the `cdn-acceleration` feature is active,
    /// this will return an optimized URL from the best available CDN mirror.
    /// Otherwise, it returns the original URL.
    pub(crate) async fn optimize_url(&self, url: &str) -> String {
        if !self.cdn_enabled {
            return url.to_string();
        }

        #[cfg(feature = "cdn-acceleration")]
        {
            match turbo_cdn::async_api::quick::optimize_url(url).await {
                Ok(optimized) => {
                    tracing::debug!(
                        original = url,
                        optimized = %optimized,
                        "CDN URL optimized"
                    );
                    optimized
                }
                Err(e) => {
                    tracing::warn!(
                        url = url,
                        error = %e,
                        "CDN optimization failed, using original URL"
                    );
                    url.to_string()
                }
            }
        }

        #[cfg(not(feature = "cdn-acceleration"))]
        {
            url.to_string()
        }
    }

    /// Build the retry strategy using backon with exponential backoff
    fn build_retry_strategy() -> ExponentialBuilder {
        ExponentialBuilder::default()
            .with_min_delay(Duration::from_millis(500))
            .with_max_delay(Duration::from_secs(5))
            .with_max_times(3)
            .with_jitter()
    }

    /// Perform a single JSON fetch attempt (used by retry logic)
    async fn fetch_json_once(
        &self,
        client: &reqwest::Client,
        url: &str,
    ) -> std::result::Result<serde_json::Value, HttpError> {
        let mut request = client.get(url);

        // GitHub API is picky about headers; also helps some proxies behave.
        if url.contains("api.github.com") {
            request = request
                .header("Accept", "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28");
        }

        // Add GitHub token for GitHub API requests
        if url.contains("api.github.com") || url.contains("github.com") {
            if let Some(token) = get_github_token() {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                HttpError::retryable(format!("Request timed out for {}: {}", url, e))
            } else if e.is_connect() {
                HttpError::retryable(format!("Connection failed for {}: {}", url, e))
            } else {
                HttpError::non_retryable(format!("Request failed: {}", e))
            }
        })?;

        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // Check for rate limit errors
        if status == reqwest::StatusCode::FORBIDDEN
            || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            let remaining = response
                .headers()
                .get("x-ratelimit-remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u32>().ok());

            if remaining == Some(0) {
                return Err(HttpError::non_retryable(
                    "GitHub API rate limit exceeded. Set GITHUB_TOKEN or GH_TOKEN environment variable to increase limit (5000 requests/hour with token vs 60/hour without).",
                ));
            }
        }

        // Check for HTTP errors
        if !status.is_success() {
            let is_retryable = HttpError::is_retryable_status(status);
            let error_msg = match status.as_u16() {
                502..=504 => {
                    format!(
                        "Network error: {} ({}).\n\n\
                        This is usually a temporary issue. Please try:\n\
                        1. Wait a moment and retry\n\
                        2. Check your internet connection\n\
                        3. If using a proxy, verify it's working correctly\n\
                        4. Try setting HTTPS_PROXY environment variable if behind a firewall",
                        status,
                        status.canonical_reason().unwrap_or("Server Error")
                    )
                }
                404 => {
                    format!(
                        "Resource not found (HTTP 404): {}\n\n\
                        The requested version may not exist or the URL has changed.",
                        url
                    )
                }
                401 | 403 => {
                    format!(
                        "Access denied (HTTP {}): {}\n\n\
                        Try setting GITHUB_TOKEN or GH_TOKEN environment variable for authentication.",
                        status.as_u16(),
                        url
                    )
                }
                _ => {
                    let body = response.text().await.unwrap_or_default();
                    // Don't show HTML content, it's not useful
                    if body.trim_start().starts_with('<') {
                        format!("HTTP {} for {}", status, url)
                    } else {
                        let preview = if body.len() > 200 {
                            format!("{}...", &body[..200])
                        } else {
                            body
                        };
                        format!("HTTP {} for {}: {}", status, url, preview)
                    }
                }
            };

            return if is_retryable {
                Err(HttpError::retryable(error_msg))
            } else {
                Err(HttpError::non_retryable(error_msg))
            };
        }

        // Be tolerant to broken/missing Content-Type headers (some proxies misbehave).
        // `reqwest::Response::json()` rejects non-JSON content-types; we parse from bytes instead.
        let bytes = response.bytes().await.map_err(|e| {
            // Check if this is a timeout error while reading the response body
            if e.is_timeout() {
                HttpError::retryable(format!(
                    "Timeout while reading response body from {}: {}",
                    url, e
                ))
            } else if e.is_body() || e.is_decode() {
                HttpError::retryable(format!("Error reading response body from {}: {}", url, e))
            } else {
                HttpError::retryable(format!("Failed to read response body from {}: {}", url, e))
            }
        })?;

        serde_json::from_slice::<serde_json::Value>(&bytes).map_err(|e| {
            let body = String::from_utf8_lossy(&bytes);
            let preview = if body.len() > 200 {
                format!("{}...", &body[..200])
            } else {
                body.to_string()
            };

            if preview.trim_start().starts_with('<') {
                HttpError::non_retryable(format!(
                    "Expected JSON but got HTML (content-type: '{}') from {}.\n\n\
                    This usually means your network/proxy replaced the GitHub API response.\n\
                    Try configuring HTTPS_PROXY / HTTP_PROXY, or set a working proxy/VPN.",
                    content_type, url
                ))
            } else {
                HttpError::non_retryable(format!(
                    "Failed to parse JSON from {} (content-type: '{}'): {}\n\n\
                    Body preview: {}",
                    url, content_type, e, preview
                ))
            }
        })
    }

    /// Extract a display name from URL (uv-style)
    ///
    /// For URLs like:
    /// - .../cpython-3.10.19+20251217-x86_64-pc-windows-msvc-install_only.tar.gz
    ///   → cpython-3.10.19-windows-x86_64
    /// - .../node-v20.10.0-win-x64.zip
    ///   → node-v20.10.0-win-x64
    pub(crate) fn extract_display_name_from_url(url: &str) -> String {
        // Get the filename from URL
        let filename = url.split('/').next_back().unwrap_or("download").to_string();

        // Try to extract a cleaner name for python-build-standalone
        // Pattern: cpython-{version}+{date}-{platform}-install_only.tar.gz
        if filename.starts_with("cpython-") {
            if let Some(caps) = regex::Regex::new(
                r"cpython-(\d+\.\d+\.\d+)\+\d+-(.+?)-install_only\.(tar\.gz|tar\.zst)",
            )
            .ok()
            .and_then(|re| re.captures(&filename))
            {
                let version = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let platform = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                // Simplify platform string
                let simplified_platform = Self::simplify_platform_string(platform);
                return format!("cpython-{}-{}", version, simplified_platform);
            }
        }

        // For other files, remove common extensions and simplify
        filename
            .trim_end_matches(".tar.gz")
            .trim_end_matches(".tar.zst")
            .trim_end_matches(".tar.xz")
            .trim_end_matches(".zip")
            .trim_end_matches(".7z")
            .to_string()
    }

    /// Simplify platform string for display
    fn simplify_platform_string(platform: &str) -> String {
        // x86_64-pc-windows-msvc-shared → windows-x86_64
        // x86_64-unknown-linux-gnu → linux-x86_64
        // aarch64-apple-darwin → darwin-aarch64
        let parts: Vec<&str> = platform.split('-').collect();

        if parts.len() >= 2 {
            let arch = parts[0];
            let os = if platform.contains("windows") {
                "windows"
            } else if platform.contains("linux") {
                "linux"
            } else if platform.contains("darwin") || platform.contains("apple") {
                "darwin"
            } else {
                parts.get(1).copied().unwrap_or("unknown")
            };
            format!("{}-{}", os, arch)
        } else {
            platform.to_string()
        }
    }
}

impl Default for RealHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Get GitHub token from environment variables or stored config
/// Checks in order: GITHUB_TOKEN, GH_TOKEN, ~/.vx/config/github_token
fn get_github_token() -> Option<String> {
    // First check environment variables (highest priority)
    if let Some(token) = std::env::var("GITHUB_TOKEN").ok().filter(|t| !t.is_empty()) {
        return Some(token);
    }

    if let Some(token) = std::env::var("GH_TOKEN").ok().filter(|t| !t.is_empty()) {
        return Some(token);
    }

    // Then check stored token file
    if let Ok(paths) = vx_paths::VxPaths::new() {
        let token_file = paths.config_dir.join("github_token");
        if token_file.exists() {
            if let Ok(token) = std::fs::read_to_string(&token_file) {
                let token = token.trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            }
        }
    }

    None
}

/// HTTP error that can be retried
#[derive(Debug)]
struct HttpError {
    message: String,
    is_retryable: bool,
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for HttpError {}

impl HttpError {
    fn retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_retryable: true,
        }
    }

    fn non_retryable(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_retryable: false,
        }
    }

    /// Check if the HTTP status code indicates a retryable error
    fn is_retryable_status(status: reqwest::StatusCode) -> bool {
        matches!(
            status.as_u16(),
            // Server errors that might be temporary
            500..=504 |
            // Rate limiting (but not auth errors)
            429
        )
    }
}

#[async_trait]
impl HttpClient for RealHttpClient {
    async fn get(&self, url: &str) -> Result<String> {
        let url = url.to_string();
        let client = self.client.clone();

        let result = (|| async {
            let mut request = client.get(&url);

            // Add GitHub token for GitHub API requests
            if url.contains("api.github.com") || url.contains("github.com") {
                if let Some(token) = get_github_token() {
                    request = request.header("Authorization", format!("Bearer {}", token));
                }
            }

            let response = request.send().await.map_err(|e| {
                if e.is_timeout() || e.is_connect() {
                    HttpError::retryable(format!("Network error: {}", e))
                } else {
                    HttpError::non_retryable(format!("Request failed: {}", e))
                }
            })?;

            let text = response
                .text()
                .await
                .map_err(|e| HttpError::non_retryable(format!("Failed to read response: {}", e)))?;

            Ok::<_, HttpError>(text)
        })
        .retry(Self::build_retry_strategy())
        .notify(|err: &HttpError, dur: Duration| {
            tracing::debug!(error = %err, retry_in = ?dur, url = %url, "Retrying HTTP request");
        })
        .when(|e: &HttpError| e.is_retryable)
        .await;

        result.map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn get_json_value(&self, url: &str) -> Result<serde_json::Value> {
        let url = url.to_string();
        let client = self.client.clone();

        let result = (|| async { self.fetch_json_once(&client, &url).await })
            .retry(Self::build_retry_strategy())
            .notify(|err: &HttpError, dur: Duration| {
                tracing::debug!(error = %err, retry_in = ?dur, url = %url, "Retrying JSON request");
            })
            .when(|e: &HttpError| e.is_retryable)
            .await;

        result.map_err(|e| anyhow::anyhow!("{}", e))
    }

    async fn download(&self, url: &str, dest: &Path) -> Result<()> {
        use futures_util::StreamExt;
        use indicatif::{ProgressBar, ProgressStyle};
        use tokio::io::AsyncWriteExt;

        // Optimize URL with CDN if enabled
        let download_url = self.optimize_url(url).await;
        let using_cdn = download_url != url;
        if using_cdn {
            tracing::info!(
                original = url,
                optimized = %download_url,
                "Using CDN accelerated URL"
            );
        }

        let response = self.client.get(&download_url).send().await;

        // If CDN URL failed, fallback to original URL
        let (response, actual_using_cdn) = match response {
            Ok(resp) if resp.status().is_success() => (resp, using_cdn),
            Ok(resp) if using_cdn => {
                tracing::warn!(
                    cdn_url = %download_url,
                    status = %resp.status(),
                    original_url = url,
                    "CDN download failed, falling back to original URL"
                );
                let fallback_resp = self.client.get(url).send().await?;
                if !fallback_resp.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Download failed: HTTP {} for {}",
                        fallback_resp.status(),
                        url
                    ));
                }
                (fallback_resp, false)
            }
            Ok(resp) => {
                return Err(anyhow::anyhow!(
                    "Download failed: HTTP {} for {}",
                    resp.status(),
                    url
                ));
            }
            Err(e) if using_cdn => {
                tracing::warn!(
                    cdn_url = %download_url,
                    error = %e,
                    original_url = url,
                    "CDN download error, falling back to original URL"
                );
                let fallback_resp = self.client.get(url).send().await?;
                if !fallback_resp.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Download failed: HTTP {} for {}",
                        fallback_resp.status(),
                        url
                    ));
                }
                (fallback_resp, false)
            }
            Err(e) => return Err(e.into()),
        };

        let total_size = response.content_length().unwrap_or(0);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        // Extract filename from URL for display (uv-style)
        let filename = Self::extract_display_name_from_url(url);
        let cdn_suffix = if actual_using_cdn { " [CDN]" } else { "" };

        // Create progress bar with uv-style format:
        // cpython-3.10.19-windows-x86_64-none (download) ━━━━━━━━━━━━━━ 1.47 MiB/21.49 MiB
        let progress_bar = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "{filename}{cdn_suffix} (download) {{wide_bar:.cyan/blue}} {{bytes}}/{{total_bytes}}"
                ))
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("━━╺"),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "{{spinner:.green}} {filename}{cdn_suffix} (download) {{bytes}}"
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

        // Finish with summary (uv-style: just clear the progress bar)
        progress_bar.finish_and_clear();

        file.flush().await?;
        Ok(())
    }

    async fn download_with_progress(
        &self,
        url: &str,
        dest: &Path,
        on_progress: &(dyn Fn(u64, u64) + Send + Sync),
    ) -> Result<()> {
        use tokio::io::AsyncWriteExt;

        // Optimize URL with CDN if enabled
        let download_url = self.optimize_url(url).await;
        if download_url != url {
            tracing::info!(
                original = url,
                optimized = %download_url,
                "Using CDN accelerated URL"
            );
        }
        let using_cdn = download_url != url;

        let response = self.client.get(&download_url).send().await;

        // If CDN URL failed, fallback to original URL
        let response = match response {
            Ok(resp) if resp.status().is_success() => resp,
            Ok(resp) if using_cdn => {
                tracing::warn!(
                    cdn_url = %download_url,
                    status = %resp.status(),
                    "CDN download failed, falling back to original URL"
                );
                let fallback_resp = self.client.get(url).send().await?;
                if !fallback_resp.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Download failed: HTTP {} for {}",
                        fallback_resp.status(),
                        url
                    ));
                }
                fallback_resp
            }
            Ok(resp) => {
                return Err(anyhow::anyhow!(
                    "Download failed: HTTP {} for {}",
                    resp.status(),
                    url
                ));
            }
            Err(e) if using_cdn => {
                tracing::warn!(
                    cdn_url = %download_url,
                    error = %e,
                    "CDN download error, falling back to original URL"
                );
                let fallback_resp = self.client.get(url).send().await?;
                if !fallback_resp.status().is_success() {
                    return Err(anyhow::anyhow!(
                        "Download failed: HTTP {} for {}",
                        fallback_resp.status(),
                        url
                    ));
                }
                fallback_resp
            }
            Err(e) => return Err(e.into()),
        };

        let total_size = response.content_length().unwrap_or(0);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = tokio::fs::File::create(dest).await?;
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            on_progress(total_size, downloaded);
        }

        file.flush().await?;
        Ok(())
    }

    async fn download_cached(&self, url: &str, dest: &Path) -> Result<bool> {
        use indicatif::{ProgressBar, ProgressStyle};

        // Check if we have a download cache
        let cache = match &self.download_cache {
            Some(c) => c,
            None => {
                // No cache, just download
                self.download(url, dest).await?;
                return Ok(false);
            }
        };

        // Check cache
        let lookup = cache.lookup(url);
        match lookup {
            vx_cache::CacheLookupResult::Hit { path, metadata } => {
                // Cache hit! Copy from cache
                let filename = Self::extract_display_name_from_url(url);
                let size_mb = metadata.size as f64 / 1_000_000.0;
                let pb = ProgressBar::new_spinner();
                pb.set_style(
                    ProgressStyle::with_template(&format!(
                        "{filename} (cached) {{spinner:.green}} {size_mb:.1} MB"
                    ))
                    .unwrap_or_else(|_| ProgressStyle::default_spinner()),
                );
                pb.enable_steady_tick(std::time::Duration::from_millis(50));

                // Ensure parent directory exists
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                std::fs::copy(&path, dest)?;
                pb.finish_and_clear();
                tracing::debug!(url = url, cached_path = ?path, "Served from download cache");
                return Ok(true);
            }
            vx_cache::CacheLookupResult::NeedsRevalidation { path, metadata } => {
                // Has ETag, could do conditional request but for simplicity use cached
                let filename = Self::extract_display_name_from_url(url);
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
                return Ok(true);
            }
            vx_cache::CacheLookupResult::Miss => {
                // Cache miss, need to download
            }
        }

        // Download to a temp file first
        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path().join("download");

        // Use standard download (which shows progress)
        self.download(url, &temp_path).await?;

        // Store in cache
        if let Err(e) = cache.store(url, &temp_path, None, None, None) {
            tracing::warn!(url = url, error = %e, "Failed to cache download");
        }

        // Move to final destination
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&temp_path, dest)?;

        Ok(false)
    }

    fn is_cached(&self, url: &str) -> bool {
        self.download_cache
            .as_ref()
            .is_some_and(|c| c.is_cached(url))
    }
}
