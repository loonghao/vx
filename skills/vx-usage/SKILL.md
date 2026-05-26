---
name: vx-usage
description: "Teaches AI agents how to use vx, the universal dev tool manager. Use when the project has vx.toml or .vx/, or when the user mentions vx, tool version management, Git/GitHub operations, or cross-platform setup. vx auto-manages Node.js, Python, Go, Rust, and 142 providers via Starlark DSL provider.star files. Also covers MCP integration patterns and GitHub Actions."
---

# VX - Universal Development Tool Manager

> **One-sentence summary**: vx = prefix any dev tool command with `vx` → it auto-installs the tool and runs it.

vx is a universal development tool manager that automatically installs and manages
development tools (Node.js, Python/uv, Go, Rust, etc.) with zero configuration.

## Core Concept

Instead of requiring users to manually install tools, prefix any command with `vx`:

```bash
vx node --version      # Auto-installs Node.js if needed
vx uv pip install x    # Auto-installs uv if needed
vx go build .          # Auto-installs Go if needed
vx cargo build         # Auto-installs Rust if needed
vx just test           # Auto-installs just if needed
```

vx is fully transparent - same commands, same arguments, just add `vx` prefix.

## Essential Commands

### Tool Execution (most common)
```bash
vx <tool> [args...]           # Run any tool (auto-installs if missing)
vx node app.js                # Run Node.js
vx python script.py           # Run Python (via uv)
vx npm install                # Run npm
vx npx create-react-app app   # Run npx
vx cargo test                 # Run cargo
vx just build                 # Run just (task runner)
vx git status                 # Run git
vx gh pr status               # Run GitHub CLI
```

### Git and GitHub for Codex

When Codex or another AI agent works in a vx-managed repository, use vx-managed
Git and GitHub CLI commands. Do not run bare `git` or bare `gh`.

```bash
vx git status --short --branch
vx git fetch origin main
vx git checkout -B fix/example origin/main
vx git diff --stat
vx git add path/to/file
vx git commit -m "fix: example"

vx gh issue view 123
vx gh pr view 456 --json title,state,headRefName
vx gh pr checks 456
vx gh run view 789 --json status,conclusion,jobs
```

### Token-Efficient Agent Workflows

vx is not just an installation wrapper. For agents, it is the stable way to use
fast search, structured GitHub queries, JSON filters, and scoped diffs without
spending tokens on irrelevant output.

Prefer narrow, structured commands before broad dumps:

```bash
# Search and file discovery
vx rg -n --glob '!target/**' --glob '!node_modules/**' "OutputRenderer"
vx rg --files -g '*.rs' -g '!target/**'
vx fd provider.star crates/vx-providers

# Git context with small output first
vx git status --short --branch
vx git diff --stat
vx git diff --name-only origin/main...HEAD
vx git grep -n "CommandOutput" origin/main -- crates/vx-cli

# GitHub context with selected fields
vx gh issue view 123 --json title,state,labels,body
vx gh pr view 456 --json title,state,headRefName,baseRefName,files
vx gh pr checks 456 --json name,state,conclusion,link
vx gh run view 789 --json status,conclusion,jobs
vx gh run view 789 --json jobs --jq '.jobs[] | {name,conclusion,startedAt,completedAt}'
vx gh run view 789 --log | vx rg -n -m 80 "error|failed|panic|Traceback|warning"

# Structured filtering
vx jq -r '.files[].path' pr.json
vx yq '.jobs | keys' .github/workflows/ci.yml
```

Token-saving defaults for agents:
- Start with `vx rg`, `vx fd`, `vx git diff --stat`, and `vx git diff --name-only`; open full files or full diffs only after locating the relevant surface.
- Use `vx gh --json ...` with selected fields, and add `--jq` when a small projection is enough.
- Use vx structured output flags when the vx command supports them: `--json`, `--fields`, `--toon`, `--compact`, or `--output-format toon|compact`.
- For forwarded runtimes like `vx node`, `vx cargo`, or `vx npm`, use that tool's own quiet, JSON, or filtering flags when available.
- Pipe large logs through vx-managed filters such as `vx rg`, `vx jq`, or `vx yq` before reading them.
- Use `vx --compact <tool> ...` only when you still need broad subprocess output after structured fields and grep-style filters are not enough. It preserves vx transparency unless explicitly requested.
- Do not expect default `vx git` or `vx gh` forwarding to shrink output; explicit `--json`, `--jq`, filtering, or `--compact` is what saves tokens.

