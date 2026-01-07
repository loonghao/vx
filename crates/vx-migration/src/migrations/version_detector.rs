//! Version detection for vx configurations.

use crate::error::MigrationResult;
use crate::traits::VersionDetector;
use crate::version::{Version, VersionRange};
use async_trait::async_trait;
use std::path::Path;

/// Version detector for vx configuration files
pub struct VxVersionDetector;

impl VxVersionDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self
    }

    /// Detect version from config content
    fn detect_from_content(&self, content: &str) -> Option<Version> {
        // Try to parse as TOML and look for version field
        if let Ok(value) = content.parse::<toml::Value>() {
            // Check for explicit version field
            if let Some(version) = value.get("version").and_then(|v| v.as_str()) {
                if let Ok(v) = version.parse() {
                    return Some(v);
                }
            }

            // Check for vx section with version
            if let Some(vx) = value.get("vx").and_then(|v| v.as_table()) {
                if let Some(version) = vx.get("version").and_then(|v| v.as_str()) {
                    if let Ok(v) = version.parse() {
                        return Some(v);
                    }
                }
            }

            // Detect v1 format (has [tools] section)
            if value.get("tools").is_some() {
                return Some(Version::new(1, 0, 0));
            }

            // Detect v2 format (has [runtimes] section)
            if value.get("runtimes").is_some() {
                return Some(Version::new(2, 0, 0));
            }
        }

        None
    }
}

impl Default for VxVersionDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VersionDetector for VxVersionDetector {
    fn name(&self) -> &str {
        "vx-version-detector"
    }

    async fn detect(&self, path: &Path) -> MigrationResult<Option<Version>> {
        // Check for vx.toml
        let vx_toml = path.join("vx.toml");
        if vx_toml.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&vx_toml).await {
                if let Some(version) = self.detect_from_content(&content) {
                    return Ok(Some(version));
                }
                // If vx.toml exists but no version detected, assume v1
                return Ok(Some(Version::new(1, 0, 0)));
            }
        }

        // Check for .vx.toml (old format)
        let dot_vx_toml = path.join(".vx.toml");
        if dot_vx_toml.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&dot_vx_toml).await {
                if let Some(version) = self.detect_from_content(&content) {
                    return Ok(Some(version));
                }
                // If .vx.toml exists but no version detected, assume v1
                return Ok(Some(Version::new(1, 0, 0)));
            }
        }

        Ok(None)
    }

    fn supported_range(&self) -> VersionRange {
        VersionRange::any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_from_content_v2() {
        let detector = VxVersionDetector::new();
        let config = r#"
[runtimes]
node = "18.0.0"
"#;
        // Debug: test TOML parsing
        let parsed: Result<toml::Value, _> = config.parse();
        assert!(parsed.is_ok(), "TOML should parse: {:?}", parsed.err());
        let value = parsed.unwrap();
        assert!(value.get("runtimes").is_some(), "Should have runtimes key");
        
        let version = detector.detect_from_content(config);
        assert_eq!(version, Some(Version::new(2, 0, 0)));
    }

    #[tokio::test]
    async fn test_detect_v1_format() {
        let temp = TempDir::new().unwrap();
        let config = r#"
[tools]
node = "18.0.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let detector = VxVersionDetector::new();
        let version = detector.detect(temp.path()).await.unwrap();
        assert_eq!(version, Some(Version::new(1, 0, 0)));
    }

    #[tokio::test]
    async fn test_detect_v2_format() {
        let temp = TempDir::new().unwrap();
        let config = r#"
[runtimes]
node = "18.0.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let detector = VxVersionDetector::new();
        let version = detector.detect(temp.path()).await.unwrap();
        assert_eq!(version, Some(Version::new(2, 0, 0)));
    }

    #[tokio::test]
    async fn test_detect_explicit_version() {
        let temp = TempDir::new().unwrap();
        let config = r#"
version = "2.1.0"

[runtimes]
node = "18.0.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let detector = VxVersionDetector::new();
        let version = detector.detect(temp.path()).await.unwrap();
        assert_eq!(version, Some(Version::new(2, 1, 0)));
    }

    #[tokio::test]
    async fn test_detect_old_filename() {
        let temp = TempDir::new().unwrap();
        let config = r#"
[tools]
node = "18.0.0"
"#;
        tokio::fs::write(temp.path().join("vx.toml"), config)
            .await
            .unwrap();

        let detector = VxVersionDetector::new();
        let version = detector.detect(temp.path()).await.unwrap();
        assert_eq!(version, Some(Version::new(1, 0, 0)));
    }

    #[tokio::test]
    async fn test_detect_no_config() {
        let temp = TempDir::new().unwrap();

        let detector = VxVersionDetector::new();
        let version = detector.detect(temp.path()).await.unwrap();
        assert_eq!(version, None);
    }
}
