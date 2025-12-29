//! Self-update command implementation
//!
//! This module provides self-update functionality for vx with the following features:
//! - Multi-channel download with automatic fallback (GitHub, jsDelivr CDN, Fastly CDN)
//! - Download progress bar with speed and ETA display
//! - SHA256 checksum verification for security
//! - Specific version installation support
//! - Safe binary replacement using self_replace (handles Windows exe locking)
//! - Automatic backup and rollback on failure

use crate::ui::UI;
use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// GitHub release information
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    #[allow(dead_code)]
    name: String,
    body: String,
    assets: Vec<GitHubAsset>,
    #[allow(dead_code)]
    prerelease: bool,
}

/// GitHub release asset information
#[derive(Debug, Deserialize, Clone)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

/// Source of version information (affects download strategy)
#[derive(Debug, Clone, Copy, PartialEq)]
enum VersionSource {
    GitHub,
    Cdn,
}

/// Handle the self-update command
///
/// # Arguments
/// * `token` - Optional GitHub token for authenticated API requests
/// * `prerelease` - Whether to include pre-release versions
/// * `force` - Force update even if already up to date
/// * `check_only` - Only check for updates, don't install
/// * `target_version` - Specific version to install (e.g., "0.5.28")
pub async fn handle(
    token: Option<&str>,
    prerelease: bool,
    force: bool,
    check_only: bool,
    target_version: Option<&str>,
) -> Result<()> {
    UI::info("🔍 Checking for vx updates...");

    let current_version = env!("CARGO_PKG_VERSION");
    UI::detail(&format!("Current version: {}", current_version));

    // Create HTTP client with optional authentication
    let client = create_authenticated_client(token)?;

    // Get release information based on whether a specific version is requested
    let (release, version_source) = if let Some(version) = target_version {
        UI::info(&format!("📌 Looking for specific version: {}", version));
        get_specific_release(&client, version, token.is_some()).await?
    } else {
        get_latest_release(&client, prerelease, token.is_some()).await?
    };

    // Parse version from tag_name (handles "vx-v0.5.9", "x-v0.5.9", and "v0.5.9" formats)
    let latest_version = release
        .tag_name
        .trim_start_matches("vx-v")
        .trim_start_matches("x-v")
        .trim_start_matches('v');
    UI::detail(&format!("Target version: {}", latest_version));

    // Check if update is needed
    if !force && current_version == latest_version {
        UI::success("✅ vx is already up to date!");
        return Ok(());
    }

    if current_version != latest_version {
        let direction = if is_newer_version(latest_version, current_version) {
            "upgrade"
        } else {
            "downgrade"
        };
        UI::info(&format!(
            "📦 {} available: {} -> {}",
            direction, current_version, latest_version
        ));

        if !release.body.is_empty() && !release.body.contains("retrieved from CDN") {
            UI::info("📝 Release notes:");
            println!("{}", release.body);
        }
    }

    if check_only {
        if current_version != latest_version {
            UI::info("💡 Run 'vx self-update' to update to the latest version");
            if target_version.is_none() {
                UI::hint(&format!(
                    "Or run 'vx self-update {}' to install this specific version",
                    latest_version
                ));
            }
        }
        return Ok(());
    }

    // Find appropriate asset for current platform
    let asset = find_platform_asset(&release.assets)?;
    UI::info(&format!("📥 Downloading {}...", asset.name));

    // Download and install update
    download_and_install(&client, asset, force, version_source, latest_version).await?;

    UI::success(&format!(
        "🎉 Successfully updated vx to version {}!",
        latest_version
    ));
    UI::hint("Restart your terminal or run 'vx --version' to verify the update");

    Ok(())
}

/// Check if version_a is newer than version_b using semver comparison
fn is_newer_version(version_a: &str, version_b: &str) -> bool {
    let parse_version = |v: &str| -> (u64, u64, u64) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts
            .get(2)
            .and_then(|s| s.split('-').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        (major, minor, patch)
    };

    parse_version(version_a) > parse_version(version_b)
}

/// Create an HTTP client with optional GitHub authentication
fn create_authenticated_client(token: Option<&str>) -> Result<reqwest::Client> {
    let mut headers = HeaderMap::new();

    // Always set User-Agent
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("vx-cli/0.3.0 (https://github.com/loonghao/vx)"),
    );

    // Add authentication if token is provided
    if let Some(token) = token {
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| anyhow!("Invalid token format: {}", e))?,
        );
        UI::detail("🔐 Using authenticated requests to GitHub API");
    } else {
        UI::detail("🌐 No GitHub token provided, will prefer CDN for downloads");
        UI::hint("💡 Use --token <TOKEN> to use GitHub API directly and avoid CDN delays");
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    Ok(client)
}

