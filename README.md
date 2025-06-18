# 🚀 vx - Universal Development Tool Manager

<div align="center">

**The Ultimate Development Tool Manager - One Tool to Rule Them All**

[中文文档](README_zh.md) | [📖 Documentation](https://docs.rs/vx) | [🚀 Quick Start](#-quick-start) | [💡 Examples](#-real-world-examples)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![Test](https://github.com/loonghao/vx/workflows/Test/badge.svg)](https://github.com/loonghao/vx/actions)
[![Release](https://github.com/loonghao/vx/workflows/Release/badge.svg)](https://github.com/loonghao/vx/actions)
[![codecov](https://codecov.io/gh/loonghao/vx/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/vx)
[![Security audit](https://github.com/loonghao/vx/workflows/Security%20audit/badge.svg)](https://github.com/loonghao/vx/actions)
[![GitHub release](https://img.shields.io/github/release/loonghao/vx.svg)](https://github.com/loonghao/vx/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/loonghao/vx/total.svg)](https://github.com/loonghao/vx/releases)
[![Crates.io](https://img.shields.io/crates/v/vx.svg)](https://crates.io/crates/vx)
[![Documentation](https://docs.rs/vx/badge.svg)](https://docs.rs/vx)

*Lightning-fast, format-agnostic development tool manager with beautiful progress tracking*

</div>

---

## 🎯 What is vx?

**vx** is a powerful, fast, and extensible development tool manager that provides a unified interface for managing, installing, and executing development tools across different languages and ecosystems. Think of it as a combination of `nvm`, `rustup`, `pyenv`, and package managers, all in one lightning-fast tool.

## 💡 Design Philosophy

### The Problem We Solve

Every time we start a new development project, we face the same frustrating cycle:
- Install Node.js and npm for frontend tools
- Set up Python and pip/uv for scripts and automation
- Configure Go for backend services
- Manage Rust toolchain for system tools
- Deal with version conflicts and PATH issues
- Repeat this process across different machines and environments

**With the rise of MCP (Model Context Protocol)**, this problem has become even more pronounced. Many MCP servers require `uvx` for Python tools and `npx` for Node.js packages, forcing developers to manage multiple tool ecosystems just to get AI assistance working.

### Our Solution: Zero Learning Curve

vx eliminates this complexity while maintaining **zero learning curve**:

```bash
# Instead of learning and managing multiple tools:
npx create-react-app my-app     # Requires Node.js setup
uvx ruff check .                # Requires Python/UV setup
go run main.go                  # Requires Go installation

# Just use vx with the same commands you already know:
vx npx create-react-app my-app  # Auto-installs Node.js if needed
vx uvx ruff check .             # Auto-installs UV if needed
vx go run main.go               # Auto-installs Go if needed
```

### 🌟 Why vx?

- **🔄 Universal Interface**: Execute any supported tool through a single, consistent interface
- **📚 Zero Learning Curve**: Use the exact same commands you already know (`npx`, `uvx`, `go`, etc.)
- **⚡ Lightning Fast**: Built in Rust with async-first architecture for maximum performance
- **🚀 Auto-Installation**: Automatically download and install missing tools with beautiful progress bars
- **🔒 Environment Isolation**: All tools run in vx-managed environments (no system PATH conflicts)
- **📦 Format Agnostic**: Supports ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and raw binaries
- **🎨 Beautiful UX**: Rich progress bars, colorful output, and intuitive commands
- **🤖 MCP Ready**: Perfect for MCP servers - just prefix commands with `vx`

### 🚀 Latest Improvements

- **🏗️ Modular Architecture**: Complete rewrite with vx-installer engine for better maintainability
- **📊 Advanced Progress Tracking**: Beautiful progress bars with ETA and transfer rates
- **🔧 Enhanced Installation System**: Support for multiple archive formats and installation methods
- **🔌 Plugin System**: Extensible architecture with built-in and external plugin support
- **🛡️ Security First**: Built-in checksum verification and secure downloads
- **🌍 Cross-Platform**: Seamless operation on Windows, macOS, and Linux

## ✨ Features

### 🎯 Core Features
- **🔄 Universal Interface**: Execute any supported tool through a single, consistent interface
- **📦 Multi-Version Management**: Install, manage, and switch between multiple versions of tools
- **⚡ Zero Configuration**: Works out of the box with intelligent defaults
- **🚀 Auto-Installation**: Automatically download and install missing tools with beautiful progress tracking
- **🎯 Project-Specific**: Support for project-level tool configurations
- **🔌 Plugin Architecture**: Modular design with extensible plugin system

### 🎨 Enhanced CLI Experience
- **📊 Beautiful Progress Bars**: Rich progress bars with ETA, transfer rates, and visual feedback
- **🌈 Colorful Output**: Better visual distinction with colored messages and emojis
- **⏳ Smooth Animations**: Elegant loading indicators and spinner animations
- **🤝 Interactive Experience**: User-friendly prompts and confirmation dialogs
- **💡 Smart Error Messages**: Helpful suggestions and clear error reporting with recovery hints
- **🔧 Environment Control**: `--use-system-path` flag for flexible tool execution

### 🛠️ Advanced Installation Engine
- **📦 Universal Format Support**: ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and raw binaries
- **🔒 Security First**: Built-in checksum verification and secure HTTPS downloads
- **⚡ Async Performance**: Lightning-fast concurrent downloads and installations
- **🎨 Customizable Progress**: Multiple progress styles (default, simple, minimal, custom)
- **🔧 Flexible Methods**: Support for archives, binaries, scripts, and package managers
- **🌍 Cross-Platform**: Seamless operation across Windows, macOS, and Linux

### 🏗️ Modern Architecture
- **📊 Package Management**: Chocolatey-like layered package management system
- **🔍 Smart Discovery**: Automatic tool detection and intelligent version resolution
- **⚙️ Configuration Management**: Global and project-level configuration with TOML support
- **📈 Dependency Tracking**: Advanced dependency management and conflict resolution
- **🧹 Maintenance Tools**: Automated cleanup of orphaned packages and cache management
- **📋 Rich CLI**: Comprehensive command-line interface with detailed help and examples

## 🚀 Quick Start

### Installation

#### Quick Install (Recommended)

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

#### Advanced Installation Options

**Install specific version:**
```bash
# Linux/macOS
VX_VERSION="0.1.0" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_VERSION="0.1.0"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

**Install to custom directory:**
```bash
# Linux/macOS
VX_INSTALL_DIR="$HOME/bin" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_INSTALL_DIR="C:\tools\vx"; powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

#### Package Managers

**Chocolatey (Windows):**
```powershell
choco install vx
```

**Scoop (Windows):**
```powershell
scoop bucket add loonghao https://github.com/loonghao/scoop-vx.git
scoop install vx
```

**WinGet (Windows):**
```powershell
winget install loonghao.vx
```

**Homebrew (macOS):**
```bash
brew tap loonghao/vx
brew install vx
```

**Arch Linux (AUR):**
```bash
# Using yay
yay -S vx-bin

# Using paru
paru -S vx-bin
```

**Cargo (from source):**
```bash
cargo install --git https://github.com/loonghao/vx
```

#### Manual Installation

Download the latest release from [GitHub Releases](https://github.com/loonghao/vx/releases) and extract to your PATH.

#### Build from Source

```bash
git clone https://github.com/loonghao/vx
cd vx

# Build and install using the installer (recommended)
# Linux/macOS
BUILD_FROM_SOURCE=true ./install.sh

# Windows
.\install.ps1 -BuildFromSource

# Or build manually
cargo build --release
cp target/release/vx ~/.local/bin/  # Linux/macOS
# copy target\release\vx.exe %USERPROFILE%\.local\bin\  # Windows
```

### ⚡ Quick Examples: Same Commands, Better Experience

```bash
# 🎯 Use the EXACT same commands you already know - just add 'vx'!

# Python development (no Python setup required)
vx uv pip install requests           # Auto-installs UV if needed
vx uvx ruff check .                  # Auto-installs ruff via UV
vx uvx black --check .               # Auto-installs black via UV

# Node.js development (no Node.js setup required)
vx npm install react                 # Auto-installs Node.js if needed
vx npx create-react-app my-app       # Auto-installs create-react-app
vx npx -y cowsay "Hello from vx!"    # One-time tool execution

# Go development (no Go setup required)
vx go build                          # Auto-installs Go if needed
vx go run main.go                    # Same commands you know

# Rust development (no Rust setup required)
vx cargo run                         # Auto-installs Rust if needed
vx cargo build --release             # Same Cargo commands

# 🤖 Perfect for MCP servers - just prefix with 'vx':
# Instead of: npx @browsermcp/mcp@latest
# Use:        vx npx @browsermcp/mcp@latest
# Instead of: uvx some-python-tool
# Use:        vx uvx some-python-tool

# 🔧 Advanced features when you need them
vx --use-system-path python --version  # Use system tools when needed
vx list                               # Show all available tools
vx stats                              # Package statistics and usage

# 🎯 Version management with beautiful progress bars
vx install uv@0.7.12                 # Install specific versions
vx install node@20.0.0               # Rich progress tracking
vx switch node@18.19.0               # Instant version switching

# ⚙️ Project configuration
vx init                               # Initialize project configuration
vx config                             # Manage global settings
```

## 📖 Supported Tools

### 🔧 Built-in Plugins

| Tool | Commands | Category | Auto-Install | Progress Bars | Description |
|------|----------|----------|--------------|---------------|-------------|
| **UV** | `vx uv pip`, `vx uv venv`, `vx uv run`, `vx uv add` | Python | ✅ | ✅ | Extremely fast Python package installer |
| **UVX** | `vx uvx <package>`, `vx uvx ruff`, `vx uvx black` | Python | ✅ | ✅ | Python application runner (via UV) |
| **Node.js** | `vx node`, `vx npm`, `vx npx` | JavaScript | ✅ | ✅ | JavaScript runtime and package manager |
| **NPX** | `vx npx <package>`, `vx npx create-react-app` | JavaScript | ✅ | ✅ | Node.js package runner |
| **Go** | `vx go build`, `vx go run`, `vx go test` | Go | ✅ | ✅ | Go programming language toolchain |
| **Rust** | `vx cargo build`, `vx cargo run`, `vx cargo test` | Rust | ✅ | ✅ | Rust programming language and Cargo |

### 🎯 Installation Features

- **📊 Beautiful Progress Bars**: Real-time download progress with ETA and transfer rates
- **🔒 Secure Downloads**: HTTPS-only with automatic checksum verification
- **📦 Multiple Formats**: ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and raw binaries
- **⚡ Concurrent Operations**: Parallel downloads for maximum speed
- **🎨 Customizable Styles**: Choose from default, simple, minimal, or custom progress styles
- **🔧 Flexible Methods**: Archives, binaries, scripts, and package managers

### 🏗️ Plugin Categories
- **Languages**: Go, Rust, Node.js, Python (via UV)
- **Package Managers**: npm, Cargo, UV (pip-compatible)
- **Package Runners**: npx, uvx (with complete environment isolation)
- **Build Tools**: Go build, Cargo, and language-specific toolchains
- **Runtimes**: Node.js with automatic version management

## 🚀 vx-installer Engine

vx is powered by the **vx-installer** engine, a state-of-the-art installation system that provides:

### ✨ Advanced Installation Features

- **📊 Beautiful Progress Tracking**: Rich progress bars with ETA, transfer rates, and visual feedback
- **📦 Universal Format Support**: ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, and raw binaries
- **⚡ Lightning Performance**: Async-first architecture with concurrent downloads
- **🔒 Security First**: Built-in checksum verification and secure HTTPS downloads
- **🎨 Customizable Experience**: Multiple progress styles and installation methods

### 🎯 Installation Methods

| Method | Description | Use Case | Progress |
|--------|-------------|----------|----------|
| **Archive** | Extract from compressed archives | Tools distributed as ZIP/TAR | ✅ |
| **Binary** | Direct binary installation | Single executable tools | ✅ |
| **Script** | Run installation scripts | Custom installation logic | ✅ |
| **Package Manager** | Use system package managers | System-wide installations | ✅ |

### 📈 Performance Benchmarks

| Operation | Archive Size | Time | Memory Usage |
|-----------|-------------|------|--------------|
| Download | 50MB | 2.3s | 8MB |
| Extract ZIP | 100MB | 1.8s | 12MB |
| Extract TAR.GZ | 100MB | 2.1s | 10MB |
| Install Binary | 25MB | 0.5s | 4MB |

*Benchmarks on Intel i7-10700K, 32GB RAM, SSD storage*

## 🔌 MCP Integration: The Perfect Solution

vx was designed with MCP (Model Context Protocol) in mind. Many MCP servers require `uvx` and `npx`, but setting up these tools can be complex and error-prone. vx solves this with **zero configuration** and **zero learning curve**.

### The MCP Challenge

MCP servers often require multiple tool ecosystems:
```bash
# Traditional setup requires managing multiple tools:
npm install -g some-package     # Requires Node.js setup
uvx install some-python-tool    # Requires Python/UV setup
# Plus dealing with PATH conflicts, version mismatches, etc.
```

### The vx Solution: Just Add `vx`

With vx, you simply prefix your existing commands with `vx` - **no learning curve, no configuration**:

### Before (Complex Setup Required)
```json
{
  "mcpServers": {
    "browsermcp": {
      "command": "npx",
      "args": ["-y", "@browsermcp/mcp@latest"]
    },
    "python-tool": {
      "command": "uvx",
      "args": ["some-python-tool@latest"]
    }
  }
}
```

### After (Zero Setup with vx)
```json
{
  "mcpServers": {
    "browsermcp": {
      "command": "vx",
      "args": ["npx", "-y", "@browsermcp/mcp@latest"]
    },
    "python-tool": {
      "command": "vx",
      "args": ["uvx", "some-python-tool@latest"]
    }
  }
}
```

### 🎯 What You Get

- **📚 Zero Learning Curve**: Use the exact same `npx` and `uvx` commands you already know
- **🚀 Zero Configuration**: No need to install Node.js, Python, UV, or manage PATH
- **🔒 Complete Isolation**: MCP tools run in isolated environments, no conflicts
- **📊 Beautiful Progress**: See exactly what's happening with rich progress bars
- **🛡️ Security First**: Automatic checksum verification and secure downloads
- **🌍 Cross-Platform**: Identical behavior on Windows, macOS, and Linux
- **⚡ Lightning Fast**: Concurrent downloads and installations

## 🏗️ Project Architecture

vx is built with a modern, modular architecture that emphasizes performance, extensibility, and maintainability. The recent vx-installer integration brings state-of-the-art installation capabilities.

### 📦 Core Components

```
vx/
├── vx-cli/              # Command-line interface with rich UX
├── vx-core/             # Core functionality and utilities
├── vx-installer/        # 🆕 Universal installation engine
├── vx-config/           # Configuration management (TOML-based)
├── vx-plugin/           # Plugin system and trait definitions
├── vx-tools/            # Built-in tool plugins
│   ├── vx-tool-node/    # Node.js support with NPX integration
│   ├── vx-tool-go/      # Go toolchain support
│   ├── vx-tool-rust/    # Rust and Cargo support
│   └── vx-tool-uv/      # UV (Python) with UVX support
└── vx-package-managers/ # Package manager plugins
    └── vx-pm-npm/       # NPM package manager integration
```

### 🚀 vx-installer Engine

The heart of vx's installation system, providing:

- **📦 Format Handlers**: ZIP, TAR.GZ, TAR.XZ, TAR.BZ2, Binary
- **📊 Progress System**: Beautiful progress bars with customizable styles
- **🔒 Security Layer**: Checksum verification and secure downloads
- **⚡ Async Core**: High-performance concurrent operations
- **🔧 Extensible**: Plugin-based format and method support

### 🎯 Design Principles

- **🔌 Plugin Architecture**: Extensible design with trait-based plugins
- **⚡ Performance First**: Rust-powered with async-first operations
- **🛡️ Safety & Security**: Memory safety, error handling, and secure downloads
- **🔧 Modularity**: Clean separation of concerns with focused crates
- **📦 Composability**: Mix and match components as needed
- **🎨 User Experience**: Beautiful CLI with progress tracking and helpful messages

## ⚙️ Configuration

### Global Configuration

`~/.config/vx/config.toml`:

```toml
[defaults]
auto_install = true        # Auto-install missing tools
check_updates = true       # Check for updates
update_interval = "24h"    # Update check frequency

[tools.uv]
version = "0.5.26"
install_method = "official"

[tools.node]
version = "20.11.0"
install_method = "official"

[tools.go]
version = "1.21.6"
```

### Project Configuration

`.vx.toml`:

```toml
[tools]
uv = "0.5.26"
node = "20.11.0"
go = "1.21.6"

[defaults]
auto_install = true
```

### Plugin Configuration

```bash
# List all plugins
vx plugin list

# Get plugin info
vx plugin info uv

# Enable/disable plugins
vx plugin enable rust
vx plugin disable go

# Search plugins
vx plugin search python
```

## 🎯 Real-World Examples

### Python Development with UV
```bash
# Create a new Python project
vx uv init my-python-app
cd my-python-app

# Add dependencies
vx uv add fastapi uvicorn
vx uv add --dev pytest black

# Run the application
vx uv run uvicorn main:app --reload

# Run tests
vx uv run pytest

# Use uvx for Python applications (with environment isolation)
vx uvx ruff check .
vx uvx black --check .
vx uvx cowsay -t "Hello from vx uvx!"

# All tools run in vx-managed environments
vx uv --version  # Uses vx-managed uv
```

### Node.js Development
```bash
# Install and use Node.js
vx npm install express
vx node server.js

# Use npx for one-time tools (with environment isolation)
vx npx create-react-app my-app
vx npx -y typescript --init
vx npx cowsay "Hello from vx!"

# All tools run in vx-managed environments
vx npm --version  # Uses vx-managed npm
vx node --version # Uses vx-managed Node.js
```

### Go Development
```bash
# Initialize Go module
vx go mod init my-go-app

# Build and run
vx go build
vx go run main.go

# Test
vx go test ./...
```

### Rust Development
```bash
# Create new Rust project
vx cargo new my-rust-app
cd my-rust-app

# Add dependencies
vx cargo add serde tokio

# Build and run
vx cargo run
```

### Multi-Language Project
```bash
# Frontend (Node.js) + Backend (Go) + Scripts (Python)
vx npm install          # Frontend dependencies
vx go mod tidy          # Backend dependencies
vx uv pip install -r requirements.txt  # Script dependencies

# Run different parts
vx npm run dev          # Frontend dev server
vx go run cmd/server/main.go  # Backend server
vx uv run python scripts/deploy.py  # Deployment script
```

## 📊 Package Management

### Multi-Version Support
```bash
# Install multiple versions
vx install go@1.20.0
vx install go@1.21.6

# List installed versions
vx stats

# Switch between versions
vx switch go@1.20.0
vx switch go@1.21.6

# Remove specific versions
vx remove go 1.20.0
vx remove go --all

# Cleanup orphaned packages
vx cleanup
```

### Package Statistics
```bash
# View package statistics
vx stats
# Output:
# 📊 Package Statistics:
#   📦 Total packages: 3
#   🔢 Total versions: 5
#   💾 Total size: 2.1 GB
#   🕒 Last updated: 2025-01-30 10:30:00 UTC
```

## 🛠️ Development

### Prerequisites

- Rust 1.70+
- Cargo

### Building

```bash
git clone https://github.com/loonghao/vx
cd vx
cargo build --release
```

### Testing

```bash
cargo test
cargo run -- --help
```

### Plugin Development

See [MODULAR_ARCHITECTURE.md](MODULAR_ARCHITECTURE.md) for detailed plugin development guide.

## 🚀 Roadmap

### Current Status (v0.2.x)
- ✅ **Core plugin architecture** with trait-based extensibility
- ✅ **6 built-in tools** (UV, UVX, Node.js, NPX, Go, Rust)
- ✅ **Environment isolation system** with complete PATH management
- ✅ **🆕 vx-installer engine** with universal format support
- ✅ **🆕 Beautiful progress bars** with ETA and transfer rates
- ✅ **🆕 Security-first downloads** with checksum verification
- ✅ **🆕 Async installation system** with concurrent operations
- ✅ **Multi-version package management** with intelligent switching
- ✅ **MCP integration support** for seamless proxy usage
- ✅ **Package runner support** (npx, uvx) with environment isolation
- ✅ **Project configuration support** with TOML-based configs

### Upcoming Features
- [ ] **Enhanced Package Managers**: pnpm, yarn, bun with full vx-installer integration
- [ ] **System Package Managers**: Homebrew, Chocolatey, apt, yum support
- [ ] **Specialized Tools**: Rez for VFX, Spack for HPC environments
- [ ] **External Plugin Support**: .dll, .so, and script-based plugins
- [ ] **Plugin Marketplace**: Community-driven plugin ecosystem
- [ ] **Advanced Installation Methods**: Docker, containers, and virtual environments
- [ ] **GUI Interface**: Desktop application with visual tool management
- [ ] **CI/CD Integrations**: GitHub Actions, GitLab CI, Jenkins plugins
- [ ] **Team Configuration Sync**: Shared configurations and tool versions
- [ ] **Performance Optimizations**: Caching, parallel operations, and smart updates

## 🤝 Contributing

We welcome contributions! Here's how you can help:

1. **Report Issues**: Found a bug? [Open an issue](https://github.com/loonghao/vx/issues)
2. **Feature Requests**: Have an idea? [Start a discussion](https://github.com/loonghao/vx/discussions)
3. **Plugin Development**: Create plugins for new tools
4. **Documentation**: Improve docs and examples
5. **Code Contributions**: Submit pull requests

### 🚀 Release Process

This project uses [Release Please](https://github.com/googleapis/release-please) for automated releases:

- **Follow [Conventional Commits](https://www.conventionalcommits.org/) specification**
- **Automatic versioning**: Version bumps based on commit types
- **Automatic changelog**: Generated from commit history
- **Automatic releases**: GitHub releases created when Release PR is merged

```bash
# New feature (bumps minor version)
git commit -m "feat: add Python plugin support"

# Bug fix (bumps patch version)
git commit -m "fix: resolve installation issue on Windows"

# Breaking change (bumps major version)
git commit -m "feat!: redesign plugin API"
```

See [Release Guide](docs/RELEASE_GUIDE.md) for detailed guidelines.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by tools like `asdf`, `mise`, `proto`, and `chocolatey`
- Built with ❤️ using Rust and modern development practices
- Special thanks to the Rust community and all contributors

## 📞 Support

- 📖 **Documentation**: [Full documentation](https://github.com/loonghao/vx/wiki)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/loonghao/vx/discussions)
- 🐛 **Issues**: [Bug Reports](https://github.com/loonghao/vx/issues)
- 📧 **Contact**: hal.long@outlook.com

---

**Made with ❤️ for developers, by developers**

