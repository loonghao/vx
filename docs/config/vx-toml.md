# vx.toml Reference

Complete reference for the `vx.toml` project configuration file.

## Overview

`vx.toml` is the project-level configuration file for vx. It declares which runtimes your project needs, defines scripts, manages environment variables, and configures development services — all in a single TOML file.

## File Location

Place `vx.toml` in your project root. vx searches for it in the current directory and parent directories.

## Minimum Version Requirement

Use `min_version` to specify the minimum vx version required to parse this configuration:

```toml
min_version = "0.6.0"
```

If the installed vx version is older than `min_version`, vx will display an error and suggest upgrading.

## Complete Example

```toml
min_version = "0.6.0"

[project]
name = "my-fullstack-app"
description = "AI-powered fullstack application"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"

[tools]
node = "20"
uv = "latest"
go = "1.22"
rustup = "latest"
just = "latest"

[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]

[tools.pwsh]
version = "7.4.13"
os = ["windows"]

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]

[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "Your API key"
DATABASE_URL = "Database connection string"

[env.optional]
CACHE_DIR = "Optional cache directory"

[env.secrets]
provider = "auto"
items = ["DATABASE_URL", "API_KEY"]

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0", "--port", "8080"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"

[hooks]
pre_setup = "echo 'Starting setup...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.app]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]
env_file = ".env.local"

[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
```

---

## Sections

### Top-level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `min_version` | string | No | Minimum vx version required (e.g., `"0.6.0"`) |

---

### `[project]`

Project metadata. All fields are optional.

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `description` | string | Project description |
| `version` | string | Project version |
| `license` | string | License identifier (e.g., `MIT`, `Apache-2.0`) |
| `repository` | string | Repository URL |

```toml
[project]
name = "my-project"
description = "A sample project"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"
```

---

### `[tools]`

Runtime versions to use. This is the primary section for declaring which runtimes your project requires.

> **Terminology**: vx uses the term "runtime" for managed executables (node, go, uv, etc.). The section is named `[tools]` for user-friendliness but manages runtimes internally.

> **Backward compatibility**: `[runtimes]` is accepted as an alias for `[tools]`. If both exist, `[tools]` takes priority.

#### Simple Version

```toml
[tools]
node = "20"          # Major version — resolves to latest 20.x.x
uv = "latest"        # Latest stable release
go = "1.21.5"        # Exact version
rustup = "latest"    # Rust toolchain manager
just = "latest"      # Just command runner
```

#### Detailed Configuration

Use table syntax for advanced per-runtime settings:

```toml
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]
install_env = { NODE_OPTIONS = "--max-old-space-size=4096" }

[tools.pwsh]
version = "7.4.13"
os = ["windows"]     # Only install on Windows

[tools.msvc]
version = "14.42"
os = ["windows"]
components = ["spectre", "mfc", "atl"]
exclude_patterns = ["Microsoft.VisualStudio.Component.VC.Llvm*"]
```

| Field | Type | Description |
|-------|------|-------------|
| `version` | string | Version specifier (see table below) |
| `postinstall` | string | Command to run after installation |
| `os` | string[] | Limit to specific operating systems (`"windows"`, `"darwin"`, `"linux"`) |
| `install_env` | table | Environment variables set during installation |
| `components` | string[] | Optional components to install (e.g., MSVC: `spectre`, `mfc`, `atl`, `asan`, `cli`) |
| `exclude_patterns` | string[] | Package ID patterns to exclude during installation |

If `os` is not specified, the runtime is installed on all platforms. When specified, vx only installs it on the listed operating systems.

#### Version Specifiers

| Format | Example | Description |
|--------|---------|-------------|
| Major | `"20"` | Latest 20.x.x |
| Minor | `"20.10"` | Latest 20.10.x |
| Exact | `"20.10.0"` | Exact version |
| Latest | `"latest"` | Latest stable release |
| LTS | `"lts"` | Latest LTS version (runtime-specific, e.g., Node.js) |
| Channel | `"stable"` | Release channel (e.g., Rust: `stable`, `nightly`, `beta`) |

