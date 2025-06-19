# vx-paths

Cross-platform path management for vx tool installations.

## Overview

`vx-paths` provides a unified interface for managing tool installation paths across different platforms, ensuring consistent directory structures and proper handling of executable file extensions.

## Features

- **Standardized Path Structure**: Enforces `~/.vx/tools/<tool>/<version>/<tool>.exe` structure
- **Cross-Platform Support**: Handles executable extensions (.exe on Windows, none on Unix)
- **Configuration Integration**: Supports custom paths via environment variables and configuration
- **Tool Discovery**: Find installed tools and their versions
- **Path Resolution**: Resolve tool paths with version preferences

## Standard Directory Structure

```
~/.vx/
├── tools/           # Tool installations
│   ├── node/
│   │   ├── 18.17.0/
│   │   │   └── node.exe    # Windows
│   │   │   └── node        # Unix
│   │   └── 20.0.0/
│   │       └── node.exe
│   └── uv/
│       └── 0.1.0/
│           └── uv.exe
├── cache/           # Download cache
├── config/          # Configuration files
└── tmp/             # Temporary files
```

## Usage

### Basic Path Management

```rust
use vx_paths::PathManager;

// Create path manager with default locations
let manager = PathManager::new()?;

// Get tool executable path
let node_path = manager.tool_executable_path("node", "18.17.0");
// Returns: ~/.vx/tools/node/18.17.0/node.exe (Windows)
//          ~/.vx/tools/node/18.17.0/node (Unix)

// Check if tool is installed
let is_installed = manager.is_tool_version_installed("node", "18.17.0");

// List all versions of a tool
let versions = manager.list_tool_versions("node")?;

// Get latest version
let latest = manager.get_latest_tool_version("node")?;
```

### Tool Discovery

```rust
use vx_paths::{PathManager, PathResolver};

let manager = PathManager::new()?;
let resolver = PathResolver::new(manager);

// Find all executables for a tool
let executables = resolver.find_tool_executables("node")?;

// Find latest executable
let latest_exe = resolver.find_latest_executable("node")?;

// Resolve with version preference
let exe_path = resolver.resolve_tool_path("node", Some("18.17.0"))?;
```

### Custom Configuration

```rust
use vx_paths::{PathConfig, PathManager};

// Create with custom base directory
let config = PathConfig::with_base_dir("/custom/vx");
let manager = config.create_path_manager()?;

// Load from environment variables
let config = PathConfig::from_env();
let manager = config.create_path_manager()?;
```

### Environment Variables

- `VX_BASE_DIR`: Custom base directory
- `VX_TOOLS_DIR`: Custom tools directory
- `VX_CACHE_DIR`: Custom cache directory
- `VX_CONFIG_DIR`: Custom config directory
- `VX_TMP_DIR`: Custom temporary directory

## Integration with vx-core

```rust
use vx_paths::PathManager;
use vx_core::VxConfig;

// Integrate with vx configuration
let path_manager = PathManager::new()?;
let config = VxConfig {
    install_dir: path_manager.tools_dir().to_path_buf(),
    cache_dir: path_manager.cache_dir().to_path_buf(),
    // ... other config
};
```

## Testing

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.