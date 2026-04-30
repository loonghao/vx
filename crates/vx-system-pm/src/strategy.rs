//! Installation strategy definitions

use serde::{Deserialize, Serialize};

/// Installation strategy for a system tool
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InstallStrategy {
    /// Use a system package manager
    PackageManager {
        /// Package manager name (choco, winget, brew, apt, etc.)
        manager: String,
        /// Package name
        package: String,
        /// Installation parameters (Chocolatey --params)
        #[serde(default)]
        params: Option<String>,
        /// Native installer arguments (Chocolatey --install-arguments)
        #[serde(default)]
        install_args: Option<String>,
        /// Priority (higher = preferred)
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Direct download from URL
    DirectDownload {
        /// URL template (supports {version}, {platform}, {arch})
        url: String,
        /// Archive format (tar.gz, zip, etc.)
        #[serde(default)]
        format: Option<String>,
        /// Path to executable within archive
        #[serde(default)]
        executable_path: Option<String>,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Run an installation script
    Script {
        /// Script URL
        url: String,
        /// Script type
        script_type: ScriptType,
        /// Script arguments
        #[serde(default)]
        args: Vec<String>,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },

    /// Tool is provided by another runtime
    ProvidedBy {
        /// Provider runtime name
        provider: String,
        /// Relative path to the tool within provider's installation
        relative_path: String,
        /// Priority
        #[serde(default = "default_priority")]
        priority: i32,
    },
}

fn default_priority() -> i32 {
    50
}

impl InstallStrategy {
    /// Get the priority of this strategy
    pub fn priority(&self) -> i32 {
        match self {
            Self::PackageManager { priority, .. } => *priority,
            Self::DirectDownload { priority, .. } => *priority,
            Self::Script { priority, .. } => *priority,
            Self::ProvidedBy { priority, .. } => *priority,
        }
    }

    /// Create a package manager strategy
    pub fn package_manager(manager: impl Into<String>, package: impl Into<String>) -> Self {
        Self::PackageManager {
            manager: manager.into(),
            package: package.into(),
            params: None,
            install_args: None,
            priority: default_priority(),
        }
    }

    /// Create a direct download strategy
    pub fn direct_download(url: impl Into<String>) -> Self {
        Self::DirectDownload {
            url: url.into(),
            format: None,
            executable_path: None,
            priority: default_priority(),
        }
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        match &mut self {
            Self::PackageManager { priority: p, .. } => *p = priority,
            Self::DirectDownload { priority: p, .. } => *p = priority,
            Self::Script { priority: p, .. } => *p = priority,
            Self::ProvidedBy { priority: p, .. } => *p = priority,
        }
        self
    }
}

impl Default for InstallStrategy {
    fn default() -> Self {
        Self::PackageManager {
            manager: String::new(),
            package: String::new(),
            params: None,
            install_args: None,
            priority: default_priority(),
        }
    }
}

/// Script types for installation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScriptType {
    /// PowerShell script (.ps1)
    PowerShell,
    /// Bash script (.sh)
    Bash,
    /// Windows batch script (.cmd, .bat)
    Cmd,
}

impl ScriptType {
    /// Get the file extension for this script type
    pub fn extension(&self) -> &str {
        match self {
            Self::PowerShell => "ps1",
            Self::Bash => "sh",
            Self::Cmd => "cmd",
        }
    }

    /// Check if this script type is supported on the current platform
    pub fn is_supported(&self) -> bool {
        match self {
            Self::PowerShell | Self::Cmd => cfg!(windows),
            Self::Bash => cfg!(unix),
        }
    }
}
