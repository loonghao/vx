//! Default configuration management
//!
//! This module handles loading and merging of default configurations,
//! including the embedded default config.toml file.

use crate::error::{ConfigError, Result};
use crate::types::VxConfig;
use crate::version_parser::{extract_version_from_output, VersionParser};

/// Embedded default configuration
const DEFAULT_CONFIG_TOML: &str = include_str!("../config.toml");

/// Load the default configuration from the embedded config.toml
pub fn load_default_config() -> Result<VxConfig> {
    toml::from_str(DEFAULT_CONFIG_TOML).map_err(|e| ConfigError::Parse {
        message: format!("Failed to parse embedded default config.toml: {}", e),
        file_type: "TOML".to_string(),
    })
}

/// Get the current platform identifier
pub fn get_current_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("windows", "x86_64") => "windows-x64".to_string(),
        ("windows", "aarch64") => "windows-aarch64".to_string(),
        ("linux", "x86_64") => "linux-x64".to_string(),
        ("linux", "aarch64") => "linux-aarch64".to_string(),
        ("macos", "x86_64") => "darwin-x64".to_string(),
        ("macos", "aarch64") => "darwin-aarch64".to_string(),
        _ => format!("{}-{}", os, arch),
    }
}

/// Get the executable name for a tool on the current platform
pub fn get_tool_executable(config: &VxConfig, tool_name: &str) -> Option<String> {
    let tool_config = config.tools.get(tool_name)?;
    let executables = tool_config.executables.as_ref()?;

    let os = std::env::consts::OS;
    executables
        .get(os)
        .cloned()
        .or_else(|| executables.get("default").cloned())
}

/// Get the download filename for a tool on the current platform
pub fn get_tool_filename(config: &VxConfig, tool_name: &str, version: &str) -> Option<String> {
    let tool_config = config.tools.get(tool_name)?;
    let platforms = tool_config.platforms.as_ref()?;

    let current_platform = get_current_platform();
    let filename_template = platforms.get(&current_platform)?;

    // Replace version placeholder
    Some(filename_template.replace("{version}", version))
}

/// Get the download URL for a tool
pub fn get_tool_download_url(config: &VxConfig, tool_name: &str, version: &str) -> Option<String> {
    let tool_config = config.tools.get(tool_name)?;
    let url_template = tool_config.download_url_template.as_ref()?;
    let filename = get_tool_filename(config, tool_name, version)?;

    // Replace placeholders in URL template
    let url = url_template
        .replace("{version}", version)
        .replace("{filename}", &filename);

    Some(url)
}

/// Get tool dependencies
pub fn get_tool_dependencies(config: &VxConfig, tool_name: &str) -> Vec<String> {
    config
        .tools
        .get(tool_name)
        .and_then(|tool_config| tool_config.depends_on.as_ref())
        .cloned()
        .unwrap_or_default()
}

/// Get execution arguments for a tool
pub fn get_tool_exec_args(config: &VxConfig, tool_name: &str) -> Vec<String> {
    let tool_config = match config.tools.get(tool_name) {
        Some(config) => config,
        None => return vec![],
    };

    let exec_args = match tool_config.exec_args.as_ref() {
        Some(args) => args,
        None => return vec![],
    };

    // Try to get platform-specific args first, then default
    let os = std::env::consts::OS;
    exec_args
        .get(os)
        .or_else(|| exec_args.get("default"))
        .cloned()
        .unwrap_or_default()
}

/// Merge user configuration with default configuration
pub fn merge_configs(default: VxConfig, user: VxConfig) -> VxConfig {
    let mut merged = default;

    // Merge global settings
    if user.global.home_dir != merged.global.home_dir {
        merged.global.home_dir = user.global.home_dir;
    }
    if user.global.tools_dir != merged.global.tools_dir {
        merged.global.tools_dir = user.global.tools_dir;
    }
    if user.global.cache_dir != merged.global.cache_dir {
        merged.global.cache_dir = user.global.cache_dir;
    }
    if user.global.shims_dir != merged.global.shims_dir {
        merged.global.shims_dir = user.global.shims_dir;
    }
    if user.global.config_dir != merged.global.config_dir {
        merged.global.config_dir = user.global.config_dir;
    }

    // Merge defaults
    merged.defaults.auto_install = user.defaults.auto_install;
    merged.defaults.cache_duration = user.defaults.cache_duration;
    merged.defaults.fallback_to_builtin = user.defaults.fallback_to_builtin;
    if user.defaults.install_dir.is_some() {
        merged.defaults.install_dir = user.defaults.install_dir;
    }
    merged.defaults.use_system_path = user.defaults.use_system_path;
    merged.defaults.download_timeout = user.defaults.download_timeout;
    merged.defaults.max_concurrent_downloads = user.defaults.max_concurrent_downloads;

    // Merge turbo-cdn settings
    merged.turbo_cdn.enabled = user.turbo_cdn.enabled;
    merged.turbo_cdn.default_region = user.turbo_cdn.default_region;
    merged.turbo_cdn.max_concurrent_chunks = user.turbo_cdn.max_concurrent_chunks;
    merged.turbo_cdn.chunk_size = user.turbo_cdn.chunk_size;
    merged.turbo_cdn.max_retries = user.turbo_cdn.max_retries;
    merged.turbo_cdn.cache_enabled = user.turbo_cdn.cache_enabled;
    merged.turbo_cdn.cache_max_size = user.turbo_cdn.cache_max_size;
    merged.turbo_cdn.cache_compression = user.turbo_cdn.cache_compression;

    // Merge tool configurations (user overrides default)
    for (tool_name, tool_config) in user.tools {
        merged.tools.insert(tool_name, tool_config);
    }

    // Merge registry configurations
    for (registry_name, registry_config) in user.registries {
        merged.registries.insert(registry_name, registry_config);
    }

    // Merge platform mappings
    for (platform, mapping) in user.platform_mappings {
        merged.platform_mappings.insert(platform, mapping);
    }

    merged
}

/// Parse version from tool output using configuration
pub fn parse_tool_version(config: &VxConfig, tool_name: &str, output: &str) -> Option<String> {
    if let Some(tool_config) = config.tools.get(tool_name) {
        if let Some(version_parser) = VersionParser::from_tool_config(tool_config) {
            if let Ok(Some(version)) = version_parser.parse_output(output) {
                return Some(version);
            }
        }
    }

    // Fallback to generic extraction
    extract_version_from_output(output, tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_config() {
        let config = load_default_config().expect("Failed to load default config");
        assert!(!config.tools.is_empty());
        assert!(config.tools.contains_key("bun"));
        assert!(config.tools.contains_key("node"));
    }

    #[test]
    fn test_get_current_platform() {
        let platform = get_current_platform();
        assert!(!platform.is_empty());
        // Should be one of the expected formats
        assert!(platform.contains('-'));
    }

    #[test]
    fn test_get_tool_executable() {
        let config = load_default_config().expect("Failed to load default config");
        let bun_exe = get_tool_executable(&config, "bun");
        assert!(bun_exe.is_some());

        if cfg!(target_os = "windows") {
            assert_eq!(bun_exe.unwrap(), "bun.exe");
        } else {
            assert_eq!(bun_exe.unwrap(), "bun");
        }
    }

    #[test]
    fn test_get_tool_dependencies() {
        let config = load_default_config().expect("Failed to load default config");
        let bunx_deps = get_tool_dependencies(&config, "bunx");
        assert_eq!(bunx_deps, vec!["bun"]);
    }
}
