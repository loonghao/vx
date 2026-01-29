# VX Knowledge Base

## What is VX?

vx is a universal development tool manager designed for the AI-native era. It follows Unix Philosophy principles and provides zero-configuration tool execution.

## Core Concepts

### Philosophy
- **One tool, one job**: vx manages all runtimes transparently
- **Zero learning curve**: Use tools the same way you always have
- **Scriptability**: Full bash integration, CI/CD ready
- **Composability**: Works with any AI coding assistant

### Key Features
- Auto-installation of tools when needed
- Version management per project
- Environment isolation
- Extension system for custom functionality
- MCP (Model Context Protocol) support

## Installation

### Quick Install

**Linux/macOS:**
```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows (PowerShell):**
```powershell
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

## Basic Usage

### Direct Tool Execution
```bash
vx node --version
vx python script.py
vx go build
vx cargo test
```

### Project Setup
```bash
# Initialize project
vx init

# Install all tools from vx.toml
vx setup

# Enter development environment
vx dev
```

## Commands Reference

### Tool Management
- `vx install <tool>[@version]` - Install a tool version
- `vx list` - List available tools
- `vx which <tool>` - Show tool location
- `vx versions <tool>` - Show available versions
- `vx uninstall <tool>` - Remove a tool

### Project Management
- `vx init` - Initialize project configuration
- `vx setup` - Install all project tools
- `vx sync` - Sync tools with configuration
- `vx add <tool>` - Add tool to project
- `vx remove <tool>` - Remove tool from project

### Script Execution
- `vx run <script>` - Run script from vx.toml
- `vx dev` - Enter development environment

### Configuration
- `vx config show` - Show current configuration
- `vx config set <key> <value>` - Set configuration
- `vx shell init` - Generate shell integration

## Supported Tools

### Language Runtimes
- **Node.js**: node, npm, npx, yarn, pnpm, bun
- **Python**: python, uv, pip
- **Rust**: rust, cargo, rustup
- **Go**: go
- **Java**: java
- **Zig**: zig

### Package Managers
- **Node.js**: npm, yarn, pnpm, bun
- **Python**: uv, pip
- **Rust**: cargo

### Build Tools
- **Web**: vite, webpack
- **Native**: cmake, ninja, msvc
- **Task Runners**: just, task

### DevOps
- **Containers**: docker
- **Cloud**: awscli, azcli, gcloud, terraform
- **Kubernetes**: kubectl, helm

### Code Quality
- **Linting**: ruff, eslint
- **Formatting**: prettier, black
- **Testing**: pytest, jest

### AI/ML
- **Ollama**: Run LLMs locally

## Configuration File (vx.toml)

### Basic Structure
```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"
docker = "latest"

[env]
NODE_ENV = "development"
PYTHONUNBUFFERED = "1"

[scripts]
dev = "npm run dev"
test = "npm test"
build = "npm run build"
```

### Project Types

#### Node.js Project
```toml
[tools]
node = "22"
npm = "bundled"

[scripts]
dev = "npm run dev"
build = "npm run build"
```

#### Python Project
```toml
[tools]
python = "3.12"
uv = "latest"

[python]
version = "3.12"
venv = ".venv"

[scripts]
install = "uv pip install -r requirements.txt"
test = "uv run pytest"
```

#### Multi-language Project
```toml
[tools]
node = "22"
python = "3.12"
go = "latest"
docker = "latest"

[scripts]
frontend = "cd frontend && npm run dev"
backend = "cd backend && go run main.go"
```

## Best Practices

### 1. Version Pinning
Always pin tool versions in vx.toml for reproducible environments:
```toml
[tools]
node = "22.0.0"  # Specific version
python = "3.12"  # Major.minor
uv = "latest"    # Latest (use with caution)
```

### 2. Environment Variables
Use environment variables for configuration:
```toml
[env]
DATABASE_URL = "postgresql://localhost/mydb"
API_KEY = "${API_KEY}"  # From system environment
```

### 3. Script Organization
Group related scripts:
```toml
[scripts]
# Development
dev = "npm run dev"
watch = "npm run watch"

# Testing
test = "npm test"
test:coverage = "npm run test:coverage"

# Building
build = "npm run build"
build:prod = "npm run build:prod"
```

### 4. CI/CD Integration
Use vx in CI/CD pipelines:
```yaml
# GitHub Actions
- name: Setup vx
  run: curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

- name: Install tools
  run: vx setup

- name: Run tests
  run: vx run test
```

## Extension System

### Installing Extensions
```bash
vx ext install github:user/vx-ext-name
vx ext list
vx ext info <name>
```

### Using Extensions
```bash
vx x <extension> [command] [args...]
vx x docker-compose up
```

## Troubleshooting

### Tool Not Found
```bash
# Check if tool is installed
vx which <tool>

# Install the tool
vx install <tool>

# Check available versions
vx versions <tool>
```

### Version Conflicts
```bash
# Check current version
vx which <tool>

# Switch version
vx switch <tool>@<version>

# Check project configuration
cat vx.toml
```

### PATH Issues
```bash
# Use system PATH instead
vx --use-system-path <tool>

# Check shell integration
vx shell init
```

## Integration with AI Assistants

vx is designed to work seamlessly with AI coding assistants:

1. **Zero Configuration**: AI can use any tool without setup
2. **Auto-installation**: Tools install automatically when needed
3. **Consistent Interface**: Same commands work across all tools
4. **MCP Support**: Works with Model Context Protocol

## Examples

### Setting up a React Project
```bash
vx init
# Add to vx.toml:
# [tools]
# node = "22"
vx setup
vx npx create-react-app my-app
```

### Python ML Project
```bash
vx init
# Add to vx.toml:
# [tools]
# python = "3.12"
# uv = "latest"
vx setup
vx uv venv
vx uv pip install torch transformers
```

### Multi-language Monorepo
```toml
[tools]
node = "22"
python = "3.12"
go = "latest"
docker = "latest"

[scripts]
frontend:dev = "cd frontend && npm run dev"
backend:dev = "cd backend && go run main.go"
ml:train = "cd ml && python train.py"
```

## Resources

- **Documentation**: https://github.com/loonghao/vx
- **GitHub**: https://github.com/loonghao/vx
- **Issues**: https://github.com/loonghao/vx/issues
