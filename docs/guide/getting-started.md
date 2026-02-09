# Quick Start

Get up and running with vx in 5 minutes.

## Prerequisites

- **Windows 10+**, **macOS 10.15+**, or **Linux** (glibc 2.17+)
- Internet connection for first-time tool downloads

## Step 1: Install vx

::: code-group
```bash [Linux / macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows (PowerShell)]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

```bash [Cargo]
cargo install vx
```
:::

Verify the installation:

```bash
vx --version
```

## Step 2: Use Your First Tool

Simply prefix any command with `vx`. Tools are auto-installed on first use:

```bash
# Node.js
vx node --version        # Downloads and installs Node.js, then prints version

# Python
vx python --version      # Downloads and installs Python, then prints version

# Go
vx go version            # Downloads and installs Go, then prints version
```

::: tip Auto-Install
When you run `vx <tool>` for the first time, vx automatically downloads and installs the latest stable version. No manual setup required!
:::

## Step 3: Install Specific Versions

```bash
# Install a specific version
vx install node@22
vx install python@3.12
vx install go@1.23

# Install multiple tools at once
vx install node@22 python@3.12 uv@latest

# Use semantic version ranges
vx install "node@^22"    # Latest 22.x.x
vx install "python@~3.12" # Latest 3.12.x
```

## Step 4: Set Up a Project

Create a `vx.toml` to define your project's toolchain:

```bash
cd my-project
vx config init
```

Edit the generated `vx.toml`:

```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"

[scripts]
dev = "vx node server.js"
test = "vx uv run pytest"
lint = "vx uvx ruff check ."
build = "vx node scripts/build.js"
```

Install all project tools:

```bash
vx setup
```

## Step 5: Run Project Scripts

```bash
# Run defined scripts
vx run dev               # Start the dev server
vx run test              # Run tests
vx run lint              # Run linter

# List available scripts
vx run --list

# Pass arguments to scripts
vx run test -- -v --coverage
```

## Step 6: Enter Development Environment

```bash
# Enter an interactive shell with all project tools on PATH
vx dev

# Or run a single command in the project environment
vx dev -c "node --version && python --version"

# Export environment for CI/CD
vx dev --export --format github >> $GITHUB_PATH
```

## Step 7: Set Up Shell Integration (Optional)

Enable automatic version switching and tab completions:

::: code-group
```bash [Bash]
echo 'eval "$(vx shell init bash)"' >> ~/.bashrc
```

```bash [Zsh]
echo 'eval "$(vx shell init zsh)"' >> ~/.zshrc
```

```bash [Fish]
echo 'vx shell init fish | source' >> ~/.config/fish/config.fish
```

```powershell [PowerShell]
Add-Content $PROFILE 'Invoke-Expression (vx shell init powershell | Out-String)'
```
:::

## Common Workflows

### Node.js Project

```bash
vx npx create-react-app my-app
cd my-app
vx npm install
vx npm start
```

### Python Project

```bash
vx uv init my-project
cd my-project
vx uv add flask pytest
vx uv run flask run
```

### Go Project

```bash
mkdir my-service && cd my-service
vx go mod init github.com/user/my-service
vx go run main.go
```

### Multi-Language Project

```toml
# vx.toml
[tools]
node = "22"
python = "3.12"
go = "1.23"
just = "latest"

[scripts]
frontend = "cd frontend && vx npm run dev"
backend = "cd backend && vx go run ."
api = "cd api && vx uv run flask run"
all = "just dev"
```

## Next Steps

- [Core Concepts](/guide/concepts) — Understand providers, runtimes, and versions
- [Configuration](/guide/configuration) — Deep dive into `vx.toml`
- [CLI Reference](/cli/overview) — Explore all available commands
- [Supported Tools](/tools/overview) — See the full list of 50+ tools
