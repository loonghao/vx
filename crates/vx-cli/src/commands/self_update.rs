//! Self-update command implementation
//!
//! This module provides self-update functionality for vx with the following features:
//! - **axoupdater fast path** (when `self-update` feature is enabled): Uses cargo-dist
//!   install receipts for zero-config updates — handles tag format, asset naming, and
//!   binary replacement automatically.
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
    UI::section("Checking for updates");

    let current_version = env!("CARGO_PKG_VERSION");
    UI::detail(&format!("Current version: {}", current_version));

    // Try axoupdater fast path first (cargo-dist receipt-based)
    // This handles tag format, asset naming, and binary replacement automatically.
    // Only used when: no specific version requested, no prerelease flag, and receipt exists.
    #[cfg(feature = "self-update")]
    if target_version.is_none() && !prerelease {
        match try_axoupdater(token, force, check_only).await {
            Ok(Some(updated)) => {
                // axoupdater handled the update successfully
                if updated {
                    UI::success("Successfully updated vx!");
                    UI::hint("Restart your terminal or run 'vx --version' to verify the update");
                }
                // If not updated (already up to date or check_only), axoupdater already printed info
                return Ok(());
            }
            Ok(None) => {
                // No receipt found — fall through to legacy path
                UI::detail("No cargo-dist install receipt found, using legacy path");
            }
            Err(e) => {
                // axoupdater failed — fall through to legacy path
                UI::warn(&format!(
                    "axoupdater failed: {}. Falling back...",
                    e
                ));
            }
        }
    }

    // Legacy update path (multi-channel CDN fallback)
    legacy_update(token, prerelease, force, check_only, target_version).await
}

/// Try the axoupdater fast path for self-update.
///
/// Returns:
/// - `Ok(Some(true))` — update was installed
/// - `Ok(Some(false))` — already up to date (or check_only)
/// - `Ok(None)` — no install receipt found (should fall through to legacy)
/// - `Err(e)` — axoupdater encountered an error (should fall through to legacy)
#[cfg(feature = "self-update")]
async fn try_axoupdater(
    token: Option<&str>,
    force: bool,
    check_only: bool,
) -> Result<Option<bool>> {
    use axoupdater::AxoUpdater;

    let mut updater = AxoUpdater::new_for("vx");
    updater.disable_installer_output();

    // Set GitHub token if provided (helps with rate limits)
    if let Some(token) = token {
        updater.set_github_token(token);
    } else if let Ok(token) = env::var("VX_GITHUB_TOKEN") {
        updater.set_github_token(&token);
    } else if let Ok(token) = env::var("GITHUB_TOKEN") {
        updater.set_github_token(&token);
    }

    // Try to load install receipt
    if updater.load_receipt().is_err() {
        // No receipt — this is expected for users who installed via
        // old install scripts or manual download. Fall through to legacy.
        return Ok(None);
    }

    // Override receipt version with actual compiled version (defensive, like uv does)
    let current_ver = env!("CARGO_PKG_VERSION")
        .parse()
        .map_err(|e| anyhow!("Failed to parse current version: {}", e))?;
    updater.set_current_version(current_ver)?;

    // Check if update is needed
    if check_only {
        let update_needed = updater.is_update_needed().await?;
        if update_needed {
            UI::info("A new version of vx is available!");
            UI::hint("Run 'vx self-update' to update to the latest version");
        } else {
            UI::success("vx is already up to date!");
        }
        return Ok(Some(!update_needed));
    }

    if !force {
        let update_needed = updater.is_update_needed().await?;
        if !update_needed {
            UI::success("vx is already up to date!");
            return Ok(Some(false));
        }
    }

    // Perform the update
    UI::info("Updating vx via axoupdater...");
    let result = updater.run().await?;

    // run() returns Option<UpdateResult> — Some means update was performed
    Ok(Some(result.is_some()))
}

