# Introduction

**vx** is a universal development tool manager that eliminates the complexity of managing multiple development runtimes. Instead of learning and configuring separate tools for Node.js, Python, Go, Rust, and more, you simply prefix your commands with `vx` and everything just works.

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

When AI agents like Claude Code need to execute commands across different ecosystems, they often face environment setup challenges. vx solves this by providing a unified interface:

```bash
# AI agents can run any command without worrying about environment
vx npx create-react-app my-app  # Works immediately
vx uvx ruff check .             # Works immediately
vx cargo build --release        # Works immediately
```

**vx enables AI to have full-stack development capabilities without worrying about environment management and dependencies.**

## Why vx?

### The Traditional Way

```bash
# Install and manage multiple tools separately
nvm install 20
nvm use 20
npm install -g typescript

pyenv install 3.11
pyenv local 3.11
pip install uv

# Deal with PATH conflicts, version mismatches...
```

### The vx Way

```bash
# Just use the tools - vx handles everything
vx node --version
vx python --version
vx npx create-react-app my-app
vx uvx ruff check .
```

## Two Ways to Use vx

### 1. Direct Execution (For Quick Tasks)

Just prefix any command with `vx` �?tools are auto-installed on first use:

```bash
vx npx create-react-app my-app
vx uvx ruff check .
vx go run main.go
```

### 2. Project Development Environment (For Teams)

Create a `vx.toml` file to define your project's tool requirements:

```toml
[project]
name = "my-project"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
```

Then run:

```bash
vx setup     # Install all project tools
vx run dev   # Run defined scripts
```

## Supported Tools

vx supports a wide range of development tools:

| Ecosystem | Tools |
|-----------|-------|
| **Node.js** | node, npm, npx, pnpm, yarn, bun |
| **Python** | python, uv, uvx, pip |
| **Go** | go |
| **Rust** | cargo, rustc |
| **DevOps** | kubectl, helm, terraform |
| **Utilities** | just, jq, ripgrep, and more |

## Next Steps

- [Installation](/guide/installation) - Install vx on your system
- [Quick Start](/guide/getting-started) - Get up and running in minutes
- [Configuration](/guide/configuration) - Learn about `vx.toml`
- [Enhanced Scripts](/guide/enhanced-scripts) - **NEW!** Advanced script system with flexible argument passing
