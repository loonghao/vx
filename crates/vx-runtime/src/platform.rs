//! Platform detection and information

use serde::{Deserialize, Serialize};

/// Operating system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Os {
    Windows,
    MacOS,
    Linux,
    FreeBSD,
    Unknown,
}

impl Os {
    /// Detect current OS
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOS
        } else if cfg!(target_os = "linux") {
            Os::Linux
        } else if cfg!(target_os = "freebsd") {
            Os::FreeBSD
        } else {
            Os::Unknown
        }
    }

    /// Get OS name for download URLs
    pub fn as_str(&self) -> &str {
        match self {
            Os::Windows => "windows",
            Os::MacOS => "darwin",
            Os::Linux => "linux",
            Os::FreeBSD => "freebsd",
            Os::Unknown => "unknown",
        }
    }

    /// Get executable extension
    pub fn exe_extension(&self) -> &str {
        match self {
            Os::Windows => ".exe",
            _ => "",
        }
    }
}

/// CPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arch {
    X86_64,
    Aarch64,
    Arm,
    X86,
    Unknown,
}

impl Arch {
    /// Detect current architecture
    pub fn current() -> Self {
        if cfg!(target_arch = "x86_64") {
            Arch::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Aarch64
        } else if cfg!(target_arch = "arm") {
            Arch::Arm
        } else if cfg!(target_arch = "x86") {
            Arch::X86
        } else {
            Arch::Unknown
        }
    }

    /// Get architecture name for download URLs
    pub fn as_str(&self) -> &str {
        match self {
            Arch::X86_64 => "x64",
            Arch::Aarch64 => "arm64",
            Arch::Arm => "arm",
            Arch::X86 => "x86",
            Arch::Unknown => "unknown",
        }
    }
}

/// Platform information (OS + Architecture)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    /// Create a new platform
    pub fn new(os: Os, arch: Arch) -> Self {
        Self { os, arch }
    }

    /// Detect current platform
    pub fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
        }
    }

    /// Get platform string for download URLs (e.g., "linux-x64", "darwin-arm64")
    pub fn as_str(&self) -> String {
        format!("{}-{}", self.os.as_str(), self.arch.as_str())
    }

    /// Check if this is a Windows platform
    pub fn is_windows(&self) -> bool {
        self.os == Os::Windows
    }

    /// Check if this is a macOS platform
    pub fn is_macos(&self) -> bool {
        self.os == Os::MacOS
    }

    /// Check if this is a Linux platform
    pub fn is_linux(&self) -> bool {
        self.os == Os::Linux
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::current()
    }
}
