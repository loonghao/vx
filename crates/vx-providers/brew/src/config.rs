//! Homebrew configuration and URL building

use vx_runtime::Platform;

/// Homebrew configuration and URL builder
pub struct BrewConfig;

impl BrewConfig {
    /// Get the installation script URL
    pub fn install_script_url() -> &'static str {
        "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh"
    }

    /// Get the common search paths for brew executable
    pub fn search_paths(platform: &Platform) -> Vec<&'static str> {
        match platform.os.as_str() {
            "macos" => vec![
                "/opt/homebrew/bin", // Apple Silicon
                "/usr/local/bin",    // Intel
            ],
            "linux" => vec!["/home/linuxbrew/.linuxbrew/bin", "/usr/local/bin"],
            _ => vec![],
        }
    }

    /// Get the brew executable name
    pub fn executable_name() -> &'static str {
        "brew"
    }

    /// Get the shell environment setup command for the platform
    pub fn shell_env_command(platform: &Platform) -> Option<String> {
        match platform.os.as_str() {
            "macos" => {
                // Try Apple Silicon first, then Intel
                Some(r#"eval "$(/opt/homebrew/bin/brew shellenv 2>/dev/null || /usr/local/bin/brew shellenv)""#.to_string())
            }
            "linux" => {
                Some(r#"eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)""#.to_string())
            }
            _ => None,
        }
    }
}
