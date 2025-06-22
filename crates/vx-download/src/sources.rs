//! VX-specific download sources and URL builders

use crate::error::{DownloadError, Result};
use std::collections::HashMap;
use url::Url;
use vx_config::types::{DownloadSourceConfig, VxConfig};

/// VX-specific source configuration
#[derive(Debug, Clone)]
pub struct VxSource {
    pub name: String,
    pub base_url: String,
    pub supported_domains: Vec<String>,
    pub url_patterns: Vec<String>,
    pub max_file_size: u64,
    pub timeout_seconds: u64,
}

impl VxSource {
    /// Create VxSource from DownloadSourceConfig
    pub fn from_config(config: &DownloadSourceConfig) -> Self {
        Self {
            name: config.name.clone(),
            base_url: config.base_url.clone(),
            supported_domains: config.supported_domains.clone(),
            url_patterns: vec![config.url_template.clone()],
            max_file_size: config.max_file_size,
            timeout_seconds: config.timeout_seconds,
        }
    }
}

/// VX source registry for development tools
pub struct VxSourceRegistry {
    sources: HashMap<String, VxSource>,
}

impl VxSourceRegistry {
    /// Create a new VX source registry with all supported sources
    pub fn new() -> Self {
        let config = vx_config::get_global_config();
        Self::from_config(config)
    }

    /// Create VX source registry from configuration
    pub fn from_config(config: &VxConfig) -> Self {
        let mut registry = Self {
            sources: HashMap::new(),
        };

        // Load sources from configuration
        for (name, source_config) in &config.download_sources {
            if source_config.enabled {
                let vx_source = VxSource::from_config(source_config);
                registry.sources.insert(name.clone(), vx_source);
            }
        }

        // If no sources in config, fall back to hardcoded defaults for backward compatibility
        if registry.sources.is_empty() {
            registry.add_fallback_sources();
        }

        registry
    }

    /// Add fallback sources when configuration is empty (backward compatibility)
    fn add_fallback_sources(&mut self) {
        // Add all vx-specific sources as fallback
        self.add_nodejs_source();
        self.add_golang_source();
        self.add_python_standalone_source();
        self.add_rust_source();
        self.add_bun_source();
        self.add_uv_source();
        self.add_additional_sources();
    }

    /// Add Node.js official source
    fn add_nodejs_source(&mut self) {
        let source = VxSource {
            name: "nodejs".to_string(),
            base_url: "https://nodejs.org".to_string(),
            supported_domains: vec!["nodejs.org".to_string()],
            url_patterns: vec![
                "https://nodejs.org/dist/v{version}/node-v{version}-{platform}.{ext}".to_string(),
                "https://nodejs.org/download/release/v{version}/node-v{version}-{platform}.{ext}"
                    .to_string(),
            ],
            max_file_size: 50 * 1024 * 1024, // 50MB
            timeout_seconds: 300,
        };
        self.sources.insert("nodejs".to_string(), source);
    }

    /// Add Go official source
    fn add_golang_source(&mut self) {
        let source = VxSource {
            name: "golang".to_string(),
            base_url: "https://golang.org".to_string(),
            supported_domains: vec!["golang.org".to_string(), "go.dev".to_string()],
            url_patterns: vec![
                "https://golang.org/dl/go{version}.{platform}.{ext}".to_string(),
                "https://go.dev/dl/go{version}.{platform}.{ext}".to_string(),
            ],
            max_file_size: 200 * 1024 * 1024, // 200MB
            timeout_seconds: 600,
        };
        self.sources.insert("golang".to_string(), source);
    }

    /// Add Python Build Standalone source
    fn add_python_standalone_source(&mut self) {
        let source = VxSource {
            name: "python_standalone".to_string(),
            base_url: "https://github.com/astral-sh/python-build-standalone".to_string(),
            supported_domains: vec!["github.com".to_string()],
            url_patterns: vec![
                "https://github.com/astral-sh/python-build-standalone/releases/download/{date}/cpython-{version}+{date}-{platform}.{ext}".to_string(),
            ],
            max_file_size: 100 * 1024 * 1024, // 100MB
            timeout_seconds: 600,
        };
        self.sources.insert("python_standalone".to_string(), source);
    }

