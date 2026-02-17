//! Platform detection and information

use serde::{Deserialize, Serialize};

/// C library implementation (primarily for Linux)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Libc {
    /// GNU libc (glibc) - default on most Linux distributions
    #[default]
    Gnu,
    /// musl libc - used in Alpine Linux and other minimal distributions
    Musl,
}

impl Libc {
    /// Detect the current libc implementation at runtime
    pub fn current() -> Self {
        // First check if we're in a musl environment via environment variable
        if std::env::var("VX_LIBC").ok().as_deref() == Some("musl") {
            return Libc::Musl;
        }

        // Only relevant for Linux
        if !cfg!(target_os = "linux") {
            return Libc::Gnu;
        }

        // Check /etc/os-release for Alpine (uses musl)
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let content_lower = content.to_lowercase();
            if content_lower.contains("alpine") {
                return Libc::Musl;
            }
        }

        // Check if /lib/ld-musl-*.so.1 exists (musl dynamic linker)
        if std::path::Path::new("/lib/ld-musl-x86_64.so.1").exists()
            || std::path::Path::new("/lib/ld-musl-aarch64.so.1").exists()
        {
            return Libc::Musl;
        }

        // Default to GNU libc
        Libc::Gnu
    }

    /// Get the libc suffix for Rust target triples
    pub fn as_str(&self) -> &'static str {
        match self {
            Libc::Gnu => "gnu",
            Libc::Musl => "musl",
        }
    }

    /// Check if this is musl libc
    pub fn is_musl(&self) -> bool {
        matches!(self, Libc::Musl)
    }

    /// Check if this is GNU libc
    pub fn is_gnu(&self) -> bool {
        matches!(self, Libc::Gnu)
    }
}

impl std::fmt::Display for Libc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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
    pub fn as_str(&self) -> &'static str {
        match self {
            Os::Windows => "windows",
            Os::MacOS => "darwin",
            Os::Linux => "linux",
            Os::FreeBSD => "freebsd",
            Os::Unknown => "unknown",
        }
    }

    /// Get executable extension
    pub fn exe_extension(&self) -> &'static str {
        match self {
            Os::Windows => ".exe",
            _ => "",
        }
    }
}

impl std::fmt::Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// CPU architecture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arch {
    X86_64,
    Aarch64,
    Arm,
    Armv7,
    X86,
    PowerPC64,
    PowerPC64LE,
    S390x,
    Riscv64,
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
        } else if cfg!(target_arch = "powerpc64") {
            if cfg!(target_endian = "little") {
                Arch::PowerPC64LE
            } else {
                Arch::PowerPC64
            }
        } else if cfg!(target_arch = "s390x") {
            Arch::S390x
        } else if cfg!(target_arch = "riscv64") {
            Arch::Riscv64
        } else {
            Arch::Unknown
        }
    }

    /// Get architecture name for download URLs
    pub fn as_str(&self) -> &'static str {
        match self {
            Arch::X86_64 => "x64",
            Arch::Aarch64 => "arm64",
            Arch::Arm => "arm",
            Arch::Armv7 => "armv7",
            Arch::X86 => "x86",
            Arch::PowerPC64 => "ppc64",
            Arch::PowerPC64LE => "ppc64le",
            Arch::S390x => "s390x",
            Arch::Riscv64 => "riscv64",
            Arch::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Platform information (OS + Architecture + Libc)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
    /// C library implementation (relevant for Linux)
    #[serde(default)]
    pub libc: Libc,
}

impl Platform {
    /// Create a new platform with default libc (GNU)
    pub fn new(os: Os, arch: Arch) -> Self {
        Self {
            os,
            arch,
            libc: Libc::default(),
        }
    }

    /// Create a new platform with specific libc
    pub fn with_libc(os: Os, arch: Arch, libc: Libc) -> Self {
        Self { os, arch, libc }
    }

    /// Detect current platform including libc
    pub fn current() -> Self {
        Self {
            os: Os::current(),
            arch: Arch::current(),
            libc: Libc::current(),
        }
    }

    /// Check if this platform uses musl libc
    pub fn is_musl(&self) -> bool {
        self.os == Os::Linux && self.libc.is_musl()
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

    /// Get OS name as a string (for platform filtering)
    pub fn os_name(&self) -> &'static str {
        match self.os {
            Os::Windows => "windows",
            Os::MacOS => "macos",
            Os::Linux => "linux",
            Os::FreeBSD => "freebsd",
            Os::Unknown => "unknown",
        }
    }

    /// Get executable name with platform-appropriate extension
    pub fn exe_name(&self, base: &str) -> String {
        if self.os == Os::Windows {
            format!("{}.exe", base)
        } else {
            base.to_string()
        }
    }

