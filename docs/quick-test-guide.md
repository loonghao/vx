# Quick Provider Testing Guide

## TL;DR

```bash
# Test everything
just test-providers

# Test with details
just test-providers-verbose

# Test specific provider
just test-providers-filter "node"
```

## What This Does

1. ✅ Creates temporary VX_HOME (isolated testing)
2. ✅ Auto-discovers all providers
3. ✅ Tests `vx list` and `vx <tool> --version`
4. ✅ Shows pass/fail results
5. ✅ Cleans up automatically

## Example Output

```
=== Discovering Providers ===
Found 45 providers

=== Testing Provider: node ===
  ✓ vx list node
  ✓ vx node --version

=== Test Summary ===
Total: 120 | Passed: 115 | Failed: 3
Success Rate: 95.83%

✓ Cache cleaned
```

## When to Use

### During Development
```bash
# Created new provider
just test-providers-filter "mynewprovider"

# Making changes
just test-providers-filter "node"
```

### Before Committing
```bash
# Test everything
just test-providers
```

### Debugging
```bash
# Keep cache for inspection
just test-providers-keep

# Set env and test manually
export VX_HOME="/tmp/vx-test-XXXXXX"
vx list node
```

## Command Options

| Command | Description |
|---------|-------------|
| `just test-providers` | Test all providers, auto-clean |
| `just test-providers-verbose` | Show detailed output |
| `just test-providers-keep` | Keep cache for debugging |
| `just test-providers-filter "node"` | Test specific provider |

## Direct Script Usage

### Windows
```powershell
.\scripts\test-all-providers.ps1
.\scripts\test-all-providers.ps1 -Verbose
.\scripts\test-all-providers.ps1 -KeepCache
.\scripts\test-all-providers.ps1 -Filter "node"
```

### Linux/macOS
```bash
./scripts/test-all-providers.sh
./scripts/test-all-providers.sh --verbose
./scripts/test-all-providers.sh --keep-cache
./scripts/test-all-providers.sh --filter "node"
```

## Troubleshooting

**"VX binary not found"**
```bash
cargo build
# or let just do it
just test-providers
```

**Tests fail unexpectedly**
```bash
# Verbose mode
just test-providers-verbose

# Keep cache and inspect
just test-providers-keep
export VX_HOME="/tmp/vx-test-XXXXXX"
```

**Need more help?**
- See [testing-providers.md](testing-providers.md) for full guide
- See [scripts/README.md](../scripts/README.md) for script docs

## CI Integration

```yaml
# GitHub Actions
- run: just test-providers

# Exit code 0 = success, 1 = failure
```

See `.github/workflows/test-providers.yml.example` for complete workflow.
