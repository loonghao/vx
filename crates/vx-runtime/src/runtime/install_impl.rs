//! Default `install()` implementation for the `Runtime` trait.
//!
//! This module contains the ~200-line default `install()` logic that was
//! previously inlined in `runtime/mod.rs`. It handles:
//!
//! - Platform support check
//! - Already-installed detection (with verification)
//! - Cached download URL from lock file
//! - Mirror URL chain construction
//! - Download with layout metadata
//! - Post-extract hook
//! - RFC 0022 normalization
//! - Final verification

use std::collections::HashMap;

use anyhow::Result;
use tracing::{debug, info};

use crate::context::RuntimeContext;
use crate::layout::LayoutContext;
use crate::platform::{Os, Platform};
use crate::types::InstallResult;

use super::verify::verify_installation_default;

/// Check if a download URL is plausible for the given platform.
///
/// Defensive check to avoid using a cached URL recorded for a different platform
/// (e.g., a Windows `.zip` URL on Linux).
pub fn is_url_plausible_for_platform(url: &str, platform: &Platform) -> bool {
    let url_lower = url.to_lowercase();

    let windows_markers = ["windows", "win32", "win64", "-msvc", ".msi"];
    let macos_markers = ["darwin", "macos", "osx", ".dmg"];
    let linux_markers = ["linux", "gnu", "musl"];

    let url_is_windows = windows_markers.iter().any(|m| url_lower.contains(m));
    let url_is_macos = macos_markers.iter().any(|m| url_lower.contains(m));
    let url_is_linux = linux_markers.iter().any(|m| url_lower.contains(m));

    if !url_is_windows && !url_is_macos && !url_is_linux {
        return true;
    }

    match platform.os {
        Os::Windows => url_is_windows,
        Os::MacOS => url_is_macos,
        Os::Linux => url_is_linux,
        _ => true,
    }
}

/// Parameters extracted from a `Runtime` for use in `default_install_inner`.
///
/// This struct avoids the `?Sized` problem by extracting all needed data from
/// `self` before calling the inner function.
pub struct InstallParams<'a> {
    pub name: &'a str,
    pub store_name: &'a str,
    pub exe_relative: String,
    pub exe_name: &'a str,
    pub exe_extensions: &'a [&'a str],
    pub layout_metadata: HashMap<String, String>,
    pub download_urls: Vec<String>,
    pub normalize_config: Option<&'a vx_runtime_core::NormalizeConfig>,
}

