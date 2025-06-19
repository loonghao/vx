# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-06-19

### Added
- Initial release of vx-dependency crate
- Dependency resolution engine for tool management
- Multi-layer dependency support (tool -> tool dependencies)
- Automatic dependency installation and management
- Dependency graph construction and analysis
- Circular dependency detection and prevention
- Version constraint resolution across dependencies
- Refactor vx-core architecture with closed-loop toolchain design

### Fixed
- Resolve import errors and clippy warnings in tool packages

### Added
- Initial release of vx-dependency crate
- Dependency resolution engine for tool management
- Multi-layer dependency support (tool -> tool dependencies)
- Automatic dependency installation and management
- Dependency graph construction and analysis
- Circular dependency detection and prevention
- Version constraint resolution across dependencies

### Features
- `DependencyResolver` for resolving tool dependencies
- `DependencyGraph` for dependency relationship management
- Automatic upstream/downstream dependency handling
- Support for optional and required dependencies
- Dependency conflict resolution
- Installation order optimization
- Dependency caching and memoization

### Architecture
- Clean separation between dependency resolution and installation
- Integration with vx-plugin for tool management
- Support for complex dependency scenarios
- Extensible resolver architecture for different dependency types

### Documentation
- Comprehensive API documentation
- Dependency resolution examples
- Architecture overview
- Best practices for dependency management

## [0.2.5] - 2025-01-18

### Added
- Core dependency resolution framework
- Basic dependency graph implementation
- Conflict detection utilities

## [0.2.0] - 2025-01-15

### Added
- Initial project setup
- Dependency management foundation

