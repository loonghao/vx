# 🔌 vx-plugin

<div align="center">

**Extensible Plugin System for the vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-plugin.svg)](https://crates.io/crates/vx-plugin)
[![Documentation](https://docs.rs/vx-plugin/badge.svg)](https://docs.rs/vx-plugin)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Powerful, trait-based plugin architecture with beautiful installation experience*

</div>

## 🎯 Overview

`vx-plugin` provides the powerful plugin architecture for vx, enabling developers to create custom tools and package managers that integrate seamlessly with the vx ecosystem. With the integration of **vx-installer**, plugins now benefit from beautiful progress tracking and advanced installation capabilities.

## ✨ Features

### 🔧 Core Plugin System
- **Tool Plugins**: Create custom tool implementations with automatic version management
- **Package Manager Plugins**: Integrate custom package managers with unified interfaces
- **Plugin Registry**: Discover and manage plugins dynamically with hot-loading support
- **Extensible Architecture**: Clean trait-based design for maximum flexibility and type safety

### 🚀 Enhanced Installation Integration
- **🎨 Beautiful Progress Bars**: Automatic progress tracking for all plugin installations
- **📦 Universal Format Support**: Plugins can handle ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and binaries
- **🔒 Security First**: Built-in checksum verification and secure downloads for plugin tools
- **⚡ Async Performance**: Non-blocking operations with concurrent installation support

### 🛠️ Advanced Features
- **Dynamic Loading**: Load plugins at runtime with dependency resolution
- **Version Constraints**: Sophisticated version matching and constraint resolution
- **Error Recovery**: Comprehensive error handling with helpful suggestions
- **Cross-Platform**: Seamless operation across Windows, macOS, and Linux

## Quick Start

### Creating a Simple Tool Plugin

```rust
use vx_plugin::{VxTool, VersionInfo, Result};
use async_trait::async_trait;

struct MyTool;

#[async_trait]
impl VxTool for MyTool {
    fn name(&self) -> &str {
        "mytool"
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // Fetch versions from your tool's API or registry
        Ok(vec![
            VersionInfo::new("1.0.0"),
            VersionInfo::new("1.1.0"),
        ])
    }

    // Optional: Provide custom installation workflow with vx-installer integration
    async fn default_install_workflow(&self, version: &str) -> Result<PathBuf> {
        // This method automatically gets beautiful progress bars and security features
        // when using the vx-installer integration
        let download_url = format!("https://releases.example.com/mytool-{}.tar.gz", version);

        // The installation will automatically show progress bars, verify checksums,
        // and handle multiple archive formats
        self.install_from_url(version, &download_url).await
    }
}
```

### Creating a Package Manager Plugin

```rust
use vx_plugin::{VxPackageManager, Ecosystem, PackageSpec, Result};
use async_trait::async_trait;
use std::path::Path;

struct MyPackageManager;

#[async_trait]
impl VxPackageManager for MyPackageManager {
    fn name(&self) -> &str {
        "mypm"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        // Install packages using your package manager
        Ok(())
    }
}
```

### Creating a Combined Plugin

```rust
use vx_plugin::{VxPlugin, VxTool, VxPackageManager};
use async_trait::async_trait;

struct MyPlugin;

#[async_trait]
impl VxPlugin for MyPlugin {
    fn name(&self) -> &str {
        "my-plugin"
    }

    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(MyTool)]
    }

    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(MyPackageManager)]
    }
}
```

## 🚀 Enhanced Installation Features

vx-plugin integrates seamlessly with the **vx-installer** engine to provide beautiful installation experiences:

### 📊 Automatic Progress Tracking

When your plugin installs tools, users automatically get:

```bash
# Beautiful progress bars appear automatically
🚀 Installing MyTool v1.0.0...
⬇️  [████████████████████████████████] 25.4MB/25.4MB (3.2MB/s, 0s remaining)
📦 Extracting archive...
🔧 Setting up tool...
✅ MyTool v1.0.0 installed successfully!
```

### 🔒 Built-in Security

All plugin installations automatically include:

- **HTTPS-only downloads** for secure connections
- **Automatic checksum verification** to ensure file integrity
- **Secure archive extraction** with path validation
- **Permission validation** before installation

### 📦 Universal Format Support

Your plugins can handle multiple archive formats without additional code:

```rust
// This automatically handles ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and raw binaries
async fn install_version(&self, version: &str) -> Result<PathBuf> {
    let download_url = self.get_download_url(version)?;

    // vx-installer automatically detects format and shows progress
    self.install_from_url(version, &download_url).await
}
```

## Core Traits

### VxTool

The `VxTool` trait is the core interface for implementing tool support. Tools can be anything from compilers and interpreters to CLI utilities and development tools.

**Required Methods:**
- `name()` - Return the tool name
- `fetch_versions()` - Fetch available versions from the tool's source

**Optional Methods (Enhanced with vx-installer):**
- `install_version()` - Install a specific version with beautiful progress bars
- `default_install_workflow()` - Custom installation with automatic progress tracking
- `execute()` - Execute the tool with arguments and environment isolation
- `get_status()` - Get tool installation status with detailed information
- `get_download_url()` - Provide download URLs for automatic installation
- `verify_installation()` - Verify tool installation integrity
- And many more with sensible defaults and enhanced UX

### VxPackageManager

The `VxPackageManager` trait provides a unified interface for different package managers across various ecosystems.

**Required Methods:**
- `name()` - Return the package manager name
- `ecosystem()` - Return the ecosystem (Node, Python, Rust, etc.)
- `install_packages()` - Install packages in a project

**Optional Methods:**
- `remove_packages()` - Remove packages
- `update_packages()` - Update packages
- `list_packages()` - List installed packages
- `search_packages()` - Search for packages

### VxPlugin

The `VxPlugin` trait is the main interface for creating plugins that can provide both tools and package managers.

**Required Methods:**
- `name()` - Return the plugin name

**Optional Methods:**
- `tools()` - Return tools provided by this plugin
- `package_managers()` - Return package managers provided by this plugin
- `initialize()` - Initialize the plugin
- `shutdown()` - Shutdown the plugin

## Plugin Registry

The `PluginRegistry` manages all loaded plugins and provides discovery functionality:

```rust
use vx_plugin::{PluginRegistry, PluginRegistryBuilder};

// Create a registry with plugins
let registry = PluginRegistryBuilder::new()
    .with_plugin(Box::new(MyPlugin))
    .build()
    .await?;

// Use the registry
let tool = registry.get_tool("mytool");
let pm = registry.get_package_manager("mypm");
```

## Examples

See the `examples/` directory for complete working examples:

- `simple_tool_plugin.rs` - Basic tool plugin implementation
- `package_manager_plugin.rs` - Package manager plugin implementation  
- `combined_plugin.rs` - Plugin providing both tools and package managers

## 🔗 Related Crates

- [`vx-installer`](../vx-installer/README.md) - 🆕 Universal installation engine with progress tracking
- [`vx-core`](../vx-core/README.md) - Core functionality and utilities
- [`vx-cli`](../vx-cli/README.md) - Command-line interface with rich UX
- [`vx-config`](../vx-config/README.md) - Configuration management system

### Example Plugins
- [`vx-tool-node`](../vx-tools/vx-tool-node/README.md) - Node.js tool plugin with NPX support
- [`vx-tool-uv`](../vx-tools/vx-tool-uv/README.md) - UV Python tool plugin with UVX support
- [`vx-tool-go`](../vx-tools/vx-tool-go/README.md) - Go toolchain plugin
- [`vx-tool-rust`](../vx-tools/vx-tool-rust/README.md) - Rust and Cargo plugin
- [`vx-pm-npm`](../vx-package-managers/vx-pm-npm/README.md) - NPM package manager plugin

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

---

<div align="center">

**Build powerful plugins for the vx ecosystem**

[🚀 Get Started](../../README.md) | [📖 Documentation](https://docs.rs/vx-plugin) | [🤝 Contributing](../../CONTRIBUTING.md)

</div>