//! NPM package manager configuration
//!
//! This module provides NPM-specific configuration,
//! including dependencies and installation methods.

use std::path::PathBuf;
use vx_core::{StandardToolConfig, ToolDependency};
use vx_installer::{InstallConfig, InstallMethod};

/// Standard configuration for NPM tool
pub struct Config;

/// Implementation of standard tool configuration for NPM
impl StandardToolConfig for Config {
    fn tool_name() -> &'static str {
        "npm"
    }
    
    fn create_install_config(version: &str, install_dir: PathBuf) -> InstallConfig {
        create_install_config(version, install_dir)
    }
    
    fn get_install_methods() -> Vec<String> {
        get_install_methods()
    }
    
    fn supports_auto_install() -> bool {
        false // NPM comes bundled with Node.js
    }
    
    fn get_manual_instructions() -> String {
        get_manual_instructions()
    }
    
    fn get_dependencies() -> Vec<ToolDependency> {
        vec![
            ToolDependency::required("node", "NPM is bundled with Node.js")
                .with_version(">=16.0.0")
        ]
    }
    
    fn get_default_version() -> &'static str {
        "latest" // Use whatever comes with Node.js
    }
}

/// Create NPM installation configuration
/// Note: NPM is typically bundled with Node.js, so this creates a "virtual" config
pub fn create_install_config(_version: &str, install_dir: PathBuf) -> InstallConfig {
    InstallConfig::builder()
        .tool_name("npm")
        .version("bundled".to_string())
        .install_method(InstallMethod::Custom { 
            method: "bundled-with-node".to_string() 
        })
        .install_dir(install_dir)
        .build()
}

/// Get available NPM installation methods
pub fn get_install_methods() -> Vec<String> {
    vec![
        "Bundled with Node.js (recommended)".to_string(),
        "Standalone installation".to_string(),
    ]
}

/// Check if NPM supports automatic installation
pub fn supports_auto_install() -> bool {
    false // NPM comes with Node.js
}

/// Get manual installation instructions for NPM
pub fn get_manual_instructions() -> String {
    "NPM is bundled with Node.js:\n\
     • Install Node.js to get NPM automatically\n\
     • Or install NPM standalone: npm install -g npm@latest"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_dependencies() {
        let deps = Config::get_dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].tool_name, "node");
        assert!(deps[0].required);
        assert_eq!(deps[0].version_requirement, Some(">=16.0.0".to_string()));
    }

    #[test]
    fn test_npm_config() {
        assert_eq!(Config::tool_name(), "npm");
        assert!(!Config::supports_auto_install());
        assert_eq!(Config::get_default_version(), "latest");
    }

    #[test]
    fn test_create_install_config() {
        let config = create_install_config("latest", PathBuf::from("/tmp/npm"));
        assert_eq!(config.tool_name, "npm");
        assert_eq!(config.version, "bundled");
    }
}
