# Enhanced Script System

vx's enhanced script system provides powerful argument passing capabilities, making it perfect for complex development workflows and tool integration.

## Overview

The enhanced script system addresses common pain points in development automation:

- **Argument conflicts**: No more issues with `-p`, `--lib`, `--fix` flags
- **Complex tool integration**: Perfect for cargo, eslint, docker, and other tools with many options
- **Script documentation**: Built-in help system for each script
- **Flexible workflows**: Support both simple and complex argument patterns

## Key Features

### 1. Advanced Argument Passing

Pass complex arguments directly to scripts without conflicts:

```bash
# Cargo testing with package selection
vx run test-pkgs -p vx-runtime --lib

# ESLint with multiple options
vx run lint --fix --ext .js,.ts src/

# Docker build with platform selection
vx run docker-build --platform linux/amd64 -t myapp .
```

### 2. Script-Specific Help

Get detailed help for individual scripts:

```bash
# Show help for a specific script
vx run test-pkgs -H
vx run deploy --script-help

# List all available scripts
vx run --list
```

### 3. Flexible Script Definitions

Use `{{args}}` for maximum flexibility:

```toml
[scripts]
# Modern approach: flexible argument handling
test-pkgs = "cargo test {{args}}"
lint = "eslint {{args}}"
build = "docker build {{args}}"

# Legacy approach: still works but limited
test-simple = "cargo test"
```

## Migration Guide

### From Simple Scripts

**Before:**
```toml
[scripts]
test = "cargo test"
lint = "eslint src/"
```

**After:**
```toml
[scripts]
test = "cargo test {{args}}"
lint = "eslint {{args}}"
```

**Benefits:**
- `vx run test -p my-package --lib` now works
- `vx run lint --fix --ext .js,.ts src/` now works

### From Complex Workarounds

**Before:**
```toml
[scripts]
test-unit = "cargo test --lib"
test-integration = "cargo test --test integration"
test-package = "cargo test -p"  # Incomplete, needs manual editing
```

**After:**
```toml
[scripts]
test = "cargo test {{args}}"
```

**Usage:**
```bash
vx run test --lib                    # Unit tests
vx run test --test integration       # Integration tests
vx run test -p my-package --lib      # Package-specific tests
```

## Best Practices

### 1. Use `{{args}}` for Tool Integration

For tools with many command-line options:

```toml
[scripts]
# ✅ Flexible - supports any cargo test arguments
test = "cargo test {{args}}"

# ✅ Flexible - supports any eslint arguments
lint = "eslint {{args}}"

# ✅ Flexible - supports any docker build arguments
build = "docker build {{args}}"

# ❌ Rigid - only works for specific use cases
test-lib = "cargo test --lib"
```

### 2. Provide Script Documentation

Add comments to explain script usage:

```toml
[scripts]
# Run tests with flexible arguments
# Examples:
#   vx run test -p my-package --lib
#   vx run test --test integration
test = "cargo test {{args}}"

# Lint code with flexible options
# Examples:
#   vx run lint --fix
#   vx run lint --ext .js,.ts src/
lint = "eslint {{args}}"
```

### 3. Combine with Environment Variables

```toml
[env]
RUST_LOG = "debug"
CARGO_TERM_COLOR = "always"

[scripts]
test = "cargo test {{args}}"
test-quiet = "RUST_LOG=error cargo test {{args}}"
```

## Advanced Usage

### Multi-Tool Workflows

```toml
[scripts]
# Format and lint in sequence
check = "cargo fmt && cargo clippy {{args}}"

# Build and test with arguments
ci = "cargo build {{args}} && cargo test {{args}}"

# Complex deployment with multiple tools
deploy = "docker build -t myapp {{args}} . && kubectl apply -f k8s/"
```

### Conditional Arguments

```toml
[scripts]
# Use environment variables for conditional behavior
test = "cargo test {{args}} ${EXTRA_TEST_ARGS:-}"
build = "cargo build {{args}} ${BUILD_PROFILE:+--profile $BUILD_PROFILE}"
```

### Integration with External Tools

```toml
[scripts]
# Perfect for tools with many options
prettier = "npx prettier {{args}}"
webpack = "npx webpack {{args}}"
terraform = "terraform {{args}}"
kubectl = "kubectl {{args}}"
```

## Troubleshooting

### Arguments Not Working

**Problem**: Arguments aren't passed to the script.

**Solution**: Ensure your script uses `{{args}}`:

```toml
# ❌ Won't receive arguments
test = "cargo test"

# ✅ Will receive all arguments
test = "cargo test {{args}}"
```

### Complex Arguments

**Problem**: Very complex arguments with quotes or special characters.

**Solution**: Use the `--` separator:

```bash
# For complex cases, use -- separator
vx run build -- --build-arg "VERSION=1.0.0" --target production
```

### Script Help Not Showing

**Problem**: `vx run script --help` shows global help instead of script help.

**Solution**: Use `-H` instead:

```bash
# ✅ Shows script-specific help
vx run script -H

# ❌ Shows global vx help
vx run script --help
```

## Examples

### Rust Development

```toml
[scripts]
test = "cargo test {{args}}"
test-all = "cargo test --workspace {{args}}"
bench = "cargo bench {{args}}"
clippy = "cargo clippy {{args}}"
doc = "cargo doc {{args}}"
```

Usage:
```bash
vx run test -p my-crate --lib
vx run clippy -- -D warnings
vx run doc --open --no-deps
```

### JavaScript/TypeScript Development

```toml
[scripts]
lint = "eslint {{args}}"
format = "prettier {{args}}"
test = "jest {{args}}"
build = "webpack {{args}}"
```

Usage:
```bash
vx run lint --fix --ext .js,.ts src/
vx run format --write "src/**/*.{js,ts}"
vx run test --watch --coverage
vx run build --mode production
```

### Docker Development

```toml
[scripts]
build = "docker build {{args}}"
run = "docker run {{args}}"
compose = "docker-compose {{args}}"
```

Usage:
```bash
vx run build -t myapp:latest --platform linux/amd64 .
vx run run -it --rm -p 3000:3000 myapp:latest
vx run compose up -d --scale web=3
```

## See Also

- [run command reference](../cli/run.md) - Complete command documentation
- [vx.toml configuration](../config/vx-toml.md) - Configuration file reference
- [Variable interpolation](../config/vx-toml.md#variable-interpolation) - Advanced variable usage