//! Java runtime implementation

use crate::config::JavaUrlBuilder;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;
use vx_runtime::{Ecosystem, Platform, Runtime, RuntimeContext, VerificationResult, VersionInfo};

/// Java runtime (Temurin JDK)
#[derive(Debug, Clone)]
pub struct JavaRuntime;

impl JavaRuntime {
    /// Create a new Java runtime
    pub fn new() -> Self {
        Self
    }

    /// Find the JDK directory in the install path
    /// Temurin archives extract to directories like jdk-21.0.1+12
    fn find_jdk_dir(install_path: &Path) -> Option<PathBuf> {
        std::fs::read_dir(install_path)
            .ok()?
            .filter_map(|e| e.ok())
            .find(|e| {
                let name = e.file_name();
                let name_str = name.to_string_lossy();
                e.path().is_dir() && name_str.starts_with("jdk-")
            })
            .map(|e| e.path())
    }
}

impl Default for JavaRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Runtime for JavaRuntime {
    fn name(&self) -> &str {
        "java"
    }

    fn description(&self) -> &str {
        "Java Development Kit (Eclipse Temurin)"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Custom("java".to_string())
    }

    fn aliases(&self) -> &[&str] {
        &["jdk", "temurin", "openjdk"]
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("homepage".to_string(), "https://adoptium.net/".to_string());
        meta.insert("ecosystem".to_string(), "java".to_string());
        meta.insert(
            "repository".to_string(),
            "https://github.com/adoptium/temurin-build".to_string(),
        );
        meta.insert(
            "license".to_string(),
            "GPL-2.0-with-classpath-exception".to_string(),
        );
        meta
    }

    /// Java archives extract to a versioned directory like jdk-21.0.1+12
    /// After post_extract flattening, the executable is at bin/java
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        let exe_name = platform.exe_name("java");
        // On macOS, the structure is different (Contents/Home/bin/java)
        if platform.os == vx_runtime::Os::MacOS {
            format!("Contents/Home/bin/{}", exe_name)
        } else {
            format!("bin/{}", exe_name)
        }
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Fetch available feature versions from Adoptium API with caching
        let url = "https://api.adoptium.net/v3/info/available_releases";

        let response = ctx
            .get_cached_or_fetch("java", || async { ctx.http.get_json_value(url).await })
            .await?;

        let mut versions = Vec::new();

        // Get available LTS versions
        if let Some(lts_versions) = response
            .get("available_lts_releases")
            .and_then(|v| v.as_array())
        {
            for v in lts_versions {
                if let Some(version_num) = v.as_u64() {
                    versions.push(VersionInfo {
                        version: version_num.to_string(),
                        released_at: None,
                        prerelease: false,
                        lts: true,
                        download_url: None,
                        checksum: None,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        // Get available non-LTS versions
        if let Some(available_versions) = response
            .get("available_releases")
            .and_then(|v| v.as_array())
        {
            for v in available_versions {
                if let Some(version_num) = v.as_u64() {
                    let version_str = version_num.to_string();
                    // Skip if already added as LTS
                    if !versions.iter().any(|vi| vi.version == version_str) {
                        versions.push(VersionInfo {
                            version: version_str,
                            released_at: None,
                            prerelease: false,
                            lts: false,
                            download_url: None,
                            checksum: None,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }

        // Sort by version number descending
        versions.sort_by(|a, b| {
            let a_num: u64 = a.version.parse().unwrap_or(0);
            let b_num: u64 = b.version.parse().unwrap_or(0);
            b_num.cmp(&a_num)
        });

        Ok(versions)
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(JavaUrlBuilder::download_url(version, platform))
    }

    /// Post-extract hook to flatten the JDK directory structure
    ///
    /// Temurin archives extract to: jdk-21.0.1+12/bin/java
    /// We flatten to: bin/java
    fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
        use std::fs;

        // Find the JDK directory (e.g., jdk-21.0.1+12)
        let jdk_dir = match Self::find_jdk_dir(install_path) {
            Some(dir) => dir,
            None => {
                debug!("No jdk-* directory found, assuming already flattened");
                return Ok(());
            }
        };

        debug!("Found JDK directory: {:?}", jdk_dir);

        // Move all contents from jdk-* to install_path
        for entry in fs::read_dir(&jdk_dir)? {
            let entry = entry?;
            let src = entry.path();
            let dst = install_path.join(entry.file_name());

            if dst.exists() {
                if dst.is_dir() {
                    fs::remove_dir_all(&dst)?;
                } else {
                    fs::remove_file(&dst)?;
                }
            }

            debug!("Moving {:?} to {:?}", src, dst);
            fs::rename(&src, &dst)?;
        }

        // Remove the now-empty JDK directory
        let _ = fs::remove_dir_all(&jdk_dir);

        debug!("Java directory flattening completed");
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
            return VerificationResult::success(exe_path);
        }

        // Try alternate paths (in case post_extract didn't run or macOS structure)
        let alt_paths = if platform.os == vx_runtime::Os::MacOS {
            vec![
                install_path.join("Contents/Home/bin/java"),
                install_path.join("bin/java"),
            ]
        } else {
            vec![install_path.join("bin").join(platform.exe_name("java"))]
        };

        for alt in alt_paths {
            if alt.exists() {
                return VerificationResult::success(alt);
            }
        }

        // Search for java executable recursively
        if let Some(entry) = walkdir::WalkDir::new(install_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
            .find(|e| {
                let name = e.file_name().to_string_lossy();
                name == "java" || name == "java.exe"
            })
        {
            return VerificationResult::success(entry.path().to_path_buf());
        }

        VerificationResult::failure(
            vec![format!(
                "Java executable not found at expected path: {}",
                exe_path.display()
            )],
            vec!["Try reinstalling Java: vx install java".to_string()],
        )
    }
}
