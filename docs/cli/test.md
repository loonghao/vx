# vx test

Test runtime availability and provider functionality. Designed for CI/CD integration and provider development.

## Synopsis

```bash
# Test a single runtime
vx test <runtime> [OPTIONS]

# Test all providers
vx test --all [OPTIONS]

# Test a local provider (for development)
vx test --local <path> [OPTIONS]

# Test a remote extension
vx test --extension <url> [OPTIONS]
```

## Description

The `vx test` command provides a comprehensive testing framework for:

- **Runtime Testing** - Verify if a tool is available and works correctly
- **Provider Testing** - Validate all registered providers in batch
- **Development Testing** - Test providers during development
- **Extension Testing** - Validate third-party provider extensions

## Quick Start

### Test a Single Runtime

```bash
# Basic availability check
vx test node

# Quick platform support check (no installation needed)
vx test node --platform-only

# Silent mode for scripts
if vx test node --quiet; then
    echo "Node.js is available"
fi
```

### Test All Providers

```bash
# Test all registered providers
vx test --all

# JSON output for CI/CD
vx test --all --json > results.json

# Platform check only (fastest)
vx test --all --platform-only
```

### Test During Development

```bash
# Test a local provider directory
cd crates/vx-providers/my-tool
vx test --local . --verbose

# Validate provider.toml configuration
vx test --local . --platform-only
```

## Options

### Target Selection

| Option | Description |
|--------|-------------|
| `<runtime>` | Runtime name to test (e.g., "node", "go") |
| `--all` | Test all registered runtimes |
| `--local <path>` | Test a local provider directory |
| `--extension <url>` | Test a remote provider from URL |

### Test Modes

| Option | Description |
|--------|-------------|
| `--platform-only` | Only check platform support (fastest) |
| `--functional` | Run functional tests (execute --version) |
| `--install` | Test the installation process |
| `--installed` | Check if installed in vx store |
| `--system` | Check if available on system PATH |

### Output Control

| Option | Description |
|--------|-------------|
| `-q, --quiet` | Silent mode, exit code only |
| `--json` | JSON output format |
| `-v, --verbose` | Show detailed test steps |
| `--detailed` | Show extended information |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All tests passed |
| `1` | One or more tests failed |

## Examples

### CI/CD Integration

**GitHub Actions:**

```yaml
name: Test Providers
on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      
      - name: Build vx
        run: cargo build --release
      
      - name: Test all providers
        run: ./target/release/vx test --all --json > results.json
      
      - name: Check results
        run: |
          if jq -e '.failed > 0' results.json; then
            echo "Some tests failed"
            exit 1
          fi
```

**Pre-flight Check in Scripts:**

```bash
#!/bin/bash
# Ensure required tools are available

vx test node --quiet || { echo "Node.js required"; exit 1; }
vx test yarn --quiet || { echo "Yarn required"; exit 1; }

# Run your commands
yarn install
yarn build
```

### Provider Development

```bash
# Step 1: Create provider directory
mkdir -p crates/vx-providers/mytool
cd crates/vx-providers/mytool

# Step 2: Create provider.toml
cat > provider.toml << 'EOF'
name = "mytool"
description = "My awesome tool"

[[runtimes]]
name = "mytool"
description = "Main executable"

[[runtimes.platforms]]
os = "windows"
arch = "x86_64"

[[runtimes.platforms]]
os = "linux"
arch = "x86_64"

[[runtimes.platforms]]
os = "macos"
arch = "x86_64"
arch_variants = ["aarch64"]
EOF

# Step 3: Test the provider
vx test --local . --verbose

# Step 4: Validate before commit
vx test --local . --json
```

### JSON Output

**Single runtime test:**

```json
{
  "runtime": "node",
  "passed": true,
  "platform_supported": true,
  "vx_installed": true,
  "system_available": false,
  "available": true,
  "installed_versions": ["20.0.0", "18.16.0"],
  "functional_test": true
}
```

**Batch test summary:**

```json
{
  "total": 25,
  "passed": 23,
  "failed": 0,
  "skipped": 2,
  "results": [...],
  "errors": []
}
```

## Use Cases

### 1. Validate Tool Availability

Check if a tool is available before running commands:

```bash
if vx test docker --quiet; then
    docker compose up -d
else
    echo "Docker is not available"
    exit 1
fi
```

### 2. Cross-Platform Testing

Test platform support without installation:

```bash
# Quick check if tool supports current platform
vx test spack --platform-only --quiet
echo "Exit code: $?"  # 0 = supported, 1 = not supported
```

### 3. Batch Provider Validation

Test all providers in CI/CD:

```bash
# Run tests and save results
vx test --all --json > test-results.json

# Parse results
FAILED=$(jq '.failed' test-results.json)
if [ "$FAILED" -gt 0 ]; then
    echo "❌ $FAILED tests failed"
    jq '.errors' test-results.json
    exit 1
fi
echo "✅ All tests passed"
```

### 4. Development Workflow

Test provider changes during development:

```bash
# Watch for changes and test
watchexec -e toml "vx test --local . --quiet && echo '✅ OK' || echo '❌ Failed'"
```

## Best Practices

1. **Use `--platform-only` for quick checks** - No installation needed
2. **Use `--json` in CI/CD** - Easy to parse and process
3. **Use `--quiet` in scripts** - Only check exit code
4. **Use `--verbose` for debugging** - See all test steps
5. **Test locally before committing** - `vx test --local .`

## Related Commands

- [`vx install`](./install.md) - Install a runtime
- [`vx list`](./list.md) - List installed runtimes
- [`vx run`](./run.md) - Run commands with specific runtime versions

## See Also

- [Provider Development Guide](../advanced/extension-development.md)
- [CI/CD Integration](../guides/github-action.md)
