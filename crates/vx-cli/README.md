# 🚀 vx-cli

<div align="center">

**Beautiful Command-Line Interface for the vx Universal Tool Manager**

[![Crates.io](https://img.shields.io/crates/v/vx-cli.svg)](https://crates.io/crates/vx-cli)
[![Documentation](https://docs.rs/vx-cli/badge.svg)](https://docs.rs/vx-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/loonghao/vx/workflows/CI/badge.svg)](https://github.com/loonghao/vx/actions)

*Lightning-fast CLI with beautiful progress bars and intelligent tool management*

</div>

## 🎯 Overview

`vx-cli` provides the beautiful command-line interface for vx, a universal development tool manager. It offers a unified interface for managing, installing, and executing development tools across different languages and ecosystems, powered by the state-of-the-art **vx-installer** engine.

## ✨ Features

### 🔧 Core Functionality

- **Universal Tool Execution**: Run any supported tool through a single, consistent interface
- **🚀 Enhanced Auto-Installation**: Download and install missing tools with beautiful progress bars
- **Version Management**: Install, switch, and manage multiple tool versions seamlessly
- **Virtual Environments**: Create isolated environments for projects with complete PATH management

### 🎨 Enhanced User Experience

- **📊 Beautiful Progress Bars**: Rich progress tracking with ETA, transfer rates, and visual feedback
- **🌈 Colorful Output**: Intuitive color-coded messages and status indicators
- **⚡ Lightning Performance**: Async-first architecture with concurrent operations
- **🔒 Security First**: Automatic checksum verification and secure HTTPS downloads
- **💡 Smart Error Messages**: Helpful suggestions and clear error reporting with recovery hints

### 🛠️ Advanced Features

- **Project Configuration**: Support for project-specific tool configurations with TOML
- **Shell Integration**: Auto-completion and shell hooks for all major shells
- **📦 Universal Format Support**: ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and raw binaries
- **🎯 Flexible Installation**: Support for archives, binaries, scripts, and package managers

## Installation

### From Crates.io

```bash
cargo install vx-cli
```

### From Source

```bash
git clone https://github.com/loonghao/vx
cd vx
cargo install --path crates/vx-cli
```

## Quick Start

### ⚡ Basic Usage with Enhanced Experience

```bash
# 🚀 Execute tools transparently with auto-installation and progress bars
vx node --version                    # Beautiful progress if Node.js needs installation
vx uv pip install requests          # Rich progress tracking for downloads
vx go build                          # Automatic Go installation with checksum verification

# 📦 Install specific versions with visual feedback
vx install node@18.17.0             # Progress bars with ETA and transfer rates
vx install uv@latest                # Secure downloads with automatic verification

# 📊 List available tools with rich formatting
vx list                              # Colorful output with status indicators
vx list --installed                  # Show only installed tools with versions

# 🎯 Create virtual environment with beautiful setup
vx venv create myproject --tools node@18.17.0,uv@latest
```

### Project Configuration

```bash
# Initialize project configuration
vx init

# Edit vx.toml
echo '[tools]
node = "18.17.0"
uv = "latest"' > vx.toml

# Sync project tools
vx sync
```

## 🚀 Enhanced Installation Experience

vx-cli is powered by the **vx-installer** engine, providing a state-of-the-art installation experience:

### 📊 Beautiful Progress Tracking

```bash
# When installing tools, you'll see beautiful progress bars like:
# 🚀 Downloading Node.js v18.17.0...
# ⬇️  [████████████████████████████████] 45.2MB/45.2MB (2.3MB/s, 0s remaining)
# 📦 Extracting archive...
# ✅ Node.js v18.17.0 installed successfully!

vx install node@18.17.0
```

### 🔒 Security & Verification

```bash
# All downloads include automatic security features:
# - HTTPS-only downloads
# - Automatic checksum verification
# - Secure archive extraction
# - Permission validation

vx install go@1.21.0  # Automatically verified for integrity
```

### 📦 Universal Format Support

```bash
# vx-cli handles multiple archive formats seamlessly:
# - ZIP archives (Windows tools)
# - TAR.GZ archives (Unix tools)
# - TAR.XZ archives (compressed tools)
# - Raw binaries (single executables)

vx install uv@latest  # Automatically detects and handles format
```

## Commands

### Tool Execution

```bash
# Direct tool execution (transparent proxy)
vx <tool> [args...]

# Examples
vx node --version
vx npm install express
vx uv pip install requests
vx go build ./...
vx cargo test
```

### Tool Management

```bash
# Install tools
vx install <tool>[@version]
vx install node@18.17.0 uv@latest

# List tools
vx list                    # All available tools
vx list --installed        # Only installed tools
vx list node              # Specific tool versions

# Update tools
vx update                 # Update all tools
vx update node            # Update specific tool

# Remove tools
vx remove node@18.17.0    # Remove specific version
vx remove node --all      # Remove all versions

# Search tools
vx search python          # Search for tools
vx search --category python
```

### Virtual Environments

```bash
# Create environments
vx venv create myproject
vx venv create myproject --tools node@18.17.0,uv@latest

# Use environments
vx venv use myproject
vx venv run myproject node --version

# Manage environments
vx venv list              # List all environments
vx venv remove myproject  # Remove environment
```

### Project Management

```bash
# Initialize project
vx init                   # Interactive setup
vx init --template node   # Use template

# Sync project tools
vx sync                   # Install project tools
vx sync --check           # Check without installing

# Configuration
vx config                 # Show current config
vx config edit            # Edit global config
vx config edit --local    # Edit project config
```

### Maintenance

```bash
# Cache Statistics
vx cache info             # Show cache usage statistics
vx cache info --detailed  # Detailed cache information

# Cleanup
vx cleanup                # Clean orphaned files
vx cleanup --dry-run      # Preview cleanup

# Global tool management
vx global list            # List global tools
vx global cleanup         # Clean unused tools
```

## Configuration

### Global Configuration

Location: `~/.vx/config/global.toml`

```toml
[auto_install]
enabled = true
timeout = 300
confirm_before_install = false

[settings]
cache_duration = "7d"
parallel_downloads = 4
use_system_path = false

[ui]
show_progress = true
use_colors = true
```

### Project Configuration

Location: `vx.toml` in project root

```toml
[tools]
node = "18.17.0"
uv = "latest"
go = "^1.21.0"

[settings]
auto_install = true
cache_duration = "7d"

[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"
```

## Shell Integration

### Bash/Zsh

```bash
# Add to ~/.bashrc or ~/.zshrc
eval "$(vx shell-init)"
source <(vx completion bash)  # or zsh
```

### Fish

```fish
# Add to ~/.config/fish/config.fish
vx shell-init | source
vx completion fish | source
```

### PowerShell

```powershell
# Add to PowerShell profile
Invoke-Expression (vx shell-init)
vx completion powershell | Out-String | Invoke-Expression
```

## Supported Tools

### Languages & Runtimes

- **Node.js**: JavaScript runtime and npm
- **Python**: UV package manager and Python tools
- **Go**: Go compiler and tools
- **Rust**: Rust compiler and Cargo

### Package Managers

- **npm**: Node.js package manager
- **UV**: Fast Python package manager
- **Cargo**: Rust package manager

### Package Runners

- **npx**: Node.js package runner
- **uvx**: Python application runner

## 🏗️ Architecture

vx-cli is built on top of a modern, modular architecture:

### Core Components

- **vx-core**: Core traits and functionality with enhanced error handling
- **🆕 vx-installer**: Universal installation engine with progress tracking
- **vx-config**: Advanced configuration management with TOML support
- **vx-plugin**: Extensible plugin system with trait-based design

### Plugin Ecosystem

- **Tool Plugins**: Individual tool implementations (Node.js, Go, Rust, UV)
- **Package Manager Plugins**: Package manager integrations (npm, Cargo)
- **UI Components**: Rich terminal interface with beautiful progress bars

### Installation Pipeline

```
User Command → vx-cli → vx-core → vx-installer → Tool Installation
                                      ↓
                              Progress Tracking & Security
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
cargo run -- --help
```

### Debugging

```bash
# Enable verbose logging
RUST_LOG=debug cargo run -- <command>

# Or use the verbose flag
cargo run -- --verbose <command>
```

## Error Handling

vx-cli provides detailed error messages and suggestions:

```bash
$ vx nonexistent-tool
Error: Tool 'nonexistent-tool' not found

Suggestions:
  - Run 'vx list' to see available tools
  - Run 'vx search nonexistent' to search for similar tools
  - Check if the tool name is spelled correctly
```

## ⚡ Performance

### Enhanced Performance Features

- **🚀 Lightning Startup**: Optimized for sub-second command execution
- **📊 Concurrent Operations**: Multiple tools downloaded and installed simultaneously
- **💾 Smart Caching**: Version information and downloads intelligently cached
- **🔧 Lazy Loading**: Plugins loaded only when needed for minimal overhead
- **⚡ Async Architecture**: Non-blocking operations with beautiful progress tracking

### Benchmarks

| Operation | Time | Memory | Notes |
|-----------|------|--------|-------|
| Tool execution | <100ms | 8MB | Cached tools |
| First-time install | 2-5s | 12MB | With progress bars |
| Version switching | <50ms | 4MB | Symlink-based |
| Configuration load | <10ms | 2MB | TOML parsing |

## Troubleshooting

### Common Issues

1. **Tool not found**: Run `vx list` to see available tools
2. **Installation fails**: Check network connection and permissions
3. **Version conflicts**: Use `vx cleanup` to remove orphaned versions
4. **Shell integration**: Ensure shell configuration is properly loaded

### Debug Mode

```bash
# Enable debug logging
vx --verbose <command>

# Check configuration
vx config --sources

# Verify installation
vx cache info
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../CONTRIBUTING.md) for more information.

## 🔗 Related Crates

- [`vx-installer`](../vx-installer/README.md) - 🆕 Universal installation engine with progress tracking
- [`vx-core`](../vx-core/README.md) - Core functionality and utilities
- [`vx-config`](../vx-config/README.md) - Configuration management system
- [`vx-plugin`](../vx-plugin/README.md) - Plugin system and trait definitions
- [`vx-tool-node`](../vx-tools/vx-tool-node/README.md) - Node.js plugin with NPX support
- [`vx-tool-uv`](../vx-tools/vx-tool-uv/README.md) - UV Python plugin with UVX support
- [`vx-pm-npm`](../vx-package-managers/vx-pm-npm/README.md) - NPM package manager plugin

---

<div align="center">

**Experience the future of development tool management**

[🚀 Get Started](../../README.md#-quick-start) | [📖 Documentation](https://docs.rs/vx-cli) | [🤝 Contributing](../../CONTRIBUTING.md)

</div>
