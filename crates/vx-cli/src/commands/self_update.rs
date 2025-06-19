//! Self-update command implementation

use crate::ui::UI;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

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

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub async fn handle(
    token: Option<&str>,
    prerelease: bool,
    force: bool,
    check_only: bool,
) -> Result<()> {
    UI::info("🔍 Checking for vx updates...");

    let current_version = env!("CARGO_PKG_VERSION");
    UI::detail(&format!("Current version: {}", current_version));

    // Create HTTP client with optional authentication
    let client = create_authenticated_client(token)?;

    // Get latest release information with smart channel selection
    let release = get_latest_release(&client, prerelease, token.is_some()).await?;

    let latest_version = release.tag_name.trim_start_matches('v');
    UI::detail(&format!("Latest version: {}", latest_version));

    // Check if update is needed
    if !force && current_version == latest_version {
        UI::success("✅ vx is already up to date!");
        return Ok(());
    }

    if current_version != latest_version {
        UI::info(&format!(
            "📦 New version available: {} -> {}",
            current_version, latest_version
        ));

        if !release.body.is_empty() {
            UI::info("📝 Release notes:");
            println!("{}", release.body);
        }
    }

    if check_only {
        if current_version != latest_version {
            UI::info("💡 Run 'vx self-update' to update to the latest version");
        }
        return Ok(());
    }

    // Find appropriate asset for current platform
    let asset = find_platform_asset(&release.assets)?;
    UI::info(&format!(
        "📥 Downloading {} ({} bytes)...",
        asset.name, asset.size
    ));

    // Download and install update
    download_and_install(&client, asset, force).await?;

    UI::success(&format!(
        "🎉 Successfully updated vx to version {}!",
        latest_version
    ));
    UI::hint("Restart your terminal or run 'vx --version' to verify the update");

    Ok(())
}

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

async fn get_latest_release(
    client: &reqwest::Client,
    prerelease: bool,
    has_token: bool,
) -> Result<GitHubRelease> {
    // If no token is provided, prefer CDN to avoid rate limits
    if !has_token {
        UI::info("🌐 No GitHub token provided, using CDN for version check...");

        // Try jsDelivr API first when no token
        match try_jsdelivr_api(client, prerelease).await {
            Ok(release) => {
                UI::info("✅ Got version info from jsDelivr CDN");
                return Ok(release);
            }
            Err(e) => {
                UI::warn(&format!("⚠️ CDN fallback failed: {}", e));
                UI::info("🔄 Falling back to GitHub API...");
            }
        }
    }

    // Try GitHub API (either as primary with token, or as fallback without token)
    match try_github_api(client, prerelease).await {
        Ok(release) => Ok(release),
        Err(e) => {
            // Check if it's a rate limit error
            if e.to_string().contains("rate limit") {
                if has_token {
                    // If we have a token but still hit rate limit, something's wrong
                    return Err(anyhow!(
                        "GitHub API rate limit exceeded even with authentication. \
                        Check your token permissions or try again later."
                    ));
                } else {
                    // If no token and we already tried CDN, we're out of options
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
                    return Ok(release);
                }
            }

            // Return the original error if all else fails
            Err(e)
        }
    }
}

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