    /// Add Rust official source
    fn add_rust_source(&mut self) {
        let source = VxSource {
            name: "rust".to_string(),
            base_url: "https://forge.rust-lang.org".to_string(),
            supported_domains: vec![
                "forge.rust-lang.org".to_string(),
                "static.rust-lang.org".to_string(),
            ],
            url_patterns: vec![
                "https://static.rust-lang.org/dist/rust-{version}-{platform}.{ext}".to_string(),
            ],
            max_file_size: 300 * 1024 * 1024, // 300MB
            timeout_seconds: 900,
        };
        self.sources.insert("rust".to_string(), source);
    }

    /// Add Bun source (GitHub releases)
    fn add_bun_source(&mut self) {
        let source = VxSource {
            name: "bun".to_string(),
            base_url: "https://github.com/oven-sh/bun".to_string(),
            supported_domains: vec!["github.com".to_string()],
            url_patterns: vec![
                "https://github.com/oven-sh/bun/releases/download/bun-v{version}/bun-{platform}.{ext}".to_string(),
            ],
            max_file_size: 50 * 1024 * 1024, // 50MB
            timeout_seconds: 300,
        };
        self.sources.insert("bun".to_string(), source);
    }

    /// Add UV source (GitHub releases)
    fn add_uv_source(&mut self) {
        let source = VxSource {
            name: "uv".to_string(),
            base_url: "https://github.com/astral-sh/uv".to_string(),
            supported_domains: vec!["github.com".to_string()],
            url_patterns: vec![
                "https://github.com/astral-sh/uv/releases/download/{version}/uv-{platform}.{ext}"
                    .to_string(),
            ],
            max_file_size: 30 * 1024 * 1024, // 30MB
            timeout_seconds: 300,
        };
        self.sources.insert("uv".to_string(), source);
    }

    /// Add additional development tool sources
    fn add_additional_sources(&mut self) {
        // Maven Central
        let maven = VxSource {
            name: "maven".to_string(),
            base_url: "https://repo1.maven.org".to_string(),
            supported_domains: vec!["repo1.maven.org".to_string()],
            url_patterns: vec![
                "https://repo1.maven.org/maven2/{group}/{artifact}/{version}/{artifact}-{version}.{ext}".to_string(),
            ],
            max_file_size: 100 * 1024 * 1024, // 100MB
            timeout_seconds: 300,
        };
        self.sources.insert("maven".to_string(), maven);

        // NuGet
        let nuget = VxSource {
            name: "nuget".to_string(),
            base_url: "https://api.nuget.org".to_string(),
            supported_domains: vec!["api.nuget.org".to_string()],
            url_patterns: vec![
                "https://api.nuget.org/v3-flatcontainer/{package}/{version}/{package}.{version}.nupkg".to_string(),
            ],
            max_file_size: 50 * 1024 * 1024, // 50MB
            timeout_seconds: 300,
        };
        self.sources.insert("nuget".to_string(), nuget);
    }

    /// Get source by name
    pub fn get_source(&self, name: &str) -> Option<&VxSource> {
        self.sources.get(name)
    }

    /// Get all sources
    pub fn get_all_sources(&self) -> &HashMap<String, VxSource> {
        &self.sources
    }

