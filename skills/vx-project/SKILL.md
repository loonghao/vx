---
name: vx-project
description: "Project management guide for vx. Use when setting up a new project, configuring vx.toml, or managing project-level tool versions and scripts."
---

# VX Project Management Guide

> **Quick start**: Run `vx init` to create `vx.toml`, `vx setup` to install all tools, `vx dev` to enter the dev environment. For existing projects, just run `vx setup` after cloning.

## Project Setup

### Initialize a Project

```bash
vx init                     # Create vx.toml interactively
vx init --template node     # Use a template
vx init --minimal           # Create minimal vx.toml
```

### Project Detection

vx automatically detects project types and suggests tools:

```bash
vx analyze                  # Analyze project (detects languages, dependencies)
vx analyze --json           # JSON output for AI parsing
```

**Detected ecosystems**: Node.js, Python, Rust, Go, Java, .NET, C/C++, Zig
**Detected frameworks**: React, Vue, Angular, Next.js, Nuxt, Svelte, Django, Flask, FastAPI, Tauri, Electron, React Native, NW.js, and more
**Detected package managers**: npm, yarn, pnpm, bun, pip, uv, cargo, go modules

The project analyzer reads indicator files like `package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, etc. to suggest the right tools.

## vx.toml Configuration

### Basic Structure

```toml
# vx.toml - Project tool configuration

[tools]
# Version constraints
node = "22"                 # Major version (any 22.x.x)
go = "1.22"                 # Minor version (any 1.22.x)
uv = "latest"               # Always use latest
just = "*"                  # Any version
# NOTE: no `rust` entry — Rust is pinned by rust-toolchain.toml, see below

# Platform-specific tools
[tools.msvc]
version = "14.42"
os = ["windows"]            # Only install on Windows

[tools.brew]
version = "latest"
os = ["macos", "linux"]

[hooks]
# Lifecycle hooks
pre_commit = ["vx run lint"]
post_setup = ["npm install", "cargo fetch"]
```

> **No `[scripts]` above on purpose.** When the repository has a `justfile`,
> tasks belong there, not in `vx.toml`. See
> [Scripts vs justfile](#scripts-vs-justfile-pick-one).

### Rust: pin with `rust-toolchain.toml`, not `[tools] rust`

`vx` resolves the `rust` provider through **rustup** — the installed runtime is
`rustup`, and `vx install rust@<version>` selects a *rustup release*, not a
`rustc` toolchain. `rustup` itself reads `rust-toolchain.toml`, which wins for
everything that runs through `cargo`/`rustc`.

Practical consequences:

| Intent | Do this | Not this |
|--------|---------|----------|
| Pin the Rust toolchain | `rust-toolchain.toml` with `channel = "1.98.1"` | `[tools] rust = "1.98.1"` |
| Install Rust at all | `vx rustup --version` (auto-installs) | `[tools] rust = "stable"` |
| Per-project override | `rust-toolchain.toml` in the repo root | editing global config |

```toml
# rust-toolchain.toml — authoritative for cargo/rustc/rustfmt/clippy
[toolchain]
channel = "1.98.1"
```

If you must list `rust` under `[tools]`, treat the pin as **advisory only** and
say so in a comment — the two version namespaces are not interchangeable.

**Mechanical check** — fail when a bare `rust` pin sits next to a toolchain file:

```bash
test -f rust-toolchain.toml && vx rg -q '^rust\s*=' vx.toml && echo "FAIL: [tools] rust conflicts with rust-toolchain.toml"
```

### Scripts vs justfile: pick one

`vx.toml` answers **which version of a tool**. A task runner answers **how work
gets run**. A `[scripts]` entry that forwards to `just <recipe>` is a second
source of truth for the same task: change the recipe and the script silently
keeps the old behaviour.

| Situation | Use |
|-----------|-----|
| `justfile` present | Add recipes there. No `[scripts]` in `vx.toml`. |
| No `justfile`, a handful of tasks | `[scripts]` in `vx.toml` is fine. |
| No `justfile`, many tasks | Create a `justfile`, keep `vx.toml` version-only. |

**Mechanical check** — fail when both exist:

```bash
test -f justfile && vx rg -q '^\[scripts\]' vx.toml && echo "FAIL: [scripts] duplicates justfile"
```

### Multi-Python Legacy Projects

For projects that need modern Python plus legacy Python 3.7 or 2.7, keep the
runtime requirements in `vx.toml` and the environment/test matrix in `justfile`:

```toml
[tools]
uv = "latest"
python = "3.12"
just = "latest"

