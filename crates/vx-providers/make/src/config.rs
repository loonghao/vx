//! Configuration for GNU Make provider
//!
//! GNU Make is a system tool that should be installed via system package managers.
//! This module provides configuration and detection utilities.

use vx_runtime::{Os, Platform};

/// Configuration helper for Make
pub struct MakeConfig;

impl MakeConfig {
    /// Get common system paths where make might be installed
    #[allow(dead_code)]
    pub fn system_paths(platform: &Platform) -> Vec<&'static str> {
        match platform.os {
            Os::MacOS => vec![
                "/usr/bin/make",
                "/usr/local/bin/make",
                "/opt/homebrew/bin/gmake",
                "/opt/homebrew/bin/make",
            ],
            Os::Linux => vec!["/usr/bin/make", "/usr/local/bin/make"],
            _ => vec![],
        }
    }

    /// Get package manager install commands for the platform
    #[allow(dead_code)]
    pub fn install_commands(platform: &Platform) -> Vec<(&'static str, &'static str)> {
        match platform.os {
            Os::MacOS => vec![("homebrew", "brew install make")],
            Os::Linux => vec![
                ("apt", "sudo apt install make"),
                ("dnf", "sudo dnf install make"),
                ("pacman", "sudo pacman -S make"),
            ],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vx_runtime::Arch;

    #[test]
    fn test_system_paths_unix() {
        let platforms = vec![
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
        ];

        for platform in platforms {
            let paths = MakeConfig::system_paths(&platform);
            assert!(
                !paths.is_empty(),
                "Should have system paths for {:?}",
                platform.os
            );
        }
    }

    #[test]
    fn test_windows_not_supported() {
        let platform = Platform::new(Os::Windows, Arch::X86_64);
        let paths = MakeConfig::system_paths(&platform);
        assert!(paths.is_empty(), "Windows should have no system paths");
    }
}
