# 📊 vx-version

<div align="center">

**Advanced Version Management for the vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-version.svg)](https://crates.io/crates/vx-version)
[![Documentation](https://docs.rs/vx-version/badge.svg)](https://docs.rs/vx-version)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Intelligent version parsing, fetching, and management with semantic version support*

</div>

---

## 🎯 Overview

`vx-version` provides comprehensive version management capabilities for the vx universal tool manager. It handles version parsing, fetching from external sources, semantic version comparison, and constraint resolution across different tool ecosystems.

## ✨ Features

### 🔍 Version Parsing & Comparison
- **Semantic Version Support**: Full semver parsing with major.minor.patch support
- **Prerelease Handling**: Intelligent detection and handling of alpha, beta, rc versions
- **Version Comparison**: Advanced comparison algorithms for sorting and constraint matching
- **Format Flexibility**: Support for various version formats (v1.0.0, go1.21.0, etc.)

### 🌐 External Version Fetching
- **GitHub Releases**: Fetch versions from GitHub releases API
- **Node.js Official**: Direct integration with Node.js distribution API
- **Go Releases**: Support for Go version fetching
- **Extensible**: Easy to add new version sources

### 🎯 Smart Version Management
- **LTS Detection**: Automatic detection of Long Term Support versions
- **Stability Filtering**: Filter stable vs prerelease versions
- **Version Constraints**: Support for version ranges and constraints
- **Caching**: Intelligent caching of version information

### ⚡ Performance & Reliability
- **Async-First**: Non-blocking operations with concurrent fetching
- **Error Handling**: Comprehensive error types with recovery suggestions
- **Retry Logic**: Built-in retry mechanisms for network operations
- **Rate Limiting**: Respectful API usage with proper rate limiting

## 🚀 Quick Start

Add `vx-version` to your `Cargo.toml`:

```toml
[dependencies]
vx-version = "0.2"
```

### Basic Usage

```rust
use vx_version::{VersionManager, GitHubVersionFetcher, VersionUtils};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Fetch versions from GitHub
    let fetcher = GitHubVersionFetcher::new("astral-sh", "uv");
    let versions = fetcher.fetch_versions(false).await?;
    
    println!("Latest UV version: {}", versions[0].version);
    
    // Get installed version
    if let Some(installed) = VersionManager::get_installed_version("uv")? {
        println!("Installed UV version: {}", installed);
    }
    
    // Compare versions
    let is_newer = VersionUtils::is_greater_than("0.7.13", "0.7.10");
    println!("0.7.13 > 0.7.10: {}", is_newer);
    
    Ok(())
}
```

## 💡 Advanced Usage

### Version Fetching

```rust
use vx_version::{GitHubVersionFetcher, NodeVersionFetcher, VersionFetcher};

// GitHub releases
let uv_fetcher = GitHubVersionFetcher::new("astral-sh", "uv");
let uv_versions = uv_fetcher.fetch_versions(true).await?; // Include prereleases

// Node.js official API
let node_fetcher = NodeVersionFetcher::new();
let node_versions = node_fetcher.fetch_versions(false).await?; // Stable only

// Get latest version
let latest_uv = uv_fetcher.get_latest_version().await?;
if let Some(version) = latest_uv {
    println!("Latest UV: {}", version.version);
}
```

### Version Parsing & Comparison

```rust
use vx_version::{Version, VersionUtils};

// Parse semantic versions
let v1 = Version::parse("1.2.3")?;
let v2 = Version::parse("1.2.4-alpha.1")?;

println!("v1: {}, prerelease: {}", v1, v1.is_prerelease());
println!("v2: {}, prerelease: {}", v2, v2.is_prerelease());

// Compare versions
assert!(v1 < v2); // Semantic comparison

// Utility functions
assert!(VersionUtils::is_greater_than("2.0.0", "1.9.9"));
assert!(VersionUtils::is_prerelease("1.0.0-beta"));
assert_eq!(VersionUtils::clean_version("v1.0.0", &["v"]), "1.0.0");
```

### Version Information

```rust
use vx_version::VersionInfo;

let version = VersionInfo::new("18.17.0".to_string())
    .with_release_date("2023-06-20".to_string())
    .with_download_url("https://nodejs.org/dist/v18.17.0/node-v18.17.0.tar.gz".to_string())
    .with_metadata("lts".to_string(), "true".to_string())
    .with_metadata("lts_name".to_string(), "Hydrogen".to_string());

println!("Version: {}", version); // "18.17.0 (LTS: Hydrogen)"
println!("Is LTS: {}", version.is_lts());
println!("Download URL: {:?}", version.download_url);
```

### Custom Version Fetchers

```rust
use vx_version::{VersionFetcher, VersionInfo, Result};
use async_trait::async_trait;

struct CustomVersionFetcher {
    tool_name: String,
}

#[async_trait]
impl VersionFetcher for CustomVersionFetcher {
    fn tool_name(&self) -> &str {
        &self.tool_name
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Custom implementation
        let versions = vec![
            VersionInfo::new("1.0.0".to_string()),
            VersionInfo::new("1.1.0".to_string()),
        ];
        Ok(versions)
    }
}
```

## 🏗️ Architecture

### Core Components

```
vx-version/
├── error.rs          # Error types and handling
├── fetcher.rs         # Version fetching traits and implementations
├── info.rs            # VersionInfo type and utilities
├── manager.rs         # Version management and comparison
├── parser.rs          # Version parsing for different tools
└── utils.rs           # Utility functions and helpers
```

### Version Sources

| Source | Tool Support | Features |
|--------|-------------|----------|
| **GitHub Releases** | UV, Rust, Go, etc. | Releases API, prerelease detection |
| **Node.js Official** | Node.js | LTS detection, release metadata |
| **Custom APIs** | Extensible | Plugin-based architecture |

### Version Formats

| Format | Example | Tools |
|--------|---------|-------|
| **Semantic** | `1.2.3` | Most tools |
| **Prefixed** | `v1.2.3` | GitHub releases |
| **Tool-specific** | `go1.21.0` | Go releases |
| **Prerelease** | `1.0.0-alpha.1` | Development versions |

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test version_parsing
cargo test version_fetching
cargo test version_comparison

# Run with network tests (requires internet)
cargo test --features network-tests

# Test with coverage
cargo tarpaulin --out Html
```

### Test Coverage

- **Unit Tests**: 95%+ coverage of core functionality
- **Integration Tests**: Real API testing with mocked responses
- **Property Tests**: Fuzz testing for version parsing
- **Performance Tests**: Benchmarks for comparison algorithms

## 🔗 Related Crates

- [`vx-installer`](../vx-installer/README.md) - Universal installation engine
- [`vx-core`](../vx-core/README.md) - Core functionality and utilities
- [`vx-cli`](../vx-cli/README.md) - Command-line interface
- [`vx-config`](../vx-config/README.md) - Configuration management
- [`vx-plugin`](../vx-plugin/README.md) - Plugin system and trait definitions

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

---

<div align="center">

**Intelligent version management for the modern developer**

[🚀 Get Started](../../README.md) | [📖 Documentation](https://docs.rs/vx-version) | [🤝 Contributing](../../CONTRIBUTING.md)

</div>