fn find_platform_asset(assets: &[GitHubAsset]) -> Result<&GitHubAsset> {
    let target_os = env::consts::OS;
    let target_arch = env::consts::ARCH;

    // Define platform-specific patterns
    let patterns = match (target_os, target_arch) {
        ("windows", "x86_64") => vec!["windows", "win64", "x86_64-pc-windows"],
        ("windows", "x86") => vec!["windows", "win32", "i686-pc-windows"],
        ("macos", "x86_64") => vec!["macos", "darwin", "x86_64-apple-darwin"],
        ("macos", "aarch64") => vec!["macos", "darwin", "aarch64-apple-darwin"],
        ("linux", "x86_64") => vec!["linux", "x86_64-unknown-linux"],
        ("linux", "aarch64") => vec!["linux", "aarch64-unknown-linux"],
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
        if patterns.iter().any(|pattern| name_lower.contains(pattern)) {
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

async fn download_and_install(
    client: &reqwest::Client,
    asset: &GitHubAsset,
    force: bool,
) -> Result<()> {
    // Get current executable path
    let current_exe = env::current_exe()?;
    let backup_path = current_exe.with_extension("bak");

    // Try downloading with multi-channel fallback
    let content = download_with_fallback(client, asset).await?;

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

    // Backup current executable
    if current_exe.exists() && !force {
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        fs::rename(&current_exe, &backup_path)?;
        UI::detail(&format!(
            "📦 Backed up current version to {}",
            backup_path.display()
        ));
    }

    // Replace current executable
    fs::rename(&temp_path, &current_exe)?;

    UI::detail(&format!(
        "✅ Installed new version to {}",
        current_exe.display()
    ));

    Ok(())
}

fn extract_from_zip(content: &[u8], output_path: &PathBuf) -> Result<()> {
    use std::io::Cursor;
    use zip::ZipArchive;

    let cursor = Cursor::new(content);
    let mut archive = ZipArchive::new(cursor)?;

    // Find the vx executable in the archive
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

    // Extract version information from jsDelivr response
    let versions = json["versions"]
        .as_array()
        .ok_or_else(|| anyhow!("No versions found in jsDelivr response"))?;

    let latest_version = if prerelease {
        // For prerelease, get the first version (latest)
        versions.first()
    } else {
        // For stable, find the first non-prerelease version
        versions.iter().find(|v| {
            if let Some(version_str) = v.as_str() {
                !version_str.contains("-") // Simple check for prerelease
            } else {
                false
            }
        })
    }
    .and_then(|v| v.as_str())
    .ok_or_else(|| anyhow!("No suitable version found"))?;

    // Create CDN-based assets for the version
    let assets = create_cdn_assets(latest_version);

    // Create a minimal GitHubRelease structure from jsDelivr data
    Ok(GitHubRelease {
        tag_name: latest_version.to_string(),
        name: format!("Release {}", latest_version),
        body: "Release information retrieved from CDN".to_string(),
        prerelease: latest_version.contains("-"),
        assets,
    })
}

fn create_cdn_assets(version: &str) -> Vec<GitHubAsset> {
    let base_url = format!("https://cdn.jsdelivr.net/gh/loonghao/vx@v{}", version);

    // Define platform-specific asset names based on our release naming convention
    let asset_configs = vec![
        ("vx-Windows-msvc-x86_64.zip", "windows", "x86_64"),
        ("vx-Windows-msvc-arm64.zip", "windows", "aarch64"),
        ("vx-Linux-musl-x86_64.tar.gz", "linux", "x86_64"),
        ("vx-Linux-musl-arm64.tar.gz", "linux", "aarch64"),
        ("vx-macOS-x86_64.tar.gz", "macos", "x86_64"),
        ("vx-macOS-arm64.tar.gz", "macos", "aarch64"),
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

async fn download_with_fallback(client: &reqwest::Client, asset: &GitHubAsset) -> Result<Vec<u8>> {
    // Extract version from the original URL for CDN fallback
    let version = extract_version_from_url(&asset.browser_download_url);

    // Define download channels in order of preference
    // If original URL is from CDN (jsDelivr), it means we got version info from CDN
    // so we should prefer CDN for downloads too
    let channels = if asset.browser_download_url.contains("jsdelivr.net") {
        // CDN-first strategy (when version came from CDN)
        vec![
            ("jsDelivr CDN", asset.browser_download_url.clone()),
            (
                "Fastly CDN",
                format!(
                    "https://fastly.jsdelivr.net/gh/loonghao/vx@v{}/{}",
                    version, asset.name
                ),
            ),
            (
                "GitHub Releases",
                format!(
                    "https://github.com/loonghao/vx/releases/download/v{}/{}",
                    version, asset.name
                ),
            ),
        ]
    } else {
        // GitHub-first strategy (when version came from GitHub API)
        vec![
            ("GitHub Releases", asset.browser_download_url.clone()),
            (
                "jsDelivr CDN",
                format!(
                    "https://cdn.jsdelivr.net/gh/loonghao/vx@v{}/{}",
                    version, asset.name
                ),
            ),
            (
                "Fastly CDN",
                format!(
                    "https://fastly.jsdelivr.net/gh/loonghao/vx@v{}/{}",
                    version, asset.name
                ),
            ),
        ]
    };

    for (channel_name, url) in channels {
        UI::detail(&format!("🔄 Trying {}: {}", channel_name, url));

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.bytes().await {
                        Ok(content) => {
                            if content.len() > 1024 {
                                // Basic size validation
                                UI::info(&format!(
                                    "✅ Downloaded from {} ({} bytes)",
                                    channel_name,
                                    content.len()
                                ));
                                return Ok(content.to_vec());
                            } else {
                                UI::warn(&format!(
                                    "⚠️ Downloaded file too small from {}, trying next channel...",
                                    channel_name
                                ));
                            }
                        }
                        Err(e) => {
                            UI::warn(&format!(
                                "⚠️ Failed to read content from {}: {}",
                                channel_name, e
                            ));
                        }
                    }
                } else {
                    UI::warn(&format!(
                        "⚠️ HTTP {} from {}, trying next channel...",
                        response.status(),
                        channel_name
                    ));
                }
            }
            Err(e) => {
                UI::warn(&format!("⚠️ Failed to connect to {}: {}", channel_name, e));
            }
        }
    }

    Err(anyhow!("Failed to download from all channels"))
}

fn extract_version_from_url(url: &str) -> String {
    // Extract version from GitHub release URL or CDN URL
    // Look for patterns like "/v1.2.3/" or "@v1.2.3"
    for part in url.split('/') {
        if part.starts_with('v') && part.len() > 1 {
            let version_part = &part[1..]; // Remove 'v' prefix
            if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                return version_part.to_string();
            }
        }
        if part.starts_with("@v") && part.len() > 2 {
            let version_part = &part[2..]; // Remove '@v' prefix
            if version_part.chars().next().unwrap_or('a').is_ascii_digit() {
                return version_part.to_string();
            }
        }
    }

    // Fallback to current version if extraction fails
    env!("CARGO_PKG_VERSION").to_string()
}