/// Inner install logic that operates on extracted parameters (no `?Sized` issue).
///
/// Called from the `Runtime::install()` default method after extracting all
/// needed data from `self`.
pub async fn default_install_inner(
    params: InstallParams<'_>,
    version: &str,
    ctx: &RuntimeContext,
    post_extract: impl FnOnce(&str, &std::path::Path) -> Result<()>,
) -> Result<InstallResult> {
    let platform = Platform::current();
    let base_install_path = ctx.paths.version_store_dir(params.store_name, version);
    // New layout: install directly to version dir (no platform subdirectory).
    // Old layout used base_install_path.join(platform.as_str()).
    let install_path = base_install_path;

    debug!(
        "Install path for {} (store: {}) {}: {} (platform: {})",
        params.name,
        params.store_name,
        version,
        install_path.display(),
        platform.as_str()
    );
    debug!("Executable relative path: {}", params.exe_relative);

    // Check if already installed
    if ctx.fs.exists(&install_path) {
        let verification = verify_installation_default(
            &params.exe_relative,
            &install_path,
            params.exe_name,
            params.exe_extensions,
        );
        if verification.valid {
            let exe_path = verification
                .executable_path
                .unwrap_or_else(|| install_path.join(&params.exe_relative));
            debug!("Already installed: {}", exe_path.display());
            return Ok(InstallResult::already_installed(
                install_path,
                exe_path,
                version.to_string(),
            ));
        } else {
            debug!(
                "Install directory exists but executable missing, cleaning up: {}",
                install_path.display()
            );
            if let Err(e) = std::fs::remove_dir_all(&install_path) {
                debug!("Failed to clean up directory: {}", e);
            }
        }
    }

    info!(
        "Downloading {} {} ({})",
        params.name,
        version,
        if params.download_urls.len() > 1 {
            format!("{} sources available", params.download_urls.len())
        } else {
            "direct".to_string()
        }
    );

    // Download with mirror fallback chain
    let mut last_error = None;
    for (i, download_url) in params.download_urls.iter().enumerate() {
        let is_mirror = i < params.download_urls.len() - 1 || params.download_urls.len() == 1;
        if i > 0 {
            info!(
                "Mirror failed, trying {} (source {}/{})",
                download_url,
                i + 1,
                params.download_urls.len()
            );
            if ctx.fs.exists(&install_path)
                && let Err(e) = std::fs::remove_dir_all(&install_path)
            {
                debug!("Failed to clean up partial download: {}", e);
            }
        } else {
            info!("Downloading from {}", download_url);
        }

        let result = if !params.layout_metadata.is_empty() {
            ctx.installer
                .download_with_layout(download_url, &install_path, &params.layout_metadata)
                .await
        } else {
            ctx.installer
                .download_and_extract(download_url, &install_path)
                .await
        };

        match result {
            Ok(()) => {
                if i > 0 {
                    info!(
                        "Successfully downloaded {} {} from fallback source",
                        params.name, version
                    );
                }
                last_error = None;
                break;
            }
            Err(e) => {
                if is_mirror && params.download_urls.len() > 1 {
                    debug!(
                        "Download from {} failed: {}, will try next source",
                        download_url, e
                    );
                }
                last_error = Some(e);
            }
        }
    }

    if let Some(err) = last_error {
        return Err(err);
    }

    // Run post-extract hook
    post_extract(version, &install_path)?;

    // RFC 0022: Post-install normalization
    if let Some(normalize_config) = params.normalize_config {
        use crate::normalizer::{NormalizeContext, Normalizer};

        let normalize_ctx = NormalizeContext::new(params.name, version);
        match Normalizer::normalize(&install_path, normalize_config, &normalize_ctx) {
            Ok(result) => {
                if result.has_changes() {
                    debug!("Normalization completed: {}", result.summary());
                }
            }
            Err(e) => {
                debug!("Normalization warning: {}", e);
            }
        }
    }

    debug!("Expected executable path pattern: {}", params.exe_relative);

    // Verify installation
    let verification = verify_installation_default(
        &params.exe_relative,
        &install_path,
        params.exe_name,
        params.exe_extensions,
    );

    if !verification.valid {
        let mut error_msg = format!(
            "Installation of {} {} failed verification.\n",
            params.name, version
        );
        error_msg.push_str("\nIssues found:\n");
        for issue in &verification.issues {
            error_msg.push_str(&format!("  - {}\n", issue));
        }
        if !verification.suggestions.is_empty() {
            error_msg.push_str("\nSuggestions:\n");
            for suggestion in &verification.suggestions {
                error_msg.push_str(&format!("  - {}\n", suggestion));
            }
        }
        return Err(anyhow::anyhow!(error_msg));
    }

    let verified_exe_path = verification
        .executable_path
        .unwrap_or_else(|| install_path.join(&params.exe_relative));

    Ok(InstallResult::success(
        install_path,
        verified_exe_path,
        version.to_string(),
    ))
}

/// Build layout metadata HashMap from the runtime's `executable_layout()`.
pub fn build_layout_metadata(
    layout: Option<&crate::layout::ExecutableLayout>,
    version: &str,
    name: &str,
    platform: &Platform,
) -> HashMap<String, String> {
    let mut layout_metadata = HashMap::new();

    let Some(layout) = layout else {
        return layout_metadata;
    };

    let layout_ctx = LayoutContext {
        version: version.to_string(),
        name: name.to_string(),
        platform: platform.clone(),
    };

    let Ok(resolved) = layout.resolve(&layout_ctx) else {
        return layout_metadata;
    };

    match resolved {
        crate::layout::ResolvedLayout::Binary {
            source_name,
            target_name,
            target_dir,
            permissions,
        } => {
            layout_metadata.insert("source_name".to_string(), source_name);
            layout_metadata.insert("target_name".to_string(), target_name);
            layout_metadata.insert("target_dir".to_string(), target_dir);
            if let Some(perms) = permissions {
                layout_metadata.insert("target_permissions".to_string(), perms);
            }
        }
        crate::layout::ResolvedLayout::Archive {
            strip_prefix,
            permissions,
            ..
        } => {
            if let Some(prefix) = strip_prefix {
                layout_metadata.insert("strip_prefix".to_string(), prefix);
            }
            if let Some(perms) = permissions {
                layout_metadata.insert("target_permissions".to_string(), perms);
            }
        }
    }

    layout_metadata
}
