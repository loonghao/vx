# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.4.0](https://github.com/loonghao/vx/compare/vx-installer-v0.3.0...vx-installer-v0.4.0) - 2025-06-19

### Added

- implement unified path management and complete crate documentation ([#112](https://github.com/loonghao/vx/pull/112))
## [Unreleased]

## [0.3.0] - 2025-01-19

### Added
- Initial release of vx-installer crate
- Universal tool installation framework
- Support for multiple installation methods (Archive, Binary, Script, Package)
- Archive format support (ZIP, TAR, TAR.GZ, TAR.XZ, TAR.BZ2)
- Download management with progress tracking
- Checksum verification (SHA256, SHA1, MD5)
- Cross-platform installation support
- Atomic installation with rollback capabilities
- Installation verification and validation

### Features
- `Installer` struct for managing tool installations
- `InstallConfig` builder pattern for configuration
- Progress reporting with indicatif integration
- Concurrent downloads with connection pooling
- Automatic extraction and file placement
- Permission handling for executables
- Installation directory management
- Error recovery and cleanup

### Security
- Checksum verification for downloaded files
- Secure temporary file handling
- Path traversal protection during extraction
- Safe file permissions on Unix systems

### Documentation
- Comprehensive README with usage examples
- API documentation with real-world examples
- Installation method guides
- Security best practices
- Performance optimization tips

## [0.2.5] - 2025-01-18

### Added
- Core installer architecture
- Download and extraction utilities
- Basic installation methods

## [0.2.0] - 2025-01-15

### Added
- Initial project setup
- Installation framework foundation

