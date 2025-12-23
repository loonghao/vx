# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1](https://github.com/loonghao/vx/compare/vx-paths-v0.3.0...vx-paths-v0.3.1) - 2025-06-19

### Added

- implement unified path management and complete crate documentation ([#112](https://github.com/loonghao/vx/pull/112))

## [Unreleased]

## [0.3.0] - 2025-01-19

### Added

- Initial release of vx-paths crate
- Cross-platform path management for vx tool installations
- Standardized directory structure enforcement
- `PathManager` for tool installation path management
- `PathResolver` for finding and resolving tool paths
- `PathConfig` for configuration integration
- Environment variable support for custom paths

### Features

- Standard vx directory structure: `~/.vx/tools/<tool>/<version>/<tool>.exe`
- Cross-platform executable extension handling (.exe on Windows)
- Tool version discovery and management
- Path resolution with version preferences
- Configuration support via environment variables (VX_BASE_DIR, etc.)
- Tool installation verification
- Directory cleanup and management

### Architecture

- Clean separation of concerns between path management and tool logic
- Integration with vx configuration system
- Extensible path configuration with custom base directories
- Comprehensive error handling and validation

### Documentation

- Complete API documentation with examples
- Path structure specification
- Configuration guide
- Cross-platform compatibility notes

## [0.2.5] - 2025-01-18

### Added

- Core path management design
- Cross-platform path utilities
- Basic directory structure definitions

## [0.2.0] - 2025-01-15

### Added

- Initial project setup
- Path management foundation
