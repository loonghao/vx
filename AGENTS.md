# VX — Universal Development Tool Manager

> **For AI agents**: This file is a **map**, not a manual. Start here, then drill into the linked docs as needed.

## What is vx?

vx is a **zero-config universal development tool manager**. Users prefix any command with `vx` (e.g., `vx node --version`, `vx cargo build`) and vx automatically installs, manages, and forwards to the correct tool version. vx currently ships **73 providers** covering language runtimes, build tools, DevOps CLIs, cloud platforms, and more — all defined via Starlark DSL (`provider.star`).

## Quick Orientation

| If you need to…                    | Start here                                       |
|-------------------------------------|--------------------------------------------------|
| Understand the architecture          | [`docs/architecture/OVERVIEW.md`](docs/architecture/OVERVIEW.md) |
| Read design decisions (RFCs)         | [`docs/rfcs/`](docs/rfcs/) — 50 RFCs             |
| Learn coding conventions             | [`docs/CONVENTIONS.md`](docs/CONVENTIONS.md)     |
| Add a new Provider (tool)            | [`docs/guide/creating-provider.md`](docs/guide/creating-provider.md) |
| Understand provider.star DSL fully   | [`docs/guide/provider-star-reference.md`](docs/guide/provider-star-reference.md) |
| Run CI tasks locally                 | [justfile](justfile) — `vx just --list`          |
| Understand the CI pipeline           | [`.github/workflows/ci.yml`](.github/workflows/ci.yml) |
| See all CLI commands                 | [`docs/cli/`](docs/cli/)                         |
| Follow unified syntax rules          | [`docs/guide/command-syntax-rules.md`](docs/guide/command-syntax-rules.md) |
| Check project configuration          | [`Cargo.toml`](Cargo.toml) (workspace root)      |
| See all 73 providers                 | [`crates/vx-providers/`](crates/vx-providers/)   |

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
│  vx-providers/*    (73 Providers — provider.star DSL)   │
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
| **Descriptor** | Dict returned by Starlark (phase 1) → interpreted by Rust (phase 2)  |

## Terminology Rules (Enforced)

| ✅ Correct     | ❌ Never use         |
|----------------|----------------------|
| Runtime        | Tool, VxTool         |
| Provider       | Plugin, Bundle       |
| provider.star  | provider config, provider.toml (legacy) |
| ProviderRegistry | BundleRegistry     |

## Starlark Provider System

vx uses a **two-phase execution model** (inspired by Buck2):

1. **Analysis Phase (Starlark)**: `provider.star` runs as pure computation, returning descriptor dicts. No I/O.
2. **Execution Phase (Rust)**: The Rust runtime interprets descriptors for actual downloads, installs, and process execution.

### Starlark Standard Library (14 modules)

Located in `crates/vx-starlark/stdlib/`:

| Module | Key exports | Purpose |
|--------|-------------|---------|
| `provider.star` | *(re-exports all)* | Unified facade — import from here for convenience |
| `runtime.star` | `runtime_def`, `bundled_runtime_def`, `dep_def` | Runtime definitions |
| `platform.star` | `platform_map`, `platform_select`, `rust_triple`, `go_os_arch`, `archive_ext`, `exe_suffix` | Platform detection & mapping |
| `env.star` | `env_set`, `env_prepend`, `env_append`, `env_unset` | Environment variable operations |
| `layout.star` | `archive_layout`, `binary_layout`, `bin_subdir_layout`, `post_extract_*`, `pre_run_ensure_deps` | Install layout, hooks, path helpers |
| `permissions.star` | `github_permissions`, `system_permissions` | Permission declarations |
| `system_install.star` | `winget_install`, `brew_install`, `apt_install`, `cross_platform_install`, etc. | System package manager strategies |
| `script_install.star` | `curl_bash_install`, `irm_iex_install`, `platform_script_install` | Script-based installation |
| `provider_templates.star` | `github_rust_provider`, `github_go_provider`, `github_binary_provider`, `system_provider` | High-level templates (cover 90% of cases) |
| `github.star` | GitHub API helpers | GitHub releases integration |
| `http.star` | HTTP descriptors | HTTP request building |
| `install.star` | Install descriptors | Installation helpers |
| `semver.star` | Version comparison | Semantic version utilities |
| `test.star` | Testing DSL | Provider test definitions |

### Provider Templates (Fastest Path)

```starlark
# Rust tool from GitHub releases (most common)
_p = github_rust_provider("BurntSushi", "ripgrep",
    asset = "rg-{version}-{triple}.{ext}", executable = "rg")

# Go tool from GitHub releases (goreleaser style)
_p = github_go_provider("cli", "cli",
    asset = "gh_{version}_{os}_{arch}.{ext}", executable = "gh")

# Single binary download (no archive)
_p = github_binary_provider("kubernetes", "kubectl",
    asset = "kubectl{exe}")

# System package manager only
_p = system_provider("7zip", executable = "7z")
```

### Template Placeholders

| Placeholder | Rust template | Go template | Description |
|-------------|---------------|-------------|-------------|
| `{version}` | ✓ | ✓ | Version number (e.g., "1.0.0") |
| `{vversion}` | ✓ | — | With v-prefix (e.g., "v1.0.0") |
| `{triple}` | ✓ | — | Rust target triple |
| `{os}` | — | ✓ | Go GOOS |
| `{arch}` | — | ✓ | Go GOARCH |
| `{ext}` | ✓ | ✓ | Archive extension (zip/tar.gz) |
| `{exe}` | ✓ | ✓ | Executable suffix (.exe/"") |

## All 73 Providers

Organized by category:

| Category | Providers |
|----------|-----------|
| **JavaScript** | node, bun, deno, pnpm, yarn, nx, turbo, vite |
| **Python** | uv, python, pre-commit |
| **Rust** | rust (cargo, rustc, rustup) |
| **Go** | go |
| **System/CLI** | git, bash, curl, pwsh, jq, yq, fd, bat, ripgrep, fzf, starship |
| **Build Tools** | just, task, cmake, ninja, make, meson, xmake, protoc, conan, vcpkg, spack |
| **DevOps** | kubectl, helm, podman, terraform, hadolint, dagu |
| **Cloud CLI** | awscli, azcli, gcloud |
| **.NET** | dotnet, msbuild, nuget |
| **C/C++** | msvc, llvm, nasm, ccache, buildcache, sccache, rcedit |
| **Media** | ffmpeg, imagemagick |
| **Java** | java |
| **Other Langs** | zig |
| **Package Managers** | brew, choco, winget |
| **AI** | ollama |
| **Misc** | gh, prek, actrun, wix, vscode, xcodebuild, systemctl, release-please, rez, 7zip |

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

### Scoped Commands (for single-file operations)

```bash
# Type-check a single crate
vx cargo check -p vx-cli

# Run tests for one crate only
vx cargo test -p vx-starlark

# Format a single file
vx cargo fmt -- --check crates/vx-cli/src/cli.rs

# Clippy a single crate
vx cargo clippy -p vx-resolver -- -D warnings
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
   load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
   load("@vx//stdlib:provider_templates.star", "github_rust_provider")

   name = "<name>"
   description = "<description>"
   ecosystem = "custom"
   runtimes = [runtime_def("<runtime>")]
   permissions = github_permissions()

   _p = github_rust_provider("owner", "repo",
       asset = "tool-{vversion}-{triple}.{ext}")
   fetch_versions   = _p["fetch_versions"]
   download_url     = _p["download_url"]
   install_layout   = _p["install_layout"]
   store_root       = _p["store_root"]
   get_execute_path = _p["get_execute_path"]
   environment      = _p["environment"]
   ```
3. Define metadata: `name`, `description`, `runtimes`, `permissions`
4. Test: `vx <runtime> --version`
5. Full guide: [`docs/guide/creating-provider.md`](docs/guide/creating-provider.md)
6. Complete DSL reference: [`docs/guide/provider-star-reference.md`](docs/guide/provider-star-reference.md)

## File Layout Conventions

```
crates/vx-<name>/
├── src/              # Source code
│   └── lib.rs        # Must have module-level doc comment
├── tests/            # Unit tests (NEVER inline #[cfg(test)])
│   └── *_tests.rs    # Use rstest framework
└── Cargo.toml

crates/vx-providers/<name>/
├── provider.star     # Provider definition (required, Starlark DSL)
├── provider.toml     # Provider manifest (metadata)
└── src/lib.rs        # Rust glue (if needed)

crates/vx-starlark/stdlib/
├── provider.star              # Unified facade (re-exports all)
├── runtime.star               # runtime_def, bundled_runtime_def
├── platform.star              # Platform detection
├── env.star                   # Environment variables
├── layout.star                # Install layout, hooks, paths
├── permissions.star           # Permission declarations
├── system_install.star        # Package manager strategies
├── script_install.star        # Script-based installation
├── provider_templates.star    # High-level templates
├── github.star                # GitHub API helpers
├── http.star                  # HTTP descriptors
├── install.star               # Install descriptors
├── semver.star                # Version comparison
└── test.star                  # Testing DSL
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
| `llms.txt` | LLM-friendly project index |
| `llms-full.txt` | Detailed LLM documentation |

## Security Considerations

- All tool downloads are from official sources (GitHub Releases, official APIs)
- Checksums are verified automatically when available
- `permissions` in `provider.star` declare exactly which network hosts a provider may access
- Never run `sudo vx install` — vx manages user-level installations under `~/.vx/`
- `GITHUB_TOKEN` should be provided for GitHub API rate limit avoidance

## Testing Conventions

- Tests go in `crates/<name>/tests/` directories — **never** inline `#[cfg(test)]` modules
- Use `rstest` for parameterized tests
- Run all tests: `vx just test`
- Run single crate: `vx just test-pkgs "-p vx-starlark"`
- E2E tests use `trycmd` for CLI snapshot testing