Compression decision tree for CI/log triage:
1. Status only: `vx gh run view <run> --json status,conclusion,jobs --jq '.jobs[] | {name,conclusion}'`.
2. Suspected failure: `vx gh run view <run> --log | vx rg -n -m 80 "error|failed|panic|Traceback|FAILED|warning"`.
3. Broad but bounded context: `vx --compact gh run view <run> --log`.
4. Last resort: full raw logs, preferably saved to a file and searched locally before being pasted into an agent prompt.

Observed on a successful 5,589-line GitHub Actions run: selected `gh --json --jq`
projection was about 500 tokens, raw `gh --log` output was about 226k tokens,
and `vx --compact gh --log` was about 15.9k tokens. That makes semantic
selection the default, compact mode the fallback for broad context, and raw logs
the exception.

### Agent Operating Principles

vx skills should help agents make small, correct, maintainable changes with
bounded context. Treat vx as a token-aware execution layer, not just a command
prefix.

Use this loop for coding tasks:
1. Inspect the narrowest relevant file, symbol, diff, log, or test output first.
2. Prefer existing project patterns over new helpers or abstractions.
3. Make the smallest maintainable change that solves the actual request.
4. Validate with the cheapest useful scoped command for the risk involved.
5. Summarize only what changed, what was checked, and any remaining risk.

Context discipline:
- Scope before printing. Search paths first, then open focused file sections.
- Avoid dumping full files, broad diffs, generated output, or full CI logs unless the task truly requires them.
- For unknown or potentially huge output, cap and filter with vx-managed tools.
- Do not cap instruction files, skill files, or agent policy files; read the relevant one fully unless it is unexpectedly huge.
- If capped output is insufficient, narrow the query before increasing the cap.

Examples:

```bash
vx rg -n -m 20 "render_token_savings|OutputRenderer" crates/vx-cli crates/vx-metrics
vx git diff --stat origin/main...HEAD
vx git diff --name-only origin/main...HEAD
vx gh run view 789 --json status,conclusion,jobs --jq '.jobs[] | {name,conclusion}'
vx gh run view 789 --log | vx rg -n -m 50 "error|failed|panic|Traceback|FAILED"
vx --compact gh run view 789 --log
vx metrics tokens --last 20 --json
```

Validation discipline:
- Use focused checks first, such as `vx cargo test -p vx-cli --test cli_parsing_tests <case>`.
- Run broader checks only when the touched surface or release risk justifies it.
- Prefer evidence from the actual failing command, CI job, or runtime behavior over speculative fixes.
- Do not add wrappers, maps, helper files, or validation layers unless they clearly reduce real complexity.

### Tool Management
```bash
vx install node@22            # Install specific version
vx install uv go rust         # Install multiple tools at once
vx list                       # List all available tools
vx list --installed           # List installed tools only
vx versions node              # Show available versions
vx switch node@20             # Switch active version
vx uninstall go@1.21          # Remove a version
```

### Project Management
```bash
vx init                       # Initialize vx.toml for project
vx sync                       # Install all tools from vx.toml
vx setup                      # Full project setup (sync + hooks)
vx dev                        # Enter dev environment with all tools
vx run test                   # Run project scripts from vx.toml
vx check                      # Verify tool constraints
vx lock                       # Generate vx.lock for reproducibility
```

### Environment & Config
```bash
vx env list                   # List environments
vx config show                # Show configuration
vx cache info                 # Show cache usage
vx search <query>             # Search available tools
vx info                       # System info and capabilities
```

## Project Configuration (vx.toml)

Projects use `vx.toml` in the root directory:

```toml
[tools]
node = "22"         # Major version
go = "1.22"         # Minor version
uv = "latest"       # Always latest
rust = "1.80"       # Specific version
just = "*"          # Any version

[scripts]
dev = "vx npm run dev"
test = "vx cargo test"
lint = "vx npm run lint && vx cargo clippy"
build = "vx just build"

[hooks]
pre_commit = ["vx run lint"]
post_setup = ["vx npm install"]
```

## Using `--with` for Multi-Runtime

When a command needs additional runtimes available:

```bash
vx --with bun node app.js     # Node.js + Bun in PATH
vx --with deno npm test        # npm + Deno available
```

## Package Aliases

vx supports **package aliases** — short commands that automatically route to ecosystem packages:

