# info

Show system information, capabilities, and build diagnostics.

## Usage

```bash
vx info [OPTIONS]
```

## Description

The `vx info` command displays comprehensive information about your vx installation, including:

- **vx version** and platform details
- **Managed runtimes** grouped by ecosystem (Node.js, Python, Go, Rust, etc.) with installation status
- **System tools** discovered on your system
- **Feature flags** (auto-install, shell mode, extensions, etc.)

This command is especially useful for debugging issues, sharing environment details in bug reports, and for AI tooling that needs to discover available capabilities programmatically.

## Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON (recommended for AI and scripting) |
| `--warnings` | Show build warnings and diagnostics |

## Examples

### Basic usage

```bash
# Show system info in human-readable format
vx info
```

Example output:

```
ℹ vx 0.7.0 capabilities

Platform: windows (x86_64)

ℹ Managed Runtimes:
  NodeJs:
    ✅ node (v22.0.0) - Node.js JavaScript runtime
    ❌ bun - Fast JavaScript runtime and package manager
  Python:
    ✅ uv (0.5.14) - Fast Python package manager
  Go:
    ✅ go (1.22.0) - Go programming language
  ...

ℹ System Tools (available):
    git [vcs] @ C:\Program Files\Git\cmd\git.exe
    cmake [build] @ C:\Program Files\CMake\bin\cmake.exe
    ...

ℹ Features:
    auto_install: true
    shell_mode: true
    project_config: true
    extensions: true
    virtual_environments: true
```

### JSON output (for AI and scripting)

```bash
vx info --json
```

Returns structured JSON containing all capabilities:

```json
{
  "version": "0.7.0",
  "platform": { "os": "windows", "arch": "x86_64" },
  "runtimes": {
    "node": {
      "name": "node",
      "description": "Node.js JavaScript runtime",
      "version": "22.0.0",
      "installed": true,
      "ecosystem": "NodeJs",
      "commands": ["node", "nodejs"]
    }
  },
  "system_tools": {
    "available": [...],
    "unavailable": [...]
  },
  "features": {
    "auto_install": true,
    "shell_mode": true,
    "project_config": true,
    "extensions": true,
    "virtual_environments": true
  }
}
```

### Show build diagnostics

```bash
vx info --warnings
```

Displays any errors or warnings that occurred during provider registry initialization. This is useful for diagnosing issues with custom providers or manifest files.

Example output when everything is healthy:

```
✓ No build warnings or errors.
```

Example output with issues:

```
✗ Build Errors (1):
  • missing factory for provider 'my-custom-provider'

⚠ Build Warnings (2):
  • provider 'legacy-tool' has deprecated configuration format
  • runtime 'old-tool' uses unsupported platform constraint syntax

Summary: 3 total diagnostic(s). Use --debug for verbose output.
```

## Advanced Usage

### Pipe JSON to other tools

```bash
# Get list of installed runtimes
vx info --json | jq '.runtimes | to_entries[] | select(.value.installed) | .key'

# Check if a specific runtime is available
vx info --json | jq '.runtimes.node.installed'

# Get platform info
vx info --json | jq '.platform'
```

### Use in CI/CD

```yaml
# GitHub Actions example
- name: Check vx environment
  run: vx info --json > vx-env.json

- name: Verify tools are installed
  run: |
    vx info --warnings
    vx info --json | jq -e '.runtimes.node.installed'
```

### Debug provider issues

When custom providers fail to load or behave unexpectedly:

```bash
# Check for build errors
vx info --warnings

# Enable debug logging for more details
VX_LOG=debug vx info --warnings
```

## Related Commands

- [`vx list`](./list.md) — List available and installed tools
- [`vx config show`](./config.md) — Show current configuration
