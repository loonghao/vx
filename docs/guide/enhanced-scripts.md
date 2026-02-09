# Enhanced Script System

vx's enhanced script system provides powerful argument passing, DAG-based dependency execution, and flexible workflow automation — making it a complete task runner built into your project configuration.

## Overview

The enhanced script system addresses common pain points in development automation:

- **DAG-based workflow execution**: Scripts can declare dependencies on other scripts, forming a directed acyclic graph (DAG) that is automatically resolved via topological sort
- **Circular dependency detection**: vx detects and reports circular dependencies at execution time
- **Argument conflicts**: No more issues with `-p`, `--lib`, `--fix` flags
- **Complex tool integration**: Perfect for cargo, eslint, docker, and other tools with many options
- **Script documentation**: Built-in help system for each script
- **Flexible workflows**: Support both simple and complex argument patterns

## DAG-Based Workflow Execution

The most powerful feature of the script system is **dependency-based execution**. You can declare that a script depends on other scripts, and vx will execute them in the correct order using topological sorting.

### How It Works

```
           ┌─────────┐
           │  deploy  │
           └────┬────┘
                │ depends
          ┌─────┴─────┐
          │           │
     ┌────▼───┐  ┌───▼────┐
     │  build  │  │  test  │
     └────┬───┘  └───┬────┘
          │           │ depends
          │     ┌─────┴─────┐
          │     │           │
          │  ┌──▼──┐  ┌───▼─────┐
          │  │ lint │  │typecheck│
          │  └─────┘  └─────────┘
          │
     ┌────▼───────┐
     │  generate   │
     └────────────┘
```

When you run `vx run deploy`, vx:

1. **Builds the dependency graph** — collects all transitive dependencies
2. **Detects cycles** — reports an error if circular dependencies exist (e.g., `A → B → A`)
3. **Topological sorts** — determines the correct execution order
4. **Executes sequentially** — runs each script once, in dependency order
5. **Fails fast** — if any dependency fails, the entire chain stops immediately

### Basic Dependency Example

```toml
[scripts]
lint = "eslint . && prettier --check ."
typecheck = "tsc --noEmit"
test = "vitest run"
build = "npm run build"

[scripts.ci]
command = "echo '✅ All checks passed!'"
description = "Run all CI checks"
depends = ["lint", "typecheck", "test", "build"]
```

```bash
vx run ci
# Execution order: lint → typecheck → test → build → ci
# (dependencies resolved via topological sort)
```

### Multi-Level Dependencies

Dependencies can be nested — vx resolves the full transitive dependency graph:

```toml
[scripts]
generate = "protoc --go_out=. *.proto"
lint = "golangci-lint run"

[scripts.build]
command = "go build -o app ./cmd/server"
description = "Build the server"
depends = ["generate"]

[scripts.test]
command = "go test ./..."
description = "Run tests"
depends = ["lint", "generate"]

[scripts.deploy]
command = "kubectl apply -f k8s/"
description = "Deploy to Kubernetes"
depends = ["build", "test"]
```

```bash
vx run deploy
# Resolved order: generate → lint → build → test → deploy
# Note: generate runs only ONCE even though both build and test depend on it
```

### Each Script Runs Once

The DAG executor tracks visited nodes — each script in the dependency graph executes **at most once**, even if multiple scripts depend on it.

### Circular Dependency Detection

vx detects circular dependencies and reports a clear error:

```toml
[scripts.a]
command = "echo a"
depends = ["b"]

[scripts.b]
command = "echo b"
depends = ["a"]    # Circular!
```

```bash
vx run a
# Error: Circular dependency detected: a -> b -> a
```

### Dependencies with Environment Variables

Each script in the dependency chain can have its own environment variables and working directory:

```toml
[env]
NODE_ENV = "development"

[scripts.migrate]
command = "prisma migrate deploy"
env = { DATABASE_URL = "postgres://localhost/myapp" }
cwd = "backend"

[scripts.seed]
command = "python seed.py"
cwd = "backend"
depends = ["migrate"]

[scripts.dev]
command = "npm run dev"
description = "Start dev server after DB setup"
depends = ["seed"]
```

## Real-World Workflow Patterns

### Full-Stack CI Pipeline

```toml
[scripts]
# Individual check tasks
lint:frontend = "cd frontend && npm run lint"
lint:backend = "cd backend && uvx ruff check ."
typecheck = "cd frontend && tsc --noEmit"
test:unit = "cd backend && uv run pytest tests/unit"
test:integration = "cd backend && uv run pytest tests/integration"
build:frontend = "cd frontend && npm run build"
build:backend = "cd backend && cargo build --release"

# Composite tasks using DAG dependencies
[scripts.lint]
command = "echo '✅ All linting passed'"
depends = ["lint:frontend", "lint:backend"]

[scripts.test]
command = "echo '✅ All tests passed'"
depends = ["test:unit", "test:integration"]

[scripts.build]
command = "echo '✅ All builds completed'"
depends = ["build:frontend", "build:backend"]

[scripts.ci]
command = "echo '🎉 CI pipeline passed!'"
description = "Run the full CI pipeline"
depends = ["lint", "typecheck", "test", "build"]
```

