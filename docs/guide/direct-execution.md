# Direct Execution

The simplest way to use vx is direct execution — just prefix any command with `vx`.

## Basic Usage

```bash
# Language runtimes
vx node --version
vx python --version
vx go version
vx cargo --version

# Package managers
vx npm install
vx uvx ruff check .
vx pnpm dev

# DevOps tools
vx terraform plan
vx kubectl get pods
vx dagu server

# Build tools
vx just build
vx cmake --build build
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

## Running Language Runtimes

### Node.js

```bash
# Run Node.js scripts
vx node app.js
vx node --eval "console.log('Hello from vx!')"

# Interactive REPL
vx node
```

### Python

```bash
# Run Python scripts
vx python main.py
vx python -c "import sys; print(sys.version)"

# Run modules directly
vx python -m http.server 8000
vx python -m json.tool data.json
```

### Go

```bash
# Build and run
vx go build -o myapp ./cmd/server
vx go run main.go
vx go test ./...

# Install Go tools
vx go install golang.org/x/tools/gopls@latest
```

### Rust / Cargo

```bash
# Build projects
vx cargo build --release
vx cargo test
vx cargo run -- --port 8080

# Create new projects
vx cargo new my-cli
vx cargo init .

# Install tools via cargo
vx cargo install ripgrep
```

## Running Package Managers

### npm / npx

```bash
# Project setup
vx npm init -y
vx npm install express typescript
vx npm run dev

# One-off commands with npx
vx npx create-react-app my-app
vx npx create-next-app@latest my-next-app
vx npx eslint --fix .
vx npx prettier --write .
vx npx tsx script.ts
```

### pnpm

```bash
# Project setup
vx pnpm init
vx pnpm add express
vx pnpm install
vx pnpm dev

# Workspace management
vx pnpm -r build          # Build all packages
vx pnpm --filter api dev  # Run dev in specific package
```

### yarn

```bash
vx yarn init
vx yarn add react react-dom
vx yarn dev
```

### bun

```bash
vx bun init
vx bun add express
vx bun run dev
vx bunx create-next-app my-app
```

### uv / uvx (Python)

```bash
# Project lifecycle
vx uv init my-project
vx uv add requests flask pytest
vx uv sync
vx uv run python main.py
vx uv run pytest

# Virtual environment management
vx uv venv
vx uv pip install -r requirements.txt

# Run CLI tools without installing (uvx)
vx uvx ruff check .              # Lint Python code
vx uvx ruff format .             # Format Python code
vx uvx black .                   # Code formatter
vx uvx mypy src/                 # Type checking
vx uvx pytest                    # Run tests
vx uvx jupyter notebook          # Start Jupyter
vx uvx cookiecutter gh:user/repo # Project scaffolding
vx uvx pre-commit run --all-files
```

## Package Aliases

vx supports **package aliases** — short commands that automatically route to ecosystem packages.

### What are Package Aliases?

Instead of remembering ecosystem prefixes like `npm:` or `uv:`, you can use familiar tool names directly:

```bash
# These are equivalent:
vx vite              # Same as: vx npm:vite
vx vite@5.0          # Same as: vx npm:vite@5.0
vx rez               # Same as: vx uv:rez
vx pre-commit        # Same as: vx uv:pre-commit
vx meson             # Same as: vx uv:meson
vx release-please    # Same as: vx npm:release-please
```

### Benefits

1. **Simpler Commands**: No need to remember ecosystem prefixes
2. **Automatic Dependency Management**: vx automatically installs the required runtime (node/python) based on your `vx.toml` configuration
3. **Unified Experience**: Works seamlessly with both runtime tools and ecosystem packages

### How It Works

When you run `vx vite`, vx:

1. Checks if `vite` has a `package_alias` definition in its provider
2. Routes the request to `npm:vite` package execution
3. Auto-installs Node.js if needed (respecting your `vx.toml` version)
4. Runs the package with your specified arguments

### Uninstalling Aliased Packages

```bash
# Uninstall works the same way:
vx uninstall vite          # Same as: vx global uninstall npm:vite
vx uninstall rez@3.0       # Same as: vx global uninstall uv:rez
```

### Available Aliases

| Short Command | Equivalent | Ecosystem |
|--------------|------------|-----------|
| `vx vite` | `vx npm:vite` | npm |
| `vx release-please` | `vx npm:release-please` | npm |
| `vx rez` | `vx uv:rez` | uv |
| `vx pre-commit` | `vx uv:pre-commit` | uv |
| `vx meson` | `vx uv:meson` | uv |

### Defining Custom Aliases

You can define package aliases in your provider's `provider.toml`:

```toml
[provider]
name = "vite"
description = "Next generation frontend build tool"

[provider.package_alias]
ecosystem = "npm"    # Target ecosystem
package = "vite"     # Package name
```

## Running DevOps Tools

### Terraform

```bash
vx terraform init
vx terraform plan
vx terraform apply -auto-approve
vx terraform destroy
```

### kubectl & Helm

```bash
vx kubectl get pods -A
vx kubectl apply -f deployment.yaml
vx helm install my-release ./chart
vx helm upgrade my-release ./chart
```

### Dagu (Workflow Engine)

```bash
# Start the web UI dashboard
vx dagu server

