//! Real installer implementation

use super::http_client::RealHttpClient;
use crate::traits::Installer;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

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

    /// Create a new installer with download caching enabled
    pub fn with_download_cache(cache_dir: std::path::PathBuf) -> Self {
        Self {
            http: RealHttpClient::new().with_download_cache(cache_dir),
        }
    }

    /// Extract filename from response headers and final URL.
    ///
    /// Tries in order:
    /// 1. Content-Disposition header
    /// 2. Final URL path (after redirects)
    fn extract_filename_from_response(
        response: &reqwest::Response,
        original_url: &str,
    ) -> Option<String> {
        // Try Content-Disposition header first
        if let Some(content_disposition) = response.headers().get("content-disposition")
            && let Ok(value) = content_disposition.to_str()
            && let Some(filename) = Self::parse_content_disposition(value)
        {
            tracing::debug!("Got filename from Content-Disposition: {}", filename);
            return Some(filename);
        }

        // Try to get filename from final URL (after redirects)
        let final_url = response.url().as_str();
        if final_url != original_url {
            let filename = final_url
                .split('/')
                .next_back()
                .unwrap_or("download")
                .split('?')
                .next()
                .unwrap_or("download");
            // Only use if it has a valid extension
            if filename.contains('.') && !filename.starts_with('.') {
                tracing::debug!("Got filename from final URL: {}", filename);
                return Some(filename.to_string());
            }
        }

        None
    }

    /// Download a file and return the detected filename from the server response.
    ///
    /// This avoids a separate HEAD request by extracting the filename from
    /// the GET response's Content-Disposition header or final redirected URL.
    /// For APIs like Adoptium that use redirect chains (307 → 302 → CDN),
    /// this saves ~3-5 seconds by eliminating the redundant HEAD round-trip.
    async fn download_and_detect_filename(&self, url: &str, dest: &Path) -> Result<Option<String>> {
        use futures_util::StreamExt;
        use indicatif::{ProgressBar, ProgressStyle};
        use tokio::io::AsyncWriteExt;

        // Check download cache first
        if let Some(cache) = &self.http.download_cache {
            let lookup = cache.lookup(url);
            match lookup {
                vx_cache::CacheLookupResult::Hit { path, metadata } => {
                    let filename = RealHttpClient::extract_display_name_from_url(url);
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
                    tracing::debug!(url = url, cached_path = ?path, "Served from download cache");
                    // Return cached filename from metadata
                    let cached_filename = if !metadata.filename.is_empty()
                        && metadata.filename.contains('.')
                        && !metadata.filename.starts_with('.')
                    {
                        Some(metadata.filename.clone())
                    } else {
                        None
                    };
                    return Ok(cached_filename);
                }
                vx_cache::CacheLookupResult::NeedsRevalidation { path, metadata } => {
                    let filename = RealHttpClient::extract_display_name_from_url(url);
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
                    return Ok(None);
                }
                vx_cache::CacheLookupResult::Miss => {}
            }
        }

        // Optimize URL with CDN if enabled
        let download_url = self.http.optimize_url(url).await;
        let using_cdn = download_url.as_str() != url;
        if using_cdn {
            tracing::info!(
                original = url,
                optimized = %download_url,
                "Using CDN accelerated URL"
            );
        }

        let response = self.http.client.get(download_url.as_str()).send().await;

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
                let fallback_resp = self.http.client.get(url).send().await?;
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
                let fallback_resp = self.http.client.get(url).send().await?;
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

        // Extract filename from response BEFORE consuming the body
        let detected_filename = Self::extract_filename_from_response(&response, url);
        let total_size = response.content_length().unwrap_or(0);

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = tokio::fs::File::create(dest).await?;
        let mut stream = response.bytes_stream();

        let filename_display = RealHttpClient::extract_display_name_from_url(url);
        let cdn_suffix = if actual_using_cdn { " [CDN]" } else { "" };

        let progress_bar = if total_size > 0 {
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "{filename_display}{cdn_suffix} (download) {{wide_bar:.cyan/blue}} {{bytes}}/{{total_bytes}}"
                ))
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("━━╺"),
            );
            pb
        } else {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::with_template(&format!(
                    "{{spinner:.green}} {filename_display}{cdn_suffix} (download) {{bytes}}"
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

        progress_bar.finish_and_clear();
        file.flush().await?;

        // Store in download cache if enabled
        if let Some(cache) = &self.http.download_cache
            && let Err(e) = cache.store(url, dest, None, None, None)
        {
            tracing::warn!(url = url, error = %e, "Failed to cache download");
        }

        Ok(detected_filename)
    }

    /// Parse filename from Content-Disposition header value
    fn parse_content_disposition(value: &str) -> Option<String> {
        // Handle: attachment; filename=xxx.zip
        // Handle: attachment; filename="xxx.zip"
        // Handle: attachment; filename*=UTF-8''xxx.zip

        for part in value.split(';') {
            let part = part.trim();

            // Standard filename parameter
            if let Some(filename) = part.strip_prefix("filename=") {
                let filename = filename.trim_matches('"').trim_matches('\'');
                if !filename.is_empty() {
                    return Some(filename.to_string());
                }
            }

            // RFC 5987 encoded filename (filename*=)
            if let Some(encoded) = part.strip_prefix("filename*=") {
                // Format: charset'language'encoded_filename
                // e.g., UTF-8''OpenJDK25U-jdk_x64_windows_hotspot_25.0.1_8.zip
                if let Some(pos) = encoded.rfind("''") {
                    let filename = &encoded[pos + 2..];
                    // URL decode the filename
                    if let Ok(decoded) = urlencoding::decode(filename)
                        && !decoded.is_empty()
                    {
                        return Some(decoded.into_owned());
                    }
                }
            }
        }

        None
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
        } else if archive_str.ends_with(".tar.zst") || archive_str.ends_with(".tzst") {
            Some("tar.zst")
        } else if archive_str.ends_with(".zip") {
            Some("zip")
        } else if archive_str.ends_with(".7z") {
            Some("7z")
        } else if archive_str.ends_with(".msi") {
            Some("msi")
        } else if archive_str.ends_with(".pkg") {
            Some("pkg")
        } else {
            // Try to detect by magic bytes
            let file = std::fs::File::open(archive)?;
            use std::io::Read;
            let mut magic = [0u8; 6];
            if (&file).take(6).read(&mut magic).is_ok() {
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
                } else if magic[0] == 0x28
                    && magic[1] == 0xB5
                    && magic[2] == 0x2F
                    && magic[3] == 0xFD
                {
                    // Zstd magic: \x28\xB5\x2F\xFD
                    Some("tar.zst")
                } else if magic[0] == 0x37
                    && magic[1] == 0x7A
                    && magic[2] == 0xBC
                    && magic[3] == 0xAF
                    && magic[4] == 0x27
                    && magic[5] == 0x1C
                {
                    // 7z magic: 7z\xBC\xAF\x27\x1C
                    Some("7z")
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
                use xz2::read::XzDecoder;
                let file = std::fs::File::open(archive)?;
                let decoder = XzDecoder::new(file);
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            Some("tar.zst") => {
                let file = std::fs::File::open(archive)?;
                let decoder = zstd::stream::read::Decoder::new(std::io::BufReader::new(file))?;
                let mut archive = tar::Archive::new(decoder);
                archive.unpack(dest)?;
            }
            Some("zip") => {
                let file = std::fs::File::open(archive)?;
                let mut archive = zip::ZipArchive::new(file)?;
                archive.extract(dest)?;
            }
            Some("7z") => {
                sevenz_rust::decompress_file(archive, dest)
                    .map_err(|e| anyhow::anyhow!("Failed to extract 7z archive: {}", e))?;
            }
            Some("msi") => {
                // Use msiexec to extract MSI packages (Windows-only)
                #[cfg(target_os = "windows")]
                {
                    let status = std::process::Command::new("msiexec")
                        .args([
                            "/a",
                            &archive.to_string_lossy(),
                            "/qn",
                            &format!("TARGETDIR={}", dest.to_string_lossy()),
                        ])
                        .status()?;
                    if !status.success() {
                        return Err(anyhow::anyhow!(
                            "msiexec failed to extract MSI (exit code: {:?})",
                            status.code()
                        ));
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Err(anyhow::anyhow!(
                        "MSI extraction is only supported on Windows: {}",
                        archive_str
                    ));
                }
            }
            Some("pkg") => {
                // Use pkgutil to extract macOS .pkg packages
                #[cfg(target_os = "macos")]
                {
                    let expand_dir = dest.join(".pkg_expand");

                    let output = std::process::Command::new("pkgutil")
                        .arg("--expand-full")
                        .arg(archive)
                        .arg(&expand_dir)
                        .output()?;

                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(anyhow::anyhow!("pkgutil --expand-full failed: {}", stderr));
                    }

                    // Promote Payload contents to dest directory
                    promote_pkg_payload_contents(&expand_dir, dest)?;

                    // Clean up expand directory
                    let _ = std::fs::remove_dir_all(&expand_dir);

                    // Flatten executables to the install root so vx can find them.
                    // macOS .pkg files typically install to paths like usr/local/bin/
                    // which are deeply nested. We copy executables to the root so
                    // that verify_installation() can locate them at <install_dir>/<exe>.
                    flatten_pkg_executables(dest)?;
                }
                #[cfg(not(target_os = "macos"))]
                {
                    return Err(anyhow::anyhow!(
                        "PKG extraction is only supported on macOS: {}",
                        archive_str
                    ));
                }
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

        // Download and detect filename in a single GET request (no separate HEAD).
        // This saves ~3-5 seconds for APIs that use redirect chains (e.g., Adoptium:
        // api.adoptium.net → github.com → objects.githubusercontent.com).
        let temp_download_path = temp_dir.path().join("download_temp");
        let detected_filename = self
            .download_and_detect_filename(url_without_fragment, &temp_download_path)
            .await?;

        let archive_name = detected_filename.unwrap_or_else(|| {
            url_without_fragment
                .split('/')
                .next_back()
                .unwrap_or("archive")
                .split('?')
                .next()
                .unwrap_or("archive")
                .to_string()
        });

        // Rename temp file to actual filename so extraction can detect format
        let temp_path = temp_dir.path().join(&archive_name);
        if temp_download_path != temp_path {
            std::fs::rename(&temp_download_path, &temp_path)?;
        }

        // Check for extension hint in URL fragment
        let extension_hint = url.split('#').nth(1);

        // Check if it's an archive or a single executable
        // First check the URL/filename, then check extension hint, then check file magic bytes
        let archive_str = archive_name.to_lowercase();
        let mut is_archive = archive_str.ends_with(".tar.gz")
            || archive_str.ends_with(".tgz")
            || archive_str.ends_with(".tar.xz")
            || archive_str.ends_with(".tar.zst")
            || archive_str.ends_with(".tzst")
            || archive_str.ends_with(".zip")
            || archive_str.ends_with(".7z")
            || archive_str.ends_with(".msi")
            || archive_str.ends_with(".pkg");

        // Check extension hint from URL fragment
        if !is_archive && let Some(hint) = extension_hint {
            is_archive = hint.ends_with(".tar.gz")
                || hint.ends_with(".tgz")
                || hint.ends_with(".tar.xz")
                || hint.ends_with(".zip")
                || hint.ends_with(".7z");
        }

        // Check file magic bytes if still uncertain
        if !is_archive && let Ok(mut file) = std::fs::File::open(&temp_path) {
            use std::io::Read;
            let mut magic = [0u8; 6];
            if file.read_exact(&mut magic).is_ok() {
                // ZIP magic: PK\x03\x04
                // GZIP magic: \x1f\x8b
                // 7z magic: 7z\xBC\xAF\x27\x1C (first 6 bytes: 37 7A BC AF 27 1C)
                is_archive = (magic[0] == 0x50 && magic[1] == 0x4B)  // ZIP
                        || (magic[0] == 0x1f && magic[1] == 0x8b) // GZIP (tar.gz)
                        || (magic[0] == 0x37 && magic[1] == 0x7A && magic[2] == 0xBC
                            && magic[3] == 0xAF && magic[4] == 0x27 && magic[5] == 0x1C);
                // 7z
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

/// Promote files from Payload directories to the target directory after pkgutil --expand-full.
///
/// After `pkgutil --expand-full`, files are nested like:
///   `expand_dir/<component>.pkg/Payload/<actual files>`
///
/// This function moves the Payload contents up to `target_dir`.
#[cfg(target_os = "macos")]
fn promote_pkg_payload_contents(expand_dir: &Path, target_dir: &Path) -> Result<()> {
    if let Ok(entries) = std::fs::read_dir(expand_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let payload_dir = path.join("Payload");
                if payload_dir.exists() && payload_dir.is_dir() {
                    copy_dir_contents_recursive(&payload_dir, target_dir)?;
                }
            }
        }
    }
    Ok(())
}

/// Recursively copy/move contents of src_dir into dst_dir.
#[cfg(target_os = "macos")]
fn copy_dir_contents_recursive(src_dir: &Path, dst_dir: &Path) -> Result<()> {
    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let src_path = entry.path();
            let file_name = match src_path.file_name() {
                Some(name) => name.to_owned(),
                None => continue,
            };
            let dst_path = dst_dir.join(&file_name);

            if src_path.is_dir() {
                std::fs::create_dir_all(&dst_path)?;
                copy_dir_contents_recursive(&src_path, &dst_path)?;
            } else {
                if let Some(parent) = dst_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::rename(&src_path, &dst_path)
                    .or_else(|_| std::fs::copy(&src_path, &dst_path).map(|_| ()))?;
            }
        }
    }
    Ok(())
}

/// Flatten executables from nested directories to the install root.
///
/// macOS .pkg files typically install to system paths like `usr/local/bin/`.
/// After promotion, the structure might be:
///   `dest/usr/local/bin/actrun`
///
/// vx expects executables at `dest/actrun` (or within 3 levels of depth).
/// This function finds all executable files in the extracted tree and
/// copies them to the install root, making them discoverable by vx.
#[cfg(target_os = "macos")]
fn flatten_pkg_executables(dest: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut executables = Vec::new();
    find_executables_recursive(dest, &mut executables, 0, 10);

    for exe_path in &executables {
        let file_name = match exe_path.file_name() {
            Some(name) => name.to_owned(),
            None => continue,
        };
        let dest_path = dest.join(&file_name);

        // Skip if already at root level
        if exe_path.parent() == Some(dest) {
            continue;
        }

        // Skip if a file with the same name already exists at root
        if dest_path.exists() {
            continue;
        }

        tracing::debug!(
            "Flattening pkg executable: {} -> {}",
            exe_path.display(),
            dest_path.display()
        );

        // Hard-link or copy the executable to root
        std::fs::hard_link(exe_path, &dest_path)
            .or_else(|_| std::fs::copy(exe_path, &dest_path).map(|_| ()))?;
    }

    Ok(())
}

/// Recursively find executable files in a directory.
#[cfg(target_os = "macos")]
fn find_executables_recursive(
    dir: &Path,
    results: &mut Vec<std::path::PathBuf>,
    depth: usize,
    max_depth: usize,
) {
    use std::os::unix::fs::PermissionsExt;

    if depth > max_depth || !dir.exists() {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            // Check if file is executable
            if let Ok(metadata) = std::fs::metadata(&path) {
                let mode = metadata.permissions().mode();
                if mode & 0o111 != 0 {
                    // Skip common non-tool files
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if !name.starts_with('.')
                        && !name.ends_with(".dylib")
                        && !name.ends_with(".so")
                        && !name.ends_with(".a")
                    {
                        results.push(path);
                    }
                }
            }
        } else if path.is_dir() {
            // Skip hidden directories and common non-relevant dirs
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !dir_name.starts_with('.') {
                find_executables_recursive(&path, results, depth + 1, max_depth);
            }
        }
    }
}