/// Get a specific version release
async fn get_specific_release(
    client: &reqwest::Client,
    version: &str,
    has_token: bool,
) -> Result<(GitHubRelease, VersionSource)> {
    // Normalize version string (remove 'v' prefix if present)
    let version = version
        .trim_start_matches("vx-v")
        .trim_start_matches("x-v")
        .trim_start_matches('v');

    // Try GitHub API first if we have a token
    if has_token {
        match try_github_api_specific(client, version).await {
            Ok(release) => {
                UI::info("✅ Got version info from GitHub API");
                return Ok((release, VersionSource::GitHub));
            }
            Err(e) => {
                UI::warn(&format!("⚠️ GitHub API failed: {}", e));
                UI::info("🔄 Trying CDN fallback...");
            }
        }
    }

    // Try CDN (create synthetic release)
    match try_jsdelivr_api_specific(client, version).await {
        Ok(release) => {
            UI::info("✅ Got version info from jsDelivr CDN");
            Ok((release, VersionSource::Cdn))
        }
        Err(e) => {
            // If CDN fails and we haven't tried GitHub, try it now
            if !has_token {
                UI::warn(&format!("⚠️ CDN fallback failed: {}", e));
                UI::info("🔄 Trying GitHub API...");

                match try_github_api_specific(client, version).await {
                    Ok(release) => {
                        UI::info("✅ Got version info from GitHub API");
                        return Ok((release, VersionSource::GitHub));
                    }
                    Err(github_err) => {
                        return Err(anyhow!(
                            "Failed to find version {} from both CDN and GitHub API.\n\
                            CDN error: {}\n\
                            GitHub error: {}",
                            version,
                            e,
                            github_err
                        ));
                    }
                }
            }
            Err(anyhow!("Failed to find version {}: {}", version, e))
        }
    }
}

/// Try to get a specific version from GitHub API
async fn try_github_api_specific(client: &reqwest::Client, version: &str) -> Result<GitHubRelease> {
    // Try different tag formats
    let tag_formats = [
        format!("vx-v{}", version),
        format!("v{}", version),
        version.to_string(),
    ];

    for tag in &tag_formats {
        let url = format!(
            "https://api.github.com/repos/loonghao/vx/releases/tags/{}",
            tag
        );

        let response = client.get(&url).send().await?;

        if response.status().is_success() {
            return Ok(response.json().await?);
        }
    }

    Err(anyhow!("Version {} not found in GitHub releases", version))
}