    /// Check if URL is supported by any vx source
    pub fn is_url_supported(&self, url: &str) -> bool {
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(domain) = parsed_url.domain() {
                return self.sources.values().any(|source| {
                    source.supported_domains.iter().any(|supported_domain| {
                        domain == supported_domain
                            || domain.ends_with(&format!(".{}", supported_domain))
                    })
                });
            }
        }
        false
    }

    /// Get source for URL
    pub fn get_source_for_url(&self, url: &str) -> Option<&VxSource> {
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(domain) = parsed_url.domain() {
                return self.sources.values().find(|source| {
                    source.supported_domains.iter().any(|supported_domain| {
                        domain == supported_domain
                            || domain.ends_with(&format!(".{}", supported_domain))
                    })
                });
            }
        }
        None
    }

    /// Build download URL for a tool
    pub fn build_download_url(
        &self,
        tool_name: &str,
        version: &str,
        platform: &str,
    ) -> Result<String> {
        let source = self
            .get_source(tool_name)
            .ok_or_else(|| DownloadError::tool_not_found(tool_name))?;

        // Use the first URL pattern for now
        if let Some(pattern) = source.url_patterns.first() {
            let url = pattern
                .replace("{version}", version)
                .replace("{platform}", platform)
                .replace("{ext}", self.get_extension_for_platform(platform));

            Ok(url)
        } else {
            Err(DownloadError::config(format!(
                "No URL pattern found for tool: {}",
                tool_name
            )))
        }
    }

    /// Get file extension for platform
    fn get_extension_for_platform(&self, platform: &str) -> &str {
        if platform.contains("windows") {
            "zip"
        } else if platform.contains("darwin") || platform.contains("macos") {
            "tar.gz"
        } else {
            "tar.gz"
        }
    }

    /// Get all supported domains
    pub fn get_all_supported_domains(&self) -> Vec<String> {
        let mut domains = Vec::new();
        for source in self.sources.values() {
            domains.extend(source.supported_domains.clone());
        }
        domains.sort();
        domains.dedup();
        domains
    }
}

impl Default for VxSourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a comprehensive domain whitelist for vx tools
pub fn create_vx_domain_whitelist() -> Vec<String> {
    let registry = VxSourceRegistry::new();
    let mut domains = registry.get_all_supported_domains();

    // Add standard CDN and package registry domains
    domains.extend(vec![
        "github.com".to_string(),
        "githubusercontent.com".to_string(),
        "cdn.jsdelivr.net".to_string(),
        "fastly.jsdelivr.net".to_string(),
        "cdnjs.cloudflare.com".to_string(),
        "registry.npmjs.org".to_string(),
        "files.pythonhosted.org".to_string(),
        "pypi.org".to_string(),
        "crates.io".to_string(),
        "proxy.golang.org".to_string(),
    ]);

    domains.sort();
    domains.dedup();
    domains
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vx_source_registry() {
        let registry = VxSourceRegistry::new();

        // Test that all expected sources are present
        assert!(registry.get_source("nodejs").is_some());
        assert!(registry.get_source("golang").is_some());
        assert!(registry.get_source("python_standalone").is_some());
        assert!(registry.get_source("rust").is_some());
        assert!(registry.get_source("bun").is_some());
        assert!(registry.get_source("uv").is_some());
    }

    #[test]
    fn test_url_support_detection() {
        let registry = VxSourceRegistry::new();

        // Test supported URLs
        assert!(
            registry.is_url_supported("https://nodejs.org/dist/v18.17.0/node-v18.17.0-win-x64.zip")
        );
        assert!(registry.is_url_supported("https://golang.org/dl/go1.21.0.windows-amd64.zip"));
        assert!(registry.is_url_supported(
            "https://github.com/oven-sh/bun/releases/download/bun-v1.0.0/bun-windows-x64.zip"
        ));

        // Test unsupported URLs
        assert!(!registry.is_url_supported("https://example.com/some-file.zip"));
    }

    #[test]
    fn test_domain_whitelist() {
        let domains = create_vx_domain_whitelist();

        // Test that key domains are included
        assert!(domains.contains(&"nodejs.org".to_string()));
        assert!(domains.contains(&"golang.org".to_string()));
        assert!(domains.contains(&"github.com".to_string()));
        assert!(domains.contains(&"cdn.jsdelivr.net".to_string()));
    }
}
