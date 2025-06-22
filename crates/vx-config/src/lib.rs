//! # VX Configuration Management
//!
//! This crate provides comprehensive configuration management for the vx tool manager.
//! It supports layered configuration from multiple sources and automatic project detection.
//!
//! ## Features
//!
//! - **Layered Configuration**: Supports builtin defaults, user config, project config, and environment variables
//! - **Project Detection**: Automatically detects Python, Rust, Node.js, and Go projects
//! - **Multiple Formats**: Supports TOML, JSON, and other configuration formats
//! - **Tool Version Management**: Manages tool versions across different project types
//!
//! ## Example
//!
//! ```rust
//! use vx_config::ConfigManager;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config_manager = ConfigManager::new().await?;
//! let tool_version = config_manager.get_tool_version("node");
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod defaults;
pub mod detection;
pub mod error;
pub mod manager;
pub mod parsers;
pub mod types;
pub mod version_parser;

// Re-export main types and functions
pub use config::*;
pub use defaults::*;
pub use error::{ConfigError, Result};
pub use manager::ConfigManager;
pub use types::*;
pub use version_parser::{extract_version_from_output, parse_github_tag, VersionParser};

/// Current version of the vx-config crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Global static configuration instance for sync access
static GLOBAL_CONFIG: std::sync::OnceLock<VxConfig> = std::sync::OnceLock::new();

/// Get global configuration instance (sync access)
/// This loads the default configuration on first access
pub fn get_global_config() -> &'static VxConfig {
    GLOBAL_CONFIG.get_or_init(|| {
        // Load minimal config with defaults only to avoid async issues
        match ConfigManager::minimal() {
            Ok(manager) => manager.config().clone(),
            Err(_) => VxConfig::default(),
        }
    })
}

/// Initialize global configuration with custom config (for testing)
#[allow(clippy::result_large_err)]
pub fn init_global_config(config: VxConfig) -> std::result::Result<(), VxConfig> {
    GLOBAL_CONFIG.set(config)
}

/// Reset global configuration (for testing)
#[cfg(test)]
pub fn reset_global_config() {
    // This is only available in tests since OnceLock doesn't have a reset method
    // In tests, we can use a different approach if needed
}

/// Get fetcher URL for a specific tool
pub fn get_tool_fetcher_url(config: &VxConfig, tool_name: &str) -> Option<String> {
    config.tools.get(tool_name)?.fetcher_url.clone()
}

/// Get tool dependencies from configuration
pub fn get_tool_dependencies(config: &VxConfig, tool_name: &str) -> Vec<String> {
    if let Some(tool_config) = config.tools.get(tool_name) {
        tool_config.depends_on.clone().unwrap_or_default()
    } else {
        vec![]
    }
}

/// Get tool metadata from global configuration (sync access)
pub fn get_tool_metadata_sync(tool_name: &str) -> std::collections::HashMap<String, String> {
    get_tool_metadata(get_global_config(), tool_name)
}

/// Get tool metadata from configuration
pub fn get_tool_metadata(
    config: &VxConfig,
    tool_name: &str,
) -> std::collections::HashMap<String, String> {
    let mut metadata = std::collections::HashMap::new();

    if let Some(tool_config) = config.tools.get(tool_name) {
        // Add basic metadata from config
        if let Some(description) = &tool_config.description {
            metadata.insert("description".to_string(), description.clone());
        }
        if let Some(homepage) = &tool_config.homepage {
            metadata.insert("homepage".to_string(), homepage.clone());
        }
        if let Some(repository) = &tool_config.repository {
            metadata.insert(
                "repository".to_string(),
                format!("https://github.com/{}", repository),
            );
        }

        // Add ecosystem based on tool name or explicit config
        let ecosystem = match tool_name {
            "node" | "npm" | "npx" | "yarn" | "pnpm" | "bun" => "javascript",
            "python" | "pip" | "uv" | "uvx" => "python",
            "go" | "gofmt" | "goimports" => "go",
            "cargo" | "rustc" | "rustfmt" | "clippy" => "rust",
            _ => "general",
        };
        metadata.insert("ecosystem".to_string(), ecosystem.to_string());

        // Add license if available (could be added to config later)
        match tool_name {
            "uv" | "cargo" | "rustc" => {
                metadata.insert("license".to_string(), "MIT OR Apache-2.0".to_string());
            }
            "node" | "npm" => {
                metadata.insert("license".to_string(), "MIT".to_string());
            }
            _ => {}
        }
    }

    metadata
}

/// Get tool download URL from global configuration (sync access)
pub fn get_tool_download_url_sync(tool_name: &str, version: &str) -> Option<String> {
    get_tool_download_url(get_global_config(), tool_name, version)
}