/// Legacy update path with multi-channel CDN fallback.
/// Used when axoupdater receipt is not available (old installations, manual installs).
async fn legacy_update(
    token: Option<&str>,
    prerelease: bool,
    force: bool,
    check_only: bool,
    target_version: Option<&str>,
) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");

    // Create HTTP client with optional authentication
    let client = create_authenticated_client(token)?;

    // Get release information based on whether a specific version is requested
    let (release, version_source) = if let Some(version) = target_version {
        UI::info(&format!("Looking for version {}...", version));
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
        UI::success("vx is already up to date!");
        return Ok(());
    }

    if current_version != latest_version {
        let direction = if is_newer_version(latest_version, current_version) {
            "Upgrade"
        } else {
            "Downgrade"
        };
        UI::info(&format!(
            "{} available: {} -> {}",
            direction, current_version, latest_version
        ));

        if !release.body.is_empty() && !release.body.contains("retrieved from CDN") {
            println!();
            UI::detail("Release notes:");
            println!("{}", release.body);
        }
    }

    if check_only {
        if current_version != latest_version {
            UI::hint("Run 'vx self-update' to update to the latest version");
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
    UI::step(&format!("Downloading {}...", asset.name));

    // Download and install update
    download_and_install(&client, asset, force, version_source, latest_version).await?;

    println!();
    UI::success(&format!(
        "Successfully updated vx to version {}!",
        latest_version
    ));
    UI::hint("Restart your terminal or run 'vx --version' to verify the update");

    Ok(())
}

/// Check if version_a is newer than version_b using semver comparison
/// Supports formats: "0.6.27", "v0.6.27", "0.6.27-beta.1"
///
/// This is a thin wrapper around vx_core::version_utils::is_newer_version
/// to ensure consistent version comparison across the codebase.
fn is_newer_version(version_a: &str, version_b: &str) -> bool {
    vx_core::version_utils::is_newer_version(version_a, version_b)
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
        UI::detail("Using authenticated GitHub API requests");
    } else {
        UI::detail("No GitHub token provided, preferring CDN downloads");
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
                UI::detail("Found version via GitHub API");
                return Ok((release, VersionSource::GitHub));
            }
            Err(e) => {
                UI::warn(&format!("GitHub API failed: {}", e));
                UI::detail("Trying CDN fallback...");
            }
        }
    }

    // Try CDN (create synthetic release)
    match try_jsdelivr_api_specific(client, version).await {
        Ok(release) => {
            UI::detail("Found version via jsDelivr CDN");
            Ok((release, VersionSource::Cdn))
        }
        Err(e) => {
            // If CDN fails and we haven't tried GitHub, try it now
            if !has_token {
                UI::warn(&format!("CDN lookup failed: {}", e));
                UI::detail("Falling back to GitHub API...");

                match try_github_api_specific(client, version).await {
                    Ok(release) => {
                        UI::detail("Found version via GitHub API");
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

    // Check if the requested version exists using normalized comparison
    let version_exists = versions.iter().any(|v| {
        if let Some(v_str) = v.as_str() {
            vx_core::version_utils::normalize_version(v_str) == version
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
        tag_name: get_tag_for_version(version),
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
        UI::detail("Using CDN for version check...");

        // Try jsDelivr API first when no token
        match try_jsdelivr_api(client, prerelease).await {
            Ok(release) => {
                UI::detail("Got version info from jsDelivr CDN");
                return Ok((release, VersionSource::Cdn));
            }
            Err(e) => {
                UI::warn(&format!("CDN version check failed: {}", e));
                UI::detail("Falling back to GitHub API...");
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
                UI::warn(&format!("GitHub API failed: {}", e));
                UI::detail("Trying CDN fallback...");

                if let Ok(release) = try_jsdelivr_api(client, prerelease).await {
                    UI::detail("Got version info from jsDelivr CDN");
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

    // Try downloading with multi-channel fallback
    let content = download_with_fallback(client, asset, version_source, version).await?;

    // Try to verify checksum if available
    if let Err(e) = verify_checksum(client, asset, &content, version_source, version).await {
        UI::debug(&format!("Checksum verification skipped: {}", e));
    }

    // Use system temp directory for the new binary to avoid permission issues
    // This is more reliable than placing temp files next to the executable
    let temp_dir = env::temp_dir();
    let temp_path = temp_dir.join(format!("vx-update-{}.tmp", std::process::id()));

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

    // Create backup in temp directory (more reliable than next to exe)
    let backup_path = if !force && current_exe.exists() {
        let backup = temp_dir.join(format!("vx-backup-{}.bak", std::process::id()));
        fs::copy(&current_exe, &backup)?;
        UI::detail(&format!(
            "Backed up current version to {}",
            backup.display()
        ));
        Some(backup)
    } else {
        None
    };

    // Use self_replace for safe binary replacement (handles Windows exe locking)
    // On Windows, self_replace works by:
    // 1. Renaming the running exe to a temp name (Windows allows this)
    // 2. Copying the new exe to the original location
    // 3. The old exe is deleted on next reboot or when no longer in use
    match self_replace::self_replace(&temp_path) {
        Ok(()) => {
            // Clean up temp file
            let _ = fs::remove_file(&temp_path);
            // Clean up backup if update succeeded
            if let Some(ref backup) = backup_path {
                let _ = fs::remove_file(backup);
            }
            UI::detail(&format!(
                "Installed to {}",
                current_exe.display()
            ));
        }
        Err(e) => {
            // On Windows, try alternative replacement methods
            #[cfg(target_os = "windows")]
            {
                let error_str = e.to_string();
                if error_str.contains("os error 5") || error_str.contains("Access is denied") {
                    UI::warn("Standard replacement failed, trying alternative method...");

                    // Try alternative: rename current exe and copy new one
                    match try_windows_alternative_replace(&current_exe, &temp_path) {
                        Ok(()) => {
                            let _ = fs::remove_file(&temp_path);
                            if let Some(ref backup) = backup_path {
                                let _ = fs::remove_file(backup);
                            }
                            UI::detail(&format!(
                                "Installed to {}",
                                current_exe.display()
                            ));
                            return Ok(());
                        }
                        Err(alt_err) => {
                            UI::warn(&format!("Alternative method failed: {}", alt_err));
                        }
                    }

                    // All methods failed, provide detailed guidance
                    UI::error("Could not replace vx executable");
                    println!();
                    UI::detail("This usually happens when:");
                    UI::detail("  1. Antivirus software is blocking the operation");
                    UI::detail("  2. Another terminal/process is using vx");
                    UI::detail("  3. File system permissions issue");
                    println!();
                    UI::detail("Solutions:");
                    UI::detail("  - Temporarily disable antivirus and try again");
                    UI::detail("  - Close ALL terminals and run update in a fresh terminal");
                    UI::detail("  - Manual update:");
                    UI::detail(&format!(
                        "    1. Download: https://github.com/loonghao/vx/releases/download/{}/{}",
                        get_tag_for_version(version),
                        asset.name
                    ));
                    UI::detail(&format!(
                        "    2. Extract and replace: {}",
                        current_exe.display()
                    ));

                    // Save the new binary for manual installation
                    let manual_path = temp_dir.join(format!("vx-{}-new.exe", version));
                    if fs::copy(&temp_path, &manual_path).is_ok() {
                        println!();
                        UI::detail(&format!(
                            "  New version saved at: {}",
                            manual_path.display()
                        ));
                        UI::detail(
                            "  You can manually copy this file to replace the current vx.exe",
                        );
                    }

                    let _ = fs::remove_file(&temp_path);
                    return Err(anyhow!(
                        "Failed to replace binary. Please try manual update or see suggestions above."
                    ));
                }
            }

            // Clean up temp file
            let _ = fs::remove_file(&temp_path);

            // Generic error handling for other platforms or errors
            if let Some(ref backup) = backup_path {
                UI::warn("Update failed. Backup is available for manual recovery.");
                UI::detail(&format!("Backup location: {}", backup.display()));
            }
            return Err(anyhow!("Failed to replace binary: {}", e));
        }
    }

    Ok(())
}

/// Alternative replacement method for Windows when self_replace fails
///
/// This tries a different approach:
/// 1. Rename the current exe to a random temp name (Windows allows renaming running exes)
/// 2. Copy the new exe to the original location
/// 3. The old renamed exe will be cleaned up on next run or reboot
#[cfg(target_os = "windows")]
fn try_windows_alternative_replace(current_exe: &PathBuf, new_exe: &PathBuf) -> Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Generate a unique suffix for the old exe
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let old_exe_renamed = current_exe.with_extension(format!("old.{}.exe", timestamp));

    // Step 1: Rename current exe (Windows allows this even while running)
    fs::rename(current_exe, &old_exe_renamed).context("Failed to rename current executable")?;

    // Step 2: Copy new exe to original location
    match fs::copy(new_exe, current_exe) {
        Ok(_) => {
            UI::detail(&format!(
                "Replaced executable (old version at {})",
                old_exe_renamed.display()
            ));

            // Try to schedule old exe for deletion (best effort)
            // The old exe will be deleted when no longer in use
            let _ = schedule_file_deletion(&old_exe_renamed);

            Ok(())
        }
        Err(e) => {
            // Rollback: try to rename back
            let _ = fs::rename(&old_exe_renamed, current_exe);
            Err(anyhow!("Failed to copy new executable: {}", e))
        }
    }
}

/// Schedule a file for deletion (Windows)
/// Uses MoveFileEx with MOVEFILE_DELAY_UNTIL_REBOOT flag
#[cfg(target_os = "windows")]
fn schedule_file_deletion(path: &PathBuf) -> Result<()> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Convert path to wide string
    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // MOVEFILE_DELAY_UNTIL_REBOOT = 0x4
    const MOVEFILE_DELAY_UNTIL_REBOOT: u32 = 0x4;

    // Call MoveFileExW with NULL destination to delete on reboot
    let result = unsafe {
        windows_sys::Win32::Storage::FileSystem::MoveFileExW(
            wide_path.as_ptr(),
            std::ptr::null(),
            MOVEFILE_DELAY_UNTIL_REBOOT,
        )
    };

    if result != 0 {
        UI::detail("Old version scheduled for deletion on next reboot");
        Ok(())
    } else {
        // Not critical if this fails
        Ok(())
    }
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

    // Find the latest version based on prerelease flag using vx_core utilities
    let version_strings: Vec<&str> = versions.iter().filter_map(|v| v.as_str()).collect();

    let latest_version = vx_core::version_utils::find_latest_version(&version_strings, !prerelease)
        .ok_or_else(|| anyhow!("No suitable version found"))?;

    let version_number = vx_core::version_utils::normalize_version(latest_version);

    let assets = create_cdn_assets(version_number);

    Ok(GitHubRelease {
        tag_name: latest_version.to_string(),
        name: format!("Release {}", version_number),
        body: "Release information retrieved from CDN".to_string(),
        prerelease: vx_core::version_utils::is_prerelease(latest_version),
        assets,
    })
}

/// Determine artifact naming format based on version
/// - v0.6.0 to v0.6.x: versioned format (vx-0.6.1-x86_64-pc-windows-msvc.zip) with tag vx-v{ver}
/// - v0.5.x and earlier: legacy/unversioned format (vx-x86_64-pc-windows-msvc.zip) with tag vx-v{ver}
/// - v0.7.0+ (cargo-dist): primarily unversioned format (vx-x86_64-pc-windows-msvc.zip) with tag v{ver}
///   but also supports versioned format (vx-0.7.0-x86_64-pc-windows-msvc.zip) as fallback
fn uses_versioned_artifact_naming(version: &str) -> bool {
    if let Some(parsed) = vx_core::version_utils::parse_version(version) {
        // Only v0.6.x uses versioned naming as PRIMARY format
        // v0.7.0+ (cargo-dist) uses unversioned as primary, versioned as fallback
        parsed.major == 0 && parsed.minor == 6
    } else {
        false
    }
}

/// Determine the git tag format for a given version
/// - v0.7.0+: uses v{version} tag format (cargo-dist)
/// - v0.6.x and earlier: uses vx-v{version} tag format (custom CI)
fn get_tag_for_version(version: &str) -> String {
    if uses_cargo_dist_tag_format(version) {
        format!("v{}", version)
    } else {
        format!("vx-v{}", version)
    }
}

/// Check if a version uses cargo-dist tag format (v{version} instead of vx-v{version})
fn uses_cargo_dist_tag_format(version: &str) -> bool {
    if let Some(parsed) = vx_core::version_utils::parse_version(version) {
        parsed.major > 0 || (parsed.major == 0 && parsed.minor >= 7)
    } else {
        false
    }
}

/// Get all possible tag formats for a version (for fallback)
fn get_tag_candidates(version: &str) -> Vec<String> {
    if uses_cargo_dist_tag_format(version) {
        // v0.7.0+: try v{ver} first, then vx-v{ver} as fallback
        vec![format!("v{}", version), format!("vx-v{}", version)]
    } else {
        // v0.6.x and earlier: try vx-v{ver} first, then v{ver} as fallback
        vec![format!("vx-v{}", version), format!("v{}", version)]
    }
}

/// Create CDN-based assets for a given version
/// Supports both legacy naming (vx-arch-platform.ext) and versioned naming (vx-version-arch-platform.ext)
fn create_cdn_assets(version: &str) -> Vec<GitHubAsset> {
    let tag = get_tag_for_version(version);
    let base_url = format!("https://cdn.jsdelivr.net/gh/loonghao/vx@{}", tag);
    let use_versioned = uses_versioned_artifact_naming(version);

    // Platform configurations: (base_name, extension, os, arch)
    let platform_configs = vec![
        ("x86_64-pc-windows-msvc", "zip", "windows", "x86_64"),
        ("aarch64-pc-windows-msvc", "zip", "windows", "aarch64"),
        ("x86_64-unknown-linux-gnu", "tar.gz", "linux", "x86_64"),
        ("aarch64-unknown-linux-gnu", "tar.gz", "linux", "aarch64"),
        ("x86_64-apple-darwin", "tar.gz", "macos", "x86_64"),
        ("aarch64-apple-darwin", "tar.gz", "macos", "aarch64"),
    ];

    platform_configs
        .into_iter()
        .map(|(platform, ext, _os, _arch)| {
            let name = if use_versioned {
                // New versioned format: vx-0.6.1-x86_64-pc-windows-msvc.zip
                format!("vx-{}-{}.{}", version, platform, ext)
            } else {
                // Legacy format: vx-x86_64-pc-windows-msvc.zip
                format!("vx-{}.{}", platform, ext)
            };
            GitHubAsset {
                name: name.clone(),
                browser_download_url: format!("{}/{}", base_url, name),
                size: 0, // Size unknown from CDN
            }
        })
        .collect()
}

/// Generate alternative asset names for fallback
/// This handles the case where we might need to try both versioned and legacy naming.
/// For v0.7.0+ (cargo-dist), the primary is unversioned but we also try versioned format
/// since some releases may use `vx-{version}-{platform}.{ext}` naming.
fn get_alternative_asset_names(asset_name: &str, version: &str) -> Vec<String> {
    let mut names = vec![asset_name.to_string()];

    // Check if current name is versioned format (vx-{version}-...)
    let versioned_prefix = format!("vx-{}-", version);
    if asset_name.starts_with(&versioned_prefix) {
        // Current is versioned, add unversioned format as alternative
        let legacy_name = asset_name.replacen(&format!("{}-", version), "", 1);
        if !names.contains(&legacy_name) {
            names.push(legacy_name);
        }
    } else if asset_name.starts_with("vx-") {
        // Current is unversioned, add versioned format as alternative
        let versioned_name = asset_name.replacen("vx-", &versioned_prefix, 1);
        if !names.contains(&versioned_name) {
            names.push(versioned_name);
        }
    }

    names
}

/// Download with multi-channel fallback and progress bar
async fn download_with_fallback(
    client: &reqwest::Client,
    asset: &GitHubAsset,
    version_source: VersionSource,
    version: &str,
) -> Result<Vec<u8>> {
    // Get both versioned and legacy asset names for fallback
    let asset_names = get_alternative_asset_names(&asset.name, version);
    // Get all possible tag formats for this version
    let tag_candidates = get_tag_candidates(version);

    // Build download channels with all possible asset names and tag formats
    let mut channels: Vec<(&str, String)> = Vec::new();

    for asset_name in &asset_names {
        for tag in &tag_candidates {
            if version_source == VersionSource::Cdn {
                // CDN-first strategy
                if channels.is_empty() {
                    channels.push(("jsDelivr CDN", asset.browser_download_url.clone()));
                } else {
                    channels.push((
                        "jsDelivr CDN (alt)",
                        format!(
                            "https://cdn.jsdelivr.net/gh/loonghao/vx@{}/{}",
                            tag, asset_name
                        ),
                    ));
                }
                channels.push((
                    "Fastly CDN",
                    format!(
                        "https://fastly.jsdelivr.net/gh/loonghao/vx@{}/{}",
                        tag, asset_name
                    ),
                ));
                channels.push((
                    "GitHub Releases",
                    format!(
                        "https://github.com/loonghao/vx/releases/download/{}/{}",
                        tag, asset_name
                    ),
                ));
            } else {
                // GitHub-first strategy
                if channels.is_empty() {
                    channels.push(("GitHub Releases", asset.browser_download_url.clone()));
                } else {
                    channels.push((
                        "GitHub Releases (alt)",
                        format!(
                            "https://github.com/loonghao/vx/releases/download/{}/{}",
                            tag, asset_name
                        ),
                    ));
                }
                channels.push((
                    "jsDelivr CDN",
                    format!(
                        "https://cdn.jsdelivr.net/gh/loonghao/vx@{}/{}",
                        tag, asset_name
                    ),
                ));
                channels.push((
                    "Fastly CDN",
                    format!(
                        "https://fastly.jsdelivr.net/gh/loonghao/vx@{}/{}",
                        tag, asset_name
                    ),
                ));
            }
        }
    }

    // Deduplicate URLs while preserving order
    let mut seen_urls = std::collections::HashSet::new();
    let channels: Vec<_> = channels
        .into_iter()
        .filter(|(_, url)| seen_urls.insert(url.clone()))
        .collect();

    for (channel_name, url) in channels {
        UI::detail(&format!("Trying {}: {}", channel_name, url));

        match download_with_progress(client, &url, asset.size, channel_name).await {
            Ok(content) => {
                if content.len() > 1024 {
                    UI::detail(&format!(
                        "Downloaded from {} ({} bytes)",
                        channel_name,
                        content.len()
                    ));
                    return Ok(content);
                } else {
                    UI::warn(&format!(
                        "Downloaded file too small from {}, trying next...",
                        channel_name
                    ));
                }
            }
            Err(e) => {
                UI::debug(&format!("{} failed: {}", channel_name, e));
            }
        }
    }

    Err(anyhow!(
        "Failed to download from all channels.\n\
        \n\
        This can happen when upgrading across major release format changes \
        (e.g., v0.6.x → v0.7.x).\n\
        If you are stuck on an older version, please re-install using the install script:\n\
        \n\
        • Windows:  powershell -c \"irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex\"\n\
        • Linux/macOS: curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash"
    ))
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
    // Get both versioned and legacy asset names for checksum lookup
    let asset_names = get_alternative_asset_names(&asset.name, version);
    // Get all possible tag formats for this version
    let tag_candidates = get_tag_candidates(version);

    // Build checksum URLs for all possible asset names and tag formats
    let mut checksum_urls = Vec::new();
    for asset_name in &asset_names {
        let checksum_filename = format!("{}.sha256", asset_name);

        for tag in &tag_candidates {
            if version_source == VersionSource::Cdn {
                checksum_urls.push(format!(
                    "https://cdn.jsdelivr.net/gh/loonghao/vx@{}/{}",
                    tag, checksum_filename
                ));
                checksum_urls.push(format!(
                    "https://github.com/loonghao/vx/releases/download/{}/{}",
                    tag, checksum_filename
                ));
            } else {
                checksum_urls.push(format!(
                    "https://github.com/loonghao/vx/releases/download/{}/{}",
                    tag, checksum_filename
                ));
                checksum_urls.push(format!(
                    "https://cdn.jsdelivr.net/gh/loonghao/vx@{}/{}",
                    tag, checksum_filename
                ));
            }
        }
    }

    // Deduplicate URLs
    let mut seen = std::collections::HashSet::new();
    let checksum_urls: Vec<_> = checksum_urls
        .into_iter()
        .filter(|url| seen.insert(url.clone()))
        .collect();

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
        anyhow!("Checksum file not found. This may be expected for older releases.")
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

    UI::detail("Checksum verified (SHA256)");
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
    fn test_uses_versioned_artifact_naming() {
        // Only v0.6.x uses versioned naming
        assert!(uses_versioned_artifact_naming("0.6.0"));
        assert!(uses_versioned_artifact_naming("0.6.1"));
        assert!(uses_versioned_artifact_naming("0.6.31"));

        // v0.7.0+ (cargo-dist) uses unversioned naming
        assert!(!uses_versioned_artifact_naming("0.7.0"));
        assert!(!uses_versioned_artifact_naming("0.7.2"));
        assert!(!uses_versioned_artifact_naming("1.0.0"));
        assert!(!uses_versioned_artifact_naming("2.5.10"));

        // v0.5.x and earlier use legacy (unversioned) naming
        assert!(!uses_versioned_artifact_naming("0.5.29"));
        assert!(!uses_versioned_artifact_naming("0.5.0"));
        assert!(!uses_versioned_artifact_naming("0.4.0"));
        assert!(!uses_versioned_artifact_naming("0.1.0"));
    }

    #[test]
    fn test_uses_cargo_dist_tag_format() {
        // v0.7.0+ uses v{version} tag format
        assert!(uses_cargo_dist_tag_format("0.7.0"));
        assert!(uses_cargo_dist_tag_format("0.7.2"));
        assert!(uses_cargo_dist_tag_format("1.0.0"));

        // v0.6.x and earlier use vx-v{version} tag format
        assert!(!uses_cargo_dist_tag_format("0.6.31"));
        assert!(!uses_cargo_dist_tag_format("0.5.29"));
    }

    #[test]
    fn test_get_tag_for_version() {
        assert_eq!(get_tag_for_version("0.7.2"), "v0.7.2");
        assert_eq!(get_tag_for_version("1.0.0"), "v1.0.0");
        assert_eq!(get_tag_for_version("0.6.31"), "vx-v0.6.31");
        assert_eq!(get_tag_for_version("0.5.28"), "vx-v0.5.28");
    }

    #[test]
    fn test_create_cdn_assets_versioned() {
        // v0.6.x uses versioned naming with vx-v{ver} tag
        let assets = create_cdn_assets("0.6.1");
        assert_eq!(assets.len(), 6);

        // Check Windows x64 asset uses versioned naming
        let windows_asset = assets
            .iter()
            .find(|a| a.name.contains("windows") && a.name.contains("x86_64"));
        assert!(windows_asset.is_some());
        let windows_asset = windows_asset.unwrap();
        assert_eq!(windows_asset.name, "vx-0.6.1-x86_64-pc-windows-msvc.zip");
        assert!(windows_asset.browser_download_url.contains("vx-v0.6.1"));

        // Check Linux asset uses versioned naming
        let linux_asset = assets
            .iter()
            .find(|a| a.name.contains("linux") && a.name.contains("x86_64"));
        assert!(linux_asset.is_some());
        assert_eq!(
            linux_asset.unwrap().name,
            "vx-0.6.1-x86_64-unknown-linux-gnu.tar.gz"
        );

        // Check macOS asset uses versioned naming
        let macos_asset = assets
            .iter()
            .find(|a| a.name.contains("apple") && a.name.contains("aarch64"));
        assert!(macos_asset.is_some());
        assert_eq!(
            macos_asset.unwrap().name,
            "vx-0.6.1-aarch64-apple-darwin.tar.gz"
        );
    }

    #[test]
    fn test_create_cdn_assets_cargo_dist() {
        // v0.7.x (cargo-dist) uses unversioned naming with v{ver} tag
        let assets = create_cdn_assets("0.7.2");
        assert_eq!(assets.len(), 6);

        // Check Windows x64 asset uses unversioned naming
        let windows_asset = assets
            .iter()
            .find(|a| a.name.contains("windows") && a.name.contains("x86_64"));
        assert!(windows_asset.is_some());
        let windows_asset = windows_asset.unwrap();
        assert_eq!(windows_asset.name, "vx-x86_64-pc-windows-msvc.zip");
        // Tag should be v0.7.2, not vx-v0.7.2
        assert!(windows_asset.browser_download_url.contains("@v0.7.2"));
        assert!(!windows_asset.browser_download_url.contains("@vx-v0.7.2"));
    }

    #[test]
    fn test_create_cdn_assets_legacy() {
        let assets = create_cdn_assets("0.5.28");
        assert_eq!(assets.len(), 6);

        // Check Windows x64 asset uses legacy naming
        let windows_asset = assets
            .iter()
            .find(|a| a.name.contains("windows") && a.name.contains("x86_64"));
        assert!(windows_asset.is_some());
        let windows_asset = windows_asset.unwrap();
        assert_eq!(windows_asset.name, "vx-x86_64-pc-windows-msvc.zip");
        assert!(windows_asset.browser_download_url.contains("vx-v0.5.28"));

        // Check Linux asset uses legacy naming
        let linux_asset = assets
            .iter()
            .find(|a| a.name.contains("linux") && a.name.contains("x86_64"));
        assert!(linux_asset.is_some());
        assert_eq!(
            linux_asset.unwrap().name,
            "vx-x86_64-unknown-linux-gnu.tar.gz"
        );
    }

    #[test]
    fn test_get_alternative_asset_names_versioned() {
        // Versioned name should generate legacy alternative
        let names = get_alternative_asset_names("vx-0.6.1-x86_64-pc-windows-msvc.zip", "0.6.1");
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "vx-0.6.1-x86_64-pc-windows-msvc.zip");
        assert_eq!(names[1], "vx-x86_64-pc-windows-msvc.zip");
    }

    #[test]
    fn test_get_alternative_asset_names_legacy() {
        // Legacy name should generate versioned alternative
        let names = get_alternative_asset_names("vx-x86_64-pc-windows-msvc.zip", "0.5.29");
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "vx-x86_64-pc-windows-msvc.zip");
        assert_eq!(names[1], "vx-0.5.29-x86_64-pc-windows-msvc.zip");
    }

    #[test]
    fn test_get_alternative_asset_names_tar_gz() {
        // Test with tar.gz extension
        let names =
            get_alternative_asset_names("vx-0.6.1-x86_64-unknown-linux-gnu.tar.gz", "0.6.1");
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "vx-0.6.1-x86_64-unknown-linux-gnu.tar.gz");
        assert_eq!(names[1], "vx-x86_64-unknown-linux-gnu.tar.gz");
    }

    #[test]
    fn test_get_alternative_asset_names_cargo_dist() {
        // v0.7.x: primary is unversioned, alternative is versioned
        let names = get_alternative_asset_names("vx-x86_64-pc-windows-msvc.zip", "0.7.7");
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "vx-x86_64-pc-windows-msvc.zip");
        assert_eq!(names[1], "vx-0.7.7-x86_64-pc-windows-msvc.zip");
    }

    #[test]
    fn test_get_alternative_asset_names_cargo_dist_reverse() {
        // If someone has the versioned name first (e.g., old binary trying to update)
        let names = get_alternative_asset_names("vx-0.7.7-x86_64-pc-windows-msvc.zip", "0.7.7");
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "vx-0.7.7-x86_64-pc-windows-msvc.zip");
        assert_eq!(names[1], "vx-x86_64-pc-windows-msvc.zip");
    }
}
