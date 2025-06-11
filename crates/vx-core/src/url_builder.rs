//! URL building utilities for different tools

use crate::platform::Platform;

/// Temporary UrlBuilder trait during migration
pub trait UrlBuilder: Send + Sync {
    fn download_url(&self, version: &str) -> Option<String>;
    fn versions_url(&self) -> &str;
}

/// URL builder for Node.js downloads
pub struct NodeUrlBuilder;

impl NodeUrlBuilder {
    /// Generate Node.js download URL for a version
    pub fn download_url(version: &str) -> Option<String> {
        let platform = Platform::current();
        let (os, arch) = platform.node_platform_string()?;
        let ext = platform.archive_extension();

        Some(format!(
            "https://nodejs.org/dist/v{}/node-v{}-{}-{}.{}",
            version, version, os, arch, ext
        ))
    }

    /// Generate Node.js versions API URL
    pub fn versions_url() -> &'static str {
        "https://nodejs.org/dist/index.json"
    }
}

impl UrlBuilder for NodeUrlBuilder {
    fn download_url(&self, version: &str) -> Option<String> {
        Self::download_url(version)
    }

    fn versions_url(&self) -> &str {
        Self::versions_url()
    }
}

/// URL builder for Go downloads
pub struct GoUrlBuilder;

impl GoUrlBuilder {
    /// Generate Go download URL for a version
    pub fn download_url(version: &str) -> Option<String> {
        let platform = Platform::current();
        let (os, arch) = platform.go_platform_string()?;
        let ext = platform.archive_extension();

        Some(format!(
            "https://go.dev/dl/go{}.{}-{}.{}",
            version, os, arch, ext
        ))
    }

    /// Generate Go versions API URL
    pub fn versions_url() -> &'static str {
        "https://go.dev/dl/?mode=json"
    }
}

impl UrlBuilder for GoUrlBuilder {
    fn download_url(&self, version: &str) -> Option<String> {
        Self::download_url(version)
    }

    fn versions_url(&self) -> &str {
        Self::versions_url()
    }
}

/// URL builder for Rust downloads
pub struct RustUrlBuilder;

impl RustUrlBuilder {
    /// Generate Rust download URL (using rustup)
    pub fn download_url() -> Option<String> {
        let platform = Platform::current();
        match platform.os {
            crate::platform::OperatingSystem::Windows => {
                Some("https://win.rustup.rs/x86_64".to_string())
            }
            _ => Some("https://sh.rustup.rs".to_string()),
        }
    }
    
    /// Generate Rust versions API URL (GitHub releases)
    pub fn versions_url() -> &'static str {
        "https://api.github.com/repos/rust-lang/rust/releases"
    }
}

/// URL builder for Python downloads
pub struct PythonUrlBuilder;

impl PythonUrlBuilder {
    /// Generate Python download URL for a version
    pub fn download_url(version: &str) -> Option<String> {
        let platform = Platform::current();
        
        let (os, arch) = match platform.os {
            crate::platform::OperatingSystem::Windows => {
                let arch = match platform.arch {
                    crate::platform::Architecture::X86_64 => "amd64",
                    crate::platform::Architecture::X86 => "win32",
                    _ => return None,
                };
                ("windows", arch)
            }
            crate::platform::OperatingSystem::MacOS => {
                let arch = match platform.arch {
                    crate::platform::Architecture::X86_64 => "universal2",
                    crate::platform::Architecture::Aarch64 => "universal2",
                    _ => return None,
                };
                ("macos", arch)
            }
            crate::platform::OperatingSystem::Linux => {
                let arch = match platform.arch {
                    crate::platform::Architecture::X86_64 => "x86_64",
                    crate::platform::Architecture::Aarch64 => "aarch64",
                    _ => return None,
                };
                ("linux", arch)
            }
            _ => return None,
        };

        let ext = if matches!(platform.os, crate::platform::OperatingSystem::Windows) {
            "exe"
        } else {
            "tgz"
        };
        
        Some(format!(
            "https://www.python.org/ftp/python/{}/python-{}-{}-{}.{}",
            version, version, os, arch, ext
        ))
    }
    
    /// Generate Python versions API URL (GitHub releases)
    pub fn versions_url() -> &'static str {
        "https://api.github.com/repos/python/cpython/releases"
    }
}

/// Generic URL builder for tools that follow common patterns
pub struct GenericUrlBuilder;

impl GenericUrlBuilder {
    /// Build GitHub releases URL
    pub fn github_releases_url(owner: &str, repo: &str) -> String {
        format!("https://api.github.com/repos/{}/{}/releases", owner, repo)
    }
    
    /// Build GitHub release download URL
    pub fn github_release_download_url(
        owner: &str,
        repo: &str,
        tag: &str,
        filename: &str,
    ) -> String {
        format!(
            "https://github.com/{}/{}/releases/download/{}/{}",
            owner, repo, tag, filename
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_node_url_builder() {
        let url = NodeUrlBuilder::download_url("18.0.0");
        assert!(url.is_some());
        
        let url = url.unwrap();
        assert!(url.contains("18.0.0"));
        assert!(url.contains("nodejs.org"));
        
        let versions_url = NodeUrlBuilder::versions_url();
        assert!(versions_url.contains("nodejs.org"));
    }
    
    #[test]
    fn test_go_url_builder() {
        let url = GoUrlBuilder::download_url("1.21.0");
        assert!(url.is_some());
        
        let url = url.unwrap();
        assert!(url.contains("1.21.0"));
        assert!(url.contains("go.dev"));
        
        let versions_url = GoUrlBuilder::versions_url();
        assert!(versions_url.contains("go.dev"));
    }
    
    #[test]
    fn test_rust_url_builder() {
        let url = RustUrlBuilder::download_url();
        assert!(url.is_some());
        
        let url = url.unwrap();
        assert!(url.contains("rustup"));
        
        let versions_url = RustUrlBuilder::versions_url();
        assert!(versions_url.contains("github.com"));
    }
    
    #[test]
    fn test_python_url_builder() {
        let url = PythonUrlBuilder::download_url("3.12.0");
        assert!(url.is_some());
        
        let url = url.unwrap();
        assert!(url.contains("3.12.0"));
        assert!(url.contains("python.org"));
        
        let versions_url = PythonUrlBuilder::versions_url();
        assert!(versions_url.contains("github.com"));
    }
    
    #[test]
    fn test_generic_url_builder() {
        let releases_url = GenericUrlBuilder::github_releases_url("owner", "repo");
        assert_eq!(releases_url, "https://api.github.com/repos/owner/repo/releases");
        
        let download_url = GenericUrlBuilder::github_release_download_url(
            "owner", "repo", "v1.0.0", "file.tar.gz"
        );
        assert_eq!(
            download_url,
            "https://github.com/owner/repo/releases/download/v1.0.0/file.tar.gz"
        );
    }
}