# Run workflows
vx dagu start my-workflow
vx dagu status my-workflow

# Dagu + vx: use vx-managed tools inside DAG definitions
# my-workflow.yaml:
#   steps:
#     - name: lint
#       command: vx uvx ruff check .
#     - name: test
#       command: vx uv run pytest
#     - name: build
#       command: vx cargo build --release
```

### GitHub CLI

```bash
vx gh repo clone owner/repo
vx gh pr create --fill
vx gh issue list
vx gh release create v1.0.0
```

## Running Build Tools

### Just (Modern Make)

```bash
# Run tasks
vx just build
vx just test
vx just --list

# Just + vx subprocess PATH: tools available without vx prefix
# justfile:
#   lint:
#       uvx ruff check .     # Works! vx tools in subprocess PATH
#       npm run lint
```

### CMake & Ninja

```bash
vx cmake -B build -G Ninja
vx cmake --build build --config Release
vx ninja -C build
```

### Task (go-task)

```bash
vx task build
vx task test
vx task --list
```

## Running Data & Media Tools

```bash
# JSON processing
vx jq '.name' package.json
vx jq -r '.dependencies | keys[]' package.json

# Video/audio processing
vx ffmpeg -i input.mp4 -c:v libx264 output.mp4
vx ffprobe -show_format video.mp4

# Image processing
vx magick input.png -resize 50% output.png
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
vx cargo build --release --target x86_64-unknown-linux-musl
```

## Environment Variables

Set environment variables before the command:

```bash
# Unix
NODE_ENV=production vx node server.js
RUST_LOG=debug vx cargo run

# Or use env
env DATABASE_URL=postgres://localhost/mydb vx uv run main.py
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

## Subprocess PATH Inheritance

When running a tool via `vx`, any subprocess spawned by that tool will automatically have access to all vx-managed tools in PATH. This means build tools, task runners, and scripts can use vx-managed tools directly without the `vx` prefix.

### Example: justfile

```makefile
# justfile — all tools available without vx prefix!
lint:
    uvx ruff check .
    uvx mypy src/

test:
    uv run pytest

build:
    npm run build
    cargo build --release
```

Run with:

```bash
vx just lint    # justfile recipes can use vx tools directly
vx just test
vx just build
```

### Example: Dagu Workflow

```yaml
# workflow.yaml — vx tools available in DAG steps
steps:
  - name: lint
    command: uvx ruff check .
  - name: test
    command: uv run pytest
    depends:
      - lint
  - name: build
    command: cargo build --release
    depends:
      - test
```

Run with:

```bash
vx dagu start workflow
```

### Example: Makefile

```makefile
# Makefile
lint:
	uvx ruff check .

test:
	npm test

build:
	go build -o app
```

Run with:

```bash
vx make lint    # Make targets can use vx tools directly
```

### Disabling PATH Inheritance

If you need to disable subprocess PATH inheritance (e.g., for isolation), you can configure it in your project's `vx.toml`:

```toml
[settings]
inherit_vx_path = false
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

## Real-World Examples

### Full-Stack Web App

```bash
# Frontend
vx npx create-next-app@latest my-app
cd my-app
vx npm install
vx npm run dev

# Backend API (Python)
vx uv init api && cd api
vx uv add fastapi uvicorn
vx uv run uvicorn main:app --reload
```

### Python Data Science

```bash
vx uv init analysis && cd analysis
vx uv add pandas numpy matplotlib scikit-learn
vx uvx jupyter notebook
vx python -c "import pandas; print(pandas.__version__)"
```

### Go Microservice

```bash
mkdir my-service && cd my-service
vx go mod init github.com/user/my-service
vx go get github.com/gin-gonic/gin
vx go run main.go
vx go build -o server .
```

### Rust CLI Tool

```bash
vx cargo new my-cli
cd my-cli
vx cargo add clap --features derive
vx cargo build --release
```

### Cross-Language Project with Dagu

```bash
# Define a workflow that uses multiple tools
# build-pipeline.yaml:
#   steps:
#     - name: frontend
#       command: npm run build
#       dir: frontend/
#     - name: backend
#       command: cargo build --release
#       dir: backend/
#     - name: deploy
#       command: terraform apply -auto-approve
#       depends: [frontend, backend]

vx dagu start build-pipeline
vx dagu server   # Monitor via web UI at http://localhost:8080
```

### DevOps Automation

```bash
# Infrastructure
vx terraform init && vx terraform plan
vx kubectl apply -f k8s/

# CI-like local workflow
vx just ci       # Run all CI checks locally
```

## Tips

::: tip First Run
The first run is slower because tools need to be downloaded and installed. Subsequent runs use cached versions and are much faster.
:::

::: tip Version Pinning
Always specify versions for reproducibility in team projects.
:::

::: tip Subprocess PATH
When using task runners like `just`, `dagu`, or `make` via vx, all vx-managed tools are automatically available in subprocesses — no `vx` prefix needed inside recipes/steps.
:::

## Next Steps

- [Project Environments](/guide/project-environments) - Set up project-specific configurations
- [Real-World Use Cases](/guides/use-cases) - More practical examples
- [CLI Reference](/cli/overview) - Complete command reference
