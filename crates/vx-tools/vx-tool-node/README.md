# 🟢 vx-tool-node

<div align="center">

**Node.js Tool Plugin for vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-tool-node.svg)](https://crates.io/crates/vx-tool-node)
[![Documentation](https://docs.rs/vx-tool-node/badge.svg)](https://docs.rs/vx-tool-node)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Beautiful Node.js installation and management with zero configuration*

</div>

## 🎯 Overview

`vx-tool-node` provides comprehensive Node.js runtime support for vx, enabling automatic installation, version management, and execution of Node.js and npm commands through the vx interface. Enhanced with the **vx-installer** engine for beautiful installation experiences.

## ✨ Features

### 🚀 Enhanced Installation Experience
- **📊 Beautiful Progress Bars**: Rich progress tracking with ETA and transfer rates during Node.js installation
- **🔒 Security First**: Automatic checksum verification and secure HTTPS downloads
- **📦 Universal Format Support**: Handles multiple Node.js distribution formats automatically
- **⚡ Lightning Fast**: Concurrent downloads and async installation process

### 🔧 Core Node.js Features
- **Node.js Runtime**: Full Node.js runtime support with intelligent version management
- **NPM Integration**: Built-in npm package manager with complete feature support
- **NPX Support**: Package runner functionality for one-time tool execution with environment isolation
- **Auto-Installation**: Zero-configuration automatic download and installation of Node.js versions
- **Cross-Platform**: Seamless operation on Windows, macOS, and Linux
- **Version Management**: Install and switch between multiple Node.js versions instantly
- **LTS Support**: Automatic detection and installation of LTS versions with recommendations

## 🚀 Installation Experience

When you first use Node.js through vx, you'll see a beautiful installation process:

```bash
$ vx node --version

🚀 Installing Node.js v20.11.0...
⬇️  [████████████████████████████████] 28.4MB/28.4MB (4.2MB/s, 0s remaining)
📦 Extracting Node.js archive...
🔧 Setting up Node.js environment...
✅ Node.js v20.11.0 installed successfully!

v20.11.0
```

### 🔒 Security Features

- **HTTPS-only downloads** from official Node.js releases
- **Automatic checksum verification** to ensure file integrity
- **Secure archive extraction** with path validation
- **Permission validation** before installation

## 💡 Supported Commands

### Node.js Runtime
```bash
# 🎯 Use the EXACT same Node.js commands you already know!
vx node script.js                    # Auto-installs Node.js if needed
vx node --version                    # Beautiful progress bars on first run
vx node -e "console.log('Hello!')"   # Zero configuration required

# Interactive REPL (same as always)
vx node
```

### NPM Package Manager
```bash
# 📦 Same npm commands, enhanced experience
vx npm install express              # Auto-installs Node.js + npm if needed
vx npm install -g typescript        # Global packages in isolated environment
vx npm uninstall lodash             # Same commands you know
vx npm update                        # Enhanced with progress tracking

# 🚀 Project management (zero learning curve)
vx npm init                          # Interactive project setup
vx npm run dev                       # Run scripts as usual
vx npm test                          # Testing workflows unchanged
vx npm publish                       # Publishing works the same

# 📊 Information commands with better output
vx npm list                           # Enhanced formatting
vx npm outdated                      # Better visual output
vx npm audit                         # Security audit with colors
```

### NPX Package Runner
```bash
# 🎯 Perfect for MCP servers - just add 'vx'!
vx npx create-react-app my-app       # Auto-installs create-react-app
vx npx -y cowsay "Hello from vx!"    # One-time tool execution
vx npx typescript --init             # TypeScript setup
vx npx @browsermcp/mcp@latest        # MCP server execution

# 🚀 Run specific versions with progress tracking
vx npx typescript@4.9.5 --version    # Version-specific execution
vx npx -p typescript@latest tsc --version  # Latest version usage

# 🤖 MCP Integration Example
# Instead of: npx @browsermcp/mcp@latest
# Use:        vx npx @browsermcp/mcp@latest
# Benefits: Auto-installation, environment isolation, progress tracking
```

## 🤖 MCP Integration

Perfect for MCP (Model Context Protocol) servers that require Node.js tools:

```json
{
  "mcpServers": {
    "browsermcp": {
      "command": "vx",
      "args": ["npx", "-y", "@browsermcp/mcp@latest"]
    },
    "typescript-tools": {
      "command": "vx",
      "args": ["npx", "typescript-language-server", "--stdio"]
    }
  }
}
```

### Benefits for MCP
- **Zero Setup**: No need to install Node.js manually
- **Environment Isolation**: MCP tools run in isolated environments
- **Automatic Updates**: Tools are automatically managed and updated
- **Cross-Platform**: Works identically on all platforms

## Installation

### Through vx CLI
```bash
# Install latest LTS version
vx install node

# Install specific version
vx install node@18.17.0
vx install node@20.10.0

# Install latest version
vx install node@latest
```

### Version Constraints
```bash
# Semantic version ranges
vx install node@^18.0.0    # Latest 18.x.x
vx install node@~18.17.0   # Latest 18.17.x
vx install node@>=18.0.0   # 18.0.0 or higher
```

## Configuration

### Project Configuration (.vx.toml)
```toml
[tools]
node = "18.17.0"          # Specific version
# node = "lts"            # Latest LTS
# node = "latest"         # Latest stable
# node = "^18.0.0"        # Version range

[tools.node]
auto_install = true
install_npm = true        # Install npm alongside Node.js
```

### Global Configuration
```toml
[tools.node]
default_version = "lts"
auto_install = true
install_timeout = 300
prefer_lts = true

[node.npm]
registry = "https://registry.npmjs.org/"
cache_dir = "~/.npm"
```

## Version Management

### Available Versions
The plugin supports all official Node.js releases:

- **LTS Versions**: 18.x, 20.x (recommended for production)
- **Current Versions**: Latest stable releases
- **Legacy Versions**: 16.x and older (limited support)

### Version Detection
```bash
# List available versions
vx list node

# Show current version
vx node --version

# Show installation path
vx which node
```

## Platform Support

### Windows
- **x64**: Full support
- **x86**: Legacy support
- **ARM64**: Windows 11 ARM support

### macOS
- **x64**: Intel Mac support
- **ARM64**: Apple Silicon (M1/M2) support
- **Universal**: Automatic architecture detection

### Linux
- **x64**: All major distributions
- **ARM64**: ARM-based systems
- **ARMv7**: Raspberry Pi and similar

## Integration

### With vx-core
```rust
use vx_core::{Tool, ToolManager};
use vx_tool_node::NodeTool;

let node_tool = NodeTool::new();
let manager = ToolManager::new();

// Install Node.js
manager.install_tool(&node_tool, "18.17.0").await?;

// Execute Node.js
manager.execute_tool(&node_tool, &["--version"]).await?;
```

### Plugin Registration
```rust
use vx_core::{Plugin, PluginManager};
use vx_tool_node::NodePlugin;

let plugin = NodePlugin::new();
let mut manager = PluginManager::new();

manager.register_plugin(Box::new(plugin))?;
```

## Development

### Building
```bash
cd crates/vx-tool-node
cargo build
```

### Testing
```bash
cargo test
```

### Integration Testing
```bash
# Test with actual Node.js installation
cargo test --features integration-tests
```

## Implementation Details

### Tool Structure
- **NodeTool**: Main Node.js runtime tool
- **NpmTool**: NPM package manager integration
- **NpxTool**: NPX package runner support

### Version Resolution
1. **Project Config**: Check `.vx.toml` for version specification
2. **Global Config**: Fall back to global default
3. **LTS Detection**: Use latest LTS if no version specified
4. **Auto-Install**: Download and install if not available

### Installation Process
1. **Version Lookup**: Query Node.js release API
2. **Download**: Fetch appropriate binary/installer
3. **Extraction**: Extract to vx tools directory
4. **Verification**: Verify installation integrity
5. **NPM Setup**: Configure npm if included

## Error Handling

### Common Errors
- **Network Issues**: Download failures, API timeouts
- **Permission Errors**: Installation directory access
- **Version Conflicts**: Multiple Node.js installations
- **Corruption**: Incomplete or corrupted downloads

### Recovery
```bash
# Reinstall corrupted version
vx install node@18.17.0 --force

# Clear cache and retry
vx cleanup --cache-only
vx install node@18.17.0

# Use system Node.js as fallback
vx --use-system-path node --version
```

## Performance

- **Fast Downloads**: Parallel downloading with progress tracking
- **Efficient Storage**: Shared installations across virtual environments
- **Quick Execution**: Minimal overhead for tool execution
- **Smart Caching**: Version metadata and download caching

## Security

- **Checksum Verification**: SHA256 verification of downloads
- **HTTPS Only**: Secure downloads from official sources
- **Signature Validation**: GPG signature verification (when available)
- **Sandboxed Execution**: Isolated execution environments

## Troubleshooting

### Installation Issues
```bash
# Check available versions
vx search node

# Verify network connectivity
curl -I https://nodejs.org/dist/

# Check disk space
vx stats

# Force reinstall
vx remove node@18.17.0
vx install node@18.17.0
```

### Runtime Issues
```bash
# Check Node.js installation
vx node --version
vx npm --version

# Verify PATH configuration
vx which node
vx which npm

# Test with system Node.js
vx --use-system-path node --version
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../../CONTRIBUTING.md) for more information.

## 🔗 Related Crates

- [`vx-installer`](../../vx-installer/README.md) - 🆕 Universal installation engine with progress tracking
- [`vx-core`](../../vx-core/README.md) - Core functionality and utilities
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface with rich UX
- [`vx-config`](../../vx-config/README.md) - Configuration management system
- [`vx-pm-npm`](../../vx-package-managers/vx-pm-npm/README.md) - NPM package manager plugin
- [`vx-tool-uv`](../vx-tool-uv/README.md) - UV Python tool plugin

---

<div align="center">

**Node.js development made effortless**

[🚀 Get Started](../../../README.md) | [📖 Documentation](https://docs.rs/vx-tool-node) | [🤝 Contributing](../../../CONTRIBUTING.md)

</div>