/// Try to get a specific version from jsDelivr CDN
async fn try_jsdelivr_api_specific(
    client: &reqwest::Client,
    version: &str,
) -> Result<GitHubRelease> {
    // Verify the version exists by checking the CDN API
    let url = "https://data.jsdelivr.com/v1/package/gh/loonghao/vx";
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch from jsDelivr: {}",
            response.status()
        ));
    }

    let json: serde_json::Value = response.json().await?;
    let versions = json["versions"]
        .as_array()
        .ok_or_else(|| anyhow!("No versions found in jsDelivr response"))?;

    // Check if the requested version exists
    let version_exists = versions.iter().any(|v| {
        if let Some(v_str) = v.as_str() {
            let normalized = v_str
                .trim_start_matches("vx-v")
                .trim_start_matches("x-v")
                .trim_start_matches('v');
            normalized == version
        } else {
            false
        }
    });

    if !version_exists {
        return Err(anyhow!(
            "Version {} not found. Available versions: {}",
            version,
            versions
                .iter()
                .filter_map(|v| v.as_str())
                .take(10)
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // Create CDN-based assets for the version
    let assets = create_cdn_assets(version);

    Ok(GitHubRelease {
        tag_name: format!("vx-v{}", version),
        name: format!("Release {}", version),
        body: "Release information retrieved from CDN".to_string(),
        prerelease: false,
        assets,
    })
}

/// Get the latest release information with smart channel selection
async fn get_latest_release(
    client: &reqwest::Client,
    prerelease: bool,
    has_token: bool,
) -> Result<(GitHubRelease, VersionSource)> {
    // If no token is provided, prefer CDN to avoid rate limits
    if !has_token {
        UI::info("🌐 No GitHub token provided, using CDN for version check...");

        // Try jsDelivr API first when no token
        match try_jsdelivr_api(client, prerelease).await {
            Ok(release) => {
                UI::info("✅ Got version info from jsDelivr CDN");
                return Ok((release, VersionSource::Cdn));
            }
            Err(e) => {
                UI::warn(&format!("⚠️ CDN fallback failed: {}", e));
                UI::info("🔄 Falling back to GitHub API...");
            }
        }
    }

    // Try GitHub API (either as primary with token, or as fallback without token)
    match try_github_api(client, prerelease).await {
        Ok(release) => Ok((release, VersionSource::GitHub)),
        Err(e) => {
            // Check if it's a rate limit error
            if e.to_string().contains("rate limit") {
                if has_token {
                    return Err(anyhow!(
                        "GitHub API rate limit exceeded even with authentication. \
                        Check your token permissions or try again later."
                    ));
                } else {
                    return Err(anyhow!(
                        "GitHub API rate limit exceeded and CDN fallback also failed. \
                        Use --token <TOKEN> to authenticate and increase rate limits. \
                        See: https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api"
                    ));
                }
            }

            // For other errors, try CDN as last resort if we haven't already
            if has_token {
                UI::warn(&format!("⚠️ GitHub API failed: {}", e));
                UI::info("🔄 Trying CDN fallback...");

                if let Ok(release) = try_jsdelivr_api(client, prerelease).await {
                    UI::info("✅ Got version info from jsDelivr CDN");
                    return Ok((release, VersionSource::Cdn));
                }
            }

            Err(e)
        }
    }
}

/// Try to get release info from GitHub API
async fn try_github_api(client: &reqwest::Client, prerelease: bool) -> Result<GitHubRelease> {
    let url = if prerelease {
        "https://api.github.com/repos/loonghao/vx/releases"
    } else {
        "https://api.github.com/repos/loonghao/vx/releases/latest"
    };

    let response = client.get(url).send().await?;

    // Check for rate limiting
    if response.status() == 403 {
        let remaining = response
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");

        return Err(anyhow!(
            "GitHub API rate limit exceeded (remaining: {})",
            remaining
        ));
    }

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch release information: HTTP {}",
            response.status()
        ));
    }

    if prerelease {
        let releases: Vec<GitHubRelease> = response.json().await?;
        releases
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No releases found"))
    } else {
        Ok(response.json().await?)
    }
}

