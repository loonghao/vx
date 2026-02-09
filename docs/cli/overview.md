# CLI Overview

vx provides a comprehensive command-line interface for managing development tools, project environments, and development workflows.

## Basic Syntax

```bash
# Subcommand mode
vx [OPTIONS] <COMMAND> [ARGS]

# Direct execution mode (transparent proxy)
vx [OPTIONS] <TOOL> [TOOL_ARGS]
```

## Global Options

| Option | Description |
|--------|-------------|
| `--verbose`, `-v` | Enable verbose output |
| `--debug` | Enable debug output |
| `--use-system-path` | Use system PATH instead of vx-managed tools |
| `--no-auto-install` | Disable auto-installation of missing tools |
| `--help`, `-h` | Show help |
| `--version`, `-V` | Show version |

## Commands at a Glance

### Tool Management

| Command | Alias | Description |
|---------|-------|-------------|
| [`install`](./install) | `i` | Install tool versions (`vx install node@22 python@3.12`) |
| `uninstall` | — | Remove an installed tool version |
| [`list`](./list) | `ls` | List installed tools and available runtimes |
| `versions` | — | Show available versions for a tool |
| `which` | `where` | Show the path of the currently active tool |
| `switch` | — | Switch to a different installed version |
| `search` | — | Search for available tools |
| [`test`](./test) | — | Test runtime availability and provider functionality |
| [`global`](./global) | `g` | Manage globally installed packages (isolated) |

### Project Management

| Command | Alias | Description |
|---------|-------|-------------|
| `init` | — | Initialize a new `vx.toml` for the project |
| `add` | — | Add a tool requirement to `vx.toml` |
| `remove` | `rm` | Remove a tool from `vx.toml` |
| `sync` | — | Sync project tools with `vx.toml` |
| `lock` | — | Generate/update `vx.lock` for reproducible environments |
| `check` | — | Check version constraints and tool availability |
| `bundle` | — | Offline development environment packaging |
| `analyze` | — | Analyze project dependencies, scripts, and tools |

### Scripts & Environment

| Command | Description |
|---------|-------------|
| [`run`](./run) | Run scripts defined in `vx.toml` |
| [`dev`](./dev) | Enter development environment (interactive shell) |
| [`setup`](./setup) | Install all project tools and run setup hooks |
| [`env`](./env) | Manage virtual environments |

### Configuration & Shell

| Command | Alias | Description |
|---------|-------|-------------|
| [`config`](./config) | `cfg` | Manage global and project configuration |
| [`shell`](./shell) | — | Shell integration (init, completions) |

### Extensions & Plugins

| Command | Alias | Description |
|---------|-------|-------------|
| [`ext`](./ext) | `extension` | Manage vx extensions |
| `x` | — | Execute extension commands |
| [`plugin`](./plugin) | — | Manage provider plugins |

### System & Maintenance

| Command | Description |
|---------|-------------|
| [`info`](./info) | Show system information, capabilities, and diagnostics |
| [`metrics`](./metrics) | View execution performance metrics |
| `cache` | Cache management (info, list, prune, purge) |
| `self-update` | Update vx itself |
| `version` | Show vx version |
| `migrate` | Migrate configuration and data formats |
| `hook` | Manage lifecycle hooks |
| `services` | Development service management |
| `container` | Container/Dockerfile management |
| `auth` | Authentication management |

## Direct Tool Execution

Run any managed tool by using it as the command:

```bash
vx node --version        # Run Node.js
vx python script.py      # Run Python script
vx go build ./...        # Build Go project
vx cargo test            # Run Rust tests
vx uv run pytest         # Run Python tests via uv
```

Tools are automatically installed on first use. Dependencies are resolved and installed as well (e.g., `vx npm install` ensures Node.js is available).

## Package Execution Syntax

Execute packages on-demand using the unified syntax:

```
vx <ecosystem>[@runtime_version]:<package>[@version][::executable] [args...]
```

### Examples

```bash
# Run package executables
vx npm:typescript::tsc --version     # TypeScript compiler
vx pip:httpie::http GET example.com  # HTTPie client
vx cargo:ripgrep::rg "pattern" .     # ripgrep search

# Scoped npm packages
vx npm:@biomejs/biome::biome check .

# With version pinning
vx npm:typescript@5.3::tsc --version
vx pip:ruff@0.3 check .

# With runtime version
vx npm@20:typescript::tsc --version  # Use Node.js 20
```

### Supported Ecosystems

| Ecosystem | Runtime | Example |
|-----------|---------|---------|
| `npm` | Node.js | `vx npm:typescript::tsc` |
| `pip` / `uv` | Python | `vx pip:httpie::http` |
| `cargo` | Rust | `vx cargo:ripgrep::rg` |
| `go` | Go | `vx go:golangci-lint` |
| `bun` | Bun | `vx bun:typescript::tsc` |
| `yarn` / `pnpm` | Node.js | `vx yarn:typescript::tsc` |

Use `::` when the package name differs from the executable name.

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
| `VX_AUTO_INSTALL` | Enable/disable auto-install (`true`/`false`) |
| `VX_VERBOSE` | Enable verbose output |
| `VX_DEBUG` | Enable debug output |
| `VX_CDN_ENABLED` | Enable CDN acceleration |

## Getting Help

```bash
# General help
vx --help

# Command-specific help
vx install --help
vx env --help
vx global --help
```
