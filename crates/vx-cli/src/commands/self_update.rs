//! Self-update command implementation
//! Allows vx to update itself to the latest version

use crate::ui::UI;
use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use tracing::{info_span, Instrument};

const GITHUB_OWNER: &str = "loonghao";
const GITHUB_REPO: &str = "vx";

/// Handle self-update command
pub async fn handle(check_only: bool, version: Option<&str>) -> Result<()> {
    let span = info_span!("Self-update", check_only = check_only);
    async {
        if check_only {
            check_for_updates().await
        } else {
            perform_self_update(version).await
        }
    }
    .instrument(span)
    .await
}

/// Check for available updates
async fn check_for_updates() -> Result<()> {
    UI::info("Checking for vx updates...");

    let current_version = get_current_version()?;
    let latest_version = get_latest_version().await?;

    if current_version == latest_version {
        UI::success(&format!("vx {} is up to date", current_version));
    } else {
        UI::info(&format!(
            "Update available: {} → {}",
            current_version, latest_version
        ));
        UI::hint("Run 'vx self-update' to update");
    }

    Ok(())
}

/// Perform self-update
async fn perform_self_update(target_version: Option<&str>) -> Result<()> {
    let current_version = get_current_version()?;

    let target_version = if let Some(v) = target_version {
        v.to_string()
    } else {
        UI::info("Fetching latest version...");
        get_latest_version().await?
    };

    if current_version == target_version {
        UI::success(&format!("vx {} is already up to date", current_version));
        return Ok(());
    }

    UI::info(&format!(
        "Updating vx: {} → {}",
        current_version, target_version
    ));

    // Get current executable path
    let current_exe = env::current_exe().context("Failed to get current executable path")?;

    // Download and replace the executable
    download_and_replace(&current_exe, &target_version).await?;

    UI::success(&format!("Successfully updated vx to {}", target_version));
    UI::hint("Restart your terminal to ensure the new version is loaded");

    Ok(())
}

/// Get current vx version
fn get_current_version() -> Result<String> {
    // Get version from Cargo.toml or environment
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

/// Get latest version from GitHub releases (with fallback strategies)
async fn get_latest_version() -> Result<String> {
    // Try API first (with better error handling)
    match get_latest_version_from_api().await {
        Ok(version) => return Ok(version),
        Err(e) => {
            UI::warn(&format!("GitHub API failed: {}", e));
            UI::info("Falling back to releases page parsing...");
        }
    }

    // Fallback to parsing releases page
    get_latest_version_from_page().await
}

/// Get latest version from GitHub API (preferred method)
async fn get_latest_version_from_api() -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases",
        GITHUB_OWNER, GITHUB_REPO
    );

    let mut request = client
        .get(&url)
        .header("User-Agent", format!("vx/{}", get_current_version()?));

    // Add GitHub token if available in environment
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .await
        .context("Failed to fetch releases from API")?;

    if !response.status().is_success() {
        let status = response.status();
        if status == 403 {
            anyhow::bail!("GitHub API rate limit exceeded");
        }
        anyhow::bail!("GitHub API request failed: {}", status);
    }

    let releases: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse GitHub API response")?;

    // Find the latest main vx release (tag starts with 'v' and doesn't contain '-')
    if let Some(releases_array) = releases.as_array() {
        for release in releases_array {
            if let Some(tag_name) = release["tag_name"].as_str() {
                if tag_name.starts_with('v') && !tag_name.contains('-') {
                    return Ok(tag_name.trim_start_matches('v').to_string());
                }
            }
        }
    }

    anyhow::bail!("No main vx release found in API response")
}

/// Get latest version from GitHub releases page (fallback method)
async fn get_latest_version_from_page() -> Result<String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://github.com/{}/{}/releases",
        GITHUB_OWNER, GITHUB_REPO
    );

    let response = client
        .get(&url)
        .header("User-Agent", format!("vx/{}", get_current_version()?))
        .send()
        .await
        .context("Failed to fetch releases page")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to fetch releases page: {}", response.status());
    }

    let html = response
        .text()
        .await
        .context("Failed to read page content")?;

    // Parse HTML to find the first main vx release tag
    // Look for patterns like: href="/loonghao/vx/releases/tag/v0.2.6"
    use regex::Regex;
    let re = Regex::new(r#"href="[^"]*?/releases/tag/v([0-9]+\.[0-9]+\.[0-9]+)""#)
        .context("Failed to create regex")?;

    for captures in re.captures_iter(&html) {
        if let Some(version) = captures.get(1) {
            return Ok(version.as_str().to_string());
        }
    }

    anyhow::bail!("Could not find main vx version in releases page")
}