> **Rust note**: Configure `rustup` in `[tools]`, not `rust`. The `rustup` version is the version of the toolchain manager itself, not the Rust compiler version. Use `vx cargo` / `vx rustc` in your scripts.

---

### `[python]`

Python-specific environment configuration. This section provides deeper integration for Python projects beyond the basic `[tools].python` version pin.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `version` | string | — | Python version |
| `venv` | string | `".venv"` | Virtual environment directory path |
| `package_manager` | string | `"uv"` | Package manager (`uv`, `pip`, `poetry`) |

```toml
[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"
```

#### `[python.dependencies]`

Python project dependencies.

| Field | Type | Description |
|-------|------|-------------|
| `requirements` | string[] | Requirements files to install from |
| `packages` | string[] | Direct package names to install |
| `git` | string[] | Git repository URLs to install from |
| `dev` | string[] | Development-only dependencies |

```toml
[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["requests", "pandas"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "black", "mypy"]
```

---

### `[env]`

Environment variables with support for required/optional declarations and secret management.

#### Static Variables

Set environment variables directly:

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"
PORT = "3000"
```

#### Required Variables

Variables that **must** be set. vx will warn if they are missing. The value is a human-readable description.

```toml
[env.required]
API_KEY = "Your API key for the service"
DATABASE_URL = "PostgreSQL connection string"
```

#### Optional Variables

Optional variables with descriptions:

```toml
[env.optional]
CACHE_DIR = "Optional cache directory"
LOG_LEVEL = "Logging level (default: info)"
```

#### Secrets

Load secrets from secure storage providers:

```toml
[env.secrets]
provider = "auto"  # auto | 1password | vault | aws-secrets
items = ["DATABASE_URL", "API_KEY"]
```

| Field | Type | Description |
|-------|------|-------------|
| `provider` | string | Secret provider (`auto`, `1password`, `vault`, `aws-secrets`) |
| `items` | string[] | Secret names to load |

---

### `[scripts]`

Runnable scripts invoked via `vx run <script_name>`. Supports both simple commands and detailed configuration with DAG-based dependencies.

#### Simple Scripts

```toml
[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
lint = "cargo clippy --workspace"
```

#### Parameterized Scripts

Use `{{args}}` to forward additional arguments:

```toml
[scripts]
test-pkgs = "cargo test {{args}}"      # vx run test-pkgs -- -p vx-cli
just = "just {{args}}"                  # vx run just -- build
```

#### Package Execution in Scripts

Use ecosystem package execution syntax directly:

```toml
[scripts]
tox = "uvx:tox {{args}}"               # Runs tox via uvx (Python)
```

#### Detailed Scripts

```toml
[scripts.start]
command = "python main.py"
description = "Start the server"
args = ["--host", "0.0.0.0", "--port", "8080"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]

[scripts.ci]
command = "echo 'All checks passed'"
description = "Run CI pipeline"
depends = ["lint", "test", "build"]     # DAG: executed in topological order
```

| Field | Type | Description |
|-------|------|-------------|
| `command` | string | Command to execute |
| `description` | string | Human-readable description (shown by `vx run --list`) |
| `args` | string[] | Default arguments appended to the command |
| `cwd` | string | Working directory (relative to project root) |
| `env` | table | Script-specific environment variables |
| `depends` | string[] | Scripts that must run first (DAG topological ordering) |

---

### `[settings]`

Behavior settings for vx within this project.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_install` | bool | `true` | Automatically install missing runtimes on first use |
| `parallel_install` | bool | `true` | Install multiple runtimes in parallel |
| `cache_duration` | string | `"7d"` | Version list cache duration (e.g., `"1h"`, `"7d"`, `"30d"`) |
| `shell` | string | `"auto"` | Default shell (`auto`, `bash`, `zsh`, `fish`, `pwsh`, `cmd`) |
| `log_level` | string | `"info"` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `isolation` | bool | `true` | Enable environment isolation in `vx dev` |
| `passenv` | string[] | — | Environment variables to pass through in isolated mode (glob patterns, e.g., `"SSH_*"`) |
| `setenv` | table | — | Explicit environment variables to set (overrides passenv) |

```toml
[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"
isolation = true
passenv = ["SSH_*", "GPG_*", "EDITOR"]
setenv = { TERM = "xterm-256color" }
```

#### Isolation Mode

When `isolation = true` (default), `vx dev` creates an isolated environment where only vx-managed runtimes are in `PATH`. System variables are filtered, with only essential ones passed through:

- **Windows**: `SYSTEMROOT`, `TEMP`, `TMP`, `PATHEXT`, `COMSPEC`, `WINDIR`
- **Unix**: `HOME`, `USER`, `SHELL`, `LANG`, `LC_*`, `TERM`

Use `passenv` to explicitly allow additional variables (supports glob patterns).

#### Experimental Features

```toml
[settings.experimental]
monorepo = false       # Monorepo workspace support
workspaces = false     # Multi-workspace support
```

---

### `[hooks]` <Badge type="tip" text="v0.6.0+" />

Lifecycle hooks for automating tasks at specific points.

| Hook | When It Runs |
|------|-------------|
| `pre_setup` | Before `vx setup` |
| `post_setup` | After `vx setup` completes |
| `pre_commit` | Before git commit (requires git hooks setup) |
| `enter` | When entering the project directory |

Hooks can be a single command string or an array of commands:

```toml
[hooks]
pre_setup = "echo 'Starting setup...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"
```

#### Custom Hooks

Define your own hooks triggered via `vx hook <name>`:

```toml
[hooks.custom]
deploy = "vx run build && vx run deploy"
release = "vx run test && vx run build && gh release create"
```

---

### `[setup]` <Badge type="tip" text="v0.6.0+" />

Configure the `vx setup` pipeline for reproducible environment bootstrapping, including CI integration.

```toml
[setup]
pipeline = ["pre_setup", "install_tools", "export_paths", "post_setup"]

[setup.hooks.install_tools]
enabled = true
parallel = true
force = false

[setup.hooks.export_paths]
enabled = true
ci_only = true
extra_paths = ["/usr/local/bin"]

[setup.ci]
enabled = true              # Auto-detect CI environment
provider = "github"         # github | gitlab | azure | circleci | jenkins | generic
path_env_file = ""          # Custom PATH export file (auto-detected for CI provider)
env_file = ""               # Custom env export file
```

#### CI Auto-detection

vx automatically detects CI environments via environment variables:

| Provider | Detection Variable |
|----------|-------------------|
| GitHub Actions | `GITHUB_ACTIONS` |
| GitLab CI | `GITLAB_CI` |
| Azure Pipelines | `TF_BUILD` |
| CircleCI | `CIRCLECI` |
| Jenkins | `JENKINS_URL` |
| Generic | `CI=true` |

#### Custom Setup Hooks

```toml
[setup.hooks.custom.setup_database]
command = "podman compose up -d postgres"
enabled = true
ci_only = false
continue_on_failure = false
working_dir = "infra"
env = { PGPASSWORD = "dev" }
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `command` | string/string[] | — | Command(s) to execute |
| `enabled` | bool | `true` | Whether this hook is active |
| `ci_only` | bool | `false` | Only run in CI environments |
| `local_only` | bool | `false` | Only run locally (not in CI) |
| `continue_on_failure` | bool | `false` | Continue pipeline if this hook fails |
| `working_dir` | string | — | Working directory for this hook |
| `env` | table | — | Environment variables for this hook |

---

### `[services]` <Badge type="tip" text="v0.6.0+" />

Service definitions for local development. Container services run with the configured runtime, which defaults to Podman, and are managed via `vx services` commands.

```toml
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
volumes = ["./data:/var/lib/postgresql/data"]
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.app]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]
env_file = ".env.local"
working_dir = "./frontend"
```

| Field | Type | Description |
|-------|------|-------------|
| `image` | string | Container image (for container services) |
| `command` | string | Command to run (for non-container services) |
| `ports` | string[] | Port mappings (`"host:container"`) |
| `env` | table | Environment variables |
| `env_file` | string | Path to `.env` file |
| `volumes` | string[] | Volume mounts (`"host:container"`) |
| `depends_on` | string[] | Services that must start first |
| `healthcheck` | string | Health check command |
| `working_dir` | string | Working directory |

> Each service must have either `image` (container) or `command` (process), but not both.

---

### `[dependencies]` <Badge type="tip" text="v0.6.0+" />

Smart dependency management configuration per ecosystem.

```toml
[dependencies]
lockfile = true
audit = true
auto_update = "minor"    # none | patch | minor | major
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `lockfile` | bool | — | Generate/use lockfile for reproducibility |
| `audit` | bool | — | Run security audit on dependencies |
| `auto_update` | string | — | Auto-update strategy |

