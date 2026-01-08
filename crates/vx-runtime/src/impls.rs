//! Real implementations of runtime traits
//!
//! This module provides production implementations of the abstract traits
//! defined in `traits.rs`.

use crate::traits::{CommandExecutor, FileSystem, HttpClient, Installer, PathProvider};
use crate::types::ExecutionResult;
use anyhow::Result;
use async_trait::async_trait;
use backon::{ExponentialBuilder, Retryable};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use vx_paths::VxPaths;

// ============================================================================
// Real Path Provider
// ============================================================================

/// Real path provider using VxPaths
pub struct RealPathProvider {
    paths: VxPaths,
}

impl RealPathProvider {
    /// Create a new real path provider
    pub fn new() -> Result<Self> {
        Ok(Self {
            paths: VxPaths::new()?,
        })
    }

    /// Create with custom base directory
    pub fn with_base_dir(base_dir: impl AsRef<Path>) -> Self {
        Self {
            paths: VxPaths::with_base_dir(base_dir),
        }
    }
}

impl Default for RealPathProvider {
    fn default() -> Self {
        Self::new().expect("Failed to create RealPathProvider")
    }
}

impl PathProvider for RealPathProvider {
    fn vx_home(&self) -> PathBuf {
        self.paths.base_dir.clone()
    }

    fn store_dir(&self) -> PathBuf {
        self.paths.store_dir.clone()
    }

    fn envs_dir(&self) -> PathBuf {
        self.paths.envs_dir.clone()
    }

    fn bin_dir(&self) -> PathBuf {
        self.paths.bin_dir.clone()
    }

    fn cache_dir(&self) -> PathBuf {
        self.paths.cache_dir.clone()
    }

    fn config_dir(&self) -> PathBuf {
        self.paths.config_dir.clone()
    }

    fn runtime_store_dir(&self, name: &str) -> PathBuf {
        self.paths.runtime_store_dir(name)
    }

    fn version_store_dir(&self, name: &str, version: &str) -> PathBuf {
        self.paths.version_store_dir(name, version)
    }

    fn executable_path(&self, name: &str, version: &str) -> PathBuf {
        let exe_name = vx_paths::with_executable_extension(name);
        self.version_store_dir(name, version)
            .join("bin")
            .join(exe_name)
    }

    fn env_dir(&self, env_name: &str) -> PathBuf {
        self.paths.env_dir(env_name)
    }

    // ========== npm-tools paths ==========

    fn npm_tools_dir(&self) -> PathBuf {
        self.paths.npm_tools_dir.clone()
    }

    fn npm_tool_dir(&self, package_name: &str) -> PathBuf {
        self.paths.npm_tool_dir(package_name)
    }

    fn npm_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.npm_tool_version_dir(package_name, version)
    }

    fn npm_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.npm_tool_bin_dir(package_name, version)
    }

    // ========== pip-tools paths ==========

    fn pip_tools_dir(&self) -> PathBuf {
        self.paths.pip_tools_dir.clone()
    }

    fn pip_tool_dir(&self, package_name: &str) -> PathBuf {
        self.paths.pip_tool_dir(package_name)
    }

    fn pip_tool_version_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.pip_tool_version_dir(package_name, version)
    }

    fn pip_tool_venv_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.pip_tool_venv_dir(package_name, version)
    }

    fn pip_tool_bin_dir(&self, package_name: &str, version: &str) -> PathBuf {
        self.paths.pip_tool_bin_dir(package_name, version)
    }
}

// ============================================================================
// Real HTTP Client
// ============================================================================

/// Real HTTP client using reqwest with optional CDN acceleration
pub struct RealHttpClient {
    client: reqwest::Client,
    /// Whether CDN acceleration is enabled (controlled by cdn-acceleration feature)
    cdn_enabled: bool,
}