```bash
vx run ci
# Runs: lint:frontend → lint:backend → lint → typecheck
#     → test:unit → test:integration → test
#     → build:frontend → build:backend → build → ci
```

### Release Workflow

```toml
[scripts]
changelog = "git-cliff -o CHANGELOG.md"
version-bump = "npm version {{arg1}}"

[scripts.build-release]
command = "cargo build --release"
depends = ["changelog"]

[scripts.package]
command = "tar czf dist/app.tar.gz -C target/release app"
depends = ["build-release"]

[scripts.publish]
command = "gh release create v{{arg1}} dist/app.tar.gz"
description = "Create a new release"
depends = ["version-bump", "package"]
```

```bash
vx run publish 1.2.0
# Runs: changelog → build-release → package → version-bump → publish
```

### Cross-Language Build Pipeline

```toml
[scripts]
proto-gen = "protoc --go_out=. --python_out=. api/*.proto"

[scripts.build:go]
command = "go build -o bin/server ./cmd/server"
depends = ["proto-gen"]

[scripts.build:python]
command = "uv run python -m build"
depends = ["proto-gen"]

[scripts.build:frontend]
command = "npm run build"
cwd = "frontend"

[scripts.build]
command = "echo '✅ All services built'"
description = "Build everything"
depends = ["build:go", "build:python", "build:frontend"]

[scripts.docker]
command = "docker compose build"
description = "Build Docker images"
depends = ["build"]
```

### Database Migration Pipeline

```toml
[scripts]
db:backup = "pg_dump $DATABASE_URL > backup.sql"

[scripts.db:migrate]
command = "prisma migrate deploy"
description = "Run database migrations"
depends = ["db:backup"]

[scripts.db:seed]
command = "python manage.py seed"
depends = ["db:migrate"]

[scripts.db:reset]
command = "prisma migrate reset --force"
description = "Reset and reseed database"
depends = ["db:backup"]
```

## Advanced Argument Passing

### The `{{args}}` Placeholder

Pass complex arguments directly to scripts without conflicts:

```bash
# Cargo testing with package selection
vx run test-pkgs -p vx-runtime --lib

# ESLint with multiple options
vx run lint --fix --ext .js,.ts src/

# Docker build with platform selection
vx run docker-build --platform linux/amd64 -t myapp .
```

### Script Definition

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

### Script-Specific Help

Get detailed help for individual scripts:

```bash
# Show help for a specific script
vx run test-pkgs -H
vx run deploy --script-help

# List all available scripts
vx run --list
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

### From Shell Script Chains

**Before (Makefile / shell script):**
```bash
# You had to manually chain commands and track dependencies
lint:
	eslint .
typecheck:
	tsc --noEmit
test: lint typecheck
	vitest run
deploy: test
	npm run build && kubectl apply -f k8s/
```

**After (vx.toml with DAG):**
```toml
[scripts]
lint = "eslint ."
typecheck = "tsc --noEmit"

[scripts.test]
command = "vitest run"
depends = ["lint", "typecheck"]

[scripts.deploy]
command = "npm run build && kubectl apply -f k8s/"
depends = ["test"]
```

**Benefits:**
- Circular dependency detection
- Each dependency runs exactly once
- Built-in script help and listing
- Cross-platform (no Makefile/bash dependency)

## Best Practices

### 1. Use `{{args}}` for Tool Integration

For tools with many command-line options:

```toml
[scripts]
# ✅ Flexible - supports any cargo test arguments
test = "cargo test {{args}}"

# ✅ Flexible - supports any eslint arguments
lint = "eslint {{args}}"

# ❌ Rigid - only works for specific use cases
test-lib = "cargo test --lib"
```

### 2. Use Dependencies for Multi-Step Tasks

Instead of chaining commands with `&&`, use `depends`:

```toml
# ❌ Fragile - no deduplication, no cycle detection
ci = "eslint . && tsc --noEmit && vitest run && npm run build"

# ✅ Robust - DAG-based execution with all benefits
[scripts]
lint = "eslint ."
typecheck = "tsc --noEmit"
test = "vitest run"
build = "npm run build"

[scripts.ci]
command = "echo 'All checks passed!'"
depends = ["lint", "typecheck", "test", "build"]
```

### 3. Add Descriptions for Complex Scripts

```toml
[scripts.deploy]
command = "kubectl apply -f k8s/"
description = "Deploy to production Kubernetes cluster"
depends = ["build", "test"]
env = { KUBECONFIG = "~/.kube/production" }
```

### 4. Combine with Environment Variables

```toml
[env]
RUST_LOG = "debug"
CARGO_TERM_COLOR = "always"

[scripts]
test = "cargo test {{args}}"
test-quiet = "RUST_LOG=error cargo test {{args}}"
```

### 5. Use cwd for Monorepo Projects

```toml
[scripts.build:api]
command = "cargo build --release"
cwd = "services/api"