```bash
# These are equivalent:
vx vite              # Same as: vx npm:vite
vx vite@5.0          # Same as: vx npm:vite@5.0
vx rez               # Same as: vx uv:rez
vx pre-commit        # Same as: vx uv:pre-commit
vx meson             # Same as: vx uv:meson
vx release-please    # Same as: vx npm:release-please
```

**Benefits**:
- Simpler commands without remembering ecosystem prefixes
- Automatic runtime dependency management (node/python installed as needed)
- Respects project `vx.toml` version configuration

**Available Aliases**:
| Short Command | Equivalent | Ecosystem |
|--------------|------------|-----------|
| `vx vite` | `vx npm:vite` | npm |
| `vx release-please` | `vx npm:release-please` | npm |
| `vx rez` | `vx uv:rez` | uv |
| `vx pre-commit` | `vx uv:pre-commit` | uv |
| `vx meson` | `vx uv:meson` | uv |

## Companion Tool Environment Injection

When `vx.toml` includes tools like MSVC, vx automatically injects discovery environment variables into **all** subprocess environments. This allows any tool needing a C/C++ compiler to discover the vx-managed installation.

```toml
# vx.toml — MSVC env vars injected for ALL tools
[tools]
node = "22"
cmake = "3.28"
rust = "1.82"

[tools.msvc]
version = "14.42"
os = ["windows"]
```

Now tools like node-gyp, CMake, Cargo (cc crate) automatically find MSVC:

```bash
# node-gyp finds MSVC via VCINSTALLDIR
vx npx node-gyp rebuild

# CMake discovers the compiler
vx cmake -B build -G "Ninja"

# Cargo cc crate finds MSVC for C dependencies
vx cargo build
```

**Injected Environment Variables** (MSVC example):
| Variable | Purpose |
|----------|---------|
| `VCINSTALLDIR` | VS install path (node-gyp, CMake) |
| `VCToolsInstallDir` | Exact toolchain path |
| `VX_MSVC_ROOT` | vx MSVC root path |

## MSVC Build Tools (Windows)

Microsoft Visual C++ compiler for Windows development:

```bash
# Install MSVC Build Tools
vx install msvc@latest
vx install msvc 14.40       # Specific version

# Using MSVC tools via namespace
vx msvc cl main.cpp -o main.exe
vx msvc link main.obj
vx msvc nmake

# Direct aliases
vx cl main.cpp              # Same as: vx msvc cl
vx nmake                    # Same as: vx msvc nmake

# Version-specific usage
vx msvc@14.40 cl main.cpp
```

**Available MSVC Tools**:
| Tool | Command | Description |
|------|---------|-------------|
| cl | `vx msvc cl` | C/C++ compiler |
| link | `vx msvc link` | Linker |
| lib | `vx msvc lib` | Library manager |
| nmake | `vx msvc nmake` | Make utility |

## Supported Tools (142 Providers)

| Category | Tools |
|----------|-------|
| **JavaScript** | node, npm, npx, bun, deno, pnpm, yarn, vite, nx, turbo |
| **JS Tooling** | oxlint, biome |
| **Python** | uv, uvx, python, pip, ruff, maturin, pre-commit |
| **Rust** | cargo, rustc, rustup |
| **Go** | go, gofmt, gws, goreleaser, golangci-lint |
| **System/CLI** | git, bash, curl, pwsh, jq, yq, fd, bat, ripgrep, fzf, starship, jj, sd, eza, dust, duf, xh, atuin, zoxide, tealdeer, gping, delta, hyperfine, watchexec, bottom |
| **TUI/Terminal** | helix, yazi, zellij, lazygit, lazydocker, k9s |
| **Build Tools** | just, task, cmake, ninja, make, meson, xmake, protoc, buf, conan, vcpkg, spack |
| **DevOps** | kubectl, helm, flux, kind, k3d, nerdctl, skaffold, podman, terraform, hadolint, dagu, actionlint |
| **Security** | gitleaks, trivy, cosign, grype, syft |
| **Cloud CLI** | awscli, azcli, gcloud |
| **.NET** | dotnet, msbuild, nuget |
| **C/C++** | msvc, llvm, nasm, ccache, buildcache, sccache, rcedit |
| **Media** | ffmpeg, imagemagick |
| **Java** | java |
| **AI** | ollama, openclaw, mcpcall |
| **Other Langs** | zig |
| **Container** | dive |
| **Config Mgmt** | chezmoi, mise |
| **Package Managers** | brew, choco, winget |
| **Data/API** | duckdb, grpcurl |
| **Misc** | gh, prek, actrun, wix, vscode, xcodebuild, systemctl, release-please, rez, 7zip, trippy |