impl RealHttpClient {
    /// Create a new real HTTP client with default timeouts
    ///
    /// CDN acceleration is automatically enabled when the `cdn-acceleration` feature is active.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(format!("vx/{}", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(30))  // 30 seconds total timeout
                .connect_timeout(Duration::from_secs(10))  // 10 seconds connect timeout
                .build()
                .expect("Failed to create HTTP client"),
            cdn_enabled: cfg!(feature = "cdn-acceleration"),
        }
    }

    /// Create a new HTTP client with explicit CDN setting and default timeouts
    pub fn with_cdn(cdn_enabled: bool) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(format!("vx/{}", env!("CARGO_PKG_VERSION")))
                .timeout(Duration::from_secs(30))  // 30 seconds total timeout
                .connect_timeout(Duration::from_secs(10))  // 10 seconds connect timeout
                .build()
                .expect("Failed to create HTTP client"),
            cdn_enabled: cdn_enabled && cfg!(feature = "cdn-acceleration"),
        }
    }

    /// Create a new HTTP client with custom timeouts
    pub fn with_timeouts(
        cdn_enabled: bool,
        connect_timeout: Duration,
        total_timeout: Duration,
    ) -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent(format!("vx/{}", env!("CARGO_PKG_VERSION")))
                .timeout(total_timeout)
                .connect_timeout(connect_timeout)
                .build()
                .expect("Failed to create HTTP client"),
            cdn_enabled: cdn_enabled && cfg!(feature = "cdn-acceleration"),
        }
    }

    /// Check if CDN acceleration is enabled
    pub fn is_cdn_enabled(&self) -> bool {
        self.cdn_enabled
    }

    /// Optimize a download URL using CDN mirrors (if enabled)
    ///
    /// When CDN acceleration is enabled and the `cdn-acceleration` feature is active,
    /// this will return an optimized URL from the best available CDN mirror.
    /// Otherwise, it returns the original URL.
    async fn optimize_url(&self, url: &str) -> String {
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
            if e.is_timeout() || e.is_connect() {
                HttpError::retryable(format!("Network error: {}", e))
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
                    "GitHub API rate limit exceeded. Set GITHUB_TOKEN or GH_TOKEN environment variable to increase limit (5000 requests/hour with token vs 60/hour without)."
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
        let bytes = response
            .bytes()
            .await
            .map_err(|e| HttpError::retryable(format!("Failed to read response body: {}", e)))?;

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
}

impl Default for RealHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Get GitHub token from environment variables
/// Checks in order: GITHUB_TOKEN, GH_TOKEN
fn get_github_token() -> Option<String> {
    std::env::var("GITHUB_TOKEN")
        .ok()
        .or_else(|| std::env::var("GH_TOKEN").ok())
        .filter(|t| !t.is_empty())
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

            let text = response.text().await.map_err(|e| {
                HttpError::non_retryable(format!("Failed to read response: {}", e))
            })?;

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

        let result = (|| async {
            self.fetch_json_once(&client, &url).await
        })
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

        let response = self.client.get(&download_url).send().await?;

        // Check for successful response
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed: HTTP {} for {}",
                response.status(),
                if using_cdn { &download_url } else { url }
            ));
        }

        let total_size = response.content_length().unwrap_or(0);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        // Create progress bar with indicatif
        let cdn_indicator = if using_cdn { " [CDN]" } else { "" };
        let progress_bar = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::with_template(
                    &format!("  {{spinner:.green}} Downloading{cdn_indicator} {{wide_bar:.cyan/blue}} {{bytes}}/{{total_bytes}} ({{bytes_per_sec}}, {{eta}})")
                )
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("━━╺"),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "  {{spinner:.green}} Downloading{cdn_indicator} {{bytes}} ({{bytes_per_sec}})"
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

        // Finish with summary
        let downloaded = progress_bar.position();
        progress_bar.finish_with_message(format!(
            "Downloaded{} {:.1} MB",
            cdn_indicator,
            downloaded as f64 / 1_000_000.0
        ));

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

        let response = self.client.get(&download_url).send().await?;

        // Check for successful response
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Download failed: HTTP {} for {}",
                response.status(),
                &download_url
            ));
        }

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
}

// ============================================================================
// Real File System
// ============================================================================

/// Real file system implementation
pub struct RealFileSystem;

impl RealFileSystem {
    /// Create a new real file system
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for RealFileSystem {
    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::remove_dir_all(path)?;
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries: Vec<PathBuf> = std::fs::read_dir(path)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();
        Ok(entries)
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        let content = std::fs::read_to_string(path)?;
        Ok(content)
    }

    fn read(&self, path: &Path) -> Result<Vec<u8>> {
        let content = std::fs::read(path)?;
        Ok(content)
    }

