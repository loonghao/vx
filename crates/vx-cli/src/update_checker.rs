//! Background update checker with caching
//!
//! This module provides non-blocking version checking for vx.
//! It checks for updates asynchronously and caches the result to avoid
//! impacting command performance or being affected by network fluctuations.
//!
//! # Design Principles
//!
//! 1. **Non-blocking**: Version check runs with timeout, never blocks main command
//! 2. **Cached**: Results cached for 24 hours by default
//! 3. **Fault-tolerant**: Network failures don't affect vx usage
//! 4. **Self-healing**: Automatically retries after cooldown period
//!
//! # Cache File Format
//!
//! `~/.vx/cache/update_check.json`:
//! ```json
//! {
//!   "last_check": "2026-05-04T10:30:00Z",
//!   "latest_version": "0.8.35",
//!   "current_version": "0.8.32",
//!   "cache_duration_hours": 24,
//!   "check_failures": 0,
//!   "last_failure_time": null,
//!   "skip_until": null
//! }
//! ```

use anyhow::{Context, Result, anyhow};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Cache file name
const CACHE_FILE_NAME: &str = "update_check.json";

/// Default cache duration (24 hours)
const DEFAULT_CACHE_DURATION_HOURS: u64 = 24;

/// Cooldown period after consecutive failures (1 hour)
const FAILURE_COOLDOWN_SECS: u64 = 3600;

/// Maximum consecutive failures before cooldown
const MAX_CONSECUTIVE_FAILURES: u32 = 3;

/// Update check cache structure
#[derive(Debug, Serialize, Deserialize, Clone)]
struct UpdateCheckCache {
    /// When the last check was performed (ISO 8601 format)
    last_check: String,

    /// Latest version found during last successful check
    latest_version: String,

    /// Version of vx during last check (for comparison)
    current_version: String,

    /// Cache duration in hours (configurable)
    #[serde(default = "default_cache_duration")]
    cache_duration_hours: u64,

    /// Consecutive check failures count
    #[serde(default)]
    check_failures: u32,

    /// Last failure time (ISO 8601 format, optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    last_failure_time: Option<String>,

    /// Skip checking until this time (ISO 8601 format, optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    skip_until: Option<String>,
}

fn default_cache_duration() -> u64 {
    DEFAULT_CACHE_DURATION_HOURS
}

impl Default for UpdateCheckCache {
    fn default() -> Self {
        Self {
            last_check: "1970-01-01T00:00:00Z".to_string(),
            latest_version: env!("CARGO_PKG_VERSION").to_string(),
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            cache_duration_hours: DEFAULT_CACHE_DURATION_HOURS,
            check_failures: 0,
            last_failure_time: None,
            skip_until: None,
        }
    }
}

/// Get the cache file path
fn get_cache_path() -> Result<PathBuf> {
    let vx_dir = dirs::home_dir()
        .map(|h| h.join(".vx").join("cache"))
        .ok_or_else(|| anyhow!("Failed to determine home directory"))?;

    // Create cache directory if it doesn't exist
    if !vx_dir.exists() {
        fs::create_dir_all(&vx_dir)
            .with_context(|| format!("Failed to create cache directory: {}", vx_dir.display()))?;
    }

    Ok(vx_dir.join(CACHE_FILE_NAME))
}

/// Load cache from disk
fn load_cache() -> UpdateCheckCache {
    match get_cache_path() {
        Ok(path) if path.exists() => match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<UpdateCheckCache>(&content) {
                Ok(cache) => cache,
                Err(e) => {
                    tracing::debug!("Failed to parse cache file: {}, using default", e);
                    UpdateCheckCache::default()
                }
            },
            Err(e) => {
                tracing::debug!("Failed to read cache file: {}, using default", e);
                UpdateCheckCache::default()
            }
        },
        _ => UpdateCheckCache::default(),
    }
}

/// Save cache to disk
fn save_cache(cache: &UpdateCheckCache) -> Result<()> {
    let path = get_cache_path()?;
    let content = serde_json::to_string_pretty(cache).context("Failed to serialize cache")?;

    fs::write(&path, content)
        .with_context(|| format!("Failed to write cache file: {}", path.display()))?;

    tracing::debug!("Saved update check cache to {}", path.display());
    Ok(())
}

