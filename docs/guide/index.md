# Introduction

## What is vx?

**vx** is a universal development tool manager that provides a **zero learning curve** experience for managing programming language runtimes, package managers, and development tools across all platforms.

Instead of learning and configuring multiple tool installers — `nvm` for Node.js, `pyenv` for Python, `rustup` for Rust, `gvm` for Go — you simply prefix any command with `vx` and everything just works.

```bash
# Just prefix any command with vx — tools are auto-installed
vx node --version        # Auto-installs Node.js if needed
vx python --version      # Auto-installs Python if needed
vx go version            # Auto-installs Go if needed
vx cargo build           # Auto-installs Rust if needed
```

## Why vx?

### The Problem

Modern software development requires a complex toolkit:

- **Multiple language runtimes** — Node.js, Python, Go, Rust, .NET, Java, Zig, etc.
- **Package managers** — npm, pnpm, yarn, uv, pip, cargo, etc.
- **DevOps tools** — Terraform, kubectl, Helm, Docker CLI, etc.
- **Build tools** — CMake, Ninja, Just, Task, protoc, etc.
- **Cloud CLIs** — AWS CLI, Azure CLI, Google Cloud CLI, etc.

Each tool has its own installer, version manager, and configuration. Teams waste hours debugging "works on my machine" issues.

### The Solution

vx provides **one tool to manage them all**:

| Feature | vx | Traditional Approach |
|---------|-----|---------------------|
| Install tools | `vx install node@22` | Download installer, configure PATH |
| Use tools | `vx node index.js` | Hope the right version is active |
| Switch versions | `vx switch node 20` | `nvm use 20` / `fnm use 20` / edit `.nvmrc` |
| Team consistency | `vx.toml` in repo | READMEs, wikis, tribal knowledge |
| CI/CD | `uses: loonghao/vx@main` | Multiple setup-* actions |

## Key Features

### Zero Learning Curve
Use commands you already know — just add `vx` in front:
```bash
vx npm install           # Same as npm install, but version-managed
vx uvx ruff check .      # Same as uvx ruff check, but auto-installed
vx go build ./...        # Same as go build, but portable
```

### 50+ Tools Supported
From language runtimes to DevOps tools, vx manages them all with a single interface. See the [full tool list](/tools/overview).

### Declarative Configuration
Define your project's toolchain in `vx.toml`:
```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"
just = "latest"
```

### Automatic Dependency Resolution
vx understands tool dependencies and installs them automatically:
```bash
vx npm --version         # Automatically installs Node.js first
vx cargo build           # Automatically installs Rust first
vx uvx ruff check .      # Automatically installs uv first
```

### Enhanced Script System
Define and run project scripts with powerful variable interpolation:
```toml
[scripts]
dev = "vx node server.js --port {{PORT}}"
test = "vx uv run pytest {{args}}"
build = "vx cargo build --release"
lint = "vx uvx ruff check . {{args}}"
```

### Cross-Platform
Works on Windows, macOS, and Linux with consistent behavior.

### Extensible
Create custom providers via TOML manifests or Rust plugins, and extend functionality with the extension system.

## How It Works

```
┌─────────────┐    ┌─────────────┐    ┌──────────────┐
│  vx node    │───>│  Resolver   │───>│  Provider    │
│  --version  │    │  (find tool │    │  (install &  │
│             │    │   & deps)   │    │   execute)   │
└─────────────┘    └─────────────┘    └──────────────┘
                         │                    │
                   ┌─────▼─────┐    ┌────────▼────────┐
                   │ Version   │    │ Content-Addressed│
                   │ Resolution│    │ Store (~/.vx/)   │
                   └───────────┘    └─────────────────┘
```

1. **Parse** — vx identifies the runtime and command
2. **Resolve** — Finds the required version and checks dependencies
3. **Install** — Downloads and installs missing tools (if needed)
4. **Execute** — Forwards the command transparently

## Quick Example

```bash
# Install vx
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Use any tool immediately — no manual setup
vx node --version        # v22.x.x
vx python --version      # Python 3.12.x
vx go version            # go1.23.x

# Set up a project
cd my-project
vx config init           # Creates vx.toml
vx setup                 # Installs all project tools

# Run project scripts
vx run dev               # Starts dev server
vx run test              # Runs tests
```

## Next Steps

- [Installation](/guide/installation) — Install vx on your system
- [Quick Start](/guide/getting-started) — Get up and running in 5 minutes
- [Core Concepts](/guide/concepts) — Understand how vx works
- [CLI Reference](/cli/overview) — Complete command documentation
