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
rust = "1.80"               # Specific version
                            # rust = "rustup-managed" defers to rustup instead
just = "*"                  # Any version

# Platform-specific tools
[tools.msvc]
version = "14.42"
os = ["windows"]            # Only install on Windows

[tools.brew]
version = "latest"
os = ["macos", "linux"]

[scripts]
# Development scripts
dev = "npm run dev"
test = "cargo test"
lint = "npm run lint && cargo clippy"
build = "just build"

# CI/CD scripts
ci = "just ci"
release = "just release"

[hooks]
# Lifecycle hooks
pre_commit = ["vx run lint"]
post_setup = ["npm install", "cargo fetch"]
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

`vx ai setup` installs built-in vx skills globally by default. Use project scope
only when the repository wants local skill copies:

```bash
vx ai setup --project
vx ai check
vx ai setup --project --force
```

Project setup records `[ai].skills_hash` in `vx.toml`. `vx ai check` compares
that hash with the embedded skills hash and reminds developers to refresh stale
project skills.

Global scope is verified too: `vx ai setup` records the hash under
`~/.vx/ai-skills.toml`, and `vx ai check` reports drift for both scopes.

```bash
vx ai check          # report global + project drift
vx ai check --fix    # refresh stale vx-builtin copies, drop project duplicates
```

A repository that ships no skills of its own needs neither scope pinned — prefer
the global install and keep the repo free of skill copies. See the
**vx-repo-contract** skill for the repository layout rules.

### `[scripts]` vs `justfile`: pick one task surface

`vx.toml` declares *tool versions*. A task runner declares *how work runs*.
Keeping both means every task has two definitions, and the stale one wins
silently.

- If the repo has a `justfile`, put tasks there and keep `[scripts]` out of
  `vx.toml`. A `[scripts]` entry that forwards to `just <recipe>` is a duplicate
  source of truth: rename the recipe and the script keeps invoking the old
  behaviour.
- Only use `[scripts]` when there is no `justfile`.

```bash
vx run test    # only meaningful when [scripts] exists
vx just test   # preferred when a justfile exists
```

**Check** — this must print nothing:

```bash
test -f justfile && vx rg -q '^\[scripts\]' vx.toml && echo "FAIL: [scripts] duplicates justfile"
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

### Rust Version Resolution

Rust is the one ecosystem where vx is **not** the authority. If the repository already
names a toolchain, vx delegates to rustup: it installs nothing, keeps its store off
`PATH`, and never runs `rustup default` (which would change the default for every other
project on the machine).

| Priority | Signal | Behaviour |
|---|---|---|
| 1 | `rust-toolchain.toml` / `rust-toolchain` | Delegate to rustup (searched upward from the working directory) |
| 2 | `RUSTUP_TOOLCHAIN` set | Delegate to rustup |
| 3 | `rust = "rustup-managed"` in `vx.toml` | Delegate to rustup, explicitly |
| 4 | `rust = "<version>"` in `vx.toml` | vx installs that toolchain into its store |

- Both (1) and (2) present → (2) wins, matching rustup's own precedence.
- A numeric `vx.toml` pin that disagrees with the effective toolchain is warned about
  on stderr and makes `vx check` exit non-zero. The toolchain file still wins; the
  warning exists so a dead pin cannot go unnoticed.
- For a repository that already pins with rustup, write `rust = "rustup-managed"`
  instead of omitting `rust` with an explanatory comment.
- An explicit `vx cargo@1.90.0` always goes through vx.

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

Group related scripts:

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

---

## Delivery Surface — External Systems Are Evidence

GitHub PRs, CI runs, and dashboards are **supporting evidence**, not the delivery
surface. The conclusion has to land where the work is tracked.

- **Record the outcome once, completely** — one issue comment carrying status,
  branch/commit/PR, what you validated, the blocker, and the next owner.
- **Verify terminal state, not intermediate state.** A green CI run or an open,
  review-ready PR is not "shipped". Confirm merged / released / deployed, then
  record *that*.
- **Keep public surfaces public-safe.** PR titles, bodies, and commit messages
  carry technical content only — no internal issue IDs, routing history,
  reviewer handoffs, local absolute paths, or internal hostnames.
- **Collect results in the foreground.** A queued or pending state is a handoff,
  not a completion; never end a turn "standing by" for background work.

For agents running in the Monica platform the delivery surface is Monica — the
issue comment plus issue metadata. A PR comment alone delivers nothing.
