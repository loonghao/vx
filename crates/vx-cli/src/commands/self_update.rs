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
    name: String,
    body: String,
    assets: Vec<GitHubAsset>,
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

    // Get latest release information
    let release = get_latest_release(&client, prerelease).await?;

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
    download_and_install(&client, &asset, force).await?;

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
        UI::detail("🌐 Using unauthenticated requests to GitHub API");
        UI::hint("💡 Use --token <TOKEN> to avoid rate limits in shared environments");
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    Ok(client)
}

async fn get_latest_release(client: &reqwest::Client, prerelease: bool) -> Result<GitHubRelease> {
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
            "GitHub API rate limit exceeded (remaining: {}). \
            Use --token <TOKEN> to authenticate and increase rate limits. \
            See: https://docs.github.com/en/rest/using-the-rest-api/rate-limits-for-the-rest-api",
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

    // Download the new binary
    let response = client.get(&asset.browser_download_url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to download asset: HTTP {}",
            response.status()
        ));
    }

    let content = response.bytes().await?;

    // Create temporary file for the new binary
    let temp_path = current_exe.with_extension("tmp");

    // Handle different asset types
    if asset.name.ends_with(".zip") {
        extract_from_zip(&content, &temp_path)?;
    } else if asset.name.ends_with(".tar.gz") {
        extract_from_tar_gz(&content, &temp_path)?;
    } else {
        // Assume it's a raw binary
        fs::write(&temp_path, &content)?;
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
