# 🔧 vx-core

<div align="center">

**Core Engine for the vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-core.svg)](https://crates.io/crates/vx-core)
[![Documentation](https://docs.rs/vx-core/badge.svg)](https://docs.rs/vx-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Foundational traits, types, and utilities powering the vx ecosystem*

</div>

## 🎯 Overview

`vx-core` provides the foundational traits, types, and utilities that power the vx tool management system. It defines the core abstractions for tool management, version handling, configuration, and plugin architecture. With the integration of **vx-installer**, it now offers state-of-the-art installation capabilities with beautiful progress tracking.

## ✨ Features

### 🔧 Core Functionality

- **Tool Management**: Advanced traits for tool installation, version management, and execution
- **Plugin Architecture**: Extensible plugin system with trait-based design for adding new tools
- **Configuration Management**: Unified configuration system using Figment with TOML support
- **Version Handling**: Semantic version parsing, comparison, and constraint resolution

### 🚀 Enhanced Installation System (via vx-installer)

- **🎨 Beautiful Progress Bars**: Rich progress tracking with ETA and transfer rates
- **📦 Universal Format Support**: ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and raw binaries
- **🔒 Security First**: Built-in checksum verification and secure HTTPS downloads
- **⚡ Async Performance**: Lightning-fast concurrent downloads and installations
- **🎯 Flexible Methods**: Support for archives, binaries, scripts, and package managers

### 🛠️ Advanced Features

- **Virtual Environments**: Symlink-based virtual environment management with isolation
- **Cross-Platform**: Seamless operation on Windows, macOS, and Linux
- **Error Handling**: Comprehensive error types with recovery suggestions
- **HTTP Utilities**: Advanced HTTP client with retry logic and timeout handling

## Core Traits

### Tool Management

```rust
use vx_core::{Tool, ToolManager, Version};

// Define a custom tool
struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &str { "mytool" }
    fn description(&self) -> &str { "My custom tool" }
    // ... implement other required methods
}
```

### Plugin System

```rust
use vx_core::{Plugin, PluginManager};

// Create a plugin for your tool
struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str { "my-plugin" }
    fn tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(MyTool)]
    }
}
```

### Configuration

```rust
use vx_core::config::{Config, ConfigManager};

// Load configuration from multiple sources
let config = ConfigManager::load()?;
let auto_install = config.auto_install.enabled;
```

## Key Components

### Version Management

- **Version**: Semantic version representation
- **VersionManager**: Version comparison and resolution
- **VersionParser**: Parse version strings and constraints

### Tool Installation (Enhanced with vx-installer)

- **InstallerAdapter**: Bridge to the powerful vx-installer engine
- **Universal Installation**: Support for multiple archive formats and installation methods
- **Progress Tracking**: Beautiful progress bars with customizable styles
- **Security**: Checksum verification and secure download protocols
- **Platform**: Cross-platform path and architecture detection with smart defaults

### Virtual Environments

- **VirtualEnvironment**: Manage isolated tool environments
- **SymlinkVenv**: Symlink-based virtual environment implementation

### Configuration System

- **Config**: Global and project-specific configuration
- **ConfigFigment**: Figment-based configuration loading
- **InstallConfigs**: Tool-specific installation configurations

## 💡 Usage Examples

### Enhanced Tool Installation with vx-installer

```rust
use vx_core::InstallerAdapter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create installer adapter with beautiful progress tracking
    let installer = InstallerAdapter::new().await?;

    // Install tools with automatic progress bars and checksum verification
    let node_path = installer
        .download_and_install(
            "node",
            "18.17.0",
            "https://nodejs.org/dist/v18.17.0/node-v18.17.0-linux-x64.tar.gz"
        )
        .await?;

    println!("✅ Node.js installed to: {}", node_path.display());

    // Check if a tool version is installed
    let is_installed = installer.is_version_installed("node", "18.17.0").await?;
    println!("Node.js 18.17.0 installed: {}", is_installed);

    Ok(())
}
```

### Basic Tool Management

```rust
use vx_core::{ToolManager, GlobalToolManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = GlobalToolManager::new()?;

    // Install a tool
    manager.install_tool("node", "18.17.0").await?;

    // List installed tools
    let tools = manager.list_installed_tools()?;
    for tool in tools {
        println!("{}: {}", tool.name, tool.version);
    }

    Ok(())
}
```

### Configuration Management

```rust
use vx_core::config::ConfigManager;

let config = ConfigManager::load()?;

// Check if auto-install is enabled
if config.auto_install.enabled {
    println!("Auto-install is enabled");
}

// Get tool-specific configuration
if let Some(node_config) = config.tools.get("node") {
    println!("Node.js version: {}", node_config.version);
}
```

### Virtual Environment

```rust
use vx_core::{VirtualEnvironment, SymlinkVenv};

let venv = SymlinkVenv::new("my-project")?;

// Add tools to the environment
venv.add_tool("node", "18.17.0")?;
venv.add_tool("uv", "latest")?;

// Activate the environment
venv.activate()?;
```

## Configuration

vx-core uses a hierarchical configuration system:

1. **Global Config**: `~/.vx/config/global.toml`
2. **Project Config**: `.vx.toml` in project root
3. **Environment Variables**: `VX_*` prefixed variables

### Example Configuration

```toml
[auto_install]
enabled = true
timeout = 300

[tools]
node = "18.17.0"
uv = "latest"

[settings]
cache_duration = "7d"
parallel_downloads = 4
```

## Error Handling

vx-core uses `anyhow::Result` for error handling:

```rust
use vx_core::error::VxError;

fn example() -> anyhow::Result<()> {
    // Operations that might fail
    let tool = manager.find_tool("nonexistent")?;
    Ok(())
}
```

## Platform Support

- **Windows**: Full support with proper path handling
- **macOS**: Native support for both Intel and Apple Silicon
- **Linux**: Support for major distributions

## 📦 Dependencies

### Core Dependencies

- **vx-installer**: Universal installation engine with progress tracking
- **vx-config**: Configuration management system
- **vx-plugin**: Plugin system and trait definitions
- **serde**: Serialization and deserialization
- **tokio**: Async runtime for high-performance operations
- **anyhow**: Comprehensive error handling

### Installation & HTTP

- **reqwest**: HTTP client for downloads
- **figment**: Advanced configuration management
- **dirs**: Platform-specific directory handling
- **walkdir**: Directory traversal utilities

### Development Dependencies

- **tempfile**: Temporary file handling for tests
- **tokio-test**: Async testing utilities

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Documentation

```bash
cargo doc --open
```

## Integration

vx-core is designed to be used by:

- **vx-cli**: Command-line interface
- **Tool Plugins**: Individual tool implementations
- **Package Manager Plugins**: Package manager integrations
- **External Applications**: Third-party integrations

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../CONTRIBUTING.md) for more information.

## 🔗 Related Crates

- [`vx-installer`](../vx-installer/README.md) - 🆕 Universal installation engine with progress tracking
- [`vx-cli`](../vx-cli/README.md) - Command-line interface with rich UX
- [`vx-config`](../vx-config/README.md) - Configuration management system
- [`vx-plugin`](../vx-plugin/README.md) - Plugin system and trait definitions
- [`vx-tool-node`](../vx-tools/vx-tool-node/README.md) - Node.js tool plugin
- [`vx-tool-uv`](../vx-tools/vx-tool-uv/README.md) - UV Python tool plugin
- [`vx-pm-npm`](../vx-package-managers/vx-pm-npm/README.md) - NPM package manager plugin

---

<div align="center">

**Built with ❤️ for the vx ecosystem**

[📖 Documentation](https://docs.rs/vx-core) | [🚀 Main Project](../../README.md) | [🤝 Contributing](../../CONTRIBUTING.md)

</div>