/// Download and replace the current executable
async fn download_and_replace(current_exe: &PathBuf, version: &str) -> Result<()> {
    use std::fs;
    use std::io::Write;

    // Determine platform-specific binary name
    let platform = get_platform_string();
    let archive_name = format!("vx-{}.zip", platform);
    let download_url = format!(
        "https://github.com/{}/{}/releases/download/v{}/{}",
        GITHUB_OWNER, GITHUB_REPO, version, archive_name
    );

    UI::info(&format!("Downloading vx {} for {}...", version, platform));

    // Create temporary directory
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;

    // Download the archive
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .send()
        .await
        .context("Failed to download update")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed: {}", response.status());
    }

    let archive_path = temp_dir.path().join(&archive_name);
    let mut file = fs::File::create(&archive_path).context("Failed to create temporary file")?;

    let content = response
        .bytes()
        .await
        .context("Failed to read download content")?;

    file.write_all(&content)
        .context("Failed to write download content")?;

    // Extract the archive
    UI::info("Extracting update...");
    extract_archive(&archive_path, temp_dir.path())?;

    // Find the new executable
    let new_exe_name = if cfg!(windows) { "vx.exe" } else { "vx" };
    let new_exe_path = find_executable_in_dir(temp_dir.path(), new_exe_name)?;

    // Replace the current executable
    UI::info("Installing update...");
    replace_executable(current_exe, &new_exe_path)?;

    Ok(())
}

/// Get platform string for download URL
fn get_platform_string() -> String {
    let os = if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "Darwin"
    } else {
        "Linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64" // fallback
    };

    let variant = if cfg!(target_os = "windows") {
        "msvc"
    } else {
        "gnu"
    };

    format!("{}-{}-{}", os, variant, arch)
}

/// Extract archive to directory
fn extract_archive(archive_path: &PathBuf, extract_dir: &std::path::Path) -> Result<()> {
    let file = std::fs::File::open(archive_path).context("Failed to open archive")?;

    let mut archive = zip::ZipArchive::new(file).context("Failed to read ZIP archive")?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .context("Failed to read archive entry")?;

        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).context("Failed to create directory")?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).context("Failed to create parent directory")?;
                }
            }
            let mut outfile =
                std::fs::File::create(&outpath).context("Failed to create output file")?;
            std::io::copy(&mut file, &mut outfile).context("Failed to extract file")?;
        }
    }

    Ok(())
}

/// Find executable in directory
fn find_executable_in_dir(dir: &std::path::Path, exe_name: &str) -> Result<PathBuf> {
    use std::fs;

    for entry in fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.is_file() && path.file_name().unwrap_or_default() == exe_name {
            return Ok(path);
        }

        if path.is_dir() {
            if let Ok(found) = find_executable_in_dir(&path, exe_name) {
                return Ok(found);
            }
        }
    }

    anyhow::bail!("Executable {} not found in archive", exe_name);
}

/// Replace the current executable with the new one
fn replace_executable(current_exe: &PathBuf, new_exe: &PathBuf) -> Result<()> {
    use std::fs;

    // On Windows, we might need to rename the current exe first
    #[cfg(windows)]
    {
        let backup_path = current_exe.with_extension("exe.old");
        if backup_path.exists() {
            fs::remove_file(&backup_path).context("Failed to remove old backup")?;
        }
        fs::rename(current_exe, &backup_path).context("Failed to backup current executable")?;

        fs::copy(new_exe, current_exe).context("Failed to install new executable")?;

        // Try to remove backup, but don't fail if we can't
        let _ = fs::remove_file(&backup_path);
    }

    #[cfg(not(windows))]
    {
        fs::copy(new_exe, current_exe).context("Failed to install new executable")?;

        // Make executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(current_exe, perms).context("Failed to set executable permissions")?;
    }

    Ok(())
}
