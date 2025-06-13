# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.1.2 (2025-06-13)

### Bug Fixes

* resolve release-please untagged PR issue by updating configuration ([8c13a25](https://github.com/loonghao/vx/commit/8c13a25dcdfa010b5b51cea581f22e8b07ee27a6))
* synchronize version to 0.1.1 and remove incorrect v0.2.0 tag ([42704e4](https://github.com/loonghao/vx/commit/42704e4ac6a998fbef2abb3ad2816c38766119bd))
* add scope placeholder to release-please PR title patterns ([3eded91](https://github.com/loonghao/vx/commit/3eded91195e02ae427e4cfacf151f89896ec6b25))

## 0.1.1 (2025-06-11)

## What's Changed
* fix: resolve GoReleaser and release-please workflow issues by @loonghao in https://github.com/loonghao/vx/pull/31
* fix: enhance CI permissions and configure release-please for PR-only mode by @loonghao in https://github.com/loonghao/vx/pull/33
* fix: resolve CI shell syntax errors and remove test workflows by @loonghao in https://github.com/loonghao/vx/pull/34
* fix: implement release-please best practices for output handling by @loonghao in https://github.com/loonghao/vx/pull/35


**Full Changelog**: https://github.com/loonghao/vx/compare/v0.1.0...v0.1.1

## 0.1.0 (2025-06-11)

## What's Changed
* chore: Configure Renovate by @renovate in https://github.com/loonghao/vx/pull/1
* fix(deps): update rust crate dirs to v6 by @renovate in https://github.com/loonghao/vx/pull/3
* fix(deps): update rust crate reqwest to 0.12 by @renovate in https://github.com/loonghao/vx/pull/2
* feat: Add GoReleaser CI/CD and improve CLI user experience by @loonghao in https://github.com/loonghao/vx/pull/5
* fix(deps): update rust crate reqwest to v0.12.20 by @renovate in https://github.com/loonghao/vx/pull/9
* fix(deps): update rust crate which to v8 by @renovate in https://github.com/loonghao/vx/pull/6
* chore(deps): update dependency go to 1.24 by @renovate in https://github.com/loonghao/vx/pull/19
* fix(deps): update rust crate zip to v4 - autoclosed by @renovate in https://github.com/loonghao/vx/pull/7
* chore(deps): update goreleaser/goreleaser-action action to v6 by @renovate in https://github.com/loonghao/vx/pull/20
* fix: resolve CI release-please configuration issues by @loonghao in https://github.com/loonghao/vx/pull/21

## New Contributors
* @renovate made their first contribution in https://github.com/loonghao/vx/pull/1
* @loonghao made their first contribution in https://github.com/loonghao/vx/pull/5

**Full Changelog**: https://github.com/loonghao/vx/commits/vx-v0.1.0

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
