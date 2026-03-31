# vx test

Test runtime availability, provider metadata, and full CI installation flows. The command supports both lightweight local checks and end-to-end provider validation in CI.

## Synopsis

```bash
# Test a single runtime
vx test <runtime> [OPTIONS]

# Batch-check all registered runtimes (metadata/platform checks)
vx test --all [OPTIONS]

# Full CI mode: install + functional tests
vx test --ci [OPTIONS]

# Full CI mode for a selected runtime subset
vx test --ci --ci-runtimes node,go,uv [OPTIONS]

# Test a local provider during development
vx test --local <path> [OPTIONS]

# Test a remote extension
vx test --extension <url> [OPTIONS]
```

## Description

The `vx test` command covers four common workflows:

- **Runtime checks**: validate a single runtime on the current machine
- **Batch registry checks**: inspect all registered runtimes with `--all`
- **Provider CI validation**: run install + functional verification with `--ci`
- **Provider development**: validate a local `provider.star` before opening a PR

## Quick Start

### Test a single runtime

```bash
# Basic availability check
vx test node

# Fast platform-only check (no installation)
vx test node --platform-only

# Quiet scripting mode
if vx test node --quiet; then
  echo "node is available"
fi
```

### Run full provider CI validation

```bash
# Test every CI-eligible runtime
vx test --ci --keep-going --detailed

# Test a focused runtime subset
vx test --ci --ci-runtimes node,go,uv --keep-going --json > results.json

# Use an isolated reusable runtime store
vx test --ci --ci-runtimes node,go --vx-root "$RUNNER_TEMP/vx-provider-test"
```

### Validate a local provider during development

```bash
cd crates/vx-providers/mytool

# Check that provider metadata loads and platform rules are valid
vx test --local . --platform-only

# Run the local provider test flow with extra logs
vx test --local . --verbose
```

## Target Selection

| Option | Description |
|--------|-------------|
| `<runtime>` | Test a specific runtime such as `node`, `go`, or `uv` |
| `--all` | Test all registered runtimes without the full CI install flow |
| `--ci` | Run full end-to-end CI validation (install + functional tests) |
| `--ci-runtimes <csv>` | Restrict CI mode to a comma-separated runtime list |
| `--ci-skip <csv>` | Skip specific runtimes in CI mode |
| `--local <path>` | Test a local provider directory containing `provider.star` |
| `--extension <url>` | Test a remote provider extension |

## Test Modes

| Option | Description |
|--------|-------------|
| `--platform-only` | Only check platform support; no install required |
| `--functional` | Run functional tests such as `--version` |
| `--install` | Test the installation flow only |
| `--installed` | Check whether the runtime already exists in the vx store |
| `--system` | Check whether the runtime exists on the system `PATH` |
| `--keep-going` | Continue CI validation after failures |
| `--cleanup` | Uninstall runtimes after CI validation and verify removal |
| `--timeout <seconds>` | Per-runtime timeout in CI mode (default: `300`) |
| `--vx-root <path>` | Use a custom vx root for an isolated or cacheable test store |
| `--temp-root` | Use a temporary vx root that is cleaned up automatically |

## Output Control

| Option | Description |
|--------|-------------|
| `-q, --quiet` | Exit code only |
| `--json` | Print JSON output for automation |
| `-v, --verbose` | Show detailed execution steps |
| `--detailed` | Show expanded end-of-run details |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All requested tests passed |
| `1` | One or more tests failed |

## Examples

### GitHub Actions provider validation

```yaml
name: Test Providers
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - name: Build vx
        run: vx cargo build --release

      - name: Test a provider subset
        run: |
          ./target/release/vx test \
            --ci \
            --ci-runtimes node,go,uv \
            --vx-root "$RUNNER_TEMP/vx-provider-test" \
            --keep-going \
            --json > results.json

      - name: Fail on CI test failures
        run: |
          if jq -e '.failed > 0' results.json; then
            exit 1
          fi
```

### Local provider authoring workflow

```bash
mkdir -p crates/vx-providers/mytool
cd crates/vx-providers/mytool

cat > provider.star <<'EOF'
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name = "mytool"
description = "My awesome tool"
runtimes = [runtime_def("mytool")]
permissions = github_permissions()

_p = github_rust_provider("example", "mytool",
    asset = "mytool-{vversion}-{triple}.{ext}",
)

fetch_versions = _p["fetch_versions"]
download_url = _p["download_url"]
install_layout = _p["install_layout"]
store_root = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment = _p["environment"]
EOF

vx test --local . --platform-only
vx test --local . --verbose
```

### Full CI smoke test from the workspace

```bash
# Match the quick CI smoke flow used in the repo
vx just test-ci-quick

# Or run a custom subset directly
./target/debug/vx test --ci --ci-runtimes node,go,uv,just --temp-root --verbose
```

## JSON Output

### Single runtime result

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

### CI summary result

```json
{
  "total": 3,
  "passed": 3,
  "failed": 0,
  "skipped": 0,
  "results": []
}
```

## Best Practices

1. **Use `--platform-only`** for fast local validation before deeper tests.
2. **Use `--ci`** when you need real install + execution coverage.
3. **Use `--vx-root`** in CI so runtime downloads can be cached between jobs.
4. **Use `--keep-going` + `--json`** for matrix-style automation and summarized failure reporting.
5. **Use `vx just test-providers-static` first** when changing provider metadata or discovery logic.

## Related Commands

- [`vx install`](./install.md) - Install a runtime explicitly
- [`vx list`](./list.md) - List runtimes available in the registry or on the system
- [`vx run`](./run.md) - Run a project script or command with vx-managed runtimes

## See Also

- [Provider Development Guide](../guide/creating-provider.md)
- [GitHub Actions Guide](../guides/github-action.md)
