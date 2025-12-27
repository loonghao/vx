# vx Extension Examples

This directory contains example extensions demonstrating the vx extension system.

## Examples

### hello-world (Python)

A simple Python-based extension demonstrating basic extension capabilities.

```bash
# Link the extension for development
vx ext dev examples/extensions/hello-world

# Run the extension
vx x hello-world
vx x hello-world greet Alice
vx x hello-world info
```

### project-info (Node.js)

A Node.js-based extension that displays project information.

```bash
# Link the extension for development
vx ext dev examples/extensions/project-info

# Run the extension
vx x project-info
vx x project-info deps
vx x project-info size
```

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

1. Add your scripts:

```
my-extension/
├── vx-extension.toml
├── main.py           # Main entry point
└── hello.py          # Subcommand script
```

1. Link for development:

```bash
vx ext dev /path/to/my-extension
```

1. Test:

```bash
vx x my-extension
vx x my-extension hello
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
