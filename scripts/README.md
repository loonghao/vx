# VX Testing Scripts

This directory contains testing and utility scripts for the VX project.

## Provider Testing

### `test-all-providers.ps1` / `test-all-providers.sh`

Comprehensive test suite that validates all VX providers by executing their commands in a clean, isolated temporary environment.

#### Features

- ✅ **Clean Environment**: Uses a temporary VX_HOME directory for testing
- ✅ **Auto-Discovery**: Automatically discovers all providers from `crates/vx-providers/`
- ✅ **Runtime Extraction**: Parses `provider.toml` to extract all runtime names
- ✅ **Command Testing**: Tests `vx list` and `vx <runtime> --version` for each runtime
- ✅ **Auto-Install**: Triggers automatic dependency installation
- ✅ **Summary Report**: Generates detailed test results and statistics
- ✅ **Cache Management**: Automatically cleans up temporary cache (optional)
- ✅ **CI Ready**: Exit codes and JSON reports for CI integration

#### Usage

**Using just commands (recommended on Windows):**

```bash
# Test all providers
just test-providers

# Test with verbose output
just test-providers-verbose

# Test and keep cache for inspection
just test-providers-keep

# Test specific providers (e.g., only "node" and "go")
just test-providers-filter "node"
just test-providers-filter "go"
```

> **Note**: The `just` commands are currently configured for Windows (PowerShell). On Linux/macOS, use the direct script execution below.

**Direct execution (cross-platform):**

```powershell
# PowerShell (Windows)
.\scripts\test-all-providers.ps1

# Options
.\scripts\test-all-providers.ps1 -Verbose        # Verbose output
.\scripts\test-all-providers.ps1 -KeepCache      # Don't delete cache
.\scripts\test-all-providers.ps1 -Filter "node"  # Test only node providers
```

```bash
# Bash (Linux/macOS)
./scripts/test-all-providers.sh

# Options
./scripts/test-all-providers.sh --verbose         # Verbose output
./scripts/test-all-providers.sh --keep-cache      # Don't delete cache
./scripts/test-all-providers.sh --filter "node"   # Test only node providers
```

#### Test Flow

1. **Setup**
   - Creates temporary VX_HOME directory (e.g., `/tmp/vx-test-20260112-143022`)
   - Sets `VX_HOME` environment variable
   - Verifies VX binary exists

2. **Discovery**
   - Scans `crates/vx-providers/` for provider directories
   - Parses each `provider.toml` to extract runtime names
   - Applies filter if specified

3. **Testing**
   - For each runtime:
     - Tests `vx list <runtime>` command
     - Tests `vx <runtime> --version` (triggers auto-install)
   - Collects test results (pass/fail/skip)

4. **Reporting**
   - Displays test summary:
     - Total tests, passed, failed, skipped
     - Success rate percentage
     - Per-provider breakdown
   - Saves JSON report to `<VX_HOME>/test-report.json`
   - Shows cache size and contents

5. **Cleanup**
   - Removes temporary VX_HOME (unless `--keep-cache`)
   - Exits with 0 (success) or 1 (failures)

#### Example Output

```
=== VX Provider Test Suite ===
Project Root: c:\github\vx
Providers Dir: c:\github\vx\crates\vx-providers
Temp VX_HOME: C:\Users\...\AppData\Local\Temp\vx-test-20260112-143022
VX Binary: c:\github\vx\target\debug\vx.exe

=== Discovering Providers ===
Found 45 providers

=== Testing Provider: node ===
  Runtimes: node, npm, npx
  Testing: node
    ✓ vx list node
    ✓ vx node --version
  Testing: npm
    ✓ vx list npm
    ✓ vx npm --version
  Testing: npx
    ✓ vx list npx
    ✓ vx npx --version

=== Testing Provider: go ===
  Runtimes: go, gofmt
  Testing: go
    ✓ vx list go
    ✓ vx go --version
  Testing: gofmt
    ✓ vx list gofmt
    ✓ vx gofmt --version

...

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
  ✓ rust: 6/6 tests passed
  ...

=== Cache Contents ===
Cache size: 2.34 GB
Cache path: C:\Users\...\AppData\Local\Temp\vx-test-20260112-143022

=== Cleaning Up ===
Removing temporary cache...
✓ Cache cleaned

=== Test Result ===
✗ Some tests failed
```

#### CI Integration

The script is designed for CI usage:

1. **Exit Codes**:
   - `0`: All tests passed
   - `1`: Some tests failed or error occurred

2. **JSON Report**:
   - Saved to `<VX_HOME>/test-report.json`
   - Contains detailed test results
   - Can be parsed by CI tools

3. **GitHub Actions Example**:
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

#### Debugging

**Keep cache for inspection:**
```bash
just test-providers-keep
# or
./scripts/test-all-providers.sh --keep-cache
```

Then manually inspect:
```bash
export VX_HOME="/tmp/vx-test-XXXXXX"  # Use actual temp path
ls -la $VX_HOME/tools/
cat $VX_HOME/test-report.json
```

**Test specific provider:**
```bash
just test-providers-filter "node"
```

**Verbose mode:**
```bash
just test-providers-verbose
```

#### Troubleshooting

**"VX binary not found"**
- Run `cargo build` first
- Or use `just test-providers` which builds automatically

**Rate limiting errors**
- Add delays between tests (already included)
- Use `--filter` to test fewer providers at once

**Permission errors on cleanup**
- Use `--keep-cache` and manually delete later
- Some tools may leave locked files (e.g., on Windows)

**Unexpected failures**
- Check `test-report.json` for details
- Run with `--verbose` for full output
- Test individual provider: `just test-providers-filter "provider-name"`

## Adding More Scripts

When adding new scripts:

1. Create both `.ps1` (PowerShell) and `.sh` (Bash) versions
2. Make `.sh` executable: `chmod +x scripts/your-script.sh`
3. Add just commands to `justfile` with OS detection
4. Document in this README
5. Use consistent error handling and exit codes