#### Node.js Dependencies

```toml
[dependencies.node]
package_manager = "pnpm"                         # npm | yarn | pnpm | bun
registry = "https://registry.npmmirror.com"      # Custom registry
```

#### Python Dependencies

```toml
[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
extra_index_urls = ["https://download.pytorch.org/whl/cu121"]
```

#### Go Dependencies

```toml
[dependencies.go]
proxy = "https://goproxy.cn,direct"
private = "github.com/myorg/*"
vendor = false
mod_mode = "readonly"     # readonly | vendor | mod
```

#### C++ Dependencies

```toml
[dependencies.cpp]
package_manager = "vcpkg"          # conan | vcpkg | cmake
vcpkg_triplet = "x64-windows"     # x64-windows | x64-linux | x64-osx
cmake_generator = "Ninja"
cmake_build_type = "Release"       # Debug | Release | RelWithDebInfo | MinSizeRel
std = "20"                          # C++ standard: 11, 14, 17, 20, 23
compiler = "msvc"                   # gcc | clang | msvc
```

#### Dependency Constraints

```toml
[dependencies.constraints]
"lodash" = ">=4.17.21"                                   # Version constraint
"*" = { licenses = ["MIT", "Apache-2.0", "BSD-3-Clause"] } # License policy
```

---

