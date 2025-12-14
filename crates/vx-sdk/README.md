# vx-sdk

Tool Development SDK for vx - Universal Development Tool Manager.

## Overview

This crate provides the unified SDK for developing tools and bundles that integrate with the vx ecosystem.

## Features

- **Tool Trait**: Core interface for implementing tool support
- **ToolBundle Trait**: Group related tools and package managers
- **PackageManager Trait**: Unified interface for package managers
- **Standard Implementations**: Ready-to-use implementations for common patterns
- **Helpers**: URL builders, version utilities, and platform helpers

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vx-sdk = "0.4"
```

## Quick Start

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

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("1.0.0")])
    }
}
```

### Creating a Tool Bundle

```rust
use vx_sdk::{ToolBundle, Tool, PackageManager, Result};
use async_trait::async_trait;

struct MyBundle;

#[async_trait]
impl ToolBundle for MyBundle {
    fn name(&self) -> &str {
        "my-bundle"
    }

    fn description(&self) -> &str {
        "A bundle providing custom tools"
    }

    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![]
    }

    fn package_managers(&self) -> Vec<Box<dyn PackageManager>> {
        vec![]
    }
}
```

## Migration from vx-plugin

If you're migrating from `vx-plugin`, the following renames apply:

| Old Name | New Name |
|----------|----------|
| `VxTool` | `Tool` |
| `VxPlugin` | `ToolBundle` |
| `VxPackageManager` | `PackageManager` |
| `StandardPlugin` | `StandardBundle` |
| `PluginRegistry` | `BundleRegistry` |

Deprecated aliases are available for backward compatibility.

## License

MIT
