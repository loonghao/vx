# VX — Universal Development Tool Manager

> **For AI agents**: This file is a **map**, not a manual. Start here, then drill into the linked docs as needed.

## What is vx?

vx is a **zero-config universal development tool manager**. Users prefix any command with `vx` (e.g., `vx node --version`, `vx cargo build`) and vx automatically installs, manages, and forwards to the correct tool version.

## Quick Orientation

| If you need to…                    | Start here                                       |
|-------------------------------------|--------------------------------------------------|
| Understand the architecture          | [`docs/architecture/OVERVIEW.md`](docs/architecture/OVERVIEW.md) |
| Read design decisions (RFCs)         | [`docs/rfcs/`](docs/rfcs/)                       |
| Learn coding conventions             | [`docs/CONVENTIONS.md`](docs/CONVENTIONS.md)     |
| Add a new Provider (tool)            | [`docs/guide/creating-provider.md`](docs/guide/creating-provider.md) |
| Run CI tasks locally                 | [justfile](justfile) — `vx just --list`          |
| Understand the CI pipeline           | [`.github/workflows/ci.yml`](.github/workflows/ci.yml) |
| See all CLI commands                 | [`docs/cli/`](docs/cli/)                         |
| Follow unified syntax rules          | [`docs/guide/command-syntax-rules.md`](docs/guide/command-syntax-rules.md) |
| Check project configuration          | [`Cargo.toml`](Cargo.toml) (workspace root)      |

## Project Structure (Layered Architecture)

```
┌─────────────────────────────────────────────────────────┐
│  vx-cli            (Application layer — CLI entry)      │
├─────────────────────────────────────────────────────────┤
│  vx-resolver       (Orchestration — resolve & execute)  │
│  vx-setup          (Environment setup)                  │
│  vx-migration      (Version migration)                  │
│  vx-extension      (Extension system)                   │
│  vx-project-analyzer (Project detection)                │
├─────────────────────────────────────────────────────────┤
│  vx-runtime        (Runtime management & registry)      │
│  vx-starlark       (Starlark DSL engine for providers)  │
│  vx-installer      (Download & install)                 │
│  vx-version-fetcher (Version resolution)                │
│  vx-config         (Configuration management)           │
│  vx-console        (Unified output & progress)          │
│  vx-env            (Environment variables)              │
│  vx-metrics        (Telemetry & tracing)                │
├─────────────────────────────────────────────────────────┤
│  vx-core           (Foundation — traits & types)        │
│  vx-paths          (Path management)                    │
│  vx-cache          (Caching layer)                      │
│  vx-versions       (Semver utilities)                   │
│  vx-manifest       (Provider manifest parsing)          │
│  vx-args           (Argument parsing)                   │
├─────────────────────────────────────────────────────────┤
│  vx-providers/*    (70+ Providers — provider.star DSL)  │
│  vx-bridge         (Generic command bridge)             │
└─────────────────────────────────────────────────────────┘
```

**Dependency rule**: Each layer may only depend on layers **below** it. Never upward.

## Key Concepts

| Concept        | Definition                                                            |
|----------------|-----------------------------------------------------------------------|
| **Runtime**    | An executable tool managed by vx (node, go, uv, ripgrep…)            |
| **Provider**   | A module that defines how to install/manage a Runtime                 |
| **provider.star** | Starlark DSL file that declaratively describes a Provider          |
| **Ecosystem**  | A language/tool family (nodejs, python, rust, go, system, custom)     |
| **Bundled Runtime** | A Runtime shipped inside another (npm bundled with node)         |

## Terminology Rules (Enforced)

| ✅ Correct     | ❌ Never use         |
|----------------|----------------------|
| Runtime        | Tool, VxTool         |
| Provider       | Plugin, Bundle       |
| provider.star  | provider config      |
| ProviderRegistry | BundleRegistry     |

## Development Workflow

```bash
# Prerequisites: Rust toolchain (1.93+), just

# Build
vx just build                  # Debug build
vx just build-release          # Release build

# Test
vx just test                   # All tests (nextest)
vx just test-fast              # Unit tests only (skip e2e)
vx just test-pkgs "-p vx-cli"  # Test specific crate

# Code Quality
vx just format                 # Format code
vx just lint                   # Clippy lints
vx just check-architecture     # Verify layer dependencies
vx just check-file-sizes       # Enforce file size limits
vx just doctor                 # Diagnose dev environment

# Quick pre-commit cycle
vx just quick                  # format → lint → test → build

# Documentation
vx just docs-dev               # Start docs dev server
```

