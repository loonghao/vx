use crate::installer::{ArchiveFormat, InstallConfig, InstallMethod};
use std::path::PathBuf;

/// Get installation configuration for a specific tool
pub fn get_install_config(tool_name: &str, version: &str) -> Option<InstallConfig> {
    let install_dir = get_tool_install_dir(tool_name);

    match tool_name {
        "uv" => Some(get_uv_config(version, install_dir)),
        "node" => Some(get_node_config(version, install_dir)),
        "go" => Some(get_go_config(version, install_dir)),
        "rust" | "cargo" => Some(get_rust_config(version, install_dir)),
        _ => None,
    }
}

/// Get tool installation directory
fn get_tool_install_dir(tool_name: &str) -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home_dir.join(".vx").join("tools").join(tool_name)
}

/// UV installation configuration
fn get_uv_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let (download_url, install_method) = if cfg!(windows) {
        // Windows: Download from GitHub releases
        let url = if version == "latest" {
            "https://github.com/astral-sh/uv/releases/latest/download/uv-x86_64-pc-windows-msvc.zip"
                .to_string()
        } else {
            format!("https://github.com/astral-sh/uv/releases/download/{version}/uv-x86_64-pc-windows-msvc.zip")
        };
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::Zip,
            },
        )
    } else if cfg!(target_os = "macos") {
        // macOS: Download binary for better compatibility
        let url = if version == "latest" {
            "https://github.com/astral-sh/uv/releases/latest/download/uv-x86_64-apple-darwin.tar.gz"
                .to_string()
        } else {
            format!("https://github.com/astral-sh/uv/releases/download/{version}/uv-x86_64-apple-darwin.tar.gz")
        };
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::TarGz,
            },
        )
    } else {
        // Linux: Download binary for better compatibility
        let url = if version == "latest" {
            "https://github.com/astral-sh/uv/releases/latest/download/uv-x86_64-unknown-linux-gnu.tar.gz"
                .to_string()
        } else {
            format!("https://github.com/astral-sh/uv/releases/download/{version}/uv-x86_64-unknown-linux-gnu.tar.gz")
        };
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::TarGz,
            },
        )
    };

    InstallConfig {
        tool_name: "uv".to_string(),
        version: version.to_string(),
        install_method,
        download_url,
        install_dir,
    }
}

/// Node.js installation configuration
fn get_node_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let (download_url, install_method) = if cfg!(windows) {
        // Windows: Download ZIP from nodejs.org
        let version_str = if version == "latest" {
            "v20.11.0"
        } else {
            version
        };
        let url = format!("https://nodejs.org/dist/{version_str}/node-{version_str}-win-x64.zip");
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::Zip,
            },
        )
    } else if cfg!(target_os = "macos") {
        // macOS: Download binary for better compatibility
        let version_str = if version == "latest" {
            "v20.11.0"
        } else {
            version
        };
        let url =
            format!("https://nodejs.org/dist/{version_str}/node-{version_str}-darwin-x64.tar.gz");
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::TarGz,
            },
        )
    } else {
        // Linux: Download tar.gz
        let version_str = if version == "latest" {
            "v20.11.0"
        } else {
            version
        };
        let url =
            format!("https://nodejs.org/dist/{version_str}/node-{version_str}-linux-x64.tar.gz");
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::TarGz,
            },
        )
    };

    InstallConfig {
        tool_name: "node".to_string(),
        version: version.to_string(),
        install_method,
        download_url,
        install_dir,
    }
}

/// Go installation configuration
fn get_go_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let (download_url, install_method) = if cfg!(windows) {
        // Windows: Download ZIP from golang.org
        let version_str = if version == "latest" {
            "1.21.6"
        } else {
            version
        };
        let url = format!("https://golang.org/dl/go{version_str}.windows-amd64.zip");
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::Zip,
            },
        )
    } else if cfg!(target_os = "macos") {
        // macOS: Download binary for better compatibility
        let version_str = if version == "latest" {
            "1.21.6"
        } else {
            version
        };
        let url = format!("https://golang.org/dl/go{version_str}.darwin-amd64.tar.gz");
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::TarGz,
            },
        )
    } else {
        // Linux: Download tar.gz
        let version_str = if version == "latest" {
            "1.21.6"
        } else {
            version
        };
        let url = format!("https://golang.org/dl/go{version_str}.linux-amd64.tar.gz");
        (
            Some(url),
            InstallMethod::Archive {
                format: ArchiveFormat::TarGz,
            },
        )
    };

    InstallConfig {
        tool_name: "go".to_string(),
        version: version.to_string(),
        install_method,
        download_url,
        install_dir,
    }
}

/// Rust installation configuration
fn get_rust_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    // Rust uses rustup for installation on all platforms
    let (download_url, install_method) = if cfg!(windows) {
        (
            Some("https://win.rustup.rs/".to_string()),
            InstallMethod::Script {
                url: "https://win.rustup.rs/".to_string(),
            },
        )
    } else {
        (
            Some("https://sh.rustup.rs".to_string()),
            InstallMethod::Script {
                url: "https://sh.rustup.rs".to_string(),
            },
        )
    };

    InstallConfig {
        tool_name: "cargo".to_string(), // Primary executable
        version: version.to_string(),
        install_method,
        download_url,
        install_dir,
    }
}

/// Get available installation methods for a tool
pub fn get_available_install_methods(tool_name: &str) -> Vec<String> {
    match tool_name {
        "uv" => vec![
            "Official installer".to_string(),
            "GitHub releases".to_string(),
            if cfg!(target_os = "macos") {
                "Homebrew".to_string()
            } else {
                "Package manager".to_string()
            },
        ],
        "node" => vec![
            "Official releases".to_string(),
            if cfg!(target_os = "macos") {
                "Homebrew".to_string()
            } else {
                "Package manager".to_string()
            },
            "Node Version Manager (nvm)".to_string(),
        ],
        "go" => vec![
            "Official releases".to_string(),
            if cfg!(target_os = "macos") {
                "Homebrew".to_string()
            } else {
                "Package manager".to_string()
            },
        ],
        "rust" | "cargo" => vec![
            "Rustup (recommended)".to_string(),
            if cfg!(target_os = "macos") {
                "Homebrew".to_string()
            } else {
                "Package manager".to_string()
            },
        ],
        _ => vec!["Manual installation required".to_string()],
    }
}

/// Check if a tool supports automatic installation
pub fn supports_auto_install(tool_name: &str) -> bool {
    matches!(tool_name, "uv" | "node" | "go" | "rust" | "cargo")
}

/// Get installation instructions for manual installation
pub fn get_manual_install_instructions(tool_name: &str) -> String {
    match tool_name {
        "uv" => "To install uv manually:\n\
             • Windows: Download from https://github.com/astral-sh/uv/releases\n\
             • macOS: brew install uv\n\
             • Linux: curl -LsSf https://astral.sh/uv/install.sh | sh"
            .to_string(),
        "node" | "npm" | "npx" => "To install Node.js manually:\n\
             • Visit: https://nodejs.org/\n\
             • Or use a version manager like nvm"
            .to_string(),
        "go" => "To install Go manually:\n\
             • Visit: https://golang.org/dl/\n\
             • Or use your system package manager"
            .to_string(),
        "rust" | "cargo" => "To install Rust manually:\n\
             • Visit: https://rustup.rs/\n\
             • Or run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            .to_string(),
        _ => {
            format!("Please install {tool_name} manually according to its official documentation.")
        }
    }
}
