# 🚀 vx - Universal Development Tool Manager

<div align="center">

**One command to rule them all — Zero setup, Zero learning curve**

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

Create a `.vx.toml` file to define your project's tool requirements:

```bash
# Initialize a new project
vx init

# Or create .vx.toml manually
cat > .vx.toml << 'EOF'
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
vx rm-tool go                   # Remove a tool
vx sync                         # Sync tools with .vx.toml
```

---

## 📋 Command Reference

### Tool Execution

| Command | Description |
|---------|-------------|
| `vx <tool> [args...]` | Execute a tool (auto-installs if needed) |
| `vx install <tool>[@version]` | Install a specific tool version |
| `vx uninstall <tool> [version]` | Uninstall tool versions |
| `vx switch <tool>@<version>` | Switch to a different version |
| `vx which <tool>` | Show which version is being used |
| `vx versions <tool>` | Show available versions |
| `vx list` | List all supported tools |
| `vx search <query>` | Search available tools |

### Project Environment

| Command | Description |
|---------|-------------|
| `vx init` | Initialize project configuration (`.vx.toml`) |
| `vx setup` | Install all tools defined in `.vx.toml` |
| `vx dev` | Enter development shell with project tools |
| `vx dev -c <cmd>` | Run a command in the dev environment |
| `vx sync` | Sync installed tools with `.vx.toml` |
| `vx add <tool>` | Add a tool to project configuration |
| `vx rm-tool <tool>` | Remove a tool from project configuration |
| `vx run <script>` | Run a script defined in `.vx.toml` |

### System Management

| Command | Description |
|---------|-------------|
| `vx stats` | Show disk usage and statistics |
| `vx clean` | Clean up cache and orphaned packages |
| `vx config` | Manage global configuration |
| `vx self-update` | Update vx itself |
| `vx plugin list` | List available plugins |

---

## 📁 Project Configuration (`.vx.toml`)

```toml
# VX Project Configuration
# Run 'vx setup' to install all tools
# Run 'vx dev' to enter the development environment

[tools]
node = "20"                     # Major version
python = "3.12"                 # Minor version
uv = "latest"                   # Always latest
go = "1.21.6"                   # Exact version
rust = ">=1.70"                 # Version range

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
```

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
cat > .vx.toml << 'EOF'
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

### DevOps Tools

| Tool | Commands | Description |
|------|----------|-------------|
| **Terraform** | `terraform` | Infrastructure as Code |
| **kubectl** | `kubectl` | Kubernetes CLI |
| **Helm** | `helm` | Kubernetes package manager |

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
| **Project Config** | ✅ `.vx.toml` | ❌ Varies by tool |
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
- uses: loonghao/vx@v1
  with:
    github-token: ${{ secrets.GITHUB_TOKEN }}

- run: vx node --version
- run: vx npm ci
- run: vx npm test
```

See [GitHub Action Guide](docs/guides/github-action.md) for full documentation.

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
