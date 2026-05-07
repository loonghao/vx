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

## Installation

witr is available for all platforms (Windows, macOS, Linux) via `vx`:

```bash
vx install witr@latest
```

vx downloads witr from the [vx-org/mirrors](https://github.com/vx-org/mirrors) repository, which provides pre-built binaries for:
- Windows (amd64, arm64)
- macOS (amd64, arm64)
- Linux (amd64, arm64)

## Usage Examples

```bash
# Basic usage
vx witr

# Check specific process
vx witr <pid>

# Verbose output
vx witr --verbose
```

## Platform Support

| Platform | Architectures | Binary Type |
|----------|----------------|-------------|
| Windows  | amd64, arm64   | `.zip` archive (contains `witr.exe`) |
| macOS    | amd64, arm64   | Direct binary |
| Linux    | amd64, arm64   | Direct binary |

## Source

- **Original repository**: [pranshuparmar/witr](https://github.com/pranshuparmar/witr)
- **Mirror source**: [vx-org/mirrors](https://github.com/vx-org/mirrors) (GitHub Releases)

## License

MIT
