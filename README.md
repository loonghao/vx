# vx - Universal Development Tool Manager

[中文文档](README_zh.md)

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

> 🚀 The ultimate development tool manager - One tool to rule them all

## ⚠️ Early Development Stage

**This project is currently in early experimental development stage.** Features and APIs may change significantly between versions. Use with caution in production environments.

- 🔬 **Experimental**: Core features are being actively developed and tested
- 🚧 **Breaking Changes**: APIs and configurations may change without notice
- 📝 **Feedback Welcome**: Please report issues and share your experience
- 🎯 **MVP Focus**: Currently supports UV, Node.js, Go, and Rust tools
- 🚀 **Release Automation**: Now with automated release management via release-plz

### ✅ Recently Implemented

- **NPX Support**: Full support for Node.js package runner with environment isolation
- **UVX Support**: Complete Python application runner integration via UV
- **Environment Isolation**: All tools run in vx-managed environments (no system PATH dependency)
- **Auto-Installation**: Automatic tool download and installation on first use
- **MCP Integration**: Ready for use as proxy in MCP server configurations
- **Improved CI/CD**: Enhanced release automation with release-please integration

### Current Limitations

- **Package Manager Integration**: Additional package managers (pnpm, yarn, bun) in development
- **Configuration**: Advanced project-specific configurations being enhanced
- **Plugin System**: External plugin support planned for future releases

`vx` is a powerful, fast, and extensible development tool manager that provides a unified interface for managing, installing, and executing development tools across different languages and ecosystems. Think of it as a combination of `nvm`, `rustup`, `pyenv`, and package managers, all in one lightning-fast tool.

## ✨ Features

### 🎯 Core Features
- **🔄 Universal Interface**: Execute any supported tool through a single, consistent interface
- **📦 Multi-Version Management**: Install, manage, and switch between multiple versions of tools
- **⚡ Zero Configuration**: Works out of the box with intelligent defaults
- **🚀 Auto-Installation**: Automatically download and install missing tools
- **🎯 Project-Specific**: Support for project-level tool configurations
- **🔌 Plugin Architecture**: Modular design with extensible plugin system

### 🎨 Enhanced CLI Experience
- **📊 Progress Bars**: Visual feedback for downloads and installations
- **🌈 Colorful Output**: Better visual distinction with colored messages
- **⏳ Spinner Animations**: Smooth loading indicators for operations
- **🤝 Interactive Confirmations**: User-friendly prompts and dialogs
- **💡 Smart Error Messages**: Helpful suggestions and clear error reporting
- **🔧 Environment Isolation**: `--use-system-path` flag for better control

### 🛠️ Advanced Features
- **📊 Package Management**: Chocolatey-like layered package management
- **🔍 Smart Discovery**: Automatic tool detection and version resolution
- **⚙️ Configuration Management**: Global and project-level configuration support
- **📈 Dependency Tracking**: Track and manage tool dependencies
- **🧹 Cleanup Tools**: Orphaned package cleanup and maintenance
- **📋 Rich CLI**: Comprehensive command-line interface with helpful output

## 🚀 Quick Start

### Installation

#### Quick Install (Recommended)

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/loonghao/vx/main/install-release.ps1 | iex
```

#### Package Managers

**Homebrew (macOS/Linux):**
```bash
brew install loonghao/tap/vx
```

**Scoop (Windows):**
```powershell
scoop bucket add loonghao https://github.com/loonghao/scoop-bucket
scoop install vx
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

# Standard release build
cargo build --release

# PGO-optimized build (recommended for best performance)
make build-pgo

# Windows: Run the build installer
.\install.ps1
```

### Basic Usage

```bash
# Execute tools through vx - they'll be auto-installed if missing!
vx uv pip install requests
vx npm install react
vx node app.js
vx go build
vx cargo run

# Package runners with environment isolation
vx npx create-react-app my-app
vx npx -y cowsay "Hello from vx!"
vx uvx cowsay -t "Hello from vx uvx!"
vx uvx ruff check .

# Use system PATH instead of vx-managed tools
vx --use-system-path python --version
vx --use-system-path node --version

# List supported tools and plugins
vx list
vx plugin list

# Install specific versions
vx install uv@0.7.12
vx install node@20.0.0
vx install go@1.21.6

# Switch between versions
vx switch uv@0.7.12
vx switch node@18.19.0

# Project configuration
vx init
vx config
```

## 📖 Supported Tools

### 🔧 Built-in Plugins

| Tool | Commands | Category | Auto-Install | Description |
|------|----------|----------|--------------|-------------|
| **UV** | `vx uv pip`, `vx uv venv`, `vx uv run`, `vx uv add` | Python | ✅ | Extremely fast Python package installer |
| **UVX** | `vx uvx <package>`, `vx uvx ruff`, `vx uvx black` | Python | ✅ | Python application runner (via UV) |
| **Node.js** | `vx node`, `vx npm`, `vx npx` | JavaScript | ✅ | JavaScript runtime and package manager |
| **NPX** | `vx npx <package>`, `vx npx create-react-app` | JavaScript | ✅ | Node.js package runner |
| **Go** | `vx go build`, `vx go run`, `vx go test` | Go | ✅ | Go programming language toolchain |
| **Rust** | `vx cargo build`, `vx cargo run`, `vx cargo test` | Rust | ✅ | Rust programming language and Cargo |

### 🎯 Plugin Categories
- **Languages**: Go, Rust, Node.js, Python (via UV)
- **Package Managers**: npm, Cargo, UV (pip-compatible)
- **Package Runners**: npx, uvx (with environment isolation)
- **Build Tools**: Go build, Cargo, etc.
- **Runtimes**: Node.js

## 🔌 MCP Integration

vx is designed to work seamlessly as a proxy for package runners in MCP (Model Context Protocol) server configurations, providing environment isolation and automatic tool management.

### Before (Direct Tool Usage)
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

### After (vx Proxy)
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

### Benefits
- **Environment Isolation**: No conflicts with system-installed tools
- **Automatic Installation**: Tools are downloaded and managed automatically
- **Version Control**: Consistent tool versions across environments
- **Cross-Platform**: Works identically on Windows, macOS, and Linux

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

### Current Status (v0.1.1)
- ✅ Core plugin architecture
- ✅ 6 built-in tools (UV, UVX, Node.js, NPX, Go, Rust)
- ✅ Environment isolation system
- ✅ Auto-installation system
- ✅ Multi-version package management
- ✅ MCP integration support
- ✅ Package runner support (npx, uvx)
- ✅ Project configuration support

### Upcoming Features
- [ ] More package managers (pnpm, yarn, bun)
- [ ] System package managers (Homebrew, Chocolatey)
- [ ] Specialized tools (Rez for VFX, Spack for HPC)
- [ ] External plugin support (.dll, .so, scripts)
- [ ] Plugin marketplace
- [ ] GUI interface
- [ ] CI/CD integrations
- [ ] Team configuration sync

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

