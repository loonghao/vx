# Introduction

**vx** is a universal development tool manager that eliminates the complexity of managing multiple development runtimes. Instead of learning and configuring separate tools for Node.js, Python, Go, Rust, and more, you simply prefix your commands with `vx` and everything just works.

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
