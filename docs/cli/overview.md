# CLI Overview

vx provides a comprehensive command-line interface for managing development tools.

## Basic Syntax

```bash
vx [OPTIONS] <COMMAND> [ARGS]
vx [OPTIONS] <TOOL> [TOOL_ARGS]
```

## Global Options

| Option | Description |
|--------|-------------|
| `--verbose`, `-v` | Enable verbose output |
| `--debug` | Enable debug output |
| `--use-system-path` | Use system PATH instead of vx-managed tools |
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version |

## Commands

### Tool Management

| Command | Description |
|---------|-------------|
| [`install`](./install) | Install a tool version |
| [`list`](./list) | List available tools |
| `uninstall` | Remove a tool version |
| `which` | Show tool location |
| `versions` | Show available versions |
| `switch` | Switch to a different version |

### Project Management

| Command | Description |
|---------|-------------|
| `init` | Initialize project configuration |
| [`setup`](./setup) | Install all project tools |
| `sync` | Sync tools with configuration |
| `add` | Add a tool to project configuration |
| `remove` | Remove a tool from project configuration |

### Script Execution

| Command | Description |
|---------|-------------|
| [`run`](./run) | Run a script from `.vx.toml` |
| [`dev`](./dev) | Enter development environment |

### Environment Management

| Command | Description |
|---------|-------------|
| [`env`](./env) | Manage environments |
| `global` | Manage global tool versions |
| `venv` | Python virtual environment management |

### Configuration

| Command | Description |
|---------|-------------|
| [`config`](./config) | Manage configuration |
| [`shell`](./shell) | Shell integration |

### Extension Management

| Command | Description |
|---------|-------------|
| [`ext`](./ext) | Manage extensions |
| `x` | Execute extension command |

### Maintenance

| Command | Description |
|---------|-------------|
| `clean` | Clean up cache and orphaned files |
| `stats` | Show disk usage statistics |
| `self-update` | Update vx itself |

## Direct Tool Execution

Run any tool by using it as the command:

```bash
vx node --version
vx python script.py
vx go build
vx cargo test
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Command not found |
| 3 | Tool not installed |
| 4 | Configuration error |

## Environment Variables

| Variable | Description |
|----------|-------------|
| `VX_HOME` | Override vx data directory |
| `VX_ENV` | Current environment name |
| `VX_AUTO_INSTALL` | Enable/disable auto-install |
| `VX_VERBOSE` | Enable verbose output |
| `VX_DEBUG` | Enable debug output |

## Getting Help

```bash
# General help
vx --help

# Command-specific help
vx install --help
vx env --help
```