[scripts]
test = "vx just test"
test-legacy = "vx just test-legacy"
```

```makefile
venv312:
    vx uv venv .venv312 --python 3.12
    vx uv pip install --python .venv312 -r requirements.txt

venv37:
    vx uv venv .venv37 --python 3.7
    vx uv pip install --python .venv37 -r requirements-py37.txt

venv27:
    vx uv venv .venv27 --python 2.7
    .venv27/bin/python -m pip install -r requirements-py27.txt

test312: venv312
    .venv312/bin/python -m pytest

test37: venv37
    .venv37/bin/python -m pytest

test27: venv27
    .venv27/bin/python -m pytest

test-legacy: test37 test27
test: test312 test-legacy
```

Agents should use `vx uv venv ... --python 3.7` and
`vx uv venv ... --python 2.7`; vx resolves those versions to managed
interpreters. Python 2.7 is a legacy compatibility path using PyPy2.7 and
PyPA's Python 2.7 `virtualenv.pyz`, so flag CPython-only extension risks early.

### Multi-Version Runtime Test Matrices

When a project promises compatibility across several runtime lines, record the
baseline runtime in `vx.toml` and put every compatibility lane in `justfile`.
This keeps local developer testing, CI, and AI-agent verification aligned.

```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"
just = "latest"

[scripts]
test = "vx just test"
test-matrix = "vx just test-matrix"

[ai]
skills_hash = "<recorded by vx ai setup --project>"
```

```makefile
test-node18:
    vx node@18 npm test

test-node20:
    vx node@20 npm test

test-node22:
    vx node@22 npm test

venv37:
    vx uv venv .venv37 --python 3.7
    vx uv pip install --python .venv37 -r requirements-py37.txt

venv312:
    vx uv venv .venv312 --python 3.12
    vx uv pip install --python .venv312 -r requirements.txt

test-py37: venv37
    .venv37/bin/python -m pytest tests/py37

test-py312: venv312
    .venv312/bin/python -m pytest tests/py312

test-matrix: test-node18 test-node20 test-node22 test-py37 test-py312
test: test-matrix
```

Agents should use the matrix recipe (`vx just test-matrix` or
`vx run test-matrix`) when changing shared code. Add the smallest missing lane
when a bug report mentions an unsupported Python, Node.js, npm, pnpm, or yarn
version.

### Project AI Skills Hash

`vx ai setup` installs built-in vx skills **globally** (one copy in the user's
agent directories), and records `[ai].skills_hash` in the project's `vx.toml`
whenever one exists — in global mode too, so drift is detectable without a
project-local install. Repositories should not carry their own copy of a
built-in vx skill.

```bash
vx ai setup            # install globally (default) + record the hash in vx.toml
vx ai check            # report drift: stale global install, stale hash, local copies
vx ai check --fix      # refresh global skills, drop local copies, re-record the hash
```

A repository that owns project-specific skills keeps them under `skills/` with
its own namespace (`acme-domain-skill`, `dcc-mcp-maya-setup`, …) and never
reuses a built-in name (`vx-usage`, `vx-project`, …). `vx ai check --fix` removes
copies of built-in skills and leaves everything else alone.

The vx repository itself sets `skills_source = true` so its own `skills/`
directory is treated as the upstream source instead of drift:

```toml
[ai]
skills_source = true
```

### Version Constraints

| Constraint | Example | Meaning |
|------------|---------|---------|
| Exact | `"1.2.3"` | Only version 1.2.3 |
| Major | `"1"` | Any 1.x.x |
| Minor | `"1.2"` | Any 1.2.x |
| Latest | `"latest"` | Always latest |
| Any | `"*"` | Any available version |
| Range | `">=1.0.0 <2.0.0"` | Range constraint |

### Platform-Specific Tools

```toml
[tools]
# Cross-platform tools
node = "22"
uv = "latest"

