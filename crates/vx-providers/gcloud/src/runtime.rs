//! Google Cloud CLI runtime implementation

use crate::config::GcloudUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use vx_runtime::{
    Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo,
};

/// Google Cloud CLI runtime
#[derive(Debug, Clone)]
pub struct GcloudRuntime;

impl GcloudRuntime {
    /// Create a new gcloud runtime
    pub fn new() -> Self {
        Self
    }

    /// Fetch versions from Google Cloud SDK version API
    async fn fetch_gcloud_versions(&self) -> Result<Vec<VersionInfo>> {
        // Google doesn't have a public version API, so we'll use known stable versions
        // In production, this could be enhanced to scrape the downloads page or use a cache
        let versions = vec![
            "509.0.0", "508.0.0", "507.0.1", "507.0.0", "506.0.0", "505.0.0", "504.0.1", "504.0.0",
            "503.0.0", "502.0.0", "501.0.0", "500.0.0", "499.0.0", "498.0.0", "497.0.0",
        ];

        Ok(versions.into_iter().map(VersionInfo::new).collect())
    }
}

impl Default for GcloudRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// gsutil runtime - Google Cloud Storage utility
/// Bundled with gcloud SDK
#[derive(Debug, Clone)]
pub struct GsutilRuntime;

impl GsutilRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GsutilRuntime {
    fn default() -> Self {
        Self::new()
    }
}

/// bq runtime - BigQuery command-line tool
/// Bundled with gcloud SDK
#[derive(Debug, Clone)]
pub struct BqRuntime;

impl BqRuntime {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BqRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for GcloudRuntime {
    fn name(&self) -> &str {
        "gcloud"
    }

    fn description(&self) -> &str {
        "Google Cloud CLI - Google Cloud Platform command-line interface"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cloud".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["google-cloud-sdk", "google-cloud-cli"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://cloud.google.com/sdk/".to_string(),
        );
        meta.insert("ecosystem".to_string(), "cloud".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/GoogleCloudPlatform/google-cloud-sdk".to_string(),
        );
        meta.insert("license".to_string(), "Apache-2.0".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    /// gcloud executable path - SDK extracts to google-cloud-sdk/
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;

        match &platform.os {
            Os::Windows => "google-cloud-sdk/bin/gcloud.cmd".to_string(),
            _ => "google-cloud-sdk/bin/gcloud".to_string(),
        }
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        self.fetch_gcloud_versions().await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(GcloudUrlBuilder::download_url(version, platform))
    }

    /// Custom post-install for gcloud
    /// gcloud requires running install.sh/install.bat
    async fn post_install(&self, _version: &str, _ctx: &RuntimeContext) -> Result<()> {
        // Google Cloud SDK installed. User should run 'gcloud init' to configure.
        Ok(())
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));
        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "gcloud executable not found at {}",
                    exe_path.display()
                )],
                vec![],
            )
        }
    }
}

#[async_trait]
impl Runtime for GsutilRuntime {
    fn name(&self) -> &str {
        "gsutil"
    }

    fn description(&self) -> &str {
        "Google Cloud Storage utility - bundled with gcloud SDK"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cloud".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://cloud.google.com/storage/docs/gsutil".to_string(),
        );
        meta.insert("bundled_with".to_string(), "gcloud".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;
        match &platform.os {
            Os::Windows => "google-cloud-sdk/bin/gsutil.cmd".to_string(),
            _ => "google-cloud-sdk/bin/gsutil".to_string(),
        }
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // gsutil versions match gcloud versions
        Ok(vec![
            VersionInfo::new("509.0.0"),
            VersionInfo::new("508.0.0"),
            VersionInfo::new("507.0.0"),
        ])
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // gsutil is bundled with gcloud, no separate download
        Ok(None)
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));
        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "gsutil executable not found at {}",
                    exe_path.display()
                )],
                vec!["Install gcloud first: vx install gcloud".to_string()],
            )
        }
    }
}

#[async_trait]
impl Runtime for BqRuntime {
    fn name(&self) -> &str {
        "bq"
    }

    fn description(&self) -> &str {
        "BigQuery command-line tool - bundled with gcloud SDK"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("cloud".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["bigquery"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert(
            "homepage".to_string(),
            "https://cloud.google.com/bigquery/docs/bq-command-line-tool".to_string(),
        );
        meta.insert("bundled_with".to_string(), "gcloud".to_string());
        meta
    }

    fn supported_platforms(&self) -> Vec<Platform> {
        Platform::all_common()
    }

    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        use vx_runtime::Os;
        match &platform.os {
            Os::Windows => "google-cloud-sdk/bin/bq.cmd".to_string(),
            _ => "google-cloud-sdk/bin/bq".to_string(),
        }
    }

    async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // bq versions match gcloud versions
        Ok(vec![
            VersionInfo::new("509.0.0"),
            VersionInfo::new("508.0.0"),
            VersionInfo::new("507.0.0"),
        ])
    }

    async fn download_url(&self, _version: &str, _platform: &Platform) -> Result<Option<String>> {
        // bq is bundled with gcloud, no separate download
        Ok(None)
    }

    fn verify_installation(
        &self,
        version: &str,
        install_path: &Path,
        platform: &Platform,
    ) -> VerificationResult {
        let exe_path = install_path.join(self.executable_relative_path(version, platform));
        if exe_path.exists() {
            VerificationResult::success(exe_path)
        } else {
            VerificationResult::failure(
                vec![format!(
                    "bq executable not found at {}",
                    exe_path.display()
                )],
                vec!["Install gcloud first: vx install gcloud".to_string()],
            )
        }
    }
}
