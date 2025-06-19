# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<<<<<<< HEAD
=======
## [Unreleased]

## [0.2.6] - 2025-01-19

### Added
- Initial release of vx-plugin crate
- `VxTool` trait for implementing tool support
- `PluginRegistry` for managing tool plugins
- Tool execution context and result types
- Version information and status management
- Default installation workflows for tools
- Cross-platform executable path resolution
- Tool metadata and configuration support

### Changed
- Updated tool installation logic to use vx-paths for standardized path management
- Improved executable path resolution with standard vx structure
- Enhanced version checking with PathManager integration
- Refactored tool removal to use centralized path management

### Features
- Simplified trait interface requiring only essential methods
- Sensible defaults for most tool operations
- Configurable tool implementation with URL builders
- Version parsing and management utilities
- Tool status reporting and installed version tracking
- Async-first design with tokio integration
- Comprehensive error handling with anyhow

### Documentation
- Complete API documentation with examples
- Plugin development guide
- Tool implementation examples
- Best practices for tool plugin development

## [0.2.5] - 2025-01-18

### Added
- Core plugin architecture design
- Basic tool trait definitions
- Plugin registry implementation

## [0.2.0] - 2025-01-15

### Added
- Initial project setup
- Plugin framework foundation
>>>>>>> fix/compilation-and-install-issues
