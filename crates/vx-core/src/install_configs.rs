use crate::installer::{ArchiveFormat, InstallConfig, InstallMethod};
use crate::url_builder::{NodeUrlBuilder, GoUrlBuilder, GenericUrlBuilder};
use crate::platform::Platform;
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
    let platform = Platform::current();

    // Use GitHub releases pattern for UV
    let (filename, format) = match platform.os {
        crate::platform::OperatingSystem::Windows => {
            ("uv-x86_64-pc-windows-msvc.zip", ArchiveFormat::Zip)
        }
        crate::platform::OperatingSystem::MacOS => {
            ("uv-x86_64-apple-darwin.tar.gz", ArchiveFormat::TarGz)
        }
        _ => {
            ("uv-x86_64-unknown-linux-gnu.tar.gz", ArchiveFormat::TarGz)
        }
    };

    let download_url = if version == "latest" {
        Some(GenericUrlBuilder::github_release_download_url(
            "astral-sh", "uv", "latest", filename
        ))
    } else {
        Some(GenericUrlBuilder::github_release_download_url(
            "astral-sh", "uv", version, filename
        ))
    };

    let install_method = InstallMethod::Archive { format };

    InstallConfig {
        tool_name: "uv".to_string(),
        version: version.to_string(),
        install_method,
        download_url,
        install_dir,
        force: false,
        checksum: None,
        metadata: std::collections::HashMap::new(),
    }
}

/// Node.js installation configuration
fn get_node_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "20.11.0" // Default to LTS version
    } else {
        version
    };

    // Use NodeUrlBuilder for consistent URL generation
    let download_url = NodeUrlBuilder::download_url(actual_version);
    let install_method = InstallMethod::Archive {
        format: if cfg!(windows) {
            ArchiveFormat::Zip
        } else {
            ArchiveFormat::TarGz
        },
    };

    InstallConfig {
        tool_name: "node".to_string(),
        version: version.to_string(),
        install_method,
        download_url,
        install_dir,
        force: false,
        checksum: None,
        metadata: std::collections::HashMap::new(),
    }
}

/// Go installation configuration
fn get_go_config(version: &str, install_dir: PathBuf) -> InstallConfig {
    let actual_version = if version == "latest" {
        "1.21.6" // Default to stable version
    } else {
        version
    };

    // Use GoUrlBuilder for consistent URL generation
    let download_url = GoUrlBuilder::download_url(actual_version);
    let install_method = InstallMethod::Archive {
        format: if cfg!(windows) {
            ArchiveFormat::Zip
        } else {
            ArchiveFormat::TarGz
        },
    };

    InstallConfig {
        tool_name: "go".to_string(),
        version: version.to_string(),
        install_method,
        download_url,
        install_dir,
        force: false,
        checksum: None,
        metadata: std::collections::HashMap::new(),
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
        force: false,
        checksum: None,
        metadata: std::collections::HashMap::new(),
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