[scripts.build:web]
command = "npm run build"
cwd = "apps/web"

[scripts.build]
command = "echo 'All services built'"
depends = ["build:api", "build:web"]
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

### Integration with Task Runners

vx scripts work seamlessly with external task runners like Dagu, Just, and Make via subprocess PATH inheritance:

```toml
[scripts]
# Use vx-managed tools inside DAG workflows
workflow = "dagu start pipeline.yaml"

# justfile recipes can access vx tools without prefix
just-ci = "just ci"
```

## Troubleshooting

### Circular Dependency Error

**Problem**: `Circular dependency detected: A -> B -> A`

**Solution**: Review your `depends` lists and break the cycle:

```toml
# ❌ Circular
[scripts.a]
command = "echo a"
depends = ["b"]

[scripts.b]
command = "echo b"
depends = ["a"]

# ✅ Fixed - extract shared dependency
[scripts]
shared = "echo shared"

[scripts.a]
command = "echo a"
depends = ["shared"]

[scripts.b]
command = "echo b"
depends = ["shared"]
```

### Dependency Script Not Found

**Problem**: `Dependency script 'build' not found in vx.toml`

**Solution**: Ensure all scripts referenced in `depends` are defined:

```toml
[scripts]
build = "cargo build"   # Must exist!

[scripts.deploy]
command = "kubectl apply -f k8s/"
depends = ["build"]     # References "build" above
```

### Arguments Not Working

**Problem**: Arguments aren't passed to the script.

**Solution**: Ensure your script uses `{{args}}`:

```toml
# ❌ Won't receive arguments
test = "cargo test"

# ✅ Will receive all arguments
test = "cargo test {{args}}"
```

### Script Help Not Showing

**Problem**: `vx run script --help` shows global help instead of script help.

**Solution**: Use `-H` instead:

```bash
# ✅ Shows script-specific help (including dependencies)
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
fmt = "cargo fmt"

[scripts.check]
command = "echo '✅ All checks passed'"
description = "Run all quality checks"
depends = ["fmt", "clippy", "test-all"]
```

Usage:
```bash
vx run test -p my-crate --lib
vx run clippy -- -D warnings
vx run doc --open --no-deps
vx run check   # Runs fmt → clippy → test-all → check
```

### JavaScript/TypeScript Development

```toml
[scripts]
lint = "eslint {{args}}"
format = "prettier {{args}}"
typecheck = "tsc --noEmit"
test = "vitest run {{args}}"
build = "vite build"

[scripts.ci]
command = "echo '✅ CI passed'"
depends = ["lint", "typecheck", "test", "build"]
```

Usage:
```bash
vx run lint --fix --ext .js,.ts src/
vx run test --watch --coverage
vx run ci   # Full pipeline
```

### Python Development

```toml
[scripts]
lint = "uvx ruff check . {{args}}"
format = "uvx ruff format . {{args}}"
typecheck = "uvx mypy src/"
test = "uv run pytest {{args}}"

[scripts.ci]
command = "echo '✅ All checks passed'"
depends = ["lint", "typecheck", "test"]

[scripts.publish]
command = "uv build && uvx twine upload dist/*"
description = "Build and publish to PyPI"
depends = ["ci"]
```

Usage:
```bash
vx run lint --fix
vx run test -x --tb=short
vx run publish   # Runs: lint → typecheck → test → ci → publish
```

### Docker Development

```toml
[scripts]
build = "docker build {{args}}"
run = "docker run {{args}}"
compose = "docker-compose {{args}}"

[scripts.up]
command = "docker compose up -d"
description = "Start all services"

[scripts.down]
command = "docker compose down"
description = "Stop all services"
```

Usage:
```bash
vx run build -t myapp:latest --platform linux/amd64 .
vx run compose up -d --scale web=3
```

## Script Configuration Reference

### Simple Script

```toml
[scripts]
dev = "npm run dev"
```

### Detailed Script

```toml
[scripts.deploy]
command = "kubectl apply -f k8s/"      # Required: command to execute
description = "Deploy to production"    # Optional: shown in --list and -H
args = ["--prune"]                     # Optional: default arguments
cwd = "infrastructure"                 # Optional: working directory
env = { KUBECONFIG = "~/.kube/prod" }  # Optional: environment variables
depends = ["build", "test"]            # Optional: dependency scripts (DAG)
```

| Field | Type | Description |
|-------|------|-------------|
| `command` | string | Command to execute |
| `description` | string | Human-readable description |
| `args` | string[] | Default arguments |
| `cwd` | string | Working directory (relative to project root) |
| `env` | table | Script-specific environment variables |
| `depends` | string[] | Scripts to run first (DAG dependencies) |

## See Also

- [run command reference](../cli/run.md) - Complete command documentation
- [vx.toml configuration](../config/vx-toml.md) - Configuration file reference
- [Variable interpolation](../config/vx-toml.md#variable-interpolation) - Advanced variable usage
- [Best Practices](./best-practices.md) - More workflow patterns
