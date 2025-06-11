//! Platform detection and utilities

use serde::{Deserialize, Serialize};

/// Supported operating systems
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperatingSystem {
    Windows,
    MacOS,
    Linux,
    FreeBSD,
    Other(String),
}

/// Supported CPU architectures
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    X86,
    Aarch64,
    Arm,
    Other(String),
}

/// Platform information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Platform {
    pub os: OperatingSystem,
    pub arch: Architecture,
}

impl Platform {
    /// Get the current platform
    pub fn current() -> Self {
        Self {
            os: Self::current_os(),
            arch: Self::current_arch(),
        }
    }
    
    /// Get the current operating system
    pub fn current_os() -> OperatingSystem {
        if cfg!(target_os = "windows") {
            OperatingSystem::Windows
        } else if cfg!(target_os = "macos") {
            OperatingSystem::MacOS
        } else if cfg!(target_os = "linux") {
            OperatingSystem::Linux
        } else if cfg!(target_os = "freebsd") {
            OperatingSystem::FreeBSD
        } else {
            OperatingSystem::Other(std::env::consts::OS.to_string())
        }
    }
    
    /// Get the current architecture
    pub fn current_arch() -> Architecture {
        if cfg!(target_arch = "x86_64") {
            Architecture::X86_64
        } else if cfg!(target_arch = "x86") {
            Architecture::X86
        } else if cfg!(target_arch = "aarch64") {
            Architecture::Aarch64
        } else if cfg!(target_arch = "arm") {
            Architecture::Arm
        } else {
            Architecture::Other(std::env::consts::ARCH.to_string())
        }
    }
    
    /// Get platform string for Node.js downloads
    pub fn node_platform_string(&self) -> Option<(String, String)> {
        let os = match self.os {
            OperatingSystem::Windows => "win",
            OperatingSystem::MacOS => "darwin",
            OperatingSystem::Linux => "linux",
            _ => return None,
        };
        
        let arch = match self.arch {
            Architecture::X86_64 => "x64",
            Architecture::X86 => "x86",
            Architecture::Aarch64 => "arm64",
            _ => return None,
        };
        
        Some((os.to_string(), arch.to_string()))
    }
    
    /// Get platform string for Go downloads
    pub fn go_platform_string(&self) -> Option<(String, String)> {
        let os = match self.os {
            OperatingSystem::Windows => "windows",
            OperatingSystem::MacOS => "darwin",
            OperatingSystem::Linux => "linux",
            OperatingSystem::FreeBSD => "freebsd",
            _ => return None,
        };
        
        let arch = match self.arch {
            Architecture::X86_64 => "amd64",
            Architecture::X86 => "386",
            Architecture::Aarch64 => "arm64",
            Architecture::Arm => "armv6l",
            _ => return None,
        };
        
        Some((os.to_string(), arch.to_string()))
    }
    
    /// Get file extension for archives on this platform
    pub fn archive_extension(&self) -> &'static str {
        match self.os {
            OperatingSystem::Windows => "zip",
            _ => "tar.gz",
        }
    }
    
    /// Get executable extension for this platform
    pub fn executable_extension(&self) -> &'static str {
        match self.os {
            OperatingSystem::Windows => "exe",
            _ => "",
        }
    }
}

impl std::fmt::Display for OperatingSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperatingSystem::Windows => write!(f, "windows"),
            OperatingSystem::MacOS => write!(f, "macos"),
            OperatingSystem::Linux => write!(f, "linux"),
            OperatingSystem::FreeBSD => write!(f, "freebsd"),
            OperatingSystem::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Display for Architecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::X86 => write!(f, "x86"),
            Architecture::Aarch64 => write!(f, "aarch64"),
            Architecture::Arm => write!(f, "arm"),
            Architecture::Other(s) => write!(f, "{}", s),
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.os, self.arch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_current_platform() {
        let platform = Platform::current();
        
        // Should detect some valid OS and arch
        assert!(!matches!(platform.os, OperatingSystem::Other(_)));
        assert!(!matches!(platform.arch, Architecture::Other(_)));
    }
    
    #[test]
    fn test_platform_strings() {
        let platform = Platform::current();
        
        // Should be able to generate platform strings for major tools
        if matches!(platform.os, OperatingSystem::Windows | OperatingSystem::MacOS | OperatingSystem::Linux) {
            assert!(platform.node_platform_string().is_some());
            assert!(platform.go_platform_string().is_some());
        }
    }
    
    #[test]
    fn test_extensions() {
        let platform = Platform::current();
        
        let archive_ext = platform.archive_extension();
        let exe_ext = platform.executable_extension();
        
        assert!(!archive_ext.is_empty());
        // exe_ext can be empty on Unix systems
    }
}
