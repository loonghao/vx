# Testing All VX Providers

## Overview

VX includes a comprehensive testing system that validates all providers by executing their commands in a clean, isolated environment. This ensures that all providers work correctly and helps catch regressions early.

## Quick Start

```bash
# Test all providers
just test-providers

# Test with verbose output
just test-providers-verbose

# Test and keep cache
just test-providers-keep

# Test specific providers
just test-providers-filter "node"
```

## How It Works

### 1. Clean Environment Setup

The test system creates a **temporary VX_HOME** directory for each test run:

```
/tmp/vx-test-20260112-143022/  (Linux/macOS)
C:\Users\...\AppData\Local\Temp\vx-test-20260112-143022\  (Windows)
```

This ensures:
- ✅ Tests don't affect your real VX installation
- ✅ Each test run starts from a clean slate
- ✅ No leftover artifacts between test runs
- ✅ Parallel CI runs don't interfere with each other

### 2. Provider Discovery

The system automatically discovers all providers:

1. Scans `crates/vx-providers/` directory
2. Finds all directories containing `provider.toml`
3. Parses each `provider.toml` to extract runtime names
4. Applies name filter if specified

Example `provider.toml` parsing:
```toml
[[runtimes]]
name = "node"  # ← Extracted

[[runtimes]]
name = "npm"   # ← Extracted

[[runtimes]]
name = "npx"   # ← Extracted
```

### 3. Command Testing

For each discovered runtime, the system tests:

#### Test 1: List Command
```bash
vx list node
```

Expected behavior:
- Returns available versions (may be empty if not installed)
- Exit code: 0

#### Test 2: Version Command
```bash
vx node --version
```

Expected behavior:
- Auto-installs runtime if not present
- Downloads and extracts latest version
- Executes `node --version`
- Returns version string
- Exit code: 0

### 4. Result Collection

Each test produces:

```json
{
  "Command": "node --version",
  "Result": {
    "Success": true,
    "Output": "v20.11.0",
    "ExitCode": 0
  }
}
```

### 5. Reporting

The system generates:

**Console Output:**
```
=== Test Summary ===
Total Tests: 120
Passed: 115
Failed: 3
Skipped: 2
Success Rate: 95.83%

=== Provider Details ===
  ✓ node: 6/6 tests passed
  ✓ go: 4/4 tests passed
  ✗ python: 2/3 tests passed
```

**JSON Report** (`test-report.json`):
```json
{
  "Total": 120,
  "Passed": 115,
  "Failed": 3,
  "Skipped": 2,
  "Providers": [
    {
      "Name": "node",
      "Runtimes": ["node", "npm", "npx"],
      "Tests": [...]
    }
  ]
}
```

### 6. Cleanup

After testing:
- Displays cache size and location
- Deletes temporary VX_HOME (unless `--keep-cache`)
- Exits with appropriate code (0 = success, 1 = failures)

## Testing Scenarios

### Local Development

**Test all providers:**
```bash
just test-providers
```

**Test during provider development:**
```bash
just test-providers-filter "mynewprovider"
```

**Debug a failing test:**
```bash
# Keep cache for inspection
just test-providers-keep

# Set VX_HOME to temp directory
export VX_HOME="/tmp/vx-test-XXXXXX"

# Manually test commands
vx list node
vx node --version
```

### CI/CD Integration

**GitHub Actions:**
```yaml
- name: Test All Providers
  run: just test-providers

- name: Upload Test Report
  if: always()
  uses: actions/upload-artifact@v6
  with:
    name: provider-test-report
    path: /tmp/vx-test-*/test-report.json
```

**GitLab CI:**
```yaml
test-providers:
  script:
    - just test-providers
  artifacts:
    when: always
    paths:
      - /tmp/vx-test-*/test-report.json
```

**Jenkins:**
```groovy
stage('Test Providers') {
    sh 'just test-providers'
    archiveArtifacts artifacts: '/tmp/vx-test-*/test-report.json', allowEmptyArchive: true
}
```

### Pull Request Validation

