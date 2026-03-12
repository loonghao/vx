# 🚀 vx - Universal Development Tool Manager

<div align="center">

**One command to rule them all — Zero setup, Zero learning curve**

*Built for the AI-native era: Unix Philosophy meets Scriptability*

[中文文档](README_zh.md) | [📖 Documentation](https://docs.rs/vx) | [🚀 Quick Start](#-quick-start) | [💡 Examples](#-real-world-examples)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.80+-blue.svg)](https://www.rust-lang.org)
[![Test](https://github.com/loonghao/vx/workflows/Test/badge.svg)](https://github.com/loonghao/vx/actions)
[![Release](https://github.com/loonghao/vx/workflows/Release/badge.svg)](https://github.com/loonghao/vx/actions)
[![codecov](https://codecov.io/gh/loonghao/vx/branch/main/graph/badge.svg)](https://codecov.io/gh/loonghao/vx)
[![GitHub release](https://img.shields.io/github/release/loonghao/vx.svg)](https://github.com/loonghao/vx/releases)
[![GitHub downloads](https://img.shields.io/github/downloads/loonghao/vx/total.svg)](https://github.com/loonghao/vx/releases)

</div>

---

## 🤖 Built for AI-Native Development

> *"Claude Code is designed as a low-level, unopinionated tool... creating a flexible, customizable, scriptable, and safe power tool."*
> — [Anthropic Engineering: Claude Code Best Practices](https://www.anthropic.com/engineering/claude-code-best-practices)

vx follows the same **Unix Philosophy** and **Scriptability** principles that Anthropic recommends for AI-native development tools:

| Principle | How vx Implements It |
|-----------|---------------------|
| **Unix Philosophy** | One tool, one job — `vx` manages all runtimes transparently |
| **Scriptability** | Full bash integration, CI/CD ready, headless mode support |
| **Composability** | Works with any AI coding assistant (Claude Code, Cursor, Copilot) |
| **Zero Configuration** | AI agents can use any tool without environment setup |

### Why This Matters for AI Coding Assistants

When AI agents like Claude Code need to execute commands across different ecosystems:

```bash
# Without vx: AI must handle complex environment setup
# "First install Node.js, then configure npm, set PATH..."

# With vx: AI just runs commands directly
vx npx create-react-app my-app  # Works immediately
vx uvx ruff check .             # Works immediately  
vx cargo build --release        # Works immediately
```

**vx enables AI to have full-stack development capabilities without worrying about environment management and dependencies.**

---

## 💡 Design Philosophy

### The Problem We Solve

Every time we start a new development project, we face the same frustrating cycle:

- Install Node.js and npm for frontend tools
- Set up Python and pip/uv for scripts and automation
- Configure Go for backend services
- Manage Rust toolchain for system tools
- Deal with version conflicts and PATH issues
- Repeat this process across different machines and environments

**With the rise of MCP (Model Context Protocol)**, this problem has become even more pronounced. Many MCP servers require `uvx` for Python tools and `npx` for Node.js packages, forcing developers to manage multiple tool ecosystems just to get AI assistance working.

### Our Solution: Zero Learning Curve

vx eliminates this complexity while maintaining **zero learning curve**:

```bash
# Instead of learning and managing multiple tools:
npx create-react-app my-app     # Requires Node.js setup
uvx ruff check .                # Requires Python/UV setup
go run main.go                  # Requires Go installation

# Just use vx with the same commands you already know:
vx npx create-react-app my-app  # Auto-installs Node.js if needed
vx uvx ruff check .             # Auto-installs UV if needed
vx go run main.go               # Auto-installs Go if needed
```

---

## 🚀 Quick Start

### Installation

**Linux/macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows (PowerShell):**

```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

### Stable Installation in Rate-Limited Networks

```bash
# 1) Pin a stable installer version (recommended for CI and enterprise networks)
VX_VERSION="0.8.4" curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# 2) Configure multi-source release mirrors (comma separated)
VX_RELEASE_BASE_URLS="https://mirror.example.com/vx/releases,https://github.com/loonghao/vx/releases" \
  curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell
# Windows mirror fallback (comma/semicolon separated)
$env:VX_RELEASE_BASE_URLS="https://mirror.example.com/vx/releases,https://github.com/loonghao/vx/releases"
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

> The installer will try all configured release base URLs automatically, then fallback across different asset naming patterns.

### Start Using Immediately


```bash
# No setup needed - just prefix your commands with 'vx'
vx node --version               # Auto-installs Node.js
vx python --version             # Auto-installs Python via UV
vx go version                   # Auto-installs Go
vx cargo --version              # Auto-installs Rust
```

---

## 🎯 Two Ways to Use vx

### 1️⃣ Direct Execution (For Quick Tasks)

Just prefix any command with `vx` — tools are auto-installed on first use:

```bash
# Run any tool instantly
vx npx create-react-app my-app
vx uvx ruff check .
vx go run main.go
vx cargo build --release
```

### 2️⃣ Project Development Environment (For Teams)

Create a `vx.toml` file to define your project's tool requirements:

```bash
# Initialize a new project
vx init

# Or create vx.toml manually
cat > vx.toml << 'EOF'
[tools]
node = "20"
python = "3.12"
uv = "latest"
go = "1.21"

[scripts]
dev = "npm run dev"
test = "npm test"
lint = "uvx ruff check ."
EOF
```

Then use the development environment commands:

```bash
# One-click setup: install all project tools
vx setup

# Enter development shell with all tools available
vx dev

# Run project scripts
vx run dev
vx run test
vx run lint

# Manage project tools
vx add bun                      # Add a tool
vx remove go                    # Remove a tool
vx sync                         # Sync tools with vx.toml
```

---

## 📋 Command Reference

### Tool Execution

| Command | Description |
|---------|-------------|
| `vx <runtime>[@version] [args...]` | Execute a runtime (auto-installs if needed) |
| `vx <runtime>[@version]::<executable> [args...]` | Execute specific executable from a runtime |
| `vx <ecosystem>:<package>[::executable] [args...]` | Execute a package (RFC 0027) |
| `vx --with <runtime>[@version] <command>` | Inject companion runtimes for this invocation |
| `vx install <runtime>@<version>` | Install a specific runtime version |
| `vx uninstall <runtime>[@version]` | Uninstall runtime versions |
| `vx switch <runtime>@<version>` | Switch to a different version |
| `vx which <runtime>` | Show which version is being used |
| `vx versions <runtime>` | Show available versions |
| `vx list` | List all supported runtimes |
| `vx search <query>` | Search available runtimes |

### Shell & Environment

| Command | Description |
|---------|-------------|
| `vx shell launch <runtime>[@version] [shell]` | Launch shell with runtime environment (canonical) |
| `vx dev` | Enter development shell with project tools |
| `vx dev -c <cmd>` | Run a command in the dev environment |

### Global Package Management (`vx pkg`)

| Command | Description |
|---------|-------------|
| `vx pkg install <ecosystem>:<package>` | Install a global package |
| `vx pkg uninstall <ecosystem>:<package>` | Uninstall a global package |
| `vx pkg list` | List globally installed packages |
| `vx pkg info <ecosystem>:<package>` | Show package information |

### Project Management

| Command | Description |
|---------|-------------|
| `vx init` | Initialize project configuration (`vx.toml`) |
| `vx setup` | Install all tools defined in `vx.toml` |
| `vx sync` | Sync installed tools with `vx.toml` |
| `vx lock` | Generate or update `vx.lock` for reproducibility |
| `vx check` | Check version constraints and tool availability |
| `vx add <runtime>` | Add a runtime to project configuration |
| `vx remove <runtime>` | Remove a runtime from project configuration |
| `vx run <script>` | Run a script defined in `vx.toml` |

### System Management

| Command | Description |
|---------|-------------|
| `vx cache info` | Show disk usage and cache statistics |
| `vx cache prune` | Clean up cache and orphaned packages |
| `vx config` | Manage global configuration |
| `vx self-update` | Update vx itself |
| `vx provider list` | List available providers |

---

## 📁 Project Configuration (`vx.toml`)

```toml
# VX Project Configuration
# Run 'vx setup' to install all tools
# Run 'vx dev' to enter the development environment

[tools]
node = "20"                     # Major version
python = "3.12"                 # Minor version
uv = "latest"                   # Always latest
go = "1.21.6"                   # Exact version
rustup = "latest"               # Rust toolchain manager

[settings]
auto_install = true             # Auto-install missing tools in dev shell
parallel_install = true         # Install tools in parallel

[env]
NODE_ENV = "development"
DEBUG = "true"

[scripts]
dev = "npm run dev"
test = "npm test && cargo test"
build = "npm run build"
lint = "uvx ruff check . && npm run lint"
format = "uvx black . && npm run format"
# Enhanced: Use {{args}} for complex tool arguments
test-pkgs = "cargo test {{args}}"
lint-fix = "eslint {{args}}"
```

### 🚀 Enhanced Script System

vx now supports **advanced argument passing** for complex tool workflows:

```bash
# Pass complex arguments directly to tools
vx run test-pkgs -p vx-runtime --lib
vx run lint-fix --fix --ext .js,.ts src/

# Get script-specific help
vx run test-pkgs -H

# List all available scripts
vx run --list
```

**Key Features:**
- ✅ **Zero conflicts**: Pass `-p`, `--lib`, `--fix` directly to scripts
- ✅ **Script help**: Use `-H` for script-specific documentation
- ✅ **Flexible arguments**: Use `{{args}}` in script definitions for maximum flexibility
- ✅ **Backward compatible**: Existing scripts continue to work

---

## 🔌 MCP Integration

vx was designed with MCP (Model Context Protocol) in mind. Just change the command from the tool name to `vx`:

### Before (Complex Setup Required)

```json
{
  "mcpServers": {
    "browsermcp": {
      "command": "npx",
      "args": ["-y", "@browsermcp/mcp@latest"]
    },
    "python-tool": {
      "command": "uvx",
      "args": ["some-python-tool@latest"]
    }
  }
}
```

### After (Zero Setup with vx)

```json
{
  "mcpServers": {
    "browsermcp": {
      "command": "vx",
      "args": ["npx", "-y", "@browsermcp/mcp@latest"]
    },
    "python-tool": {
      "command": "vx",
      "args": ["uvx", "some-python-tool@latest"]
    }
  }
}
```

---

## 🎯 Real-World Examples

### Team Onboarding

```bash
# New team member joins the project
git clone https://github.com/your-org/your-project
cd your-project

# One command to set up everything
vx setup

# Start developing
vx dev
```

### Multi-Language Project

```bash
# Frontend (Node.js) + Backend (Go) + Scripts (Python)
cat > vx.toml << 'EOF'
[tools]
node = "20"
go = "1.21"
uv = "latest"

[scripts]
frontend = "npm run dev"
backend = "go run cmd/server/main.go"
migrate = "uvx alembic upgrade head"
EOF

# Install everything
vx setup

# Run different parts
vx run frontend
vx run backend
vx run migrate
```

### Python Development

```bash
vx uv init my-python-app
cd my-python-app
vx uv add fastapi uvicorn
vx uv add --dev pytest black ruff
vx uv run uvicorn main:app --reload
vx uvx ruff check .
```

### Node.js Development

```bash
vx npx create-react-app my-app
cd my-app
vx npm install
vx npm run dev
```

### Go Development

```bash
vx go mod init my-go-app
vx go run main.go
vx go build -o app
```

### Rust Development

```bash
vx cargo new my-rust-app
cd my-rust-app
vx cargo add serde tokio
vx cargo run
```

---

## 📖 Supported Tools

### Language Runtimes

| Tool | Commands | Description |
|------|----------|-------------|
| **Node.js** | `node`, `npm`, `npx` | JavaScript runtime and package manager |
| **Bun** | `bun`, `bunx` | Fast all-in-one JavaScript runtime |
| **Deno** | `deno` | Secure JavaScript/TypeScript runtime |
| **Go** | `go` | Go programming language |
| **Rust** | `cargo`, `rustc`, `rustup` | Rust toolchain |
| **Java** | `java`, `javac` | Java Development Kit |
| **Zig** | `zig` | Zig programming language |

### Package Managers

| Tool | Commands | Description |
|------|----------|-------------|
| **UV** | `uv`, `uvx` | Fast Python package manager |
| **pnpm** | `pnpm`, `pnpx` | Fast, disk-efficient package manager |
| **Yarn** | `yarn` | JavaScript package manager |

### Build Tools

| Tool | Commands | Description |
|------|----------|-------------|
| **Vite** | `vite` | Next generation frontend tooling |
| **Just** | `just` | Command runner for project tasks |
| **Task** | `task` | Task runner / build tool (go-task) |
| **CMake** | `cmake` | Cross-platform build system generator |
| **Ninja** | `ninja` | Small build system focused on speed |
| **protoc** | `protoc` | Protocol Buffers compiler |

### DevOps Tools

| Tool | Commands | Description |
|------|----------|-------------|
| **Docker** | `docker` | Container runtime and tooling |
| **Terraform** | `terraform` | Infrastructure as Code |
| **kubectl** | `kubectl` | Kubernetes CLI |
| **Helm** | `helm` | Kubernetes package manager |

### Cloud CLI Tools

| Tool | Commands | Description |
|------|----------|-------------|
| **AWS CLI** | `aws` | Amazon Web Services CLI |
| **Azure CLI** | `az` | Microsoft Azure CLI |
| **gcloud** | `gcloud` | Google Cloud Platform CLI |

### Code Quality Tools

| Tool | Commands | Description |
|------|----------|-------------|
| **pre-commit** | `pre-commit` | Pre-commit hook framework |

### Other Tools

| Tool | Commands | Description |
|------|----------|-------------|
| **VS Code** | `code` | Visual Studio Code editor |
| **Rez** | `rez` | Package management system |
| **rcedit** | `rcedit` | Windows resource editor |

---

## 🌟 Why vx?

| Feature | vx | nvm/pyenv/etc. |
|---------|-----|----------------|
| **Zero Learning Curve** | ✅ Same commands you know | ❌ New commands to learn |
| **Multi-Language** | ✅ One tool for all | ❌ One tool per language |
| **Auto-Install** | ✅ On first use | ❌ Manual installation |
| **Project Config** | ✅ `vx.toml` | ❌ Varies by tool |
| **Team Sync** | ✅ `vx setup` | ❌ Manual coordination |
| **MCP Ready** | ✅ Just add `vx` | ❌ Complex setup |
| **Cross-Platform** | ✅ Windows/macOS/Linux | ⚠️ Varies |

---

## ⚙️ Advanced Configuration

### Global Configuration

`~/.config/vx/config.toml`:

```toml
[defaults]
auto_install = true
check_updates = true
update_interval = "24h"

[tools.node]
version = "20"

[tools.uv]
version = "latest"
```

### Shell Integration

```bash
# Add to your shell profile for auto-completion
eval "$(vx shell init bash)"   # Bash
eval "$(vx shell init zsh)"    # Zsh
vx shell init fish | source    # Fish
```

### Self-Update with GitHub Token

```bash
# Avoid rate limits in shared environments
vx self-update --token ghp_your_token_here

# Or set environment variable
export GITHUB_TOKEN=ghp_your_token_here
vx self-update
```

---

## 📦 Installation Options

### Package Managers

```bash
# Windows
winget install loonghao.vx
choco install vx
scoop install vx

# macOS
brew tap loonghao/vx && brew install vx

# Arch Linux
yay -S vx-bin

# Cargo
cargo install --git https://github.com/loonghao/vx
```

### Docker

```bash
docker pull loonghao/vx:latest
docker run --rm loonghao/vx --version
```

### GitHub Actions

Use vx in your CI/CD workflows:

```yaml
- uses: loonghao/vx@vx-v0.5.15
  with:
    github-token: ${{ secrets.GITHUB_TOKEN }}

- run: vx node --version
- run: vx npm ci
- run: vx npm test
```

> **Note**: Use a specific version tag (e.g., `vx-v0.5.15`) instead of `v1`. Check [releases](https://github.com/loonghao/vx/releases) for the latest version.

See [GitHub Action Guide](docs/guides/github-action.md) for full documentation.

---

## 🧪 Testing

vx includes a comprehensive test suite for all providers:

```bash
# Test all providers in a clean temporary environment
just test-providers

# Test with verbose output
just test-providers-verbose

# Test specific providers only
just test-providers-filter "node"

# Keep cache for inspection
just test-providers-keep
```

The test suite:
- ✅ Uses temporary VX_HOME (auto-cleaned after tests)
- ✅ Auto-discovers all providers from source
- ✅ Tests command execution and auto-installation
- ✅ Generates detailed test reports
- ✅ CI/CD ready with exit codes and JSON output

See [scripts/README.md](scripts/README.md) for detailed documentation.

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

1. **Report Issues**: [Open an issue](https://github.com/loonghao/vx/issues)
2. **Feature Requests**: [Start a discussion](https://github.com/loonghao/vx/discussions)
3. **Code Contributions**: Submit pull requests

---

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

## 📞 Support

- 📖 **Documentation**: [GitHub Wiki](https://github.com/loonghao/vx/wiki)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/loonghao/vx/discussions)
- 🐛 **Issues**: [Bug Reports](https://github.com/loonghao/vx/issues)
- 📧 **Contact**: <hal.long@outlook.com>

---

<div align="center">

**Made with ❤️ for developers, by developers**

</div>