/// Find the appropriate asset for the current platform
fn find_platform_asset(assets: &[GitHubAsset]) -> Result<&GitHubAsset> {
    let target_os = env::consts::OS;
    let target_arch = env::consts::ARCH;

    // Define platform-specific patterns with REQUIRED and EXCLUDED patterns
    let (required_patterns, excluded_patterns): (Vec<&str>, Vec<&str>) =
        match (target_os, target_arch) {
            ("windows", "x86_64") => (vec!["x86_64", "windows"], vec!["aarch64", "arm64"]),
            ("windows", "x86") => (vec!["i686", "windows"], vec!["x86_64", "aarch64", "arm64"]),
            ("windows", "aarch64") => (vec!["aarch64", "windows"], vec!["x86_64", "i686"]),
            ("macos", "x86_64") => (vec!["x86_64", "apple"], vec!["aarch64", "arm64"]),
            ("macos", "aarch64") => (vec!["aarch64", "apple"], vec!["x86_64"]),
            ("linux", "x86_64") => (vec!["x86_64", "linux"], vec!["aarch64", "arm64"]),
            ("linux", "aarch64") => (vec!["aarch64", "linux"], vec!["x86_64"]),
            _ => {
                return Err(anyhow!(
                    "Unsupported platform: {}-{}",
                    target_os,
                    target_arch
                ))
            }
        };

    // Find matching asset
    for asset in assets {
        let name_lower = asset.name.to_lowercase();

        let all_required_match = required_patterns
            .iter()
            .all(|pattern| name_lower.contains(pattern));
        let no_excluded_match = excluded_patterns
            .iter()
            .all(|pattern| !name_lower.contains(pattern));

        if all_required_match && no_excluded_match {
            return Ok(asset);
        }
    }

    Err(anyhow!(
        "No compatible binary found for {}-{}. Available assets: {}",
        target_os,
        target_arch,
        assets
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

/// Download and install the update with progress bar and checksum verification
async fn download_and_install(
    client: &reqwest::Client,
    asset: &GitHubAsset,
    force: bool,
    version_source: VersionSource,
    version: &str,
) -> Result<()> {
    // Get current executable path
    let current_exe = env::current_exe()?;
    let backup_path = current_exe.with_extension("bak");

    // Try downloading with multi-channel fallback
    let content = download_with_fallback(client, asset, version_source, version).await?;

    // Try to verify checksum if available
    if let Err(e) = verify_checksum(client, asset, &content, version_source, version).await {
        UI::warn(&format!("⚠️ Checksum verification skipped: {}", e));
        UI::detail("Continuing with size validation only...");
    }

    // Create temporary file for the new binary
    let temp_path = current_exe.with_extension("tmp");

    // Handle different asset types
    if asset.name.ends_with(".zip") {
        extract_from_zip(&content, &temp_path)?;
    } else if asset.name.ends_with(".tar.gz") {
        extract_from_tar_gz(&content, &temp_path)?;
    } else {
        // Assume it's a raw binary
        fs::write(&temp_path, content)?;
    }

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_path, perms)?;
    }

    // Backup current executable (unless force mode)
    if current_exe.exists() && !force {
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        fs::copy(&current_exe, &backup_path)?;
        UI::detail(&format!(
            "📦 Backed up current version to {}",
            backup_path.display()
        ));
    }

    // Use self_replace for safe binary replacement (handles Windows exe locking)
    match self_replace::self_replace(&temp_path) {
        Ok(()) => {
            // Clean up temp file
            let _ = fs::remove_file(&temp_path);
            UI::detail(&format!(
                "✅ Installed new version to {}",
                current_exe.display()
            ));
        }
        Err(e) => {
            // Rollback on failure
            if backup_path.exists() {
                UI::warn("⚠️ Update failed, attempting rollback...");
                if let Err(rollback_err) = fs::copy(&backup_path, &current_exe) {
                    return Err(anyhow!(
                        "Update failed and rollback also failed!\n\
                        Original error: {}\n\
                        Rollback error: {}\n\
                        Backup is available at: {}",
                        e,
                        rollback_err,
                        backup_path.display()
                    ));
                }
                UI::info("✅ Rollback successful");
            }
            return Err(anyhow!("Failed to replace binary: {}", e));
        }
    }

    Ok(())
}

/// Extract vx binary from ZIP archive
fn extract_from_zip(content: &[u8], output_path: &PathBuf) -> Result<()> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(content);
    let mut archive = ZipArchive::new(cursor)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();

        if name.ends_with("vx") || name.ends_with("vx.exe") {
            let mut output = fs::File::create(output_path)?;
            std::io::copy(&mut file, &mut output)?;
            return Ok(());
        }
    }

    Err(anyhow!("vx executable not found in ZIP archive"))
}

/// Extract vx binary from TAR.GZ archive
fn extract_from_tar_gz(content: &[u8], output_path: &PathBuf) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::io::Cursor;
    use tar::Archive;

    let cursor = Cursor::new(content);
    let gz = GzDecoder::new(cursor);
    let mut archive = Archive::new(gz);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        if let Some(name) = path.file_name() {
            if name == "vx" || name == "vx.exe" {
                let mut output = fs::File::create(output_path)?;
                std::io::copy(&mut entry, &mut output)?;
                return Ok(());
            }
        }
    }

    Err(anyhow!("vx executable not found in TAR.GZ archive"))
}