    /// Get executable name with custom extension options for Windows
    pub fn executable_with_extensions(&self, base: &str, extensions: &[&str]) -> String {
        if self.os == Os::Windows {
            let ext = extensions.first().copied().unwrap_or(".exe");
            format!("{}{}", base, ext)
        } else {
            base.to_string()
        }
    }

    /// Get all possible executable names for this platform
    pub fn all_executable_names(&self, base: &str, extensions: &[&str]) -> Vec<String> {
        if self.os == Os::Windows {
            let mut names: Vec<String> = extensions
                .iter()
                .map(|ext| format!("{}{}", base, ext))
                .collect();
            names.push(base.to_string());
            names
        } else {
            vec![base.to_string()]
        }
    }

    /// Get Rust target triple for this platform
    pub fn rust_target_triple(&self) -> &'static str {
        match (&self.os, &self.arch, &self.libc) {
            // Windows
            (Os::Windows, Arch::X86_64, _) => "x86_64-pc-windows-msvc",
            (Os::Windows, Arch::X86, _) => "i686-pc-windows-msvc",
            (Os::Windows, Arch::Aarch64, _) => "aarch64-pc-windows-msvc",
            // macOS
            (Os::MacOS, Arch::X86_64, _) => "x86_64-apple-darwin",
            (Os::MacOS, Arch::Aarch64, _) => "aarch64-apple-darwin",
            // Linux with musl
            (Os::Linux, Arch::X86_64, Libc::Musl) => "x86_64-unknown-linux-musl",
            (Os::Linux, Arch::Aarch64, Libc::Musl) => "aarch64-unknown-linux-musl",
            (Os::Linux, Arch::X86, Libc::Musl) => "i686-unknown-linux-musl",
            (Os::Linux, Arch::Arm, Libc::Musl) => "arm-unknown-linux-musleabihf",
            (Os::Linux, Arch::Armv7, Libc::Musl) => "armv7-unknown-linux-musleabihf",
            // Linux with glibc
            (Os::Linux, Arch::X86_64, Libc::Gnu) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Aarch64, Libc::Gnu) => "aarch64-unknown-linux-gnu",
            (Os::Linux, Arch::X86, Libc::Gnu) => "i686-unknown-linux-gnu",
            (Os::Linux, Arch::Arm, Libc::Gnu) => "arm-unknown-linux-gnueabihf",
            (Os::Linux, Arch::Armv7, Libc::Gnu) => "armv7-unknown-linux-gnueabihf",
            (Os::Linux, Arch::PowerPC64, Libc::Gnu) => "powerpc64-unknown-linux-gnu",
            (Os::Linux, Arch::PowerPC64LE, Libc::Gnu) => "powerpc64le-unknown-linux-gnu",
            (Os::Linux, Arch::S390x, Libc::Gnu) => "s390x-unknown-linux-gnu",
            (Os::Linux, Arch::Riscv64, Libc::Gnu) => "riscv64gc-unknown-linux-gnu",
            // Default fallback
            _ => "x86_64-unknown-linux-gnu",
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::current()
    }
}

impl Platform {
    /// Returns all commonly supported platforms
    pub fn all_common() -> Vec<Platform> {
        vec![
            Platform::new(Os::Windows, Arch::X86_64),
            Platform::new(Os::Windows, Arch::Aarch64),
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
        ]
    }

    /// Returns all Windows platforms
    pub fn windows_only() -> Vec<Platform> {
        vec![
            Platform::new(Os::Windows, Arch::X86_64),
            Platform::new(Os::Windows, Arch::Aarch64),
            Platform::new(Os::Windows, Arch::X86),
        ]
    }

    /// Returns all Unix-like platforms (macOS + Linux)
    pub fn unix_only() -> Vec<Platform> {
        vec![
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
        ]
    }

    /// Returns all Linux platforms
    pub fn linux_only() -> Vec<Platform> {
        vec![
            Platform::new(Os::Linux, Arch::X86_64),
            Platform::new(Os::Linux, Arch::Aarch64),
        ]
    }

    /// Returns all macOS platforms
    pub fn macos_only() -> Vec<Platform> {
        vec![
            Platform::new(Os::MacOS, Arch::X86_64),
            Platform::new(Os::MacOS, Arch::Aarch64),
        ]
    }

    /// Check if this platform matches another
    pub fn matches(&self, other: &Platform) -> bool {
        self.os == other.os && self.arch == other.arch
    }
}

/// Simple semver comparison for sorting versions
pub fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |v: &str| -> Vec<u64> {
        v.split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse::<u64>().ok())
            .collect()
    };

    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        match a_part.cmp(b_part) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    a_parts.len().cmp(&b_parts.len())
}
