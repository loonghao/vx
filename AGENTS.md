# VX — Universal Development Tool Manager

> **For AI agents**: This file is an **orientation guide and quick reference** — start here, then follow the links into dedicated docs as needed.
> If you are working on a project that uses vx, **always prefix commands with `vx`** (e.g., `vx npm install`, `vx cargo build`).
> Also see: [`CLAUDE.md`](CLAUDE.md) for Claude Code, [`llms.txt`](llms.txt) for a concise LLM-friendly project index, [`llms-full.txt`](llms-full.txt) for detailed LLM documentation.
>
> **Compatibility**: This file follows the [AGENTS.md](https://agents.md/) open standard (managed by Agentic AI Foundation / Linux Foundation). It is recognized by OpenAI Codex, Google Jules, GitHub Copilot, Cursor, Amp, Factory, Aider, Zed, Warp, JetBrains Junie, Devin, and other AI coding agents.

## What is vx?

vx is a **zero-config universal development tool manager** (v0.8.32, MIT-licensed, written in Rust). Users prefix any command with `vx` (e.g., `vx node --version`, `vx cargo build`) and vx automatically installs, manages, and forwards to the correct tool version. vx currently ships **129 providers** covering language runtimes, build tools, DevOps CLIs, cloud platforms, and more — all defined via Starlark DSL (`provider.star`).

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
| See all 129 providers                | [`crates/vx-providers/`](crates/vx-providers/)   |
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
10. **Provider count is 129** — Update any docs that reference old counts (78, 73, 70+, 50+, 122, 124, 126, etc.)

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
| User says "global install CLI (all languages)" | Prefer ecosystem-native global form (e.g., `vx npm install -g <pkg>`, `vx pip install --user <pkg>`, `vx cargo install <pkg>`, `vx go install <module>@<ver>`, `vx gem install <pkg>`) |
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
│  vx-providers/*    (129 Providers — provider.star DSL)  │
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

vx uses a **two-phase execution model**: `provider.star` runs as pure Starlark computation (no I/O) returning descriptor dicts, which the Rust runtime then interprets for actual downloads, installs, and process execution. Four high-level templates cover 90% of cases: `github_rust_provider`, `github_go_provider`, `github_binary_provider`, and `system_provider`.

For the complete DSL reference — stdlib modules, context object fields, template placeholders, and real-world examples — see [`docs/guide/provider-star-reference.md`](docs/guide/provider-star-reference.md).

## All Providers

vx supports 129 providers across JavaScript, Python, Rust, Go, system tools, DevOps, security, cloud, .NET, C/C++, media, and more. See [`docs/tools/overview.md`](docs/tools/overview.md) for the complete list.

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

```
1. Command-line: vx node@22 app.js   (highest)
2. Project vx.toml: [tools] node = "22"
3. Parent directory vx.toml (traverses up)
4. User global: ~/.config/vx/config.toml
5. Provider default: latest stable   (lowest)
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

# Cross-ecosystem global install (auto-register + shim generation)
vx npm install -g @tencent-ai/codebuddy-code
vx pnpm add -g eslint
vx yarn global add typescript
vx pip install --user ruff
vx cargo install ripgrep
vx go install golang.org/x/tools/gopls@latest
vx gem install bundler
```

### Global Install Contract (Agent-Facing)

When a user runs ecosystem-native global install commands through `vx`, vx should
preserve the familiar command style while ensuring vx-managed behavior:

1. Install into vx-managed package isolation directory (`~/.vx/packages/...`)
2. Register package metadata in `~/.vx/config/global-packages.json`
3. Create/update shims under `~/.vx/shims`
4. Keep executables discoverable by `vx <exe>` and direct shell usage (with shims dir in `PATH`)

This contract avoids ecosystem-specific drift and provides consistent behavior across languages.

### For agents developing vx itself

```bash
# One-time setup
vx just setup-hooks            # Install git hooks

# Build & test
vx just build                  # Debug build
vx just build-release          # Release build
vx just test                   # All tests (nextest)
vx just test-fast              # Unit tests only
vx just test-pkgs "-p vx-cli"  # Test specific crate

# Code quality
vx just format                 # Format
vx just lint                   # Clippy
vx just check-architecture     # Verify layer deps
vx just quick                  # format → lint → test → build (pre-commit)

# Scoped (faster)
vx cargo check -p vx-cli
vx cargo test -p vx-starlark
vx cargo clippy -p vx-resolver -- -D warnings
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

Change-aware CI — only tests affected crates:

```
detect-changes → build-vx (multi-platform) → code-quality
                                            → test-targeted / test-full
                                            → security-audit
                                            → coverage / cross-build (main only)
                                            → docs-build
```

`codecov` is informational only; `sccache` accelerates compilation; `cargo-nextest` for parallel tests.

## File Layout Conventions

```
crates/vx-<name>/
├── src/lib.rs        # Must have module-level doc comment
├── tests/*_tests.rs  # Unit tests (NEVER inline #[cfg(test)]) — use rstest
└── Cargo.toml

crates/vx-providers/<name>/
├── provider.star     # Provider definition (required, Starlark DSL)
└── src/lib.rs        # Rust glue (required for built-in providers)

crates/vx-starlark/stdlib/
├── provider.star              # Unified facade (re-exports all 14 modules)
├── provider_templates.star    # github_rust/go/binary_provider, system_provider
├── platform.star, env.star, layout.star, permissions.star
└── runtime.star, github.star, http.star, install.star, semver.star, test.star, ...
# Full stdlib reference: docs/guide/provider-star-reference.md
```

## Command Syntax Guardrails

**Single source of truth**: [`docs/guide/command-syntax-rules.md`](docs/guide/command-syntax-rules.md). Canonical forms:

| Pattern | Example |
|---------|---------|
| `vx <runtime>[@version] [args...]` | `vx node@22 app.js` |
| `vx <runtime>[@version]::<executable> [args...]` | `vx msvc@14.42::cl main.cpp` |
| `vx <ecosystem>:<package>[@version] [args...]` | `vx uvx:pyinstaller --version` |
| `vx --with <runtime> <target_command>` | `vx --with bun@1.1.0 node app.js` |
| `vx shell launch <runtime>[@version] [shell]` | `vx shell launch node@22 powershell` |
| `vx pkg <install\|uninstall\|list\|info\|update> ...` | `vx pkg install npm:typescript` |
| `vx run`, `vx sync`, `vx lock`, `vx check` | Project-aware execution |

## Security Considerations

- Downloads are from official sources (GitHub Releases, official APIs); checksums verified automatically
- `permissions` in `provider.star` declare which network hosts a provider may access
- Never run `sudo vx install` — vx manages user-level installations under `~/.vx/`
- Set `GITHUB_TOKEN` to avoid GitHub API rate limits

## PR and Commit Guidelines

- **Commit messages**: [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`
- **Branch naming**: `<type>/<short-description>` (e.g., `feat/add-zig-provider`)
- **Before PR**: Run `vx just quick` (format → lint → test → build)
- **Pre-commit hooks**: Run `vx prek install` once after cloning
- **PR scope**: One feature or fix per PR; each new provider in its own PR
- **Tests**: New functionality needs tests in `crates/<name>/tests/`
- **Docs**: Update relevant docs when changing user-facing behavior

## Code Style

- **Language**: Rust (edition 2024, MSRV 1.93+)
- **Formatting/Linting**: `vx cargo fmt` before committing; `clippy` with `-D warnings`
- **Imports**: stdlib → external → internal, separated by blank lines
- **Error handling**: `anyhow::Result` for app code, `thiserror` for library error types
- **Async**: Tokio-based async/await for all I/O
- **Logging**: `tracing::info!`, `tracing::debug!` — never `println!` or `eprintln!`
- **Naming**: `PascalCase` types, `snake_case` functions/modules, `SCREAMING_SNAKE_CASE` constants
- **Docs**: All `pub` items need doc comments; every `lib.rs` needs a module-level description
- **File size**: Keep source files under 500 lines

## Testing Conventions

- Tests go in `crates/<name>/tests/` — **never** inline `#[cfg(test)]` modules
- Use `rstest` for parameterized tests; name files `<feature>_tests.rs`
- `vx just test` (all), `vx just test-pkgs "-p vx-starlark"` (single crate)
- E2E tests use `trycmd`; provider static checks: `vx just test-providers-static`
- Mock all network calls in unit tests; keep tests independent (no shared mutable state)

## Allowed vs. Needs-Approval Actions

### ✅ Allowed Without Asking
- Read any file; run `vx just quick/test/lint/format`; `vx cargo check/test -p <crate>`
- Create or modify `crates/vx-providers/` (new providers) or `crates/*/tests/`
- Run `vx <tool> --version`

### ⚠️ Ask First
- Delete files; modify `Cargo.toml` workspace deps; `git push`
- Modify CI workflows; install system packages; run full E2E suite; change layer boundaries

## GitHub Actions Integration

```yaml
- uses: loonghao/vx@main
  with:
    tools: 'node@22 uv'
    setup: 'true'
    cache: 'true'
    github-token: ${{ secrets.GITHUB_TOKEN }}
- run: vx node --version
- run: vx npm test
```

Use `@main` for latest, or pin to a release tag. Full guide: [`docs/guides/github-action.md`](docs/guides/github-action.md).

## AI Agent Ecosystem

vx is designed to be **AI-agent-first**, providing configuration files for 17+ AI coding assistants. Traditional tool management creates friction for AI agents: without vx, an AI must guess if Node.js is installed, check `nvm` vs `fnm` vs system install, and handle version mismatches — all before running a single `npm install`. With vx, the AI just prefixes `vx` → tool auto-installs → command runs. Zero ambiguity, zero setup.

### How AI Agents Should Use vx

1. **Check for `vx.toml`** in the project root → if present, always use `vx` prefix
2. **Never suggest manual installation** of any tool — `vx` handles it
3. **Use structured output** for parsing: `vx list --json` or `vx list --format toon` (saves 40-60% tokens)
4. **For MCP servers**, replace `npx`/`uvx` with `vx` in the config:
   ```json
   {
     "mcpServers": {
       "server": {
         "command": "vx",
         "args": ["npx", "-y", "@example/mcp-server@latest"]
       }
     }
   }
   ```
5. **For CI/CD**, use `loonghao/vx@main` GitHub Action with `cache: 'true'`

## Quick Diagnostics for AI Agents

```bash
vx doctor                      # 1. Check vx health
vx list --installed            # 2. Check installed tools
vx which node && vx node --version  # 3. Verify a specific tool
vx --debug node --version      # 4. Verbose debug output
vx cache clean && vx install node --force  # 5. Clean & retry
vx check --json                # 6. Check project config
```

For a full error-by-error decision tree, see [`docs/appendix/troubleshooting.md`](docs/appendix/troubleshooting.md).

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

```bash
vx list --json             # JSON output
vx list --format toon      # Token-optimized (saves 40-60% tokens)
vx analyze --json          # Project analysis
vx ai context --json       # Full AI context
export VX_OUTPUT=json      # Default all commands to JSON
```

## Documentation Map

```
# Root files
AGENTS.md         # THIS FILE — primary AI agent entry point
CLAUDE.md         # Claude Code instructions
llms.txt          # LLM-friendly project index
llms-full.txt     # Detailed LLM documentation

# Agent config files: .github/copilot-instructions.md, .github/instructions/,
#   .cursorrules, .cursor/rules/*.mdc, .clinerules, .windsurfrules,
#   .kiro/steering/, .trae/rules/

# Documentation site
docs/
├── architecture/     # System architecture (OVERVIEW.md)
├── guide/            # User guides — getting-started, provider-star-reference, etc.
├── cli/              # CLI command reference (17 commands)
├── config/           # vx-toml, env-vars, etc.
├── tools/            # Tool category docs (14 categories)
├── advanced/         # Contributing, security, extension development
├── guides/           # GitHub Actions, use cases
├── rfcs/             # 50 design decision documents
├── appendix/         # FAQ, troubleshooting
└── zh/               # Chinese translations

# AI Agent Skills (vx ai setup)
skills/
├── vx-usage, vx-commands, vx-project, vx-best-practices, vx-troubleshooting
```
