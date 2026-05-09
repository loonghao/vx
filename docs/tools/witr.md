# witr - Process Introspection Tool

witr ("Why is this running?") is a process introspection tool that helps you understand what processes are running and why.

## Quick Start

```bash
# Install witr
vx install witr@latest

# Check witr version
vx witr --version

# Use witr to introspect processes
vx witr
```

## About witr

witr helps you answer questions like:
- Why is this process running?
- What started this process?
- What dependencies does this process have?
- What is the process ancestry (parent/child relationships)?

## Installation

witr is available for all platforms (Windows, macOS, Linux) via `vx`:

```bash
vx install witr@latest
```

vx downloads witr from the [vx-org/mirrors](https://github.com/vx-org/mirrors) repository (GitHub Releases), which provides pre-built binaries for:
- Windows (amd64, arm64) - `.zip` archive containing `witr.exe`
- macOS (amd64, arm64) - Direct binary
- Linux (amd64, arm64) - Direct binary

**Note**: The `vx-org/mirrors` repository provides a permanent archive of witr binaries, ensuring reliable downloads.

## Usage Examples

```bash
# Basic usage - list all processes
vx witr

# Check specific process by name
vx witr nginx

# Look up by PID
vx witr --pid 1234

# Find process listening on a port
vx witr --port 5432

# Show full process ancestry (tree view)
vx witr postgres --tree

# Show warnings (suspicious env, arguments, parents)
vx witr docker --warnings

# JSON output for scripting
vx witr chrome --json
```

## Integration with vx

witr is integrated into vx as a companion tool for process introspection. You can use it to debug running processes and understand "why is this running?":

```bash
# Find which process is using a specific port
vx witr --port 8080

# Check if a specific tool is running
vx witr node

# Inspect vx-managed tool processes
vx witr python
```

## Platform Support

| Platform | Architectures | Binary Type | Download Source |
|----------|----------------|-------------|-----------------|
| Windows  | amd64, arm64   | `.zip` archive (contains `witr.exe`) | vx-org/mirrors |
| macOS    | amd64, arm64   | Direct binary | vx-org/mirrors |
| Linux    | amd64, arm64   | Direct binary | vx-org/mirrors |

## Source

- **Original repository**: [pranshuparmar/witr](https://github.com/pranshuparmar/witr)
- **Mirror source**: [vx-org/mirrors](https://github.com/vx-org/mirrors) (GitHub Releases)

## License

MIT