# Windows-only
[tools.msvc]
version = "14.42"
os = ["windows"]

# macOS/Linux only
[tools.brew]
version = "latest"
os = ["macos", "linux"]
```

## Project Commands

### Setup & Sync

```bash
vx setup                    # Full project setup (sync + hooks)
vx sync                     # Install all tools from vx.toml
vx sync --clean             # Remove unlisted tools
vx sync --check             # Check without installing
```

### Running Scripts

```bash
vx run dev                  # Run development server
vx run test                 # Run tests
vx run build                # Build project
vx run --list               # List available scripts
```

### Lock File

```bash
vx lock                     # Generate vx.lock
vx lock --update            # Update locked versions
vx lock --check             # Verify lock file
```

The `vx.lock` file ensures reproducible builds:

```toml
# vx.lock - Auto-generated, do not edit
[tools]
node = { version = "22.0.0", checksum = "sha256:..." }
go = { version = "1.22.0", checksum = "sha256:..." }
```

## Environment Management

### Project Environment

```bash
vx dev                      # Enter project environment
vx env list                 # List environments
vx env activate             # Print activation commands
eval $(vx env activate)     # Activate in shell
```

### Environment Variables

Define in `vx.toml`:

```toml
[env]
NODE_ENV = "development"
DATABASE_URL = "postgresql://localhost:5432/dev"
API_KEY = { env = "API_KEY", required = true }
```

## Dependency Management

### Add/Remove Tools

```bash
vx add node@22              # Add tool to vx.toml
vx add go rust uv           # Add multiple tools
vx remove node              # Remove tool from vx.toml
```

### Check Constraints

```bash
vx check                    # Verify tool constraints
vx check --json             # JSON output
vx check --fix              # Auto-fix issues
```

## Multi-Package Projects

### Monorepo Support

For monorepos, create `vx.toml` in root:

```toml
# Root vx.toml
[tools]
node = "22"
pnpm = "latest"

[scripts]
install = "pnpm install"
build = "pnpm -r build"
test = "pnpm -r test"
```

### Workspace Packages

Individual packages can have their own `vx.toml`:

```toml
# packages/backend/vx.toml
[tools]
go = "1.22"

[scripts]
dev = "go run ./cmd/server"
```

## Best Practices

### 1. Version Pinning

Pin versions for CI/CD:

```toml
[tools]
node = "22.0.0"             # Exact version for CI
```

### 2. Lock Files

Always commit `vx.lock`:

```bash
git add vx.lock
```

### 3. Scripts Organization

Only when there is no `justfile`. Group related scripts and keep them
self-contained — never mirror a `just` recipe:

```toml
[scripts]
# Development
dev = "..."
watch = "..."

# Testing
test = "..."
test:watch = "..."

# Build
build = "..."
build:prod = "..."
```

With a `justfile`, the same grouping lives in recipes and `vx.toml` stays
version-only:

```bash
vx just --list     # discover tasks
vx just test       # run the task
```

### 4. Hooks for Quality

Use hooks for automated checks:

```toml
[hooks]
pre_commit = ["vx run lint", "vx run test"]
post_checkout = ["vx sync"]
```

## Project Templates

Create reusable templates:

```bash
# Create template from current project
vx template create my-template

# Use template
vx init --template my-template
```

### Template Structure

```
~/.vx/templates/my-template/
├── vx.toml
├── .gitignore
├── README.md
└── hooks/
    └── post_setup.sh
```


## Delivery surface — external systems are evidence, not the answer

Pull requests, CI runs, and dashboards are **supporting evidence**. The
conclusion has to land where the work is tracked.

- Record status, branch/commit/PR, validation run, blocker, and next owner in one
  issue or task comment before ending a turn.
- Never treat a green CI run as proof the work shipped — verify the terminal
  state (merged / released / deployed) and record *that*.
- Keep internal routing, issue IDs, and local absolute paths off public GitHub
  surfaces; PR text stays technical and public-safe.

**Mechanical check** — a turn that touched code ends with exactly one task-system
comment carrying status + commit/PR + validation + next owner. A PR comment alone
fails the check.