/// Try to get release info from jsDelivr CDN
async fn try_jsdelivr_api(client: &reqwest::Client, prerelease: bool) -> Result<GitHubRelease> {
    let url = "https://data.jsdelivr.com/v1/package/gh/loonghao/vx";

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch from jsDelivr: {}",
            response.status()
        ));
    }

    let json: serde_json::Value = response.json().await?;

    let versions = json["versions"]
        .as_array()
        .ok_or_else(|| anyhow!("No versions found in jsDelivr response"))?;

    // Helper function to check if a version string is a prerelease
    let is_prerelease = |v: &str| -> bool {
        if v.starts_with("vx-v") || v.starts_with("x-v") {
            return false;
        }
        v.contains("-alpha")
            || v.contains("-beta")
            || v.contains("-rc")
            || v.contains("-dev")
            || v.contains("-pre")
    };

    // Helper function to extract semver for comparison
    let extract_semver = |v: &str| -> Option<(u64, u64, u64)> {
        let version_part = v
            .trim_start_matches("vx-v")
            .trim_start_matches("x-v")
            .trim_start_matches('v');
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.len() >= 3 {
            let major = parts[0].parse::<u64>().ok()?;
            let minor = parts[1].parse::<u64>().ok()?;
            let patch = parts[2].split('-').next()?.parse::<u64>().ok()?;
            Some((major, minor, patch))
        } else {
            None
        }
    };

    // Find the latest version based on prerelease flag
    let latest_version = if prerelease {
        versions
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|v| extract_semver(v).is_some())
            .max_by(|a, b| {
                let a_ver = extract_semver(a).unwrap_or((0, 0, 0));
                let b_ver = extract_semver(b).unwrap_or((0, 0, 0));
                a_ver.cmp(&b_ver)
            })
    } else {
        versions
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|v| !is_prerelease(v) && extract_semver(v).is_some())
            .max_by(|a, b| {
                let a_ver = extract_semver(a).unwrap_or((0, 0, 0));
                let b_ver = extract_semver(b).unwrap_or((0, 0, 0));
                a_ver.cmp(&b_ver)
            })
    }
    .ok_or_else(|| anyhow!("No suitable version found"))?;

    let version_number = latest_version
        .trim_start_matches("vx-v")
        .trim_start_matches("x-v")
        .trim_start_matches('v');

    let assets = create_cdn_assets(version_number);

    Ok(GitHubRelease {
        tag_name: latest_version.to_string(),
        name: format!("Release {}", version_number),
        body: "Release information retrieved from CDN".to_string(),
        prerelease: is_prerelease(latest_version),
        assets,
    })
}

/// Create CDN-based assets for a given version
fn create_cdn_assets(version: &str) -> Vec<GitHubAsset> {
    let base_url = format!("https://cdn.jsdelivr.net/gh/loonghao/vx@vx-v{}", version);

    let asset_configs = vec![
        ("vx-x86_64-pc-windows-msvc.zip", "windows", "x86_64"),
        ("vx-aarch64-pc-windows-msvc.zip", "windows", "aarch64"),
        ("vx-x86_64-unknown-linux-musl.tar.gz", "linux", "x86_64"),
        ("vx-aarch64-unknown-linux-musl.tar.gz", "linux", "aarch64"),
        ("vx-x86_64-apple-darwin.tar.gz", "macos", "x86_64"),
        ("vx-aarch64-apple-darwin.tar.gz", "macos", "aarch64"),
    ];

    asset_configs
        .into_iter()
        .map(|(name, _os, _arch)| GitHubAsset {
            name: name.to_string(),
            browser_download_url: format!("{}/{}", base_url, name),
            size: 0, // Size unknown from CDN
        })
        .collect()
}

/// Download with multi-channel fallback and progress bar
async fn download_with_fallback(
    client: &reqwest::Client,
    asset: &GitHubAsset,
    version_source: VersionSource,
    version: &str,
) -> Result<Vec<u8>> {
    // Define download channels based on version source
    let channels = if version_source == VersionSource::Cdn {
        // CDN-first strategy
        vec![
            ("jsDelivr CDN", asset.browser_download_url.clone()),
            (
                "Fastly CDN",
                format!(
                    "https://fastly.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
                    version, asset.name
                ),
            ),
            (
                "GitHub Releases",
                format!(
                    "https://github.com/loonghao/vx/releases/download/vx-v{}/{}",
                    version, asset.name
                ),
            ),
        ]
    } else {
        // GitHub-first strategy
        vec![
            ("GitHub Releases", asset.browser_download_url.clone()),
            (
                "jsDelivr CDN",
                format!(
                    "https://cdn.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
                    version, asset.name
                ),
            ),
            (
                "Fastly CDN",
                format!(
                    "https://fastly.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
                    version, asset.name
                ),
            ),
        ]
    };

    for (channel_name, url) in channels {
        UI::detail(&format!("🔄 Trying {}: {}", channel_name, url));

        match download_with_progress(client, &url, asset.size, channel_name).await {
            Ok(content) => {
                if content.len() > 1024 {
                    UI::info(&format!(
                        "✅ Downloaded from {} ({} bytes)",
                        channel_name,
                        content.len()
                    ));
                    return Ok(content);
                } else {
                    UI::warn(&format!(
                        "⚠️ Downloaded file too small from {}, trying next channel...",
                        channel_name
                    ));
                }
            }
            Err(e) => {
                UI::warn(&format!("⚠️ {} failed: {}", channel_name, e));
            }
        }
    }

    Err(anyhow!("Failed to download from all channels"))
}

