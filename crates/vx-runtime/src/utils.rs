//! Shared utility functions for vx-runtime.

use anyhow::Result;

use crate::{RuntimeContext, VersionInfo, platform::compare_semver};

/// Fetch available versions for a package from PyPI.
///
/// Used by both [`crate::manifest_runtime::ManifestDrivenRuntime`] (for pip-based tools)
/// and [`crate::package_runtime::PackageRuntime`] (for Python ecosystem packages).
pub async fn fetch_pypi_versions(
    package_name: &str,
    ctx: &RuntimeContext,
) -> Result<Vec<VersionInfo>> {
    let url = format!("https://pypi.org/pypi/{}/json", package_name);
    let response = ctx.http.get_json_value(&url).await?;

    let mut versions = Vec::new();

    if let Some(releases) = response.get("releases").and_then(|v| v.as_object()) {
        for (version, files) in releases {
            // Skip yanked/empty releases
            if files.as_array().map(|a| a.is_empty()).unwrap_or(true) {
                continue;
            }

            let mut version_info = VersionInfo::new(version.clone());

            // Detect pre-release markers
            if version.contains('a')
                || version.contains('b')
                || version.contains("rc")
                || version.contains("dev")
            {
                version_info = version_info.with_prerelease(true);
            }

            versions.push(version_info);
        }
    }

    // Sort newest first
    versions.sort_by(|a, b| compare_semver(&b.version, &a.version));

    Ok(versions)
}
