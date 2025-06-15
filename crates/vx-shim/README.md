# vx-shim

[![Crates.io](https://img.shields.io/crates/v/vx-shim.svg)](https://crates.io/crates/vx-shim)
[![Documentation](https://docs.rs/vx-shim/badge.svg)](https://docs.rs/vx-shim)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Cross-platform shim executable for the vx universal tool manager.

## Overview

`vx-shim` is a lightweight, cross-platform executable that acts as a proxy to other executables. It's inspired by [scoop-better-shimexe](https://github.com/71/scoop-better-shimexe) but written in Rust for better cross-platform support and modern features.

## Features

- **Cross-Platform**: Works on Windows, macOS, and Linux
- **Fast Execution**: Minimal overhead with efficient process management
- **Signal Handling**: Proper Ctrl+C and signal forwarding to child processes
- **Process Management**: Automatic cleanup of child processes
- **Flexible Configuration**: Support for both TOML and legacy Scoop formats
- **Environment Variables**: Support for custom environment variables
- **Working Directory**: Configurable working directory for target executables
- **Windows Features**: Job objects, console hiding, elevation support
- **Unix Features**: Fork/exec with proper signal handling

## How It Works

1. **Shim Discovery**: Looks for a `.shim` file with the same name as the executable
2. **Configuration Parsing**: Reads the shim configuration (TOML or legacy format)
3. **Process Execution**: Executes the target program with proper signal handling
4. **Cleanup**: Ensures child processes are cleaned up when the shim exits

## Shim Configuration

### TOML Format (Recommended)

```toml
# Path to the target executable
path = "/usr/bin/git"

# Optional arguments to prepend
args = "status -u"

# Working directory (optional)
working_dir = "/workspace"

# Environment variables (optional)
[env]
GIT_EDITOR = "vim"
PAGER = "less"

# Signal handling configuration (optional)
[signal_handling]
ignore_sigint = true
forward_signals = true
kill_on_exit = true

# Windows-specific options (optional)
hide_console = false
run_as_admin = false
```

### Legacy Scoop Format

```
path = C:\Program Files\Git\git.exe
args = status -u
working_dir = C:\workspace
env.GIT_EDITOR = vim
env.PAGER = less
```

## Usage

### As a Library

```rust
use vx_shim::{ShimConfig, Executor};

// Load configuration
let config = ShimConfig::load("my-tool.shim")?;

// Execute with arguments
let executor = Executor::new(config);
let exit_code = executor.execute(&["--version"])?;
```

### As an Executable

```bash
# The shim executable looks for a .shim file with the same name
# For example, git.exe looks for git.shim

# Create a shim configuration
echo 'path = "/usr/bin/git"' > git.shim
echo 'args = "status -u"' >> git.shim

# Copy vx-shim to git (or git.exe on Windows)
cp vx-shim git

# Now running ./git will execute: /usr/bin/git status -u [additional args]
./git --short
# Executes: /usr/bin/git status -u --short
```

## Installation

### From Crates.io

```bash
cargo install vx-shim
```

### From Source

```bash
git clone https://github.com/loonghao/vx
cd vx/crates/vx-shim
cargo build --release
```

## Platform-Specific Features

### Windows

- **Job Objects**: Automatic cleanup of child processes using Windows job objects
- **Console Management**: Option to hide console windows for GUI applications
- **Elevation Support**: Automatic handling of UAC elevation requests
- **Signal Handling**: Proper handling of Ctrl+C and other console signals

### Unix (Linux/macOS)

- **Fork/Exec**: Uses fork/exec for better signal handling
- **Signal Forwarding**: Proper forwarding of signals to child processes
- **Process Groups**: Management of process groups for cleanup
- **File Permissions**: Automatic handling of executable permissions

## Advantages over Alternatives

### vs. Batch/Shell Scripts

- **Performance**: No shell interpreter overhead
- **Signal Handling**: Proper signal forwarding and cleanup
- **Cross-Platform**: Single binary works everywhere
- **Error Handling**: Better error reporting and handling

### vs. scoop-better-shimexe

- **Cross-Platform**: Works on Unix systems, not just Windows
- **Modern Language**: Written in Rust for memory safety and performance
- **Flexible Configuration**: Support for both TOML and legacy formats
- **Better Testing**: Comprehensive test suite and CI

### vs. Symlinks

- **Argument Injection**: Can prepend arguments to commands
- **Environment Control**: Can set custom environment variables
- **Working Directory**: Can change working directory
- **Windows Compatibility**: Works on Windows without admin privileges

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Cross-Platform Testing

```bash
# Test on Windows
cargo test --target x86_64-pc-windows-msvc

# Test on Linux
cargo test --target x86_64-unknown-linux-gnu

# Test on macOS
cargo test --target x86_64-apple-darwin
```

## Integration with VX

vx-shim is designed to work seamlessly with the vx tool manager:

1. **Tool Installation**: vx creates shim files when installing tools
2. **Version Management**: Different tool versions can have different shims
3. **Virtual Environments**: Shims can be environment-specific
4. **Path Management**: Shims are placed in vx-managed PATH directories

## Examples

### Simple Command Proxy

```toml
# echo.shim
path = "/bin/echo"
args = "Hello from shim:"
```

```bash
./echo world
# Output: Hello from shim: world
```

### Development Tool with Environment

```toml
# node.shim
path = "/usr/local/bin/node"
working_dir = "/workspace"

[env]
NODE_ENV = "development"
DEBUG = "*"
```

### Windows GUI Application

```toml
# notepad.shim
path = "C:\\Windows\\System32\\notepad.exe"
hide_console = true
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../CONTRIBUTING.md) for more information.

## Related Projects

- [scoop-better-shimexe](https://github.com/71/scoop-better-shimexe) - Original inspiration (Windows-only)
- [Scoop](https://scoop.sh/) - Windows package manager that uses shims
- [vx](https://github.com/loonghao/vx) - Universal tool manager that uses vx-shim
