# 🛠️ vx-sdk

<div align="center">

**Tool Development SDK for vx - Universal Development Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-sdk.svg)](https://crates.io/crates/vx-sdk)
[![Documentation](https://docs.rs/vx-sdk/badge.svg)](https://docs.rs/vx-sdk)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

*The recommended way to create tool plugins for the vx ecosystem*

</div>

---

## 🎯 Overview

`vx-sdk` is the unified SDK for developing tools and bundles that integrate with the vx ecosystem. It provides a clean, modern API for creating tool plugins with automatic version management, beautiful progress tracking, and cross-platform support.

> **Note**: This is the recommended crate for plugin development. It supersedes the older `vx-plugin` API with cleaner naming and better ergonomics.

## ✨ Features

- **Tool Trait**: Core interface for implementing tool support
- **ToolBundle Trait**: Group related tools and package managers
- **PackageManager Trait**: Unified interface for package managers
- **Standard Implementations**: Ready-to-use implementations for common patterns
- **Helpers**: URL builders, version utilities, and platform helpers
- **Registry**: Dynamic tool and bundle registration

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
vx-sdk = "0.4"
```

### Creating a Simple Tool

```rust
use vx_sdk::{Tool, VersionInfo, Result};
use async_trait::async_trait;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "mytool"
    }

    fn description(&self) -> &str {
        "My awesome development tool"
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Fetch versions from your tool's release API
        Ok(vec![
            VersionInfo::new("1.0.0"),
            VersionInfo::new("1.1.0"),
            VersionInfo::new("2.0.0"),
        ])
    }

    fn get_download_url(&self, version: &str) -> Result<String> {
        Ok(format!(
            "https://releases.example.com/mytool-{}-{}.tar.gz",
            version,
            std::env::consts::OS
        ))
    }
}
```

### Creating a Tool Bundle

A bundle groups related tools and package managers together:

```rust
use vx_sdk::{ToolBundle, Tool, PackageManager, Result};
use async_trait::async_trait;

struct NodeEcosystemBundle;

#[async_trait]
impl ToolBundle for NodeEcosystemBundle {
    fn name(&self) -> &str {
        "node-ecosystem"
    }

    fn description(&self) -> &str {
        "Node.js runtime and related package managers"
    }

    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(NodeTool),
        ]
    }

    fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
        vec![
            Box::new(NpmPackageManager),
            Box::new(YarnPackageManager),
        ]
    }
}
```

### Using the Registry

```rust
use vx_sdk::{BundleRegistry, BundleRegistryBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let registry = BundleRegistryBuilder::new()
        .with_bundle(Box::new(NodeEcosystemBundle))
        .with_bundle(Box::new(PythonEcosystemBundle))
        .build()
        .await?;

    // Find a tool
    if let Some(tool) = registry.get_tool("node") {
        let versions = tool.fetch_versions(false).await?;
        println!("Available Node.js versions: {:?}", versions);
    }

    // Find a package manager
    if let Some(pm) = registry.get_package_manager("npm") {
        println!("Found package manager: {}", pm.name());
    }

    Ok(())
}
```

## 📦 Core Types

### Tool Trait

The `Tool` trait is the core interface for implementing tool support:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    /// Tool name (e.g., "node", "go", "rust")
    fn name(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str { "No description" }

    /// Fetch available versions from the tool's source
    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>>;

    /// Get download URL for a specific version
    fn get_download_url(&self, version: &str) -> Result<String> { ... }

    /// Install a specific version
    async fn install_version(&self, version: &str) -> Result<PathBuf> { ... }

    /// Execute the tool with arguments
    async fn execute(&self, version: &str, args: &[String]) -> Result<ExitStatus> { ... }
}
```

### VersionInfo

```rust
use vx_sdk::VersionInfo;

let version = VersionInfo::new("18.17.0")
    .with_release_date("2023-06-20")
    .with_download_url("https://nodejs.org/dist/v18.17.0/...")
    .with_lts("Hydrogen");

println!("Version: {}", version.version);
println!("Is LTS: {}", version.is_lts());
```

### Platform Helpers

```rust
use vx_sdk::helpers::{PlatformUrlBuilder, UrlUtils};

// Build platform-specific URLs
let url = PlatformUrlBuilder::new("https://releases.example.com")
    .with_version("1.0.0")
    .with_platform_suffix()
    .build()?;

// Utility functions
let filename = UrlUtils::extract_filename(&url)?;
let extension = UrlUtils::get_extension(&url)?;
```

## 🔄 Migration from vx-plugin

If you're migrating from `vx-plugin`, the following renames apply:

| Old Name (vx-plugin) | New Name (vx-sdk) |
|---------------------|-------------------|
| `VxTool` | `Tool` |
| `VxPlugin` | `ToolBundle` |
| `VxPackageManager` | `PackageManager` |
| `StandardPlugin` | `StandardBundle` |
| `PluginRegistry` | `BundleRegistry` |
| `PluginRegistryBuilder` | `BundleRegistryBuilder` |

Deprecated aliases are available for backward compatibility, but new code should use the new names.

## 🏗️ Architecture

```
vx-sdk/
├── traits/
│   ├── tool.rs          # Tool trait definition
│   ├── bundle.rs        # ToolBundle trait definition
│   └── package_manager.rs # PackageManager trait definition
├── standard/
│   ├── tool.rs          # ConfigurableTool implementation
│   ├── bundle.rs        # StandardBundle implementation
│   └── package_manager.rs # StandardPackageManager implementation
├── registry/
│   ├── bundle.rs        # BundleRegistry
│   └── tool.rs          # ToolRegistry
├── helpers/
│   ├── platform.rs      # Platform detection and URL building
│   ├── url.rs           # URL utilities
│   └── version.rs       # Version utilities
└── types/
    ├── version_info.rs  # VersionInfo type
    └── ecosystem.rs     # Ecosystem enum
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=vx_sdk=debug cargo test

# Run specific test
cargo test tool_trait
```

## 🔗 Related Crates

- [`vx-core`](../vx-core/README.md) - Core engine and tool management
- [`vx-installer`](../vx-installer/README.md) - Universal installation engine
- [`vx-version`](../vx-version/README.md) - Version parsing and fetching
- [`vx-plugin`](../vx-plugin/README.md) - Legacy plugin API (use vx-sdk instead)

### Example Tool Implementations

- [`vx-tool-node`](../vx-tools/node/README.md) - Node.js implementation
- [`vx-tool-go`](../vx-tools/go/README.md) - Go implementation
- [`vx-tool-uv`](../vx-tools/uv/README.md) - UV Python implementation

## 📄 License

MIT License - see [LICENSE](../../LICENSE) for details.

---

<div align="center">

**Build powerful tools for the vx ecosystem**

[🚀 Get Started](../../README.md) | [📖 Documentation](https://docs.rs/vx-sdk) | [🤝 Contributing](../../CONTRIBUTING.md)

</div>