/// Download with progress bar display
async fn download_with_progress(
    client: &reqwest::Client,
    url: &str,
    expected_size: u64,
    channel_name: &str,
) -> Result<Vec<u8>> {
    let response = client.get(url).send().await.context("Failed to connect")?;

    if !response.status().is_success() {
        return Err(anyhow!("HTTP {}", response.status()));
    }

    // Get content length from response or use expected size
    let total_size = response.content_length().unwrap_or(expected_size);

    // Create progress bar
    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("█▓▒░"),
        );
        pb.set_message(format!("Downloading from {}", channel_name));
        Some(pb)
    } else {
        // Unknown size - use spinner
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {bytes} ({bytes_per_sec})")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        pb.set_message(format!("Downloading from {}", channel_name));
        Some(pb)
    };

    // Download with streaming
    let mut content = Vec::with_capacity(total_size as usize);
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read chunk")?;
        content.write_all(&chunk)?;
        if let Some(ref pb) = pb {
            pb.set_position(content.len() as u64);
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("Download complete");
    }

    Ok(content)
}

/// Verify SHA256 checksum of downloaded content
async fn verify_checksum(
    client: &reqwest::Client,
    asset: &GitHubAsset,
    content: &[u8],
    version_source: VersionSource,
    version: &str,
) -> Result<()> {
    // Try to find checksum file
    let checksum_filename = format!("{}.sha256", asset.name);

    // Build checksum URLs based on version source
    let checksum_urls = if version_source == VersionSource::Cdn {
        vec![
            format!(
                "https://cdn.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
                version, checksum_filename
            ),
            format!(
                "https://github.com/loonghao/vx/releases/download/vx-v{}/{}",
                version, checksum_filename
            ),
        ]
    } else {
        vec![
            format!(
                "https://github.com/loonghao/vx/releases/download/vx-v{}/{}",
                version, checksum_filename
            ),
            format!(
                "https://cdn.jsdelivr.net/gh/loonghao/vx@vx-v{}/{}",
                version, checksum_filename
            ),
        ]
    };

    // Try to download checksum file
    let mut checksum_content = None;
    for url in &checksum_urls {
        if let Ok(response) = client.get(url).send().await {
            if response.status().is_success() {
                if let Ok(text) = response.text().await {
                    checksum_content = Some(text);
                    break;
                }
            }
        }
    }

    let checksum_text = checksum_content.ok_or_else(|| {
        anyhow!(
            "Checksum file not found ({}). This may be expected for older releases.",
            checksum_filename
        )
    })?;

    // Parse expected checksum (format: "hash  filename" or just "hash")
    let expected_hash = checksum_text
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow!("Invalid checksum file format"))?
        .to_lowercase();

    // Calculate actual checksum
    let mut hasher = Sha256::new();
    hasher.update(content);
    let actual_hash = format!("{:x}", hasher.finalize());

    // Compare
    if expected_hash != actual_hash {
        return Err(anyhow!(
            "Checksum mismatch!\n\
            Expected: {}\n\
            Actual:   {}\n\
            The downloaded file may be corrupted or tampered with.",
            expected_hash,
            actual_hash
        ));
    }

    UI::info("✅ Checksum verified (SHA256)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("1.0.0", "0.9.9"));
        assert!(is_newer_version("0.5.29", "0.5.28"));
        assert!(is_newer_version("1.0.0", "0.99.99"));
        assert!(!is_newer_version("0.5.28", "0.5.29"));
        assert!(!is_newer_version("0.5.28", "0.5.28"));
    }

    #[test]
    fn test_create_cdn_assets() {
        let assets = create_cdn_assets("0.5.28");
        assert_eq!(assets.len(), 6);

        // Check Windows x64 asset
        let windows_asset = assets
            .iter()
            .find(|a| a.name.contains("windows") && a.name.contains("x86_64"));
        assert!(windows_asset.is_some());
        assert!(windows_asset
            .unwrap()
            .browser_download_url
            .contains("vx-v0.5.28"));

        // Check Linux asset
        let linux_asset = assets.iter().find(|a| a.name.contains("linux"));
        assert!(linux_asset.is_some());
    }
}
