# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.6] - 2025-01-19

### Added
- Initial release of vx-config crate
- Configuration management with figment integration
- Support for TOML, JSON, and environment variable configuration sources
- Hierarchical configuration merging
- Type-safe configuration structures with serde
- Default configuration values and validation
- Configuration file discovery and loading
- Environment variable prefix support

### Features
- `VxConfig` struct for main application configuration
- `ConfigBuilder` for programmatic configuration construction
- Support for multiple configuration file formats
- Automatic configuration file discovery in standard locations
- Environment variable override support with `VX_` prefix
- Validation and error handling for configuration values
- Integration with workspace configuration standards

### Documentation
- Comprehensive README with usage examples
- API documentation with examples
- Configuration schema documentation
- Best practices guide for configuration management

## [0.2.5] - 2025-01-18

### Added
- Project initialization and basic structure
- Core configuration traits and types
- Initial figment integration

## [0.2.0] - 2025-01-15

### Added
- Initial project setup
- Basic configuration framework