/// Check if cache is still valid
fn is_cache_valid(cache: &UpdateCheckCache) -> bool {
    // Check if we should skip checking due to failures
    if let Some(skip_until) = &cache.skip_until
        && let Ok(skip_time) = parse_iso8601_time(skip_until)
        && let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH)
        && skip_time > now.as_secs()
    {
        tracing::debug!("Skipping update check until {}", skip_until);
        return true; // Use cached data
    }

    // Check cache age
    if let Ok(last_check) = parse_iso8601_time(&cache.last_check)
        && let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH)
    {
        let age_hours = (now.as_secs() - last_check) / 3600;
        return age_hours < cache.cache_duration_hours;
    }

    false
}

/// Parse ISO 8601 time string to Unix timestamp
fn parse_iso8601_time(s: &str) -> Result<u64> {
    let dt = s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| anyhow!("Failed to parse time: {}", e))?;

    Ok(dt.timestamp() as u64)
}

/// Format SystemTime to ISO 8601 string
fn format_time(t: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Utc> = t.into();
    datetime.to_rfc3339()
}

/// Fetch latest version from CDN (non-blocking, best-effort)
async fn fetch_latest_version() -> Result<String> {
    use reqwest::header::{AUTHORIZATION, USER_AGENT};

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5)) // Short timeout to avoid blocking
        .build()
        .context("Failed to create HTTP client")?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        USER_AGENT,
        reqwest::header::HeaderValue::from_static("vx-cli/0.3.0"),
    );

    // Try CDN first (no auth needed, no rate limits)
    let cdn_url = "https://data.jsdelivr.com/v1/package/gh/loonghao/vx";

    tracing::debug!("Sending request to CDN...");
    match client.get(cdn_url).headers(headers.clone()).send().await {
        Ok(response) if response.status().is_success() => {
            tracing::debug!("CDN request successful, status: {}", response.status());
            if let Ok(json) = response.json::<serde_json::Value>().await
                && let Some(version) = json["versions"]
                    .as_array()
                    .and_then(|v| v.first())
                    .and_then(|v| v.as_str())
            {
                return Ok(version.to_string());
            }
        }
        _ => {}
    }

    // Fallback to GitHub API (may have rate limits)
    let github_url = "https://api.github.com/repos/loonghao/vx/releases/latest";

    // Add authorization if token is available
    if let Ok(token) = env::var("GITHUB_TOKEN").or_else(|_| env::var("VX_GITHUB_TOKEN"))
        && let Ok(header_value) =
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
    {
        headers.insert(AUTHORIZATION, header_value);
    }

    tracing::debug!("Sending request to GitHub API...");
    match client.get(github_url).headers(headers).send().await {
        Ok(response) if response.status().is_success() => {
            tracing::debug!(
                "GitHub API request successful, status: {}",
                response.status()
            );
            if let Ok(json) = response.json::<serde_json::Value>().await
                && let Some(tag) = json["tag_name"].as_str()
            {
                let version = tag
                    .trim_start_matches('v')
                    .trim_start_matches("vx-v")
                    .to_string();
                return Ok(version);
            }
        }
        _ => {}
    }

    Err(anyhow!("Failed to fetch latest version from all sources"))
}

/// Compare versions using vx_runtime_core utilities
fn is_newer_version(version_a: &str, version_b: &str) -> bool {
    vx_runtime_core::version_utils::is_newer_version(version_a, version_b)
}

/// Perform background update check
async fn perform_update_check(cache: &mut UpdateCheckCache) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    match fetch_latest_version().await {
        Ok(latest_version) => {
            // Reset failure count on success
            cache.check_failures = 0;
            cache.last_failure_time = None;
            cache.skip_until = None;

            // Update cache
            cache.last_check = format_time(SystemTime::now());
            cache.latest_version = latest_version;
            cache.current_version = current_version.to_string();

            save_cache(cache)?;

            tracing::debug!(
                "Update check successful: latest version = {}",
                cache.latest_version
            );
            Ok(())
        }
        Err(e) => {
            // Update failure info
            cache.check_failures += 1;
            cache.last_failure_time = Some(format_time(SystemTime::now()));

            // If too many failures, set cooldown
            if cache.check_failures >= MAX_CONSECUTIVE_FAILURES {
                let skip_until = SystemTime::now() + Duration::from_secs(FAILURE_COOLDOWN_SECS);
                cache.skip_until = Some(format_time(skip_until));

                tracing::warn!(
                    "Too many update check failures ({}), skipping for {} seconds",
                    cache.check_failures,
                    FAILURE_COOLDOWN_SECS
                );
            }

            save_cache(cache)?;

            Err(e)
        }
    }
}

