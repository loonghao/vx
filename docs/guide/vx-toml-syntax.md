# vx.toml Syntax Guide

This guide covers syntax patterns, common recipes, and best practices for writing `vx.toml` configuration files. For a complete field reference, see [vx.toml Reference](/config/vx-toml).

## Quick Start

The minimal `vx.toml` only needs a `[tools]` section:

```toml
[tools]
node = "20"
```

This tells vx: "this project needs Node.js 20.x". Running `vx node --version` will auto-install Node.js 20 if needed.

## Syntax Fundamentals

### TOML Basics

`vx.toml` uses [TOML](https://toml.io/) format. Key concepts:

```toml
# Comments start with #
key = "value"                        # String
enabled = true                       # Boolean
count = 42                           # Integer

[section]                            # Table (section)
field = "value"

[section.nested]                     # Nested table
field = "value"

inline = { key1 = "a", key2 = "b" } # Inline table

list = ["item1", "item2"]           # Array
```

### Version Specifiers

vx supports several version specifier formats:

```toml
[tools]
# Partial versions — resolve to latest matching
node = "20"              # Latest 20.x.x (e.g., 20.18.1)
go = "1.22"              # Latest 1.22.x (e.g., 1.22.7)

# Exact versions — pinned precisely
python = "3.12.1"        # Exactly 3.12.1

# Special keywords
uv = "latest"            # Latest stable release
node = "lts"             # Latest LTS release (runtime-specific)

# Channel names (Rust-specific)
rustup = "stable"        # Stable channel
rustup = "nightly"       # Nightly channel
rustup = "beta"          # Beta channel
```

**Resolution order**: When `vx <runtime>` is invoked:
1. CLI explicit version (`vx node@22`)
2. `vx.lock` (if present)
3. `vx.toml` `[tools]` section
4. Latest compatible version

### Simple vs. Detailed Configuration

Every `[tools]` entry supports two forms:

```toml
# Simple: just a version string
node = "20"

# Detailed: table with version + options
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]
install_env = { NODE_OPTIONS = "--max-old-space-size=4096" }
```

Both forms can coexist in the same file for different runtimes.

### Script Syntax

Scripts also support simple and detailed forms:

```toml
[scripts]
# Simple: just a command string
test = "cargo test --workspace"

# Detailed: table with command + options
[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]
```

---

## Recipes

### Node.js Project

```toml
min_version = "0.6.0"

[project]
name = "my-web-app"

[tools]
node = "20"

[env]
NODE_ENV = "development"

[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"
lint = "npm run lint"

[settings]
auto_install = true

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"
```

### Python Project

```toml
[project]
name = "ml-pipeline"

[tools]
uv = "latest"

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt"]
dev = ["pytest", "ruff", "mypy"]

[scripts]
test = "pytest"
lint = "ruff check ."
format = "ruff format ."
typecheck = "mypy src/"

[env]
PYTHONPATH = "src"
```

### Go Project

```toml
[project]
name = "api-server"

[tools]
go = "1.22"

[scripts]
build = "go build -o bin/server ./cmd/server"
test = "go test ./..."
lint = "golangci-lint run"
run = "go run ./cmd/server"

[dependencies.go]
proxy = "https://goproxy.cn,direct"
private = "github.com/myorg/*"
```

### Rust Project

```toml
[project]
name = "my-cli-tool"

[tools]
rustup = "latest"
just = "latest"

[scripts]
build = "cargo build"
build-release = "cargo build --release"
test = "cargo test --workspace"
lint = "cargo clippy --workspace --all-targets -- -D warnings"
fmt = "cargo fmt --all"
fmt-check = "cargo fmt --all -- --check"
doc = "cargo doc --open"
```

### Fullstack Project

```toml
min_version = "0.6.0"

[project]
name = "fullstack-app"
description = "React + Python API"

[tools]
node = "20"
uv = "latest"

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[env]
NODE_ENV = "development"
API_URL = "http://localhost:8000"

[env.required]
DATABASE_URL = "PostgreSQL connection string"

[scripts]
dev = "npm run dev"
api = "uvicorn main:app --reload"
test = "pytest && npm test"
build = "npm run build"

[scripts.start]
command = "npm run start"
description = "Start production server"
depends = ["build"]

[services.postgres]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"

[dependencies.node]
package_manager = "pnpm"

[hooks]
post_setup = ["vx run db:migrate"]
enter = "vx sync --check"
```

### CI/CD Integration

```toml
[tools]
node = "20"
uv = "latest"

[setup]
pipeline = ["install_tools", "export_paths", "post_setup"]

[setup.hooks.install_tools]
parallel = true

[setup.hooks.export_paths]
ci_only = true

[setup.ci]
enabled = true        # Auto-detects GitHub Actions, GitLab CI, etc.

[scripts]
ci-test = "npm test && pytest"
ci-build = "npm run build"
ci-lint = "npm run lint && ruff check ."
```

Use in GitHub Actions:

```yaml
- name: Setup tools
  run: |
    curl -fsSL https://get.vx.dev | sh
    vx setup
```

### Platform-Specific Runtimes

```toml
[tools]
node = "20"
go = "1.22"

# PowerShell only on Windows
[tools.pwsh]
version = "7.4.13"
os = ["windows"]

# MSVC only on Windows with components
[tools.msvc]
version = "14.42"
os = ["windows"]
components = ["spectre", "mfc", "atl"]
```

### Monorepo with Shared Configuration

```toml
[project]
name = "my-monorepo"

[tools]
node = "20"
uv = "latest"
go = "1.22"
just = "latest"

[scripts]
# Delegate to just for monorepo task orchestration
just = "just {{args}}"
build-all = "just build-all"
test-all = "just test-all"
lint-all = "just lint-all"

[settings]
parallel_install = true

[settings.experimental]
monorepo = true
workspaces = true
```

---

## Script Patterns

### Argument Forwarding

Use `{{args}}` to pass CLI arguments through:

```toml
[scripts]
test = "cargo test {{args}}"
# Usage: vx run test -- --nocapture -p vx-cli
# Expands to: cargo test --nocapture -p vx-cli
```

### Package Execution

Reference ecosystem packages directly in scripts:

```toml
[scripts]
tox = "uvx:tox {{args}}"              # Python: run tox via uvx
create-app = "npm:create-react-app"    # Node.js: run create-react-app via npx
```

### Script Dependencies (DAG)

Scripts can declare dependencies that run first in topological order:

```toml
[scripts]
lint = "cargo clippy"
test = "cargo test"
build = "cargo build --release"

[scripts.ci]
command = "echo 'All checks passed!'"
description = "Run full CI pipeline"
depends = ["lint", "test", "build"]    # Runs lint → test → build → ci
```

### Environment Per Script

Each script can have its own environment variables:

```toml
[scripts.dev]
command = "npm run dev"
env = { NODE_ENV = "development", DEBUG = "*" }

[scripts.prod]
command = "npm run start"
env = { NODE_ENV = "production" }
```

---

## Environment Variable Patterns

### Layered Configuration

```toml
# Static values available to all scripts
[env]
APP_NAME = "my-app"
LOG_FORMAT = "json"

# Required — vx warns if missing
[env.required]
DATABASE_URL = "PostgreSQL connection string"
REDIS_URL = "Redis connection string"

# Optional — documented but not enforced
[env.optional]
SENTRY_DSN = "Sentry error tracking DSN"
CACHE_TTL = "Cache TTL in seconds (default: 3600)"
```

### Secret Management

```toml
[env.secrets]
provider = "1password"
items = ["DATABASE_URL", "API_KEY", "STRIPE_SECRET"]
```

Supported providers:
- `auto` — Auto-detect available provider
- `1password` — 1Password CLI
- `vault` — HashiCorp Vault
- `aws-secrets` — AWS Secrets Manager

---

## Hooks Patterns

### Setup Automation

```toml
[hooks]
pre_setup = "echo 'Checking prerequisites...'"
post_setup = [
    "vx run db:migrate",
    "vx run seed",
    "echo 'Setup complete!'"
]
```

### Git Workflow

```toml
[hooks]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"
```

### Custom Deploy Hook

```toml
[hooks.custom]
deploy-staging = "vx run build && kubectl apply -f k8s/staging/"
deploy-prod = "vx run build && kubectl apply -f k8s/production/"
```

---

## Validation & Troubleshooting

### Validate Configuration

```bash
vx config validate
```

Common validation errors:

| Error | Cause | Fix |
|-------|-------|-----|
| Invalid TOML syntax | Malformed TOML | Check quotes, brackets, commas |
| Unknown runtime | Unrecognized name in `[tools]` | Check `vx list` for available runtimes |
| Invalid version | Malformed version string | Use `"20"`, `"20.10.0"`, `"latest"`, etc. |
| Circular dependency | Script `depends` creates a cycle | Remove the cycle in `depends` |
| Missing image/command | Service has neither | Add `image` or `command` to each service |

### Check Project State

```bash
# Verify runtimes match vx.toml
vx sync --check

# Verify lockfile is up to date
vx lock --check
```

---

## Best Practices

1. **Pin versions in production** — Use exact versions (`"20.18.1"`) for reproducibility
2. **Use partial versions in development** — `"20"` tracks latest minor/patch automatically
3. **Always set `min_version`** — Prevents confusion when team members have different vx versions
4. **Use `[env.required]`** — Documents what secrets/config new team members need
5. **Define scripts** — Makes project tasks discoverable via `vx run --list`
6. **Use `depends` for CI** — DAG-based script dependencies ensure correct ordering
7. **Platform-filter with `os`** — Avoid installing irrelevant runtimes on CI
8. **Use `{{args}}` forwarding** — Keeps scripts flexible without duplicating entries
9. **Commit `vx.toml` to git** — Share environment configuration with the team
10. **Generate `vx.lock`** — Use `vx lock` for fully reproducible builds

---

## See Also

- [vx.toml Reference](/config/vx-toml) — Complete field-by-field reference
- [Configuration Guide](/guide/configuration) — Getting started overview
- [Command Syntax Rules](/guide/command-syntax-rules) — Canonical CLI forms