Test only changed providers:

```bash
# Get list of changed providers
CHANGED=$(git diff --name-only main...HEAD | \
  grep '^crates/vx-providers/' | \
  cut -d'/' -f3 | \
  sort -u)

# Test them
for provider in $CHANGED; do
  just test-providers-filter "$provider"
done
```

## Command Reference

### `just test-providers`

Tests all providers with default settings.

**Features:**
- Auto-builds VX if needed
- Uses temporary VX_HOME
- Cleans up after completion
- Exit code indicates success/failure

**Usage:**
```bash
just test-providers
```

### `just test-providers-verbose`

Same as `test-providers` but with verbose output.

**Shows:**
- Command output for each test
- Detailed error messages
- Installed versions

**Usage:**
```bash
just test-providers-verbose
```

### `just test-providers-keep`

Tests all providers but keeps the cache for inspection.

**Use cases:**
- Debugging failed tests
- Inspecting installed files
- Verifying download URLs
- Manual testing

**Usage:**
```bash
just test-providers-keep
# Cache location printed in output
export VX_HOME="/tmp/vx-test-XXXXXX"
vx list node
```

### `just test-providers-filter FILTER`

Tests only providers matching the filter.

**Examples:**
```bash
# Test only Node.js provider
just test-providers-filter "node"

# Test all Python-related providers
just test-providers-filter "python"

# Test multiple providers (regex)
just test-providers-filter "node|go|rust"
```

## Script Options

### PowerShell (`test-all-providers.ps1`)

```powershell
# Verbose output
.\scripts\test-all-providers.ps1 -Verbose

# Keep cache
.\scripts\test-all-providers.ps1 -KeepCache

# Filter providers
.\scripts\test-all-providers.ps1 -Filter "node"

# Combine options
.\scripts\test-all-providers.ps1 -Verbose -KeepCache -Filter "go"
```

### Bash (`test-all-providers.sh`)

```bash
# Verbose output
./scripts/test-all-providers.sh --verbose

# Keep cache
./scripts/test-all-providers.sh --keep-cache

# Filter providers
./scripts/test-all-providers.sh --filter "node"

# Combine options
./scripts/test-all-providers.sh --verbose --keep-cache --filter "go"
```

## Troubleshooting

### "VX binary not found"

**Problem:** Test script can't find compiled VX binary.

**Solution:**
```bash
cargo build
# or
just test-providers  # Auto-builds first
```

### Rate Limiting

**Problem:** GitHub API rate limiting when testing many providers.

**Symptoms:**
- HTTP 403 errors
- "API rate limit exceeded" messages

**Solutions:**
1. Test fewer providers at once:
   ```bash
   just test-providers-filter "node"
   ```

2. Add delay between tests (already included in scripts)

3. Set GitHub token (if testing locally):
   ```bash
   export GITHUB_TOKEN="your_token"
   just test-providers
   ```

### Permission Errors on Cleanup

**Problem:** Can't delete temporary cache on Windows.

**Cause:** Some tools leave file locks (antivirus, indexing).

**Solutions:**
1. Use `--keep-cache` and delete manually later:
   ```bash
   just test-providers-keep
   # Delete manually when ready
   ```

2. Close programs that might lock files

3. Restart and retry

### Test Failures

**Problem:** Some tests fail unexpectedly.

**Investigation steps:**

1. **Run with verbose output:**
   ```bash
   just test-providers-verbose
   ```

2. **Keep cache and investigate:**
   ```bash
   just test-providers-keep
   export VX_HOME="/tmp/vx-test-XXXXXX"
   vx node --version  # Reproduce manually
   ```

3. **Check test report:**
   ```bash
   cat /tmp/vx-test-XXXXXX/test-report.json | jq '.Providers[] | select(.Name == "node")'
   ```

4. **Test specific provider:**
   ```bash
   just test-providers-filter "node"
   ```

### Disk Space Issues

**Problem:** Running out of disk space during tests.

**Cause:** Large downloads for all providers (2-10GB).

