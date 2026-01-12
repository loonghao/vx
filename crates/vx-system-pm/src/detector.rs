//! Package manager detection utilities

use crate::Result;
use std::collections::HashMap;
use tracing::debug;

/// Package manager detector
pub struct PackageManagerDetector {
    /// Cache of detected package managers
    cache: HashMap<String, bool>,
}

impl PackageManagerDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Check if a package manager is available
    pub async fn is_available(&mut self, name: &str) -> bool {
        if let Some(&cached) = self.cache.get(name) {
            return cached;
        }

        let available = self.detect(name).await;
        self.cache.insert(name.to_string(), available);
        available
    }

    /// Detect if a package manager is installed
    async fn detect(&self, name: &str) -> bool {
        match name {
            "choco" | "chocolatey" => self.detect_chocolatey().await,
            "winget" => self.detect_winget().await,
            "scoop" => self.detect_scoop().await,
            "brew" | "homebrew" => self.detect_homebrew().await,
            "apt" | "apt-get" => self.detect_apt().await,
            "yum" => self.detect_yum().await,
            "dnf" => self.detect_dnf().await,
            "pacman" => self.detect_pacman().await,
            "zypper" => self.detect_zypper().await,
            _ => {
                debug!("Unknown package manager: {}", name);
                false
            }
        }
    }

    /// Detect Chocolatey
    async fn detect_chocolatey(&self) -> bool {
        #[cfg(windows)]
        {
            which::which("choco").is_ok()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    /// Detect winget
    async fn detect_winget(&self) -> bool {
        #[cfg(windows)]
        {
            which::which("winget").is_ok()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    /// Detect Scoop
    async fn detect_scoop(&self) -> bool {
        #[cfg(windows)]
        {
            which::which("scoop").is_ok()
        }
        #[cfg(not(windows))]
        {
            false
        }
    }

    /// Detect Homebrew
    async fn detect_homebrew(&self) -> bool {
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        {
            which::which("brew").is_ok()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            false
        }
    }

    /// Detect APT
    async fn detect_apt(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            which::which("apt-get").is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Detect YUM
    async fn detect_yum(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            which::which("yum").is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Detect DNF
    async fn detect_dnf(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            which::which("dnf").is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Detect Pacman
    async fn detect_pacman(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            which::which("pacman").is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Detect Zypper
    async fn detect_zypper(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            which::which("zypper").is_ok()
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    /// Get the preferred package manager for the current platform
    pub async fn get_preferred(&mut self) -> Option<String> {
        #[cfg(windows)]
        {
            // Windows: prefer winget > choco > scoop
            for pm in ["winget", "choco", "scoop"] {
                if self.is_available(pm).await {
                    return Some(pm.to_string());
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if self.is_available("brew").await {
                return Some("brew".to_string());
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: try common package managers
            for pm in ["apt", "dnf", "yum", "pacman", "zypper"] {
                if self.is_available(pm).await {
                    return Some(pm.to_string());
                }
            }
        }

        None
    }

    /// Get all available package managers
    pub async fn get_all_available(&mut self) -> Vec<String> {
        let mut available = Vec::new();

        #[cfg(windows)]
        let candidates = vec!["winget", "choco", "scoop"];

        #[cfg(target_os = "macos")]
        let candidates = vec!["brew"];

        #[cfg(target_os = "linux")]
        let candidates = vec!["apt", "dnf", "yum", "pacman", "zypper", "brew"];

        #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
        let candidates: Vec<&str> = vec![];

        for pm in candidates {
            if self.is_available(pm).await {
                available.push(pm.to_string());
            }
        }

        available
    }

    /// Clear the detection cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for PackageManagerDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if running with elevated privileges
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        is_elevated_windows()
    }
    #[cfg(unix)]
    {
        is_elevated_unix()
    }
    #[cfg(not(any(windows, unix)))]
    {
        false
    }
}

#[cfg(windows)]
fn is_elevated_windows() -> bool {
    use std::mem;
    use std::ptr;

    // Use Windows API to check elevation
    unsafe {
        use std::ffi::c_void;

        #[repr(C)]
        struct TokenElevation {
            token_is_elevated: u32,
        }

        type Handle = *mut c_void;
        const TOKEN_QUERY: u32 = 0x0008;

        #[link(name = "advapi32")]
        extern "system" {
            fn OpenProcessToken(
                process_handle: Handle,
                desired_access: u32,
                token_handle: *mut Handle,
            ) -> i32;
            fn GetTokenInformation(
                token_handle: Handle,
                token_information_class: u32,
                token_information: *mut c_void,
                token_information_length: u32,
                return_length: *mut u32,
            ) -> i32;
            fn CloseHandle(handle: Handle) -> i32;
        }

        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> Handle;
        }

        let mut token: Handle = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation: TokenElevation = mem::zeroed();
        let mut size: u32 = 0;

        // TokenElevation = 20
        let result = GetTokenInformation(
            token,
            20,
            &mut elevation as *mut _ as *mut c_void,
            mem::size_of::<TokenElevation>() as u32,
            &mut size,
        );

        CloseHandle(token);

        result != 0 && elevation.token_is_elevated != 0
    }
}

#[cfg(unix)]
fn is_elevated_unix() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detector_creation() {
        let detector = PackageManagerDetector::new();
        assert!(detector.cache.is_empty());
    }

    #[tokio::test]
    async fn test_get_preferred() {
        let mut detector = PackageManagerDetector::new();
        // This test just ensures the method doesn't panic
        let _ = detector.get_preferred().await;
    }
}
