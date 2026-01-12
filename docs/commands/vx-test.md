# vx test

Test runtime availability and platform support. Designed for CI/CD integration.

## Synopsis

```bash
vx test <runtime> [OPTIONS]
vx test --all [OPTIONS]
```

## Description

The `vx test` command checks if a runtime is available and supports the current platform. It's specifically designed for:

- **Local testing**: Verify tools before running scripts
- **CI/CD pipelines**: Test tool availability in automated workflows
- **Platform validation**: Check cross-platform compatibility

## Usage Examples

### Test a Single Runtime

```bash
# Test if yarn is available
vx test yarn

# Output:
# ✓ Runtime 'yarn' is installed in vx store
# ✗ Runtime 'yarn' is not available on system PATH
# 
# ⚠ Runtime 'yarn' is not available but can be auto-installed
#   Run: vx install yarn
```

### Platform-Only Check (Fast)

```bash
# Check if deno supports the current platform
vx test deno --platform-only

# Exit code: 0 (supported) or 1 (not supported)
```

### CI/CD Integration (Silent Mode)

```bash
# Silent check for CI/CD
if vx test node --quiet; then
    echo "Node.js is available"
else
    echo "Node.js is not available"
    exit 1
fi
```

### JSON Output for CI

```bash
# Get structured data for parsing
vx test go --json

# Output:
# {
#   "runtime": "go",
#   "platform_supported": true,
#   "vx_installed": true,
#   "system_available": false,
#   "available": true,
#   "auto_installable": true,
#   "installed_versions": ["1.21.0"]
# }
```

### Test All Runtimes

```bash
# Test all registered runtimes
vx test --all

# Output:
# ✓ node - available
# ✓ go - available
# ✗ rust - not available
# ⚠ spack - platform not supported
# 
# === Test Summary ===
# Total: 50
# Passed: 45
# Failed: 5
```

### Detailed Information

```bash
# Show detailed detection information
vx test yarn --detailed

# Output:
# ✓ Runtime 'yarn' supports the current platform
# ✓ Runtime 'yarn' is installed in vx store
#   Versions: 1.22.19, 3.6.0
# ✗ Runtime 'yarn' is not available on system PATH
# ✓ Runtime 'yarn' can be auto-installed by vx
```

## Options

| Option | Description |
|--------|-------------|
| `<runtime>` | Runtime name to test (e.g., "yarn", "node", "go") |
| `--all` | Test all registered runtimes |
| `--platform-only` | Only test platform support (fastest) |
| `--installed` | Check if runtime is installed in vx store |
| `--system` | Check if runtime is available on system PATH |
| `--detailed` | Show detailed detection information |
| `-q, --quiet` | Silent mode: exit with code 0 if passes, 1 if fails |
| `--json` | JSON output format (for CI integration) |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Runtime is available (installed or on PATH) |
| `1` | Runtime is not available or doesn't support platform |

## CI/CD Examples

### GitHub Actions

```yaml
name: Test Tools
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
      
      - name: Install vx
        run: curl -sSfL https://vx.pm/install.sh | bash
      
      - name: Test Node.js availability
        run: vx test node --quiet
      
      - name: Test all tools
        run: vx test --all --json > test-results.json
      
      - name: Upload results
        uses: actions/upload-artifact@v6
        with:
          name: test-results
          path: test-results.json
```

### GitLab CI

```yaml
test:tools:
  script:
    - vx test node --quiet || exit 1
    - vx test go --quiet || exit 1
    - vx test rust --quiet || exit 1
```

### Jenkins

```groovy
pipeline {
    agent any
    stages {
        stage('Test Tools') {
            steps {
                sh 'vx test --all --json > test-results.json'
                archiveArtifacts artifacts: 'test-results.json'
            }
        }
    }
}
```

## Comparison with `vx check`

`vx test` is the modern replacement for `vx check`:

| Feature | `vx check` | `vx test` |
|---------|-----------|----------|
| Platform check | ✓ | ✓ |
| Installation check | ✓ | ✓ |
| JSON output | ✗ | ✓ |
| Test all runtimes | ✗ | ✓ |
| CI-friendly | Basic | Advanced |
| Status | Deprecated | Recommended |

## Use Cases

### 1. Pre-flight Check in Scripts

```bash
#!/bin/bash
# Ensure required tools are available before running

vx test node --quiet || { echo "Node.js required"; exit 1; }
vx test yarn --quiet || { echo "Yarn required"; exit 1; }

# Run your script
yarn install
yarn build
```

### 2. Cross-Platform Validation

```bash
# Test if a tool supports the current platform
if vx test spack --platform-only --quiet 2>/dev/null; then
    echo "Spack is supported on this platform"
else
    echo "Spack is not supported, using alternative..."
fi
```

### 3. CI Matrix Testing

```yaml
strategy:
  matrix:
    tool: [node, go, rust, python]
steps:
  - name: Test ${{ matrix.tool }}
    run: vx test ${{ matrix.tool }} --json > result-${{ matrix.tool }}.json
```

## Related Commands

- [`vx install`](./vx-install.md) - Install a runtime
- [`vx list`](./vx-list.md) - List installed runtimes
- [`vx doctor`](./vx-doctor.md) - Diagnose vx installation

## See Also

- [CI/CD Integration Guide](../guides/ci-cd.md)
- [Testing Best Practices](../guides/testing.md)
- [Platform Support](../reference/platforms.md)
