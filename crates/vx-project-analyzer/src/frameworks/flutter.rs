//! Flutter framework detector
//! Detects Flutter projects via pubspec.yaml and platform folders.

use super::{FrameworkDetector, FrameworkInfo, ProjectFramework};
use crate::dependency::{Dependency, InstallMethod};
use crate::ecosystem::Ecosystem;
use crate::error::AnalyzerResult;
use crate::types::{RequiredTool, Script};
use async_trait::async_trait;
use std::path::Path;
use tracing::debug;

/// Flutter framework detector
pub struct FlutterDetector;

impl FlutterDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self
    }

    fn pubspec_path(root: &Path) -> std::path::PathBuf {
        root.join("pubspec.yaml")
    }

    fn has_flutter_sdk_marker(content: &str) -> bool {
        // Light-weight detection to avoid new dependencies
        content.contains("sdk: flutter") || content.contains("flutter:")
    }

    fn detect_platform_dirs(root: &Path, info: &mut FrameworkInfo) {
        let platforms = [
            ("android", "android"),
            ("ios", "ios"),
            ("macos", "macos"),
            ("linux", "linux"),
            ("windows", "windows"),
            ("web", "web"),
        ];

        for (dir, name) in platforms {
            if root.join(dir).is_dir() {
                info.target_platforms.push(name.to_string());
            }
        }
    }
}

impl Default for FlutterDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FrameworkDetector for FlutterDetector {
    fn detect(&self, root: &Path) -> bool {
        let pubspec = Self::pubspec_path(root);
        if pubspec.exists() {
            if let Ok(content) = std::fs::read_to_string(&pubspec) {
                if Self::has_flutter_sdk_marker(&content) {
                    debug!("Detected Flutter via pubspec.yaml");
                    return true;
                }
            }
        }

        false
    }

    fn framework(&self) -> ProjectFramework {
        ProjectFramework::Flutter
    }

    async fn get_info(&self, root: &Path) -> AnalyzerResult<FrameworkInfo> {
        let mut info = FrameworkInfo::new(ProjectFramework::Flutter);

        let pubspec = Self::pubspec_path(root);
        if pubspec.exists() {
            info = info.with_config_path(pubspec.clone());
        }

        // Build tool for Flutter projects
        info = info.with_build_tool("flutter");

        // Platform inference
        Self::detect_platform_dirs(root, &mut info);

        Ok(info)
    }

    fn required_tools(&self, _deps: &[Dependency], _scripts: &[Script]) -> Vec<RequiredTool> {
        vec![
            RequiredTool::new(
                "flutter",
                Ecosystem::Unknown,
                "Flutter SDK",
                InstallMethod::Manual {
                    instructions:
                        "Install Flutter SDK: https://docs.flutter.dev/get-started/install"
                            .to_string(),
                },
            ),
            RequiredTool::new(
                "dart",
                Ecosystem::Unknown,
                "Dart SDK (bundled with Flutter)",
                InstallMethod::Manual {
                    instructions: "Install Flutter SDK which includes Dart".to_string(),
                },
            )
            .available(),
        ]
    }

    async fn additional_scripts(&self, _root: &Path) -> AnalyzerResult<Vec<Script>> {
        // Flutter relies on flutter tool; scripts usually not needed from package.json
        Ok(Vec::new())
    }
}
