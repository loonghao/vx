# VX — Universal Development Tool Manager

> **For AI agents**: This file is a **map**, not a manual. Start here, then drill into the linked docs as needed.
> If you are working on a project that uses vx, **always prefix commands with `vx`** (e.g., `vx npm install`, `vx cargo build`).
> Also see: [`llms.txt`](llms.txt) for a concise LLM-friendly project index, [`llms-full.txt`](llms-full.txt) for detailed LLM documentation.
>
> **Compatibility**: This file follows the [AGENTS.md](https://agents.md/) open standard (managed by Agentic AI Foundation / Linux Foundation). It is recognized by OpenAI Codex, Google Jules, GitHub Copilot, Cursor, Amp, Factory, Aider, Zed, Warp, JetBrains Junie, Devin, and other AI coding agents.

## What is vx?

vx is a **zero-config universal development tool manager** (v0.8.16, MIT-licensed, written in Rust). Users prefix any command with `vx` (e.g., `vx node --version`, `vx cargo build`) and vx automatically installs, manages, and forwards to the correct tool version. vx currently ships **105 providers** covering language runtimes, build tools, DevOps CLIs, cloud platforms, and more — all defined via Starlark DSL (`provider.star`).

**Key insight for agents**: vx is a transparent proxy. The user writes the exact same commands they already know — just prepended with `vx`. There is **no new syntax to learn** for tool execution.

```bash
# These are identical to native commands, just add `vx`:
vx node --version          # Auto-installs Node.js if needed
vx cargo build --release   # Auto-installs Rust if needed
vx uv pip install flask    # Auto-installs uv if needed
vx npx create-react-app x  # Auto-installs Node.js + runs npx
```

### One-Sentence Summary

**vx = prefix any dev tool command with `vx` → it auto-installs the tool and runs it.**

### How vx Works (Execution Flow)

When you run `vx node --version`, this happens internally:

```
1. CLI parses "node" as the runtime name
2. Resolver looks up "node" in ProviderRegistry (via provider.star)
3. Resolver checks if node is installed (in ~/.vx/store/node/<version>/)
4. If missing → Installer downloads from URL returned by provider.star's download_url()
5. Environment is prepared via provider.star's environment() function
6. Command is forwarded to the actual node binary with all args
```

This entire flow is **automatic** — the user never needs to know about it.

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
| See all 105 providers                | [`crates/vx-providers/`](crates/vx-providers/)   |
| Contribute to the project            | [`docs/advanced/contributing.md`](docs/advanced/contributing.md) |
| Understand vx.toml configuration     | [`docs/config/vx-toml.md`](docs/config/vx-toml.md) |
| Troubleshoot issues                  | [`docs/appendix/troubleshooting.md`](docs/appendix/troubleshooting.md) |

## Mental Model for AI Agents

Think of vx as a **universal shim layer** between the user and their development tools:

```
User Command: vx npm install
                 │
                 ▼
          ┌─────────────┐
          │   vx CLI     │  ← Parses "npm" as the runtime name
          └──────┬───────┘
                 │
          ┌──────▼───────┐
          │   Resolver    │  ← Looks up "npm" → bundled with "node" Provider
          └──────┬───────┘
                 │
          ┌──────▼───────┐
          │  Is node      │  ← Checks ~/.vx/store/node/<version>/
          │  installed?   │
          └──┬────────┬──┘
          No │        │ Yes
             ▼        ▼
       ┌──────────┐  ┌──────────┐
       │ Install   │  │ Prepare  │  ← Sets PATH, env vars from provider.star
       │ node      │  │ env      │
       └─────┬────┘  └────┬─────┘
             └──────┬─────┘
                    ▼
          ┌─────────────┐
          │ Forward cmd  │  ← Executes: npm install (with correct PATH)
          └─────────────┘
```

**Key principle**: vx never modifies commands — it only ensures the right tool version is available, then transparently forwards the command.

## Critical Rules for AI Agents

**These rules MUST be followed when working on or with vx:**

1. **Always use `vx` prefix** — Never suggest `npm install`, always `vx npm install`
2. **Never suggest manual tool installation** — vx handles it automatically
3. **Use correct terminology** — Runtime (not Tool), Provider (not Plugin), provider.star (not provider config)
4. **Tests go in `tests/` directories** — Never write inline `#[cfg(test)]` modules in source files
5. **Use `rstest`** for parameterized tests
6. **Use `vx just`** for task runner commands (see [justfile](justfile))
7. **Check `vx.toml`** first to understand project tool requirements
8. **New providers use Starlark DSL only** — No Rust code required for new tool definitions
9. **Layer dependencies go downward only** — Never import from a higher architectural layer
10. **Provider count is 105** — Update any docs that reference old counts (78, 73, 70+, 50+, etc.)

### Setup Commands

```bash
# Install vx itself
# Linux/macOS:
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
# Windows (PowerShell):
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"

# Clone and set up vx development environment
git clone https://github.com/loonghao/vx.git
cd vx
vx just quick                  # format → lint → test → build

# Set up any vx-managed project
cd <project-with-vx-toml>
vx setup                       # Install all tools from vx.toml
vx dev                         # Enter dev environment
```

### Common AI Agent Scenarios

| Scenario | What to do |
|----------|------------|
| User says "install Node.js" | Run `vx node --version` (auto-installs) or `vx install node@22` |
| User says "run npm test" | Run `vx npm test` |
| User says "set up project" | Check for `vx.toml`, then run `vx setup` |
| User says "add Python to project" | Run `vx add python@3.12` then `vx sync` |
| User says "use vite" | Run `vx vite` (package alias, auto-routes to `vx npm:vite`) |
| MCP server needs npx | Use `"command": "vx", "args": ["npx", ...]` in MCP config |
| Need to check tool version | Run `vx which <tool>` or `vx <tool> --version` |
| CI/CD setup | Use `loonghao/vx@main` GitHub Action with `setup: 'true'` |
| Developing vx itself | Run `vx just quick` for format → lint → test → build cycle |
| User encounters errors | Run `vx doctor` first, then `vx --debug <command>` |
| Need to update vx itself | Run `vx self-update` |
| Analyze project structure | Run `vx analyze --json` for AI-parseable project analysis |

### What NOT to Do

| ❌ Don't | ✅ Do instead |
|----------|---------------|
| `npm install` | `vx npm install` |
| `pip install flask` | `vx uv pip install flask` |
| `go build` | `vx go build` |
| Tell user to "install Node.js first" | Just run `vx node --version` (auto-installs) |
| Suggest `nvm use 22` | Suggest `vx switch node@22` or set in `vx.toml` |
| Write `#[cfg(test)] mod tests {}` | Create tests in `crates/<name>/tests/` directory |
| Use `println!` for logging | Use `tracing::info!`, `tracing::debug!`, etc. |

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
│  vx-providers/*    (105 Providers — provider.star DSL)  │
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
| **Package Alias** | Short command that routes to an ecosystem package (e.g., `vx vite` = `vx npm:vite`) |

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

### Minimal Complete Provider Example

This is the **simplest possible provider** — copy-paste and modify for new tools:

```starlark
# crates/vx-providers/mytool/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "mytool"
description = "My awesome tool"
ecosystem   = "custom"

runtimes    = [runtime_def("mytool", aliases = ["mt"])]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "mytool-{vversion}-{triple}.{ext}")

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

### Real-World Provider Examples (for reference)

| Provider | Pattern | File |
|----------|---------|------|
| ripgrep | Template (github_rust_provider) | `crates/vx-providers/ripgrep/provider.star` |
| just | Template (github_rust_provider) | `crates/vx-providers/just/provider.star` |
| uv | Template (github_rust_provider) | `crates/vx-providers/uv/provider.star` |
| go | Hand-written (custom download_url) | `crates/vx-providers/go/provider.star` |
| node | Hand-written (official API) | `crates/vx-providers/node/provider.star` |

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

### Starlark Context Object (`ctx`)

All provider.star functions receive a `ctx` object with these fields:

| Field | Type | Description |
|-------|------|-------------|
| `ctx.platform.os` | string | `"windows"`, `"macos"`, `"linux"` |
| `ctx.platform.arch` | string | `"x64"`, `"arm64"` |
| `ctx.install_dir` | string | Where the tool will be/is installed |
| `ctx.store_dir` | string | Global store root (`~/.vx/store`) |
| `ctx.cache_dir` | string | Cache directory |
| `ctx.home_dir` | string | User home directory |

**Example usage in provider.star**:
```starlark
def download_url(ctx, version):
    if ctx.platform.os == "windows":
        return "https://example.com/tool-{}-win.zip".format(version)
    return "https://example.com/tool-{}-unix.tar.gz".format(version)

def environment(ctx, _version):
    return [
        env_prepend("PATH", ctx.install_dir + "/bin"),
        env_set("TOOL_HOME", ctx.install_dir),
    ]
```

## All 105 Providers

Organized by category:

| Category | Providers |
|----------|-----------|
| **JavaScript** | node, bun, deno, pnpm, yarn, nx, turbo, vite |
| **JS Tooling** | oxlint, biome |
| **Python** | uv, python, pre-commit, maturin, ruff |
| **Rust** | rust (cargo, rustc, rustup) |
| **Go** | go, gws |
| **System/CLI** | git, bash, curl, pwsh, jq, yq, fd, bat, ripgrep, fzf, starship, jj, sd, eza, dust, duf, xh, atuin, zoxide, tealdeer, gping, delta, hyperfine, watchexec, bottom |
| **TUI/Terminal** | helix, yazi, zellij, lazygit, lazydocker, k9s |
| **Build Tools** | just, task, cmake, ninja, make, meson, xmake, protoc, conan, vcpkg, spack |
| **DevOps** | kubectl, helm, podman, terraform, hadolint, dagu, actionlint |
| **Security** | gitleaks, trivy |
| **Cloud CLI** | awscli, azcli, gcloud |
| **.NET** | dotnet, msbuild, nuget |
| **C/C++** | msvc, llvm, nasm, ccache, buildcache, sccache, rcedit |
| **Media** | ffmpeg, imagemagick |
| **Java** | java |
| **Other Langs** | zig |
| **Container** | dive |
| **Config Mgmt** | chezmoi, mise |
| **Package Managers** | brew, choco, winget |
| **AI** | ollama, openclaw |
| **Misc** | gh, prek, actrun, wix, vscode, xcodebuild, systemctl, release-please, rez, 7zip, trippy |

## Decision Framework for AI Agents

When an AI agent encounters a task involving vx, follow this decision tree:

```
Is the user working ON vx (developing vx itself)?
├── YES → Use `vx just <task>` for builds, tests, linting
│         Tests go in crates/<name>/tests/ (never inline)
│         Use rstest, tracing, and correct terminology
│         New providers: create provider.star in crates/vx-providers/<name>/
│
└── NO → Is the user working WITH vx (using vx in their project)?
    ├── YES → Always prefix commands: `vx npm install`, `vx cargo build`
    │         Check vx.toml for project tool requirements
    │         Use `vx run <script>` for project scripts
    │         Never suggest manual tool installation
    │
    └── UNCLEAR → Check for vx.toml or .vx/ in project root
                  If found → treat as vx-managed project
                  If not → ask user or suggest `vx init`
```

### Provider Development Decision Tree

```
Need to add a new tool to vx?
├── Tool releases on GitHub?
│   ├── Rust target triple naming? → github_rust_provider (most common)
│   ├── Go goreleaser style?       → github_go_provider
│   └── Single binary (no archive)?→ github_binary_provider
├── System package manager only?   → system_provider
└── Custom download source?        → Hand-write download_url function
```

### Version Resolution Priority

When determining which tool version to use, vx follows this priority:

```
1. Command-line override: vx node@22 app.js  (highest)
2. Project vx.toml: [tools] node = "22"
3. Parent directory vx.toml (traverses up)
4. User global config: ~/.config/vx/config.toml
5. Provider default: latest stable version    (lowest)
```

## Common Tasks Quick Reference

### For agents working on vx-managed projects

```bash
# Run any tool (auto-installs if missing)
vx node app.js
vx cargo build --release
vx npm install
vx python script.py

# Project setup from vx.toml
vx setup                       # Install all project tools
vx dev                         # Enter dev environment
vx run test                    # Run project scripts

# Package aliases (shorter commands)
vx vite                        # Same as: vx npm:vite
vx meson                       # Same as: vx uv:meson
```

### For agents developing vx itself

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

# Scoped commands (faster feedback)
vx cargo check -p vx-cli              # Type-check one crate
vx cargo test -p vx-starlark          # Test one crate
vx cargo clippy -p vx-resolver -- -D warnings  # Lint one crate
```

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
| `llms.txt` | LLM-friendly project index (summary) |
| `llms-full.txt` | Detailed LLM documentation (complete) |

## Security Considerations

- All tool downloads are from official sources (GitHub Releases, official APIs)
- Checksums are verified automatically when available
- `permissions` in `provider.star` declare exactly which network hosts a provider may access
- Never run `sudo vx install` — vx manages user-level installations under `~/.vx/`
- `GITHUB_TOKEN` should be provided for GitHub API rate limit avoidance

## PR and Commit Guidelines

- **Commit messages**: Use [Conventional Commits](https://www.conventionalcommits.org/) format: `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`
- **Branch naming**: `<type>/<short-description>` (e.g., `feat/add-zig-provider`, `fix/node-install-windows`)
- **Before creating a PR**: Run `vx just quick` (format → lint → test → build) to ensure CI will pass
- **Pre-commit hooks**: Run `vx prek install` once after cloning, then hooks run automatically
- **PR scope**: Keep PRs focused — one feature or fix per PR. Split large changes into smaller PRs
- **New providers**: Each new provider should be a separate PR with its `provider.star` file
- **Test coverage**: Include tests for any new functionality. Tests go in `crates/<name>/tests/`
- **Documentation**: Update relevant docs when changing user-facing behavior

## Code Style

- **Language**: Rust (edition 2024, MSRV 1.93+)
- **Formatting**: `rustfmt` via `vx cargo fmt` — always run before committing
- **Linting**: `clippy` with warnings as errors — `vx cargo clippy -- -D warnings`
- **Imports**: Group by stdlib → external crates → internal crates, separated by blank lines
- **Error handling**: Use `anyhow::Result` for application code, `thiserror` for library error types
- **Async**: Tokio-based async/await for all I/O operations
- **Logging**: Always use `tracing` macros (`tracing::info!`, `tracing::debug!`), never `println!` or `eprintln!`
- **Naming**: `PascalCase` for types, `snake_case` for functions/modules, `SCREAMING_SNAKE_CASE` for constants
- **Documentation**: All `pub` items must have doc comments; every `lib.rs` needs a module-level description
- **File size**: Keep source files under 500 lines; split large files into modules

## Testing Conventions

- Tests go in `crates/<name>/tests/` directories — **never** inline `#[cfg(test)]` modules
- Use `rstest` for parameterized tests
- Test file naming: `<feature>_tests.rs` (e.g., `resolver_tests.rs`, `executor_tests.rs`)
- Run all tests: `vx just test`
- Run single crate: `vx just test-pkgs "-p vx-starlark"`
- E2E tests use `trycmd` for CLI snapshot testing
- Provider static checks: `vx just test-providers-static`
- Mock network calls in unit tests — never use real HTTP in unit tests
- Each test should be independent — no shared mutable state between tests

## Allowed vs. Needs-Approval Actions

This section helps AI agents understand their operational boundaries:

### ✅ Allowed Without Asking
- Read any file in the repository
- Run `vx just quick`, `vx just test`, `vx just lint`, `vx just format`
- Run `vx cargo check -p <crate>` or `vx cargo test -p <crate>`
- Create or modify files under `crates/vx-providers/` (new providers)
- Create or modify test files under `crates/*/tests/`
- Run `vx <tool> --version` to check tool availability

### ⚠️ Ask First
- Deleting any file
- Modifying `Cargo.toml` workspace dependencies
- Running `git push` or `git push --force`
- Modifying CI workflows (`.github/workflows/`)
- Installing new system-level packages
- Running full E2E test suite (can be slow)
- Changing architecture layer boundaries

## GitHub Actions Integration

vx provides a GitHub Action for CI/CD. See [`docs/guides/github-action.md`](docs/guides/github-action.md) for the full guide.

```yaml
# Minimal CI usage
- uses: loonghao/vx@main
  with:
    tools: 'node@22 uv'
    setup: 'true'
    cache: 'true'
    github-token: ${{ secrets.GITHUB_TOKEN }}
- run: vx node --version
- run: vx npm test
```

> **Tip**: Use `@main` for latest, or pin to a release tag (e.g., `@vx-v0.8.15`).
> Check [releases](https://github.com/loonghao/vx/releases) for available versions.

## Documentation Map

```
# AI Agent Ecosystem (root files)
AGENTS.md                 # THIS FILE — primary AI agent entry point
llms.txt                  # LLM-friendly project index (for llmstxt.org protocol)
llms-full.txt             # Detailed LLM documentation
.github/copilot-instructions.md  # GitHub Copilot-specific instructions
.cursorrules              # Cursor IDE agent rules

# Documentation site
docs/
├── architecture/     # System architecture (OVERVIEW.md)
├── guide/            # User guides (22 files — getting-started, provider-star-reference, etc.)
├── cli/              # CLI command reference (17 commands)
├── config/           # Configuration reference (vx-toml, env-vars, etc.)
├── tools/            # Tool category docs (14 categories)
├── advanced/         # Contributing, security, extension development
├── guides/           # Practical guides (GitHub Actions, use cases)
├── rfcs/             # 50 design decision documents
├── appendix/         # FAQ, troubleshooting
└── zh/               # Chinese translations (72+ files)

# AI Agent Skills (distributed via vx ai setup and ClawHub)
skills/
├── vx-usage/SKILL.md           # Core usage guide
├── vx-commands/SKILL.md        # CLI command reference
├── vx-project/SKILL.md         # Project management
├── vx-best-practices/SKILL.md  # Best practices & provider development
└── vx-troubleshooting/SKILL.md # Troubleshooting & recovery
```

### Crate Responsibilities Quick Reference

When you need to modify specific functionality, here's where to look:

| What you need | Crate | Key files |
|---------------|-------|-----------|
| Add CLI subcommand | `vx-cli` | `src/cli.rs`, `src/commands/` |
| Modify execution logic | `vx-resolver` | `src/executor.rs`, `src/resolver.rs` |
| Add/modify Starlark stdlib | `vx-starlark` | `stdlib/*.star`, `src/stdlib.rs` |
| Change provider loading | `vx-starlark` | `src/engine.rs`, `src/handle.rs` |
| Modify runtime registry | `vx-runtime` | `src/registry/` |
| Change install behavior | `vx-installer` | `src/lib.rs` |
| Modify version fetching | `vx-version-fetcher` | `src/lib.rs` |
| Change path conventions | `vx-paths` | `src/lib.rs` |
| Modify config loading | `vx-config` | `src/lib.rs` |
| Add project detection | `vx-project-analyzer` | `src/frameworks/` |
| Change console output | `vx-console` | `src/lib.rs` |
| Core trait definitions | `vx-core` | `src/lib.rs` |
| Add a new provider | `vx-providers/<name>` | `provider.star` |

### Key Configuration Files

| File | Purpose | When to modify |
|------|---------|----------------|
| `Cargo.toml` (root) | Workspace deps, build profiles | Adding dependencies, new crates |
| `justfile` | All dev commands | Adding new dev workflows |
| `clippy.toml` | Lint configuration | Adjusting lint rules |
| `codecov.yml` | Coverage thresholds | Changing coverage requirements |
| `vx.toml` | Project tool versions | Updating project tool requirements |
| `action.yml` | GitHub Action definition | Modifying CI/CD action inputs/outputs |

## Quick Diagnostics for AI Agents

If something doesn't work, try these steps in order:

```bash
# 1. Check vx health
vx doctor

# 2. Check what's installed
vx list --installed

# 3. Verify specific tool
vx which node
vx node --version

# 4. Debug with verbose output
vx --debug node --version

# 5. Clean cache and retry
vx cache clean
vx install node --force

# 6. Check project config
vx check --json
```

### Troubleshooting Decision Tree

```
User reports an issue with vx:
│
├─ "command not found: vx"
│  → vx is not installed
│  → Linux/macOS: curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
│  → Windows: powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
│
├─ "Failed to download" / network error (exit code 5)
│  → vx cache clean && vx install <tool> --verbose
│  → If in China: vx config set cdn_acceleration true
│  → Ensure GITHUB_TOKEN is set for API rate limits
│
├─ "version not found" (exit code 4)
│  → vx versions <tool> to list available versions
│  → Check for typos in version string
│  → Try: vx install <tool>@latest
│
├─ "permission denied" (exit code 6)
│  → Check permissions: ls -la ~/.vx (Unix) or icacls %USERPROFILE%\.vx (Windows)
│  → Fix: chmod -R u+rw ~/.vx
│  → Never use sudo with vx
│
├─ Wrong tool version running
│  → vx which <tool> to see active version
│  → Check vx.toml for version constraints
│  → vx switch <tool>@<version>
│
├─ vx.toml not being picked up (exit code 7)
│  → Ensure file is in project root (same dir as .git)
│  → vx check to validate syntax
│
├─ CI failing with vx
│  → Use GitHub Action: loonghao/vx@main
│  → Add github-token for rate limit avoidance
│  → Use cache: 'true' for faster CI
│
└─ General error (exit code 1)
   → vx doctor for full diagnostics
   → vx --debug <command> for detailed logs
   → vx cache clean to clear corrupted state
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Tool not found |
| 3 | Installation failed |
| 4 | Version not found |
| 5 | Network error |
| 6 | Permission error |
| 7 | Configuration error |

### AI-Optimized Output

vx supports structured output for efficient agent consumption:

```bash
vx list --json                    # JSON output
vx list --format toon             # Token-optimized output (saves 40-60% tokens)
vx analyze --json                 # Project analysis as JSON
vx ai context --json              # Full AI-friendly context
export VX_OUTPUT=json             # Default all commands to JSON
```

## MCP Integration Guide

vx is **MCP-ready** — replace `npx`/`uvx` with `vx` in any MCP server configuration. This eliminates the need for users to pre-install Node.js, Python, or any other runtime.

### MCP Server Configuration Pattern

```json
{
  "mcpServers": {
    "example-server": {
      "command": "vx",
      "args": ["npx", "-y", "@example/mcp-server@latest"]
    },
    "python-server": {
      "command": "vx",
      "args": ["uvx", "some-python-mcp-server@latest"]
    }
  }
}
```

**Key benefit**: Users don't need to install Node.js or Python separately — vx handles it automatically when the MCP server starts.

### When to recommend vx for MCP

| User's current config | Suggest |
|----------------------|---------|
| `"command": "npx"` | `"command": "vx", "args": ["npx", ...]` |
| `"command": "uvx"` | `"command": "vx", "args": ["uvx", ...]` |
| `"command": "node"` | `"command": "vx", "args": ["node", ...]` |
| `"command": "python"` | `"command": "vx", "args": ["python", ...]` |

## Skills Distribution

vx ships AI agent skills in the [`skills/`](skills/) directory. These skills are the **single source of truth** shared across 13+ AI agents:

```bash
# Install skills to all AI agents
vx ai setup

# Skills directory structure
skills/
├── vx-usage/SKILL.md           # Core usage guide
├── vx-commands/SKILL.md        # CLI command reference
├── vx-project/SKILL.md         # Project management
├── vx-best-practices/SKILL.md  # Best practices & provider development
└── vx-troubleshooting/SKILL.md # Troubleshooting & recovery
```

These skills trigger automatically when the project contains `vx.toml` or `.vx/`, or when the user mentions `vx`, tool version management, or cross-platform setup.