## CI Pipeline Overview

The CI is **change-aware** — it detects which crates changed and only tests affected code.

```
detect-changes → build-vx (multi-platform) → code-quality
                                            → test-targeted / test-full
                                            → security-audit
                                            → coverage (main only)
                                            → cross-build (main only)
                                            → docs-build
```

**Key CI decisions**:
- `codecov` is **informational only** (won't block merge)
- `cancel-in-progress` prevents stale runs
- `sccache` accelerates Rust compilation
- `cargo-nextest` for parallel test execution

## Adding a New Provider

1. Create `crates/vx-providers/<name>/provider.star`
2. Use a template (covers 90% of cases):
   ```starlark
   load("@vx//stdlib:provider_templates.star", "github_rust_provider")
   _p = github_rust_provider("owner", "repo",
       asset = "tool-{vversion}-{triple}.{ext}")
   ```
3. Define metadata: `name`, `description`, `runtimes`, `permissions`
4. Test: `vx <runtime> --version`
5. Full guide: [`docs/guide/creating-provider.md`](docs/guide/creating-provider.md)

## File Layout Conventions

```
crates/vx-<name>/
├── src/              # Source code
│   └── lib.rs        # Must have module-level doc comment
├── tests/            # Unit tests (NEVER inline #[cfg(test)])
│   └── *_tests.rs    # Use rstest framework
└── Cargo.toml

crates/vx-providers/<name>/
├── provider.star     # Provider definition (required)
├── provider.toml     # Provider manifest (metadata)
└── src/lib.rs        # Rust glue (if needed)
```

## Command Syntax Guardrails

**Single source of truth**: all syntax decisions and evolution rules must follow [`docs/guide/command-syntax-rules.md`](docs/guide/command-syntax-rules.md).

Use these canonical forms consistently in docs and examples:

- Runtime execution: `vx <runtime>[@version] [args...]`
- Runtime executable override: `vx <runtime>[@version]::<executable> [args...]`
  - Example: `vx msvc@14.42::cl main.cpp`
- Package execution: `vx <ecosystem>[@runtime_version]:<package>[@version][::executable] [args...]`
  - Example: `vx uvx:pyinstaller::pyinstaller --version`
- Multi-runtime composition: `vx --with <runtime>[@version] [--with ...] <target_command>`
  - Example: `vx --with bun@1.1.0 --with deno node app.js`
- Runtime shell launch (canonical): `vx shell launch <runtime>[@version] [shell]`
  - Example: `vx shell launch node@22 powershell`
  - Compatibility alias: `vx <runtime>::<shell>` (e.g., `vx node::powershell`)
- Global package management (canonical): `vx pkg <install|uninstall|list|info|update> ...`
  - Example: `vx pkg install npm:typescript`
  - Compatibility alias: `vx global ...`
- Project-aware execution and synchronization: `vx run`, `vx sync`, `vx lock`, `vx check`

Parsing and state synchronization guardrails:

- Keep parser behavior, docs, and CLI help examples synchronized with the canonical rules.
- Keep project state synchronization explicit via `vx sync` + `vx lock` contracts.
- For compatibility aliases, provide clear migration hints; avoid silent reinterpretation.

Rust conventions:


- Prefer `vx cargo` / `vx rustc` for daily use.
- Configure `rustup` in `vx.toml` (not `rust` toolchain versions).
- `rustup` version is not the same as `rustc`/`cargo` toolchain version.

## Key Files for Context

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace members, shared deps, build profiles |
| `justfile` | All development commands |
| `clippy.toml` | Clippy configuration |
| `codecov.yml` | Coverage thresholds |
| `Cross.toml` | Cross-compilation config |
| `action.yml` | GitHub Action for external users |
| `.github/workflows/ci.yml` | Main CI pipeline |
| `.github/workflows/maintenance.yml` | Automated tech debt scanning |
| `vx.toml` | Project-level tool versions |
