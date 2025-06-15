# VX Shim Integration

This document describes the integration of shim technology in vx for seamless tool version switching.

## Overview

VX now uses shim technology to provide transparent tool version switching without requiring PATH manipulation or shell activation scripts. This approach is similar to tools like `scoop` and `nvm` but implemented in Rust for better cross-platform support.

## How It Works

### 1. Shim Creation

When you switch tool versions using `vx switch`, the system:

1. Creates a lightweight executable shim in the vx shim directory
2. The shim reads a configuration file to determine the target executable
3. The shim forwards all arguments to the actual tool executable
4. Proper signal handling ensures Ctrl+C and other signals work correctly

### 2. Directory Structure

```
~/.vx/
├── shims/           # Shim executables and configurations
│   ├── node.exe     # Shim executable (Windows)
│   ├── node.shim    # Shim configuration file
│   ├── python.exe   # Another tool shim
│   └── python.shim  # Its configuration
├── tools/           # Actual tool installations
│   ├── node/
│   │   ├── 18.17.0/
│   │   └── 20.10.0/
│   └── python/
│       ├── 3.10.0/
│       └── 3.11.0/
└── bin/             # VX executables
    └── vx-shim.exe  # The shim template
```

## Usage Examples

### Basic Tool Switching

```bash
# Install different versions
vx install node@18.17.0
vx install node@20.10.0

# Switch to a specific version
vx switch node@20.10.0

# The shim is automatically created/updated
# Now 'node' command uses version 20.10.0
node --version  # v20.10.0
```

### Global vs Session Switching

```bash
# Switch globally (affects all new terminal sessions)
vx switch node@20.10.0 --global

# Session-level switch (current implementation)
vx switch node@18.17.0
```

### Virtual Environment Integration

```bash
# Create a virtual environment with specific tool versions
vx venv create myproject --tools node@18.17.0,python@3.11.0

# Activate the environment
vx venv activate myproject

# Tools in the venv use the specified versions
node --version  # v18.17.0 (from venv)
python --version  # Python 3.11.0 (from venv)
```

## Implementation Details

### VxShimManager

The `VxShimManager` class handles all shim operations:

```rust
use vx_core::{VxEnvironment, VxShimManager};

// Create shim manager
let env = VxEnvironment::new()?;
let shim_manager = VxShimManager::new(env)?;

// Create a shim for a tool
shim_manager.create_tool_shim("node", "/path/to/node", "20.10.0", None)?;

// Switch tool version
shim_manager.switch_tool_version("node", "18.17.0", "/path/to/node-18")?;

// List all shims
let shims = shim_manager.list_shims()?;
```

### Shim Configuration Format

Each shim has a corresponding `.shim` configuration file in TOML format:

```toml
# node.shim
path = "/home/user/.vx/tools/node/20.10.0/bin/node"
args = ""
working_dir = ""

[env]
# Optional environment variables

[signal_handling]
forward_signals = true
kill_on_exit = true
```

### Cross-Platform Support

- **Windows**: Shims are `.exe` files that can be executed directly
- **Unix/Linux**: Shims are executable binaries with proper permissions
- **Signal Handling**: Proper Ctrl+C forwarding on all platforms
- **Process Management**: Automatic cleanup of child processes

## Benefits

### 1. No PATH Manipulation

- No need to modify PATH environment variable
- No shell-specific activation scripts
- Works across different shells and terminals

### 2. Transparent Operation

- Tools work exactly as if they were installed globally
- No performance overhead (minimal shim execution time)
- Proper signal handling and process management

### 3. Isolation

- Different projects can use different tool versions
- Virtual environments are truly isolated
- No conflicts between tool versions

### 4. Update Safety

- Updating tools doesn't break existing shims
- Shims can be updated independently
- No file locking issues during updates

## Configuration

### Adding Shim Directory to PATH

To use shims, add the shim directory to your PATH:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export PATH="$HOME/.vx/shims:$PATH"
```

On Windows:
```powershell
# Add to your PowerShell profile
$env:PATH = "$env:USERPROFILE\.vx\shims;$env:PATH"
```

### Environment Variables

- `VX_SHIM_DEBUG`: Enable debug output for shim operations
- `VX_SHIM_TIMEOUT`: Set timeout for shim operations (default: 30s)

## Troubleshooting

### Shim Not Found

If you get "vx-shim executable not found" error:

1. Ensure vx-shim is built: `cargo build --release`
2. Check if vx-shim is in PATH or vx bin directory
3. Verify file permissions on Unix systems

### Tool Not Switching

If tools don't switch versions:

1. Check if shim directory is in PATH
2. Verify shim configuration file exists
3. Ensure target tool version is installed

### Permission Issues

On Unix systems, ensure shims have execute permissions:

```bash
chmod +x ~/.vx/shims/*
```

## Future Enhancements

1. **Automatic PATH Management**: Automatically add shim directory to PATH
2. **Shell Integration**: Better integration with shell completion
3. **Performance Optimization**: Cache shim configurations for faster startup
4. **Advanced Configuration**: Support for tool-specific environment variables
5. **Monitoring**: Track tool usage and performance metrics

## Related Commands

- [`vx switch`](./cli/switch.md) - Switch tool versions
- [`vx install`](./cli/install.md) - Install tool versions
- [`vx venv`](./cli/venv.md) - Virtual environment management
- [`vx list`](./cli/list.md) - List installed versions