**Solutions:**
1. Test fewer providers:
   ```bash
   just test-providers-filter "node|go"
   ```

2. Clean up between test runs:
   ```bash
   just test-providers  # Auto-cleans
   ```

3. Use `--keep-cache` carefully (can accumulate)

## Best Practices

### During Development

1. **Test your provider immediately:**
   ```bash
   just test-providers-filter "mynewprovider"
   ```

2. **Keep cache during debugging:**
   ```bash
   just test-providers-keep
   ```

3. **Test related providers:**
   ```bash
   # If your provider depends on node
   just test-providers-filter "node|mynewprovider"
   ```

### Before Committing

1. **Test all providers:**
   ```bash
   just test-providers
   ```

2. **Fix any failures before pushing**

3. **Update provider.toml if needed**

### In CI/CD

1. **Run on all platforms:**
   - Ubuntu (Linux)
   - macOS
   - Windows

2. **Upload test reports:**
   ```yaml
   artifacts:
     paths:
       - /tmp/vx-test-*/test-report.json
   ```

3. **Fail fast on errors:**
   ```yaml
   - run: just test-providers
   # Exit code 1 fails the pipeline
   ```

4. **Cache Cargo builds:**
   ```yaml
   cache:
     key: ${{ runner.os }}-cargo
     paths:
       - target/
       - ~/.cargo/
   ```

## Integration Examples

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Test only changed providers
CHANGED=$(git diff --cached --name-only | \
  grep '^crates/vx-providers/' | \
  cut -d'/' -f3 | \
  sort -u)

if [ -n "$CHANGED" ]; then
  echo "Testing changed providers: $CHANGED"
  for provider in $CHANGED; do
    just test-providers-filter "$provider" || exit 1
  done
fi
```

### GitHub Actions Matrix

```yaml
test-providers:
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest, windows-latest]
      provider-group:
        - "node|bun|deno"
        - "go|rust"
        - "python|uv"
  runs-on: ${{ matrix.os }}
  steps:
    - run: just test-providers-filter "${{ matrix.provider-group }}"
```

### Makefile Integration

```makefile
.PHONY: test-all test-quick test-providers

test-all: test test-providers

test-quick:
	cargo test --lib

test-providers:
	just test-providers
```

## Performance Tips

### Speed Up Tests

1. **Parallel testing** (future enhancement):
   - Run provider tests in parallel
   - Requires thread-safe VX_HOME handling

2. **Cache downloads:**
   - Reuse downloaded archives
   - Share cache between test runs

3. **Selective testing:**
   ```bash
   # Test only core providers
   just test-providers-filter "node|go|python"
   ```

### Reduce Disk Usage

1. **Clean up automatically** (default behavior)

2. **Test in batches:**
   ```bash
   for group in "node|go" "python|rust" "docker|kubectl"; do
     just test-providers-filter "$group"
   done
   ```

3. **Skip large providers in dev:**
   ```bash
   # Focus on your work
   just test-providers-filter "mynewprovider"
   ```

## Future Enhancements

Planned improvements:

1. **Parallel execution** - Test multiple providers simultaneously
2. **Download caching** - Reuse downloaded archives across test runs
3. **Selective version testing** - Test specific versions, not just latest
4. **Network mock mode** - Test without internet (using cached data)
5. **Performance benchmarks** - Track installation and execution times
6. **Coverage reports** - Track which providers/commands are tested
7. **Flakiness detection** - Identify and retry flaky tests

## Contributing

When adding new providers:

1. **Add provider.toml** with all runtimes
2. **Test locally:**
   ```bash
   just test-providers-filter "yournewprovider"
   ```
3. **Ensure all tests pass**
4. **Update documentation** if adding new features
5. **Submit PR** with passing CI tests

## See Also

- [scripts/README.md](../scripts/README.md) - Script documentation
- [Provider Development Guide](provider-development.md) - How to create providers
- [CI/CD Guide](ci-cd.md) - Setting up CI pipelines
- [GitHub Actions Example](.github/workflows/test-providers.yml.example) - Ready-to-use workflow