/// Get tool filename from global configuration (sync access)
pub fn get_tool_filename_sync(tool_name: &str, version: &str) -> Option<String> {
    get_tool_filename(get_global_config(), tool_name, version)
}

/// Get platform string for downloads (sync access)
pub fn get_platform_string() -> String {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-pc-windows-msvc".to_string()
        } else if cfg!(target_arch = "x86") {
            "i686-pc-windows-msvc".to_string()
        } else {
            "unknown-pc-windows-msvc".to_string()
        }
    } else if cfg!(target_os = "macos") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-apple-darwin".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "aarch64-apple-darwin".to_string()
        } else {
            "unknown-apple-darwin".to_string()
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            "x86_64-unknown-linux-gnu".to_string()
        } else if cfg!(target_arch = "aarch64") {
            "aarch64-unknown-linux-gnu".to_string()
        } else {
            "unknown-unknown-linux-gnu".to_string()
        }
    } else {
        "unknown".to_string()
    }
}

/// Get manual instructions from global configuration (sync access)
pub fn get_tool_manual_instructions_sync(tool_name: &str) -> String {
    // For now, just return fallback instructions since manual_instructions field doesn't exist
    // TODO: Add manual_instructions field to ToolConfig if needed

    // Fallback instructions based on tool name
    match tool_name {
        "uv" => {
            "Visit https://docs.astral.sh/uv/getting-started/installation/ to install UV manually"
                .to_string()
        }
        "node" => "Visit https://nodejs.org/en/download/ to install Node.js manually".to_string(),
        "go" => "Visit https://golang.org/doc/install to install Go manually".to_string(),
        "rust" => "Visit https://rustup.rs/ to install Rust manually".to_string(),
        _ => format!(
            "Please install {} manually from its official website",
            tool_name
        ),
    }
}

/// Get tool download URL from configuration
pub fn get_tool_download_url(config: &VxConfig, tool_name: &str, version: &str) -> Option<String> {
    if let Some(tool_config) = config.tools.get(tool_name) {
        if let Some(url_template) = &tool_config.download_url_template {
            // Get platform-specific filename
            let filename = get_tool_filename(config, tool_name, version)?;

            // Replace placeholders in URL template
            let url = url_template
                .replace("{version}", version)
                .replace("{filename}", &filename);

            return Some(url);
        }
    }
    None
}

/// Get tool filename from configuration
pub fn get_tool_filename(config: &VxConfig, tool_name: &str, version: &str) -> Option<String> {
    if let Some(tool_config) = config.tools.get(tool_name) {
        if let Some(platforms) = &tool_config.platforms {
            let platform_key = get_current_platform_key();
            if let Some(filename_template) = platforms.get(&platform_key) {
                // Replace {version} placeholder in filename
                let filename = filename_template.replace("{version}", version);
                return Some(filename);
            }
        }
    }
    None
}

/// Create a standard InstallConfig for any tool (sync access)
pub fn create_standard_install_config(
    tool_name: &str,
    version: &str,
    install_dir: std::path::PathBuf,
) -> Option<vx_installer::InstallConfig> {
    use vx_installer::{ArchiveFormat, InstallConfig, InstallMethod, LifecycleHooks};

    let config = get_global_config();

    // Get download URL from config
    let download_url = get_tool_download_url(config, tool_name, version)?;

    // Determine archive format based on platform
    let format = if cfg!(windows) {
        ArchiveFormat::Zip
    } else {
        ArchiveFormat::TarGz
    };

    let install_method = InstallMethod::Archive { format };

    // Create basic lifecycle hooks
    let hooks = LifecycleHooks::default();

    Some(
        InstallConfig::builder()
            .tool_name(tool_name)
            .version(version)
            .install_method(install_method)
            .download_url(download_url)
            .install_dir(install_dir)
            .lifecycle_hooks(hooks)
            .build(),
    )
}

/// Get tool executable name for current platform (sync access)
pub fn get_tool_executable_name(tool_name: &str) -> String {
    let config = get_global_config();

    // Check if tool has custom executable configuration
    if let Some(tool_config) = config.tools.get(tool_name) {
        if let Some(executables) = &tool_config.executables {
            let platform_key = get_current_platform_key();
            if let Some(exe_name) = executables.get(&platform_key) {
                return exe_name.clone();
            }
        }
    }

    // Fallback to standard naming
    if cfg!(windows) {
        format!("{}.exe", tool_name)
    } else {
        tool_name.to_string()
    }
}

/// Get current platform key for configuration lookup
fn get_current_platform_key() -> String {
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };

    if cfg!(target_os = "windows") {
        format!("windows-{}", arch)
    } else if cfg!(target_os = "macos") {
        format!("darwin-{}", arch)
    } else if cfg!(target_os = "linux") {
        format!("linux-{}", arch)
    } else {
        format!("unknown-{}", arch)
    }
}
