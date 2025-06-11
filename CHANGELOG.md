# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Features

- **Virtual Environment Support**: Added `vx venv` command for creating and managing isolated development environments
  - `vx venv create <name>` - Create new virtual environment with specific tool versions
  - `vx venv activate <name>` - Generate activation script for shell integration
  - `vx venv list` - List all virtual environments
  - `vx venv remove <name>` - Remove virtual environment
  - `vx venv current` - Show current active environment
- **Rust Toolchain Separation**: Split Rust tool into separate `cargo` and `rustc` tools
  - `vx cargo` - Rust package manager and build tool
  - `vx rustc` - Rust compiler
- **Environment Isolation Improvements**: Enhanced tool execution to better support isolated environments
- Initial implementation of vx - Universal Development Tool Manager
- Support for UV (Python package manager)
- Support for Node.js and npm
- Support for Go toolchain
- Support for Rust and Cargo
- Plugin architecture for extensibility
- Multi-platform support (Linux, macOS, Windows, FreeBSD)
- Automatic tool installation and version management
- Project-specific configuration support

### Documentation

- Comprehensive README with installation instructions
- Chinese translation (README_zh.md)
- Plugin documentation and examples

### Build System

- GoReleaser configuration for multi-platform releases
- GitHub Actions CI/CD pipeline
- Docker image support
- Package manager integration (Homebrew, Scoop)

## [0.1.0] - 2025-01-09

### Features

- Initial release of vx
- Basic plugin system
- Core tool support (UV, Node.js, Go, Rust)
- Command-line interface
- Configuration management

[Unreleased]: https://github.com/loonghao/vx/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/loonghao/vx/releases/tag/v0.1.0
