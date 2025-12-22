# Quick Start

This guide will get you up and running with vx in just a few minutes.

## Prerequisites

- A terminal (bash, zsh, PowerShell, etc.)
- Internet connection for downloading tools

## Step 1: Install vx

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

:::

## Step 2: Use Any Tool Instantly

Just prefix your commands with `vx`. Tools are automatically installed on first use:

```bash
# Run Node.js
vx node --version

# Run Python
vx python --version

# Run Go
vx go version

# Run npm/npx
vx npx create-react-app my-app

# Run uv/uvx
vx uvx ruff check .
```

**That's it!** No configuration, no setup, no learning new commands.

## Step 3: Set Up a Project (Optional)

For team projects, create a `.vx.toml` file to ensure everyone uses the same tool versions:

```bash
# Initialize a new project configuration
vx init
```

Or create manually:

```toml
[project]
name = "my-project"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
test = "npm test"
```

Then run:

```bash
# Install all project tools
vx setup

# Run a script
vx run dev
```

## Common Commands

| Command | Description |
|---------|-------------|
| `vx <tool> [args]` | Run a tool (auto-installs if needed) |
| `vx install <tool>` | Install a specific tool |
| `vx list` | List available tools |
| `vx setup` | Install all project tools from `.vx.toml` |
| `vx run <script>` | Run a script defined in `.vx.toml` |
| `vx dev` | Enter development environment |
| `vx --help` | Show help |

## Example Workflows

### Web Development

```bash
# Create a React app
vx npx create-react-app my-app
cd my-app

# Start development server
vx npm start
```

### Python Development

```bash
# Run a Python script
vx python script.py

# Use uvx to run tools
vx uvx ruff check .
vx uvx black .
```

### Go Development

```bash
# Build a Go project
vx go build -o myapp

# Run tests
vx go test ./...
```

## Next Steps

- [Configuration Guide](/guide/configuration) - Learn about `.vx.toml` configuration
- [CLI Reference](/cli/overview) - Complete command reference
- [Shell Integration](/guide/shell-integration) - Set up shell integration
