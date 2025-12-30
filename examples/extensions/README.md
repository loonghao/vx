# vx Extension Examples

This directory contains example extensions demonstrating the vx extension system.

## Quick Start

```bash
# Install the hello-world example directly from GitHub
vx ext install https://github.com/loonghao/vx/tree/main/examples/extensions/hello-world

# Run the extension
vx x hello-world
vx x hello-world greet Alice
```

### Other Install Formats

```bash
# GitHub shorthand with path
vx ext install github:loonghao/vx/examples/extensions/hello-world

# Install a standalone extension repository
vx ext install github:user/vx-ext-name
vx ext install github:user/vx-ext-name@v1.0.0
```

## Examples

### hello-world (Python)

A simple Python-based extension demonstrating basic extension capabilities.

```bash
# Install from GitHub
vx ext install https://github.com/loonghao/vx/tree/main/examples/extensions/hello-world

# Or link locally for development
vx ext dev ./examples/extensions/hello-world

# Run the extension
vx x hello-world
vx x hello-world greet Alice
vx x hello-world info
```

### project-info (Node.js)

A Node.js-based extension that displays project information.

```bash
# Install from GitHub
vx ext install https://github.com/loonghao/vx/tree/main/examples/extensions/project-info

# Or link locally for development
vx ext dev ./examples/extensions/project-info

# Run the extension
vx x project-info
vx x project-info deps
vx x project-info size
```

## Command Reference

| Command | Description |
|---------|-------------|
| `vx ext list` | List all installed extensions |
| `vx ext install <source>` | Install extension from remote source |
| `vx ext dev <path>` | Link a local extension for development |
| `vx ext dev --unlink <name>` | Unlink a development extension |
| `vx ext uninstall <name>` | Uninstall an extension |
| `vx ext info <name>` | Show extension details |
| `vx ext update <name>` | Update an extension |
| `vx ext check --all` | Check for updates |
| `vx x <extension> [args]` | Run an extension |

### Supported Install Sources

| Format | Example |
|--------|---------|
| GitHub tree URL | `https://github.com/user/repo/tree/branch/path/to/ext` |
| GitHub shorthand | `github:user/repo` |
| GitHub with path | `github:user/repo/path/to/ext` |
| GitHub with version | `github:user/repo@v1.0.0` |
| GitHub HTTPS URL | `https://github.com/user/repo` |
| GitHub SSH URL | `git@github.com:user/repo.git` |

## Creating Your Own Extension

1. Create a directory with a `vx-extension.toml` file:

```toml
[extension]
name = "my-extension"
version = "1.0.0"
description = "My custom extension"
type = "command"

[runtime]
requires = "python >= 3.8"  # or "node >= 16", "bash", etc.

[entrypoint]
main = "main.py"

[commands.hello]
description = "Say hello"
script = "hello.py"
```

2. Add your scripts:

```
my-extension/
├── vx-extension.toml
├── main.py           # Main entry point
└── hello.py          # Subcommand script
```

3. Link for development:

```bash
vx ext dev /path/to/my-extension
```

4. Test:

```bash
vx x my-extension
vx x my-extension hello
```

5. Publish (optional):

```bash
# Create a GitHub repository with vx-extension.toml at the root
# Users can then install with:
vx ext install github:your-username/my-extension
```

## Environment Variables

Extensions receive these environment variables:

| Variable | Description |
|----------|-------------|
| `VX_VERSION` | Current vx version |
| `VX_EXTENSION_DIR` | Extension's directory |
| `VX_EXTENSION_NAME` | Extension name |
| `VX_PROJECT_DIR` | Current working directory |
| `VX_RUNTIMES_DIR` | vx runtimes directory |
| `VX_HOME` | vx home directory |

## Extension Types

- **command**: Provides CLI commands via `vx x <extension>`
- **hook**: Executes at lifecycle events (future)
- **provider**: Provides new runtime support (future)

## Extension Locations

Extensions are discovered from (in priority order):

1. `~/.vx/extensions-dev/` - Development extensions (highest priority)
2. `.vx/extensions/` - Project-level extensions
3. `~/.vx/extensions/` - User-installed extensions
4. Built-in extensions (lowest priority)
