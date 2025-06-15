# vx-tool-node

[![Crates.io](https://img.shields.io/crates/v/vx-tool-node.svg)](https://crates.io/crates/vx-tool-node)
[![Documentation](https://docs.rs/vx-tool-node/badge.svg)](https://docs.rs/vx-tool-node)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Node.js tool support for the vx universal tool manager.

## Overview

`vx-tool-node` provides Node.js runtime support for vx, enabling automatic installation, version management, and execution of Node.js and npm commands through the vx interface.

## Features

- **Node.js Runtime**: Full Node.js runtime support with version management
- **NPM Integration**: Built-in npm package manager support
- **NPX Support**: Package runner functionality for one-time tool execution
- **Auto-Installation**: Automatic download and installation of Node.js versions
- **Cross-Platform**: Windows, macOS, and Linux support
- **Version Management**: Install and switch between multiple Node.js versions
- **LTS Support**: Automatic detection and installation of LTS versions

## Supported Commands

### Node.js Runtime
```bash
# Execute Node.js scripts
vx node script.js
vx node --version
vx node -e "console.log('Hello, World!')"

# Interactive REPL
vx node
```

### NPM Package Manager
```bash
# Package management
vx npm install express
vx npm install -g typescript
vx npm uninstall lodash
vx npm update

# Project management
vx npm init
vx npm run dev
vx npm test
vx npm publish

# Information commands
vx npm list
vx npm outdated
vx npm audit
```

### NPX Package Runner
```bash
# Run packages without installing
vx npx create-react-app my-app
vx npx typescript --init
vx npx cowsay "Hello from vx!"

# Run specific versions
vx npx typescript@4.9.5 --version
vx npx -p typescript@latest tsc --version
```

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

## Related Crates

- [`vx-core`](../../vx-core/README.md) - Core functionality
- [`vx-cli`](../../vx-cli/README.md) - Command-line interface
- [`vx-pm-npm`](../../vx-package-managers/vx-pm-npm/README.md) - NPM package manager
- [`vx-tool-uv`](../vx-tool-uv/README.md) - UV Python tool