/// Synchronous version check with timeout (for `notify_if_update_available`)
///
/// This function creates its own runtime and performs the update check with timeout.
/// It's designed to be called from `tokio::task::spawn_blocking()`.
fn do_update_check_sync() -> Option<String> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let mut cache = load_cache();

    // Check if we need to refresh cache
    if !is_cache_valid(&cache) {
        // Create a new runtime for this blocking task
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                let timeout_duration = Duration::from_secs(10);
                match rt.block_on(async {
                    tokio::time::timeout(timeout_duration, perform_update_check(&mut cache)).await
                }) {
                    Ok(Ok(())) => {
                        tracing::debug!("Synchronously refreshed update cache");
                    }
                    Ok(Err(e)) => {
                        tracing::debug!("Update check failed: {}", e);
                    }
                    Err(_timeout) => {
                        tracing::debug!(
                            "Update check timed out after {}s",
                            timeout_duration.as_secs()
                        );
                    }
                }
            }
            Err(e) => {
                tracing::debug!("Failed to create tokio runtime: {}", e);
            }
        }
    }

    // Use cached data to check if update is available
    if is_newer_version(&cache.latest_version, &current_version) {
        Some(cache.latest_version)
    } else {
        None
    }
}

/// Synchronous version check (for `vx self-update --check-only`)
///
/// This function performs an immediate version check and returns the result.
/// Used when the user explicitly requests a check.
pub async fn check_for_updates_sync() -> Result<Option<String>> {
    let current_version = env!("CARGO_PKG_VERSION");

    match fetch_latest_version().await {
        Ok(latest_version) => {
            let mut cache = load_cache();
            cache.last_check = format_time(SystemTime::now());
            cache.latest_version = latest_version.clone();
            cache.current_version = current_version.to_string();
            cache.check_failures = 0;
            cache.last_failure_time = None;
            cache.skip_until = None;

            let _ = save_cache(&cache);

            if is_newer_version(&latest_version, current_version) {
                Ok(Some(latest_version))
            } else {
                Ok(None)
            }
        }
        Err(e) => Err(e),
    }
}

/// Display update notification if a newer version is available
///
/// This function should be called after the main command execution.
/// It displays a non-intrusive notification to the user.
///
/// # Output Channel
///
/// Notifications are sent to **stderr** to avoid polluting structured output
/// (JSON/TOML) when vx is used by AI agents. This follows CLI best practices:
/// - stdout: Structured data (JSON, TOML, command output)
/// - stderr: Errors, warnings, hints, update notifications
///
/// # Returns
///
/// Returns a `JoinHandle` that should be awaited (with timeout) by the caller.
///
/// # Implementation
///
/// Uses `tokio::task::spawn_blocking()` to run a synchronous check
/// in a blocking thread, avoiding nested runtime issues.
pub fn notify_if_update_available() -> tokio::task::JoinHandle<()> {
    // Use spawn_blocking to run synchronous check
    let handle = tokio::runtime::Handle::current();
    handle.spawn_blocking(move || {
        if let Some(latest_version) = do_update_check_sync() {
            let current_version = env!("CARGO_PKG_VERSION");

            // Output to stderr to avoid polluting JSON/TOML stdout
            eprintln!(
                "{} A new version of vx is available: {} → {}",
                "ℹ".blue(),
                current_version,
                latest_version
            );
            eprintln!(
                "{} {}",
                "💡".cyan(),
                "Run 'vx self-update' to update to the latest version".dimmed()
            );
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.8.35", "0.8.32"));
        assert!(!is_newer_version("0.8.32", "0.8.35"));
        assert!(!is_newer_version("0.8.32", "0.8.32"));
    }

    #[test]
    fn test_update_check_cache_default() {
        let cache = UpdateCheckCache::default();
        assert_eq!(cache.cache_duration_hours, DEFAULT_CACHE_DURATION_HOURS);
        assert_eq!(cache.check_failures, 0);
        assert!(cache.last_failure_time.is_none());
        assert!(cache.skip_until.is_none());
    }

    #[test]
    fn test_cache_path() {
        let path = get_cache_path().unwrap();
        assert!(path.to_string_lossy().contains(".vx"));
        assert!(path.to_string_lossy().contains("update_check.json"));
    }
}