## Provider System (Starlark DSL)

All 142 providers are defined using **provider.star** (Starlark DSL) — a declarative, zero-compilation approach. Each provider lives in `crates/vx-providers/<name>/provider.star`.

vx uses a **two-phase execution model** (inspired by Buck2):
1. **Analysis Phase (Starlark)**: `provider.star` runs as pure computation, returning descriptor dicts. No I/O.
2. **Execution Phase (Rust)**: The Rust runtime interprets descriptors for actual downloads, installs, and process execution.

### How to add a new tool

```starlark
# crates/vx-providers/mytool/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "mytool"
description = "My awesome tool"
ecosystem   = "custom"

runtimes = [runtime_def("mytool", aliases=["mt"])]
permissions = github_permissions()

# Use a template — covers 90% of tools
_p = github_rust_provider("owner", "mytool",
    asset = "mytool-{vversion}-{triple}.{ext}")
fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

### Available templates

| Template | Use case | Example |
|----------|----------|---------|
| `github_rust_provider` | Rust tools on GitHub (most common) | ripgrep, fd, bat, just, uv |
| `github_go_provider` | Go tools on GitHub (goreleaser style) | gh, task |
| `github_binary_provider` | Single binary download (no archive) | kubectl |
| `system_provider` | System package manager only | 7zip |

### Template Placeholders

| Placeholder | Rust template | Go template | Description |
|-------------|---------------|-------------|-------------|
| `{version}` | ✓ | ✓ | Version number (e.g., "1.0.0") |
| `{vversion}` | ✓ | — | With v-prefix (e.g., "v1.0.0") |
| `{triple}` | ✓ | — | Rust target triple (e.g., "x86_64-unknown-linux-musl") |
| `{os}` | — | ✓ | Go GOOS (linux, darwin, windows) |
| `{arch}` | — | ✓ | Go GOARCH (amd64, arm64) |
| `{ext}` | ✓ | ✓ | Archive extension (zip/tar.gz) |
| `{exe}` | ✓ | ✓ | Executable suffix (.exe/"") |

### Starlark Standard Library (14 modules)

Located in `crates/vx-starlark/stdlib/`. The main entry point is `provider.star` which re-exports everything:

```starlark
# Import everything from the unified facade
load("@vx//stdlib:provider.star",
     "runtime_def", "bundled_runtime_def", "dep_def",
     "github_permissions", "platform_map",
     "env_set", "env_prepend",
     "archive_layout", "binary_layout")
```

Key modules:
- `provider.star` — Unified facade (re-exports all)
- `runtime.star` — `runtime_def`, `bundled_runtime_def`, `dep_def`
- `platform.star` — `platform_map`, `platform_select`, `rust_triple`, `go_os_arch`
- `env.star` — `env_set`, `env_prepend`, `env_append`, `env_unset`
- `layout.star` — `archive_layout`, `binary_layout`, `bin_subdir_layout`, hooks
- `provider_templates.star` — High-level templates (4 templates)
- `permissions.star` — `github_permissions`, `system_permissions`
- `system_install.star` — `winget_install`, `brew_install`, `apt_install`

## Important Rules for AI Agents

1. **Always use `vx` prefix** when running tools in vx-managed projects
2. **Use `vx just`** instead of `just` for task runner commands
3. **Use `vx npm`** instead of `npm` directly
4. **Use `vx cargo`** instead of `cargo` directly
5. **Use `vx git`** instead of `git` for repository operations
6. **Use `vx gh`** instead of `gh` for GitHub issue, PR, checks, and workflow operations
7. **Use `vx rg`, `vx fd`, `vx jq`, and `vx yq`** for scoped search and structured filtering
8. **Prefer token-efficient output**: `--json`, selected fields, `--jq`, `--toon`, `--compact`, `--output-format toon|compact`, and narrow globs
9. **Check `vx.toml`** first to understand project tool requirements
10. **Use `vx run <script>`** for project-defined scripts
11. **Never suggest manual tool installation** - vx handles it automatically
12. Commands like `vx install` are for explicit pre-installation; normal usage auto-installs
13. **Use correct terminology**: Runtime (not Tool), Provider (not Plugin), provider.star (not provider config)
14. **Provider development**: New tools are added via `provider.star` Starlark DSL in `crates/vx-providers/<name>/`
15. **Tests go in `tests/` dirs** — never inline `#[cfg(test)]` in source files

