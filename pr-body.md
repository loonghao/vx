## Summary

Add rez package manager provider and centralize path management in vx-paths crate.

## Changes

### New Features
- **vx-provider-rez**: New crate for rez package manager support
  - Install rez via pip in isolated virtual environment
  - Auto-fix pip installation warning with production marker file
  - Auto-fix memcache.py SyntaxWarning for Python 3.12+

- **PackageRuntime trait**: New trait for npm/pip package installations
  - Unified interface for package-based runtimes (vite, rez, etc.)
  - Automatic dependency resolution (e.g., node for npm packages, uv for pip packages)

### Improvements
- **Centralized path management**: Unified ToolLocation API in vx-paths
  - New `ToolLocation` struct with path, version, and source info
  - New `ToolSource` enum (Store, NpmTools, PipTools)
  - Simplified vx-resolver by removing duplicate path-finding logic

### Bug Fixes
- Fix rez showing pip installation warning
- Fix rez memcache.py SyntaxWarning in Python 3.12+

## Testing
- `cargo run -- rez --version` works without warnings
- `cargo run -- rez env` enters rez shell correctly
