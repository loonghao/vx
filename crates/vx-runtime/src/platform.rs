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
