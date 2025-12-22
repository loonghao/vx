# Direct Execution

The simplest way to use vx is direct execution - just prefix any command with `vx`.

## Basic Usage

```bash
# Run any tool
vx node --version
vx python --version
vx go version
vx cargo --version
```

If the tool isn't installed, vx will install it automatically.

## Specifying Versions

Use `@` to specify a version:

```bash
# Specific major version
vx node@18 --version

# Exact version
vx node@18.19.0 --version

# Latest
vx node@latest --version
```

## Running Package Managers

### npm/npx

```bash
# Run npm commands
vx npm install
vx npm run build

# Run npx
vx npx create-react-app my-app
vx npx eslint .
```

### Python/UV

```bash
# Run Python
vx python script.py
vx python -m pytest

# Run uv
vx uv pip install requests
vx uv venv .venv

# Run uvx (uv tool run)
vx uvx ruff check .
vx uvx black .
vx uvx mypy src/
```

### Go

```bash
# Run Go commands
vx go build
vx go test ./...
vx go run main.go

# Install Go tools
vx go install golang.org/x/tools/gopls@latest
```

### Rust/Cargo

```bash
# Run Cargo
vx cargo build --release
vx cargo test
vx cargo run

# Run rustc
vx rustc --version
```

## Passing Arguments

All arguments after the tool name are passed through:

```bash
# These are equivalent
vx node script.js --port 3000
node script.js --port 3000  # (if node is in PATH)

# Complex arguments work too
vx npm run build -- --mode production
vx go build -ldflags "-s -w" -o app
```

## Environment Variables

Set environment variables before the command:

```bash
# Unix
NODE_ENV=production vx node server.js

# Or use env
env NODE_ENV=production vx node server.js
```

## Working Directory

vx runs commands in the current directory:

```bash
cd my-project
vx npm install  # Runs in my-project/
```

## Using System Tools

If you want to use a system-installed tool instead of vx-managed:

```bash
vx --use-system-path node --version
```

## Verbose Output

For debugging, use verbose mode:

```bash
vx --verbose node --version
```

This shows:

- Version resolution
- Installation steps
- Execution details

## Examples

### Create a React App

```bash
vx npx create-react-app my-app
cd my-app
vx npm start
```

### Python Data Science

```bash
vx uvx jupyter notebook
vx python -c "import pandas; print(pandas.__version__)"
```

### Go Web Server

```bash
vx go mod init myserver
vx go get github.com/gin-gonic/gin
vx go run main.go
```

### Rust CLI Tool

```bash
vx cargo new my-cli
cd my-cli
vx cargo build --release
```

## Tips

::: tip First Run
The first run is slower because tools need to be downloaded and installed. Subsequent runs use cached versions and are much faster.
:::

::: tip Version Pinning
Always specify versions for reproducibility in team projects.
:::

## Next Steps

- [Project Environments](/guide/project-environments) - Set up project-specific configurations
- [CLI Reference](/cli/overview) - Complete command reference
