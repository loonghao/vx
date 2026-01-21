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
    ///
    /// This checks if running on a musl-based system by examining /etc/os-release
    /// or checking for musl-specific indicators.
    pub fn current() -> Self {
        // First check if we're in a musl environment via environment variable
        // This allows explicit override in containers
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
    pub fn as_str(&self) -> &str {
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
    ///
    /// Returns "windows", "macos", "linux", etc.
    pub fn os_name(&self) -> &str {
        match self.os {
            Os::Windows => "windows",
            Os::MacOS => "macos",
            Os::Linux => "linux",
            Os::FreeBSD => "freebsd",
            Os::Unknown => "unknown",
        }
    }

    /// Get executable name with platform-appropriate extension
    ///
    /// On Windows, appends ".exe" to the base name.
    /// On other platforms, returns the base name unchanged.
    ///
    /// # Example
    /// ```
    /// use vx_runtime::{Platform, Os, Arch};
    ///
    /// let windows = Platform::new(Os::Windows, Arch::X86_64);
    /// assert_eq!(windows.exe_name("cargo"), "cargo.exe");
    ///
    /// let linux = Platform::new(Os::Linux, Arch::X86_64);
    /// assert_eq!(linux.exe_name("cargo"), "cargo");
    /// ```
    pub fn exe_name(&self, base: &str) -> String {
        if self.os == Os::Windows {
            format!("{}.exe", base)
        } else {
            base.to_string()
        }
    }

    /// Get executable name with custom extension options for Windows
    ///
    /// On Windows, returns the base name with the first extension from the list.
    /// On other platforms, returns the base name unchanged.
    ///
    /// This is useful for tools like npm, npx, yarn that use `.cmd` instead of `.exe`.
    ///
    /// # Example
    /// ```
    /// use vx_runtime::{Platform, Os, Arch};
    ///
    /// let windows = Platform::new(Os::Windows, Arch::X86_64);
    /// // npm uses .cmd on Windows
    /// assert_eq!(windows.executable_with_extensions("npm", &[".cmd", ".exe"]), "npm.cmd");
    ///
    /// let linux = Platform::new(Os::Linux, Arch::X86_64);
    /// assert_eq!(linux.executable_with_extensions("npm", &[".cmd", ".exe"]), "npm");
    /// ```
    pub fn executable_with_extensions(&self, base: &str, extensions: &[&str]) -> String {
        if self.os == Os::Windows {
            let ext = extensions.first().copied().unwrap_or(".exe");
            format!("{}{}", base, ext)
        } else {
            base.to_string()
        }
    }

    /// Get all possible executable names for this platform
    ///
    /// On Windows, returns names with all provided extensions.
    /// On other platforms, returns just the base name.
    ///
    /// # Example
    /// ```
    /// use vx_runtime::{Platform, Os, Arch};
    ///
    /// let windows = Platform::new(Os::Windows, Arch::X86_64);
    /// let names = windows.all_executable_names("npm", &[".cmd", ".exe"]);
    /// assert_eq!(names, vec!["npm.cmd", "npm.exe", "npm"]);
    ///
    /// let linux = Platform::new(Os::Linux, Arch::X86_64);
    /// let names = linux.all_executable_names("npm", &[".cmd", ".exe"]);
    /// assert_eq!(names, vec!["npm"]);
    /// ```
    pub fn all_executable_names(&self, base: &str, extensions: &[&str]) -> Vec<String> {
        if self.os == Os::Windows {
            let mut names: Vec<String> = extensions
                .iter()
                .map(|ext| format!("{}{}", base, ext))
                .collect();
            // Also include the base name without extension as fallback
            names.push(base.to_string());
            names
        } else {
            vec![base.to_string()]
        }
    }

    /// Get Rust target triple for this platform
    ///
    /// Returns the Rust target triple string (e.g., "x86_64-unknown-linux-gnu")
    /// used for Rust toolchain downloads.
    ///
    /// For Linux, this takes into account the libc type (gnu vs musl).
    ///
    /// # Example
    /// ```
    /// use vx_runtime::{Platform, Os, Arch, Libc};
    ///
    /// let linux = Platform::new(Os::Linux, Arch::X86_64);
    /// assert_eq!(linux.rust_target_triple(), "x86_64-unknown-linux-gnu");
    ///
    /// let alpine = Platform::with_libc(Os::Linux, Arch::X86_64, Libc::Musl);
    /// assert_eq!(alpine.rust_target_triple(), "x86_64-unknown-linux-musl");
    ///
    /// let windows = Platform::new(Os::Windows, Arch::X86_64);
    /// assert_eq!(windows.rust_target_triple(), "x86_64-pc-windows-msvc");
    /// ```
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
            (Os::Linux, Arch::Arm, Libc::Musl) => "arm-unknown-linux-musleabihf",
            // Linux with glibc (default)
            (Os::Linux, Arch::X86_64, Libc::Gnu) => "x86_64-unknown-linux-gnu",
            (Os::Linux, Arch::Aarch64, Libc::Gnu) => "aarch64-unknown-linux-gnu",
            (Os::Linux, Arch::Arm, Libc::Gnu) => "arm-unknown-linux-gnueabihf",
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
    ///
    /// This includes the major OS/architecture combinations that most tools support.
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

    /// Check if this platform matches another (for supported_platforms checks)
    ///
    /// This is a simple equality check, but can be extended for more complex matching.
    pub fn matches(&self, other: &Platform) -> bool {
        self.os == other.os && self.arch == other.arch
    }
}

/// Simple semver comparison for sorting versions
///
/// This function compares two version strings by parsing numeric components.
/// It handles various version formats like "1.2.3", "1.2.3-beta", etc.
///
/// # Example
/// ```
/// use vx_runtime::compare_semver;
/// use std::cmp::Ordering;
///
/// assert_eq!(compare_semver("1.2.3", "1.2.4"), Ordering::Less);
/// assert_eq!(compare_semver("2.0.0", "1.9.9"), Ordering::Greater);
/// assert_eq!(compare_semver("1.0.0", "1.0.0"), Ordering::Equal);
/// ```
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
