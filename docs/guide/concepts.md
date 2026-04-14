# Core Concepts

Understanding the core concepts behind vx helps you use it effectively and extend it for your needs.

## Architecture Overview

```
┌────────────────────────────────────────────────────┐
│                    vx CLI                           │
│  vx <runtime> [args]  │  vx run <script>           │
└──────────┬─────────────┴───────────────┬────────────┘
           │                             │
     ┌─────▼──────┐              ┌───────▼────────┐
     │  Resolver   │              │ Script Engine  │
     │  (deps +    │              │ (interpolation │
     │   versions) │              │  + .env)       │
     └─────┬──────┘              └───────┬────────┘
           │                             │
     ┌─────▼──────────────────────────────▼──────┐
     │            Provider Registry               │
     │  ┌────────┐ ┌────────┐ ┌────────┐        │
     │  │ Node   │ │ Python │ │  Go    │  ...   │
     │  │Provider│ │Provider│ │Provider│        │
     │  └───┬────┘ └───┬────┘ └───┬────┘        │
     │      │          │          │              │
     │  ┌───▼──┐  ┌────▼───┐ ┌───▼──┐          │
     │  │node  │  │python  │ │ go   │  ...     │
     │  │npm   │  │uv      │ │gofmt │          │
     │  │npx   │  │uvx     │ └──────┘          │
     │  └──────┘  └────────┘                    │
     └──────────────────────┬────────────────────┘
                            │
     ┌──────────────────────▼────────────────────┐
     │          Content-Addressed Store           │
     │  ~/.vx/store/<runtime>/<version>/          │
     └───────────────────────────────────────────┘
```

## Provider

A **Provider** is a module that supplies one or more related runtimes. It is the organizational unit in vx.

```
Provider (e.g., NodeProvider)
├── Runtime: node       (Node.js runtime)
├── Runtime: npm        (Node package manager)
└── Runtime: npx        (Node package executor)
```

Each provider handles:
- **Version discovery** — fetching available versions from upstream
- **Installation** — downloading and extracting binaries
- **Execution** — running commands with the correct environment
- **Platform support** — handling OS/architecture differences

### Built-in Providers

vx ships with **48+ built-in providers** covering major ecosystems:

| Ecosystem | Providers |
|-----------|-----------|
| **Node.js** | node, npm, npx, pnpm, yarn, bun |
| **Python** | python, uv, uvx |
| **Go** | go, gofmt |
| **Rust** | rust (rustc, cargo, rustup) |
| **.NET** | dotnet, msbuild, nuget |
| **DevOps** | terraform, kubectl, helm, podman |
| **Cloud** | awscli, azcli, gcloud |
| **Build** | cmake, ninja, just, task, make, meson, protoc |
| **Media** | ffmpeg, imagemagick |
| **AI** | ollama |
| **Other** | git, jq, deno, zig, java, gh, curl, pwsh... |

### Starlark-Driven Providers

You can define custom providers using `provider.star` (Starlark DSL) without writing Rust code:

```starlark
# ~/.vx/providers/mytool/provider.star
load("@vx//stdlib:provider.star", "runtime_def", "github_permissions")
load("@vx//stdlib:provider_templates.star", "github_rust_provider")

name        = "mytool"
description = "My custom tool"
ecosystem   = "custom"

runtimes    = [runtime_def("mytool")]
permissions = github_permissions()

_p = github_rust_provider("myorg", "mytool",
    asset = "mytool-{vversion}-{triple}.{ext}")

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

See [Provider Development Guide](/guide/provider-star-reference) for details.

## Runtime

A **Runtime** is a single executable tool managed by a provider. Each runtime has:

- **Name** — primary identifier (e.g., `node`, `python`, `go`)
- **Aliases** — alternative names (e.g., `nodejs` → `node`, `golang` → `go`)
- **Ecosystem** — the ecosystem it belongs to (Node.js, Python, Go, etc.)
- **Dependencies** — other runtimes it requires (e.g., `npm` depends on `node`)

### Runtime Dependencies

vx automatically resolves and installs dependencies:

```
npm ──depends on──> node
npx ──depends on──> node
uvx ──depends on──> uv
cargo ──depends on──> rust
gofmt ──depends on──> go
```

When you run `vx npm install`, vx ensures Node.js is installed first.

## Version Resolution

vx supports multiple version specification formats:

| Format | Example | Description |
|--------|---------|-------------|
| Exact | `22.11.0` | Specific version |
| Major | `22` | Latest 22.x.x |
| Minor | `22.11` | Latest 22.11.x |
| Range | `^22.0.0` | Compatible with 22.x.x |
| Range | `~22.11.0` | Compatible with 22.11.x |
| Latest | `latest` | Latest stable release |
| LTS | `lts` | Latest LTS release (Node.js) |
| Channel | `stable` / `beta` / `nightly` | Release channels (Rust) |

### Version Resolution Order

When determining which version to use, vx checks in this order:

1. **Command line** — `vx install node@22`
2. **Environment variable** — `VX_NODE_VERSION=22`
3. **Project config** — `vx.toml` in current or parent directory
4. **Lock file** — `vx.lock` for exact pinned versions
5. **Global config** — `~/.config/vx/config.toml`
6. **Auto-detect** — latest stable version

## Content-Addressed Store

All tools are stored in a global **content-addressed store**:

```
~/.vx/
├── store/                      # Global tool storage
│   ├── node/
│   │   ├── 22.11.0/           # Complete installation
│   │   └── 20.18.0/
│   ├── python/
│   │   └── 3.12.8/
│   └── go/
│       └── 1.23.4/
├── cache/                      # Download cache
│   └── downloads/
├── bin/                        # Global shims
└── config/                     # Configuration
```

### Benefits

- **Deduplicated** — same version stored only once, shared across projects
- **Isolated** — each version in its own directory, no conflicts
- **Fast** — environments created via symlinks, not copies
- **Recoverable** — `vx setup` re-installs from `vx.toml`

## Project Configuration

A `vx.toml` file defines the project's tool requirements:

```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"
just = "latest"

[scripts]
dev = "vx node server.js"
test = "vx uv run pytest"
lint = "vx uvx ruff check ."
build = "vx node scripts/build.js"

[env]
NODE_ENV = "development"
```

See [Configuration](/guide/configuration) for the complete reference.

## Execution Model

When you run `vx <tool> [args...]`:

1. **Tool lookup** — finds the provider that manages the tool
2. **Version resolution** — determines which version to use
3. **Dependency check** — ensures all dependencies are available
4. **Auto-install** — installs missing tools if `auto_install` is enabled
5. **Environment setup** — sets PATH and environment variables
6. **Forward execution** — runs the tool with the original arguments
7. **Exit code passthrough** — returns the tool's exit code

The execution is **transparent** — tools behave exactly as if run directly.

## Ecosystem

An **Ecosystem** groups related tools together:

| Ecosystem | Tools |
|-----------|-------|
| `NodeJs` | node, npm, npx, yarn, pnpm, bun, vite, deno |
| `Python` | python, uv, uvx, pip |
| `Rust` | rust, cargo, rustc, rustup |
| `Go` | go, gofmt |
| `DotNet` | dotnet, msbuild, nuget |
| `System` | git, jq, curl, pwsh |

Ecosystems help vx understand relationships between tools and optimize dependency resolution.

## Next Steps

- [Direct Execution](/guide/direct-execution) — How command forwarding works
- [Version Management](/guide/version-management) — Advanced version control
- [Project Environments](/guide/project-environments) — Team collaboration
- [CLI Reference](/cli/overview) — Complete command documentation
