# Migrating from `vx check` to `vx test`

## Why Migrate?

`vx test` is the new recommended command for testing runtime availability. It offers:

- ✅ **Better CI/CD integration** - JSON output, silent mode, exit codes
- ✅ **More flexible** - Test single runtime or all at once
- ✅ **Platform-only mode** - Fast platform validation
- ✅ **Clearer semantics** - "test" is more intuitive than "check"

`vx check` is now deprecated and will be removed in a future version.

## Migration Guide

### Basic Usage

**Before (`vx check`):**
```bash
vx check yarn
```

**After (`vx test`):**
```bash
vx test yarn
```

### Silent Mode

**Before:**
```bash
vx check yarn --quiet
```

**After:**
```bash
vx test yarn --quiet
```

### Platform Check

**Before (not supported):**
```bash
# Had to parse error messages
vx check spack 2>&1 | grep "does not support"
```

**After:**
```bash
# Built-in platform-only flag
vx test spack --platform-only
```

### CI/CD Scripts

**Before:**
```bash
#!/bin/bash
if vx check node --quiet; then
    echo "Node available"
fi
```

**After:**
```bash
#!/bin/bash
# Same syntax, just replace 'check' with 'test'
if vx test node --quiet; then
    echo "Node available"
fi
```

### JSON Output (NEW)

**Before (not supported):**
```bash
# Had to parse text output
```

**After:**
```bash
# Get structured JSON
vx test go --json
```

Output:
```json
{
  "runtime": "go",
  "platform_supported": true,
  "vx_installed": true,
  "system_available": false,
  "available": true,
  "auto_installable": true,
  "installed_versions": ["1.21.0"]
}
```

### Test All Runtimes (NEW)

**Before (not supported):**
```bash
# Had to loop manually
for runtime in node go rust; do
    vx check $runtime
done
```

**After:**
```bash
# Single command
vx test --all
```

## Automated Migration

Use this script to update your codebase:

```bash
# Replace in shell scripts
find . -type f \( -name "*.sh" -o -name "*.bash" \) \
  -exec sed -i 's/vx check/vx test/g' {} +

# Replace in CI configs
find . -type f \( -name "*.yml" -o -name "*.yaml" \) \
  -exec sed -i 's/vx check/vx test/g' {} +

# Replace in Makefiles
find . -type f -name "Makefile*" \
  -exec sed -i 's/vx check/vx test/g' {} +
```

## Feature Comparison

| Feature | `vx check` | `vx test` |
|---------|-----------|----------|
| Platform check | ✓ | ✓ |
| Installation check | ✓ | ✓ |
| System PATH check | ✓ | ✓ |
| Platform-only mode | ✗ | ✓ |
| JSON output | ✗ | ✓ |
| Test all runtimes | ✗ | ✓ |
| Silent mode | ✓ | ✓ |
| Detailed output | ✓ | ✓ |
| Exit codes | ✓ | ✓ |
| Status | **Deprecated** | **Recommended** |

## Timeline

- **v0.6.x**: `vx check` deprecated, `vx test` introduced
- **v0.7.x**: `vx check` hidden from help (still works)
- **v1.0.0**: `vx check` removed

## Backward Compatibility

`vx check` currently redirects to `vx test` internally, so existing scripts will continue to work. However, we recommend migrating to `vx test` as soon as possible.

## Need Help?

- [vx test documentation](../commands/vx-test.md)
- [CI/CD Integration Guide](../guides/ci-cd.md)
- [GitHub Discussions](https://github.com/your-org/vx/discussions)
