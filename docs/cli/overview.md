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
| [`run`](./run) | Run a script from `vx.toml` |
| [`dev`](./dev) | Enter development environment |

### Environment Management

| Command | Description |
|---------|-------------|
| [`env`](./env) | Manage environments |
| [`global`](./global) | Manage global packages (isolated) |
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

## Package Execution (RFC 0027)

Execute globally installed packages or run packages on-demand using the unified syntax:

### Syntax

```
vx <ecosystem>[@runtime_version]:<package>[@version][::executable] [args...]
```

### Examples

```bash
# Run installed package executables directly
vx tsc --version                    # Execute tsc from installed typescript

# Explicit package syntax
vx npm:typescript::tsc --version    # Package name differs from executable
vx pip:httpie::http GET example.com # httpie package provides 'http' command
vx npm:eslint .                     # Package name = executable name

# Scoped npm packages
vx npm:@openai/codex::codex         # @scope/package::executable
vx npm:@biomejs/biome::biome check .

# With package version
vx npm:typescript@5.3::tsc --version
vx pip:ruff@0.3 check .

# With runtime version
vx npm@20:typescript::tsc --version  # Use Node.js 20
vx pip@3.11:black .                  # Use Python 3.11
```

### Supported Ecosystems

| Ecosystem | Runtime | Example |
|-----------|---------|---------|
| `npm` | Node.js | `vx npm:typescript::tsc` |
| `pip` | Python | `vx pip:httpie::http` |
| `uv` | Python | `vx uv:ruff` |
| `cargo` | Rust | `vx cargo:ripgrep::rg` |
| `go` | Go | `vx go:golangci-lint` |
| `bun` | Bun | `vx bun:typescript::tsc` |
| `yarn` | Node.js | `vx yarn:typescript::tsc` |
| `pnpm` | Node.js | `vx pnpm:typescript::tsc` |

### The `::` Separator

Use `::` when the package name differs from the executable:

| Package | Executable | Command |
|---------|------------|---------|
| `typescript` | `tsc` | `vx npm:typescript::tsc` |
| `httpie` | `http` | `vx pip:httpie::http` |
| `@openai/codex` | `codex` | `vx npm:@openai/codex::codex` |
| `ripgrep` | `rg` | `vx cargo:ripgrep::rg` |

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
