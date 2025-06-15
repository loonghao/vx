# vx-cli

[![Crates.io](https://img.shields.io/crates/v/vx-cli.svg)](https://crates.io/crates/vx-cli)
[![Documentation](https://docs.rs/vx-cli/badge.svg)](https://docs.rs/vx-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Command-line interface for the vx universal tool manager.

## Overview

`vx-cli` provides the command-line interface for vx, a universal development tool manager. It offers a unified interface for managing, installing, and executing development tools across different languages and ecosystems.

## Features

- **Universal Tool Execution**: Run any supported tool through a single interface
- **Automatic Installation**: Download and install missing tools automatically
- **Version Management**: Install, switch, and manage multiple tool versions
- **Virtual Environments**: Create isolated environments for projects
- **Project Configuration**: Support for project-specific tool configurations
- **Interactive UI**: Rich terminal interface with progress bars and colors
- **Shell Integration**: Auto-completion and shell hooks

## Installation

### From Crates.io
```bash
cargo install vx-cli
```

### From Source
```bash
git clone https://github.com/loonghao/vx
cd vx
cargo install --path crates/vx-cli
```

## Quick Start

### Basic Usage
```bash
# Execute tools transparently
vx node --version
vx uv pip install requests
vx go build

# Install specific versions
vx install node@18.17.0
vx install uv@latest

# List available tools
vx list

# Create virtual environment
vx venv create myproject --tools node@18.17.0,uv@latest
```

### Project Configuration
```bash
# Initialize project configuration
vx init

# Edit .vx.toml
echo '[tools]
node = "18.17.0"
uv = "latest"' > .vx.toml

# Sync project tools
vx sync
```

## Commands

### Tool Execution
```bash
# Direct tool execution (transparent proxy)
vx <tool> [args...]

# Examples
vx node --version
vx npm install express
vx uv pip install requests
vx go build ./...
vx cargo test
```

### Tool Management
```bash
# Install tools
vx install <tool>[@version]
vx install node@18.17.0 uv@latest

# List tools
vx list                    # All available tools
vx list --installed        # Only installed tools
vx list node              # Specific tool versions

# Update tools
vx update                 # Update all tools
vx update node            # Update specific tool

# Remove tools
vx remove node@18.17.0    # Remove specific version
vx remove node --all      # Remove all versions

# Search tools
vx search python          # Search for tools
vx search --category python
```

### Virtual Environments
```bash
# Create environments
vx venv create myproject
vx venv create myproject --tools node@18.17.0,uv@latest

# Use environments
vx venv use myproject
vx venv run myproject node --version

# Manage environments
vx venv list              # List all environments
vx venv remove myproject  # Remove environment
```

### Project Management
```bash
# Initialize project
vx init                   # Interactive setup
vx init --template node   # Use template

# Sync project tools
vx sync                   # Install project tools
vx sync --check           # Check without installing

# Configuration
vx config                 # Show current config
vx config edit            # Edit global config
vx config edit --local    # Edit project config
```

### Maintenance
```bash
# Statistics
vx stats                  # Show usage statistics
vx stats --detailed       # Detailed statistics

# Cleanup
vx cleanup                # Clean orphaned files
vx cleanup --dry-run      # Preview cleanup

# Global tool management
vx global list            # List global tools
vx global cleanup         # Clean unused tools
```

## Configuration

### Global Configuration
Location: `~/.vx/config/global.toml`

```toml
[auto_install]
enabled = true
timeout = 300
confirm_before_install = false

[settings]
cache_duration = "7d"
parallel_downloads = 4
use_system_path = false

[ui]
show_progress = true
use_colors = true
```

### Project Configuration
Location: `.vx.toml` in project root

```toml
[tools]
node = "18.17.0"
uv = "latest"
go = "^1.21.0"

[settings]
auto_install = true
cache_duration = "7d"

[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"
```

## Shell Integration

### Bash/Zsh
```bash
# Add to ~/.bashrc or ~/.zshrc
eval "$(vx shell-init)"
source <(vx completion bash)  # or zsh
```

### Fish
```fish
# Add to ~/.config/fish/config.fish
vx shell-init | source
vx completion fish | source
```

### PowerShell
```powershell
# Add to PowerShell profile
Invoke-Expression (vx shell-init)
vx completion powershell | Out-String | Invoke-Expression
```

## Supported Tools

### Languages & Runtimes
- **Node.js**: JavaScript runtime and npm
- **Python**: UV package manager and Python tools
- **Go**: Go compiler and tools
- **Rust**: Rust compiler and Cargo

### Package Managers
- **npm**: Node.js package manager
- **UV**: Fast Python package manager
- **Cargo**: Rust package manager

### Package Runners
- **npx**: Node.js package runner
- **uvx**: Python application runner

## Architecture

vx-cli is built on top of several core components:

- **vx-core**: Core traits and functionality
- **Tool Plugins**: Individual tool implementations
- **Package Manager Plugins**: Package manager integrations
- **UI Components**: Terminal interface and progress tracking

## Development

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Running
```bash
cargo run -- --help
```

### Debugging
```bash
# Enable verbose logging
RUST_LOG=debug cargo run -- <command>

# Or use the verbose flag
cargo run -- --verbose <command>
```

## Error Handling

vx-cli provides detailed error messages and suggestions:

```bash
$ vx nonexistent-tool
Error: Tool 'nonexistent-tool' not found

Suggestions:
  - Run 'vx list' to see available tools
  - Run 'vx search nonexistent' to search for similar tools
  - Check if the tool name is spelled correctly
```

## Performance

- **Fast Startup**: Optimized for quick command execution
- **Parallel Downloads**: Multiple tools can be downloaded simultaneously
- **Caching**: Version information and downloads are cached
- **Lazy Loading**: Plugins are loaded only when needed

## Troubleshooting

### Common Issues

1. **Tool not found**: Run `vx list` to see available tools
2. **Installation fails**: Check network connection and permissions
3. **Version conflicts**: Use `vx cleanup` to remove orphaned versions
4. **Shell integration**: Ensure shell configuration is properly loaded

### Debug Mode
```bash
# Enable debug logging
vx --verbose <command>

# Check configuration
vx config --sources

# Verify installation
vx stats
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Contributing

Contributions are welcome! Please see the [contributing guidelines](../../CONTRIBUTING.md) for more information.

## Related Crates

- [`vx-core`](../vx-core/README.md) - Core functionality
- [`vx-tool-node`](../vx-tools/vx-tool-node/README.md) - Node.js plugin
- [`vx-tool-uv`](../vx-tools/vx-tool-uv/README.md) - UV Python plugin
- [`vx-pm-npm`](../vx-package-managers/vx-pm-npm/README.md) - NPM plugin
