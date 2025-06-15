# vx-core

[![Crates.io](https://img.shields.io/crates/v/vx-core.svg)](https://crates.io/crates/vx-core)
[![Documentation](https://docs.rs/vx-core/badge.svg)](https://docs.rs/vx-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Core traits and interfaces for the vx tool manager ecosystem.

## Overview

`vx-core` provides the foundational traits, types, and utilities that power the vx tool management system. It defines the core abstractions for tool management, version handling, configuration, and plugin architecture.

## Features

- **Tool Management**: Core traits for tool installation, version management, and execution
- **Plugin Architecture**: Extensible plugin system for adding new tools
- **Configuration Management**: Unified configuration system using Figment
- **Version Handling**: Semantic version parsing and comparison
- **Download & Installation**: HTTP downloading with progress tracking
- **Virtual Environments**: Symlink-based virtual environment management
- **Cross-Platform**: Windows, macOS, and Linux support

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

### Tool Installation
- **Installer**: Download and install tools
- **Downloader**: HTTP client with progress tracking
- **Platform**: Cross-platform path and architecture detection

### Virtual Environments
- **VirtualEnvironment**: Manage isolated tool environments
- **SymlinkVenv**: Symlink-based virtual environment implementation

### Configuration System
- **Config**: Global and project-specific configuration
- **ConfigFigment**: Figment-based configuration loading
- **InstallConfigs**: Tool-specific installation configurations

## Usage Examples

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

## Dependencies

- **serde**: Serialization and deserialization
- **tokio**: Async runtime
- **reqwest**: HTTP client
- **figment**: Configuration management
- **anyhow**: Error handling
- **dirs**: Platform-specific directories

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

## Related Crates

- [`vx-cli`](../vx-cli/README.md) - Command-line interface
- [`vx-tool-node`](../vx-tools/vx-tool-node/README.md) - Node.js tool plugin
- [`vx-tool-uv`](../vx-tools/vx-tool-uv/README.md) - UV Python tool plugin
- [`vx-pm-npm`](../vx-package-managers/vx-pm-npm/README.md) - NPM package manager plugin
