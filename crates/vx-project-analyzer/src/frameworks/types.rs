//! Framework type definitions

use serde::{Deserialize, Serialize};

/// Detected application framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectFramework {
    /// Electron - JavaScript/TypeScript desktop applications
    Electron,
    /// Tauri - Rust + Web technology desktop applications
    Tauri,
    /// React Native - Cross-platform mobile applications
    ReactNative,
    /// Flutter - Cross-platform mobile/desktop applications
    Flutter,
    /// Capacitor - Cross-platform mobile applications
    Capacitor,
    /// NW.js (node-webkit) - Desktop applications
    NwJs,
}

impl ProjectFramework {
    /// Get display name for the framework
    pub fn display_name(&self) -> &'static str {
        match self {
            ProjectFramework::Electron => "Electron",
            ProjectFramework::Tauri => "Tauri",
            ProjectFramework::ReactNative => "React Native",
            ProjectFramework::Flutter => "Flutter",
            ProjectFramework::Capacitor => "Capacitor",
            ProjectFramework::NwJs => "NW.js",
        }
    }

    /// Get the framework's official website
    pub fn website(&self) -> &'static str {
        match self {
            ProjectFramework::Electron => "https://www.electronjs.org/",
            ProjectFramework::Tauri => "https://tauri.app/",
            ProjectFramework::ReactNative => "https://reactnative.dev/",
            ProjectFramework::Flutter => "https://flutter.dev/",
            ProjectFramework::Capacitor => "https://capacitorjs.com/",
            ProjectFramework::NwJs => "https://nwjs.io/",
        }
    }

    /// Get common file indicators for this framework
    pub fn indicator_files(&self) -> &'static [&'static str] {
        match self {
            ProjectFramework::Electron => &[
                "electron.vite.config.js",
                "electron.vite.config.ts",
                "electron-builder.json",
                "electron-builder.yml",
                "electron-builder.yaml",
                "builder-debug.config.ts",
                "forge.config.js",
                "forge.config.ts",
            ],
            ProjectFramework::Tauri => &[
                "tauri.conf.json",
                "tauri.conf.json5",
                "Tauri.toml",
                "src-tauri/tauri.conf.json",
                "src-tauri/Cargo.toml",
            ],
            ProjectFramework::ReactNative => {
                &["app.json", "metro.config.js", "react-native.config.js"]
            }
            ProjectFramework::Flutter => &["pubspec.yaml"],
            ProjectFramework::Capacitor => &["capacitor.config.json", "capacitor.config.ts"],
            ProjectFramework::NwJs => &["package.json"], // Detected via package.json main field
        }
    }
}

impl std::fmt::Display for ProjectFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Detailed information about a detected framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkInfo {
    /// The framework type
    pub framework: ProjectFramework,

    /// Framework version (if detectable)
    pub version: Option<String>,

    /// Configuration file path
    pub config_path: Option<std::path::PathBuf>,

    /// Build tool used (e.g., "electron-builder", "tauri-cli", "electron-forge")
    pub build_tool: Option<String>,

    /// Target platforms (e.g., "win32", "darwin", "linux")
    pub target_platforms: Vec<String>,

    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl FrameworkInfo {
    /// Create a new framework info
    pub fn new(framework: ProjectFramework) -> Self {
        Self {
            framework,
            version: None,
            config_path: None,
            build_tool: None,
            target_platforms: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the config path
    pub fn with_config_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Set the build tool
    pub fn with_build_tool(mut self, tool: impl Into<String>) -> Self {
        self.build_tool = Some(tool.into());
        self
    }

    /// Add a target platform
    pub fn with_platform(mut self, platform: impl Into<String>) -> Self {
        self.target_platforms.push(platform.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}
