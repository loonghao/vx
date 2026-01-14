//! Configuration for GNU Make provider
//!
//! GNU Make is a system tool that should be installed via system package managers.
//! This module provides configuration and detection utilities.

use vx_runtime::{Os, Platform};

/// Configuration helper for Make
pub struct MakeConfig;

impl MakeConfig {
    /// Get the executable name for the platform
    pub fn get_executable_name(platform: &Platform) -> &'static str {
        match platform.os {
            Os::Windows => "make.exe",
            _ => "make",
        }
    }

    /// Get common system paths where make might be installed
    #[allow(dead_code)]
    pub fn system_paths(platform: &Platform) -> Vec<&'static str> {
        match platform.os {
            Os::Windows => vec![
                "C:\\Program Files\\GnuWin32\\bin\\make.exe",
                "C:\\ProgramData\\chocolatey\\bin\\make.exe",
                "C:\\msys64\\usr\\bin\\make.exe",
                "C:\\cygwin64\\bin\\make.exe",
            ],
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
            Os::Windows => vec![
                ("chocolatey", "choco install make"),
                ("winget", "winget install GnuWin32.Make"),
                ("scoop", "scoop install make"),
            ],
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
    fn test_executable_name_windows() {
        let platform = Platform {
            os: Os::Windows,
            arch: Arch::X86_64,
        };
        assert_eq!(MakeConfig::get_executable_name(&platform), "make.exe");
    }

    #[test]
    fn test_executable_name_unix() {
        let platform = Platform {
            os: Os::Linux,
            arch: Arch::X86_64,
        };
        assert_eq!(MakeConfig::get_executable_name(&platform), "make");
    }

    #[test]
    fn test_system_paths_not_empty() {
        let platforms = vec![
            Platform {
                os: Os::Windows,
                arch: Arch::X86_64,
            },
            Platform {
                os: Os::MacOS,
                arch: Arch::Aarch64,
            },
            Platform {
                os: Os::Linux,
                arch: Arch::X86_64,
            },
        ];

        for platform in platforms {
            let paths = MakeConfig::system_paths(&platform);
            assert!(!paths.is_empty(), "Should have system paths for {:?}", platform.os);
        }
    }
}