    fn write(&self, path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    fn write_bytes(&self, path: &Path, content: &[u8]) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<()> {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(from, to)?;
        Ok(())
    }

    fn hard_link(&self, src: &Path, dst: &Path) -> Result<()> {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::hard_link(src, dst)?;
        Ok(())
    }

    fn symlink(&self, src: &Path, dst: &Path) -> Result<()> {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(src, dst)?;
        #[cfg(windows)]
        {
            if src.is_dir() {
                std::os::windows::fs::symlink_dir(src, dst)?;
            } else {
                std::os::windows::fs::symlink_file(src, dst)?;
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    fn set_permissions(&self, path: &Path, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, permissions)?;
        Ok(())
    }
}

// ============================================================================
// Real Command Executor
// ============================================================================

/// Real command executor
pub struct RealCommandExecutor;

impl RealCommandExecutor {
    /// Create a new real command executor
    pub fn new() -> Self {
        Self
    }
}

impl Default for RealCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CommandExecutor for RealCommandExecutor {
    async fn execute(
        &self,
        program: &str,
        args: &[String],
        working_dir: Option<&Path>,
        env: &HashMap<String, String>,
        capture_output: bool,
    ) -> Result<ExecutionResult> {
        use std::process::Stdio;
        use tokio::process::Command;

        let mut cmd = Command::new(program);
        cmd.args(args);

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in env {
            cmd.env(key, value);
        }

        if capture_output {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let output = cmd.output().await?;

            Ok(ExecutionResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: Some(String::from_utf8_lossy(&output.stdout).to_string()),
                stderr: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            })
        } else {
            cmd.stdin(Stdio::inherit());
            cmd.stdout(Stdio::inherit());
            cmd.stderr(Stdio::inherit());

            let status = cmd.status().await?;

            Ok(ExecutionResult {
                exit_code: status.code().unwrap_or(-1),
                stdout: None,
                stderr: None,
            })
        }
    }

    fn which(&self, program: &str) -> Option<PathBuf> {
        which::which(program).ok()
    }
}

// ============================================================================
// Real Installer
// ============================================================================

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
        } else if archive_str.ends_with(".zip") {
            Some("zip")
        } else {
            // Try to detect by magic bytes
            let file = std::fs::File::open(archive)?;
            use std::io::Read;
            let mut magic = [0u8; 4];
            if (&file).take(4).read(&mut magic).is_ok() {
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
            Some("zip") => {
                let file = std::fs::File::open(archive)?;
                let mut archive = zip::ZipArchive::new(file)?;
                archive.extract(dest)?;
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
        let archive_name = url_without_fragment
            .split('/')
            .next_back()
            .unwrap_or("archive");

        // Check for extension hint in URL fragment
        let extension_hint = url.split('#').nth(1);

        let temp_path = temp_dir.path().join(archive_name);

        // Download
        self.http.download(url_without_fragment, &temp_path).await?;

        // Check if it's an archive or a single executable
        // First check the URL/filename, then check extension hint, then check file magic bytes
        let archive_str = archive_name.to_lowercase();
        let mut is_archive = archive_str.ends_with(".tar.gz")
            || archive_str.ends_with(".tgz")
            || archive_str.ends_with(".tar.xz")
            || archive_str.ends_with(".zip");

        // Check extension hint from URL fragment
        if !is_archive {
            if let Some(hint) = extension_hint {
                is_archive = hint.ends_with(".tar.gz")
                    || hint.ends_with(".tgz")
                    || hint.ends_with(".tar.xz")
                    || hint.ends_with(".zip");
            }
        }

        // Check file magic bytes if still uncertain
        if !is_archive {
            if let Ok(mut file) = std::fs::File::open(&temp_path) {
                use std::io::Read;
                let mut magic = [0u8; 4];
                if file.read_exact(&mut magic).is_ok() {
                    // ZIP magic: PK\x03\x04
                    // GZIP magic: \x1f\x8b
                    is_archive = (magic[0] == 0x50 && magic[1] == 0x4B)  // ZIP
                        || (magic[0] == 0x1f && magic[1] == 0x8b); // GZIP (tar.gz)
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

// ============================================================================
// Context Factory
// ============================================================================

use crate::context::RuntimeContext;
use crate::version_cache::VersionCache;
use std::sync::Arc;

/// Create a real runtime context for production use
pub fn create_runtime_context() -> Result<RuntimeContext> {
    let paths = Arc::new(RealPathProvider::new()?);
    let http = Arc::new(RealHttpClient::new());
    let fs = Arc::new(RealFileSystem::new());
    let installer = Arc::new(RealInstaller::new());

    // Create version cache in the cache directory
    let cache_dir = paths.cache_dir().join("versions");
    let version_cache = VersionCache::new(cache_dir);

    Ok(RuntimeContext::new(paths, http, fs, installer).with_version_cache(version_cache))
}

/// Create a real runtime context with custom base directory
pub fn create_runtime_context_with_base(base_dir: impl AsRef<Path>) -> RuntimeContext {
    let base_dir = base_dir.as_ref();
    let paths = Arc::new(RealPathProvider::with_base_dir(base_dir));
    let http = Arc::new(RealHttpClient::new());
    let fs = Arc::new(RealFileSystem::new());
    let installer = Arc::new(RealInstaller::new());

    // Create version cache in the cache directory
    let cache_dir = paths.cache_dir().join("versions");
    let version_cache = VersionCache::new(cache_dir);

    RuntimeContext::new(paths, http, fs, installer).with_version_cache(version_cache)
}