## Planned Sections

The following sections are designed and have Rust struct definitions, but are in development:

| Section | Phase | Description |
|---------|-------|-------------|
| `[ai]` | Phase 2 | AI code generation integration |
| `[docs]` | Phase 2 | Documentation auto-generation |
| `[team]` | Phase 3 | Team collaboration rules (code owners, review, conventions) |
| `[remote]` | Phase 3 | Remote development environments (Codespaces, Gitpod, DevContainer) |
| `[security]` | Phase 4 | Security scanning (audit, secret detection, SAST) |
| `[test]` | Phase 4 | Test pipeline configuration (coverage, environments) |
| `[telemetry]` | Phase 4 | Performance monitoring and tracing (OTLP) |
| `[container]` | Phase 5 | Container deployment (Dockerfile generation, registry, multi-stage) |
| `[versioning]` | Phase 5 | Version control strategy (semver, calver) |

See [RFC 0001: vx.toml v2 Enhancement](../rfcs/0001-vx-toml-v2-enhancement.md) for the full roadmap.

---

## Validation

vx validates `vx.toml` on load. Validation checks include:

- TOML syntax correctness
- `min_version` format
- Runtime name validity (alphanumeric + hyphens)
- Script name validity
- Version specifier format
- Service definitions (must have `image` or `command`)
- Port mapping format (`"host:container"`)
- Circular script dependency detection

Run validation manually:

```bash
vx config validate
```

---

## Tips

1. **Commit to git** — Share `vx.toml` with your team for consistent environments
2. **Use specific versions** — Pin exact versions for production reproducibility
3. **Use `os` filtering** — Limit platform-specific runtimes (e.g., `pwsh` on Windows only)
4. **Define scripts** — Make common tasks discoverable via `vx run --list`
5. **Use `depends`** — Script dependencies ensure correct execution order
6. **Use `{{args}}`** — Forward CLI arguments to scripts for flexibility
7. **Document env vars** — Use `[env.required]` descriptions to help new contributors
8. **Use hooks** — Automate `vx setup` and git workflows

---

## See Also

- [Configuration Guide](../guide/configuration) — Getting started with configuration
- [vx.toml Syntax Guide](../guide/vx-toml-syntax) — Syntax patterns and best practices
- [Global Configuration](./global) — User-wide default settings
- [Command Syntax Rules](../guide/command-syntax-rules) — Canonical command forms