## Version Resolution Priority

vx resolves tool versions in this order (highest to lowest):

1. **Command-line override**: `vx node@22 app.js`
2. **Project vx.toml**: `[tools] node = "22"`
3. **Parent directory vx.toml** (traverses up to root)
4. **User global config**: `~/.config/vx/config.toml`
5. **Provider default**: latest stable version

## MCP Integration

vx is **MCP-ready** — replace `npx`/`uvx` with `vx` in MCP server configurations.
This eliminates the "install Node.js/Python first" requirement for all MCP servers.

### Configuration Pattern

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

### Real-World MCP Examples

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-filesystem", "/path/to/dir"]
    },
    "github": {
      "command": "vx",
      "args": ["npx", "-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "<token>" }
    },
    "sqlite": {
      "command": "vx",
      "args": ["uvx", "mcp-server-sqlite", "--db-path", "/path/to/db.sqlite"]
    }
  }
}
```

### Testing MCP Servers with mcpcall

Use the built-in `mcpcall` provider for scriptable MCP smoke tests. Prefer
compact vx output plus mcpcall JSON output when agents need concise logs:

```bash
vx install mcpcall@0.4.0
vx --compact mcpcall list --url http://127.0.0.1:8765/mcp --json
vx --compact mcpcall doctor --url http://127.0.0.1:8765/mcp --json
vx --compact mcpcall call --url http://127.0.0.1:8765/mcp dcc_status --json
```

### Migration Pattern

| Original | vx-powered |
|----------|------------|
| `"command": "npx"` | `"command": "vx", "args": ["npx", ...]` |
| `"command": "uvx"` | `"command": "vx", "args": ["uvx", ...]` |
| `"command": "node"` | `"command": "vx", "args": ["node", ...]` |
| `"command": "python"` | `"command": "vx", "args": ["python", ...]` |
| `"command": "bun"` | `"command": "vx", "args": ["bun", ...]` |

### Benefits for AI Agents

- **Zero-config**: No need to check if Node.js/Python is installed before starting MCP servers
- **Version consistency**: MCP servers always use the version specified in `vx.toml`
- **Cross-platform**: Same MCP config works on Windows, macOS, and Linux
- **CI/CD ready**: MCP servers in CI pipelines just work with vx

## GitHub Actions Integration

vx provides a GitHub Action (`action.yml`) for CI/CD workflows. Use it in `.github/workflows/` files:

### Basic Usage

```yaml
- uses: loonghao/vx@main
  with:
    version: 'latest'           # vx version (default: latest)
    github-token: ${{ secrets.GITHUB_TOKEN }}
```

### Pre-install Tools

```yaml
- uses: loonghao/vx@main
  with:
    tools: 'node go uv'         # Space-separated tools to pre-install
    cache: 'true'               # Enable tool caching (default: true)
```

### Project Setup (vx.toml)

```yaml
- uses: loonghao/vx@main
  with:
    setup: 'true'               # Run `vx setup --ci` for vx.toml projects
```

### Full Example

```yaml
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          tools: 'node@22 uv'
          setup: 'true'
          cache: 'true'

      - run: vx node --version
      - run: vx npm test
```

### Action Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `version` | `latest` | vx version to install |
| `github-token` | `${{ github.token }}` | GitHub token for API requests |
| `tools` | `''` | Space-separated tools to pre-install |
| `cache` | `true` | Enable caching of ~/.vx directory |
| `cache-key-prefix` | `vx-tools` | Custom prefix for cache key |
| `setup` | `false` | Run `vx setup --ci` for vx.toml projects |

### Action Outputs

| Output | Description |
|--------|-------------|
| `version` | The installed vx version |
| `cache-hit` | Whether the cache was hit |

## Container Image Support

vx also provides a container image for containerized workflows, and it can be consumed from Podman-compatible environments:


```dockerfile
# Use vx as base image
FROM ghcr.io/loonghao/vx:latest

# Tools are auto-installed on first use
RUN vx node --version
RUN vx uv pip install mypackage
```

### Multi-stage Build with vx

```dockerfile
FROM ghcr.io/loonghao/vx:latest AS builder
RUN vx node --version && vx npm ci && vx npm run build

FROM nginx:alpine
COPY --from=builder /home/vx/dist /usr/share/nginx/html
```

### GitHub Actions Container Jobs

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/loonghao/vx:latest
    steps:
      - uses: actions/checkout@v6
      - run: vx node --version
      - run: vx npm test
```
