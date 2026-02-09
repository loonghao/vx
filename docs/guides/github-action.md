---
outline: deep
---

# Using vx in GitHub Actions

vx provides an official GitHub Action that makes it easy to use vx in your CI/CD workflows. This allows you to have consistent development tool versions across local development and CI environments.

## Quick Start

Add the following to your GitHub Actions workflow:

```yaml
- uses: loonghao/vx@main
  with:
    github-token: ${{secrets.GITHUB_TOKEN}}
```

> **Note**: You can use `@main` for the latest version, or pin to a specific release tag (e.g., `@vx-v0.6.4`). Check [releases](https://github.com/loonghao/vx/releases) for available versions.

Then use vx to run any supported tool:

```yaml
- run: vx node --version
- run: vx npm install
- run: vx uv pip install -r requirements.txt
- run: vx go build ./...
```

## Full Example

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      # Setup vx with caching
      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'node uv'  # Pre-install these tools
          cache: 'true'

      # Use vx to run tools
      - name: Install dependencies
        run: vx npm ci

      - name: Run tests
        run: vx npm test

      - name: Build
        run: vx npm run build
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `version` | vx version to install (e.g., "0.5.7", "latest") | `latest` |
| `github-token` | GitHub token for API requests (avoids rate limiting) | `github.token` |
| `tools` | Space-separated list of tools to pre-install (e.g., "node go uv") | `''` |
| `cache` | Enable caching of vx tools directory | `true` |
| `cache-key-prefix` | Custom prefix for cache key | `vx-tools` |

## Outputs

| Output | Description |
|--------|-------------|
| `version` | The installed vx version |
| `cache-hit` | Whether the cache was hit |

## Use Cases

### Automatic Dependency Installation

vx automatically detects and installs missing dependencies before running commands. This is especially useful in CI environments where you don't want to manually add install steps.

| Tool | Trigger Command | Auto-runs | Detection |
|------|-----------------|-----------|-----------|
| **uv** | `vx uv run` | `uv sync` | `pyproject.toml` exists, `.venv` missing |
| **npm** | `vx npm run` | `npm install` | `package.json` exists, `node_modules` missing |
| **pnpm** | `vx pnpm run` | `pnpm install` | `package.json` exists, `node_modules` missing |
| **yarn** | `vx yarn run` | `yarn install` | `package.json` exists, `node_modules` missing |
| **bun** | `vx bun run` | `bun install` | `package.json` exists, `node_modules` missing |
| **go** | `vx go run` | `go mod download` | `go.mod` exists, `vendor` missing |

This means your CI workflow can be as simple as:

```yaml
- run: vx uv run pytest      # Auto-syncs dependencies first
- run: vx npm run build      # Auto-installs node_modules first
- run: vx go run main.go     # Auto-downloads modules first
```

### Node.js Project

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'node'

      - run: vx npm ci
      - run: vx npm test
      - run: vx npm run build
```

> **Note**: vx automatically installs dependencies before running scripts. When running `vx npm run`, `vx pnpm run`, `vx yarn run`, or `vx bun run`, it will automatically execute the corresponding install command first if `package.json` exists but `node_modules` doesn't.

### Python Project with UV

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'uv'

      # vx automatically runs 'uv sync' before 'uv run' if .venv doesn't exist
      - run: vx uv run pytest
      - run: vx uvx ruff check .
```

> **Note**: vx automatically detects if dependencies need to be installed. When running `vx uv run`, it will automatically execute `uv sync` first if `pyproject.toml` exists but `.venv` doesn't.

### Go Project

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'go'

      - run: vx go build ./...
      - run: vx go test ./...
```

> **Note**: vx automatically downloads Go modules before running. When running `vx go run`, it will automatically execute `go mod download` first if `go.mod` exists but `vendor` directory doesn't.

### Multi-Language Project

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'node uv go'

      # Frontend
      - run: vx npm ci
      - run: vx npm run build

      # Backend (Python)
      - run: vx uv sync
      - run: vx uv run pytest

      # Services (Go)
      - run: vx go build ./cmd/...
```

### Cross-Platform CI

```yaml
jobs:
  build:
    runs-on: ${{matrix.os}}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}

      # Same commands work on all platforms!
      - run: vx node --version
      - run: vx npm ci
      - run: vx npm test
```

### Using with Version Pinning

If your project has a `vx.toml` configuration file, vx will automatically use the versions specified there:

```toml
# vx.toml
[tools]
node = "20.10.0"
uv = "0.4.0"
go = "1.22.0"
```

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}

      # vx will use versions from vx.toml
      - run: vx node --version  # Uses 20.10.0
      - run: vx uv self version    # Uses 0.4.0
```

### One-Click Setup with vx setup

The most powerful way to use vx in CI is with `vx setup`. This command reads your `vx.toml` and automatically:

1. Installs all required tools with pinned versions
2. Sets up Python virtual environment (if configured)
3. Installs Python dependencies
4. Verifies environment variables

**Example `vx.toml`:**

```toml
[project]
name = "my-fullstack-app"
description = "A full-stack application"

[tools]
node = "20"
uv = "latest"
go = "1.22"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]

[env]
NODE_ENV = "production"

[scripts]
test = "pytest"
build = "npm run build"
lint = "ruff check ."
```

**GitHub Actions workflow:**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}

      # One command to setup everything!
      - name: Setup development environment
        run: vx setup

      # Now run your scripts
      - run: vx run lint
      - run: vx run test
      - run: vx run build
```

This approach ensures:

- **Consistency**: Local and CI environments use identical tool versions
- **Simplicity**: One command replaces multiple setup actions
- **Reproducibility**: Just copy `vx.toml` to any project for the same setup

::: tip Share Your Configuration
Commit `vx.toml` to your repository. New team members can run `vx setup` to get the exact same development environment in seconds.
:::

### Environment Export for Subsequent Steps

When you need tools to be available in subsequent workflow steps without the `vx` prefix, use `vx env export`:

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}

      - name: Setup development environment
        run: vx setup

      # Export tool paths to GITHUB_PATH
      - name: Setup vx environment
        run: |
          if [ -f "vx.toml" ]; then
            vx env export --format github >> $GITHUB_PATH
          fi

      # Now tools are available directly!
      - run: node --version   # No 'vx' prefix needed
      - run: uv --version
      - run: npm ci
```

This is particularly useful when:

- Using tools that don't work well with the `vx` prefix
- Running scripts that expect tools to be directly in PATH
- Integrating with other actions that invoke tools directly

::: info How it works
`vx env export --format github` outputs tool paths (one per line) that GitHub Actions appends to `$GITHUB_PATH`. These paths become available in all subsequent steps.
:::

## Caching

The action automatically caches the vx tools directory (`~/.vx`) to speed up subsequent runs. You can customize the cache behavior:

```yaml
- uses: loonghao/vx@main
  with:
    cache: 'true'
    cache-key-prefix: 'my-project-vx'
```

To disable caching:

```yaml
- uses: loonghao/vx@main
  with:
    cache: 'false'
```

## Troubleshooting

### Rate Limiting

If you encounter GitHub API rate limiting, make sure to provide a GitHub token:

```yaml
- uses: loonghao/vx@main
  with:
    github-token: ${{secrets.GITHUB_TOKEN}}
```

The vx action automatically exports the `GITHUB_TOKEN` environment variable to all subsequent steps, so you don't need to set it manually in each step. However, if you're running vx commands in a separate job or workflow, make sure to pass the token:

```yaml
- name: Run vx command
  env:
    GITHUB_TOKEN: ${{secrets.GITHUB_TOKEN}}
  run: vx uv run pytest
```

### Tool Installation Failures

If a tool fails to install, check:

1. The tool is supported by vx (`vx list` to see all supported tools)
2. Network connectivity to the tool's download source
3. Sufficient disk space

### Cache Issues

If you're experiencing cache-related issues, try:

1. Using a different cache key prefix
2. Disabling caching temporarily
3. Clearing the cache in GitHub Actions settings

## Supported Tools

vx supports many popular development tools:

- **JavaScript/TypeScript**: Node.js, npm, npx, Bun, Deno, pnpm, Yarn, Vite
- **Python**: UV, uvx
- **Go**: Go
- **Rust**: Cargo, rustc, rustup
- **Java**: Java, javac
- **DevOps**: Terraform, kubectl, Helm
- **Others**: Just, Zig, and more

Run `vx list` to see all available tools.

## Migration from Other Actions

### From actions/setup-node

Before:

```yaml
- uses: actions/setup-node@v4
  with:
    node-version: '20'
- run: npm ci
```

After:

```yaml
- uses: loonghao/vx@main
- run: vx npm ci
```

### From actions/setup-python + pip

Before:

```yaml
- uses: actions/setup-python@v5
  with:
    python-version: '3.12'
- run: pip install -r requirements.txt
```

After:

```yaml
- uses: loonghao/vx@main
- run: vx uv pip install -r requirements.txt
```

### From actions/setup-go

Before:

```yaml
- uses: actions/setup-go@v5
  with:
    go-version: '1.22'
- run: go build ./...
```

After:

```yaml
- uses: loonghao/vx@main
- run: vx go build ./...
```

## Benefits of Using vx

1. **Unified Tool Management**: One action to manage all development tools
2. **Version Consistency**: Same tool versions in CI as local development
3. **Cross-Platform**: Works on Linux, macOS, and Windows
4. **Caching**: Automatic caching for faster CI runs
5. **Simplicity**: No need to configure multiple setup actions

## Using Docker Images

vx provides official Docker images that can be used directly in your CI/CD workflows. These images are available on both Docker Hub and GitHub Container Registry.

### Available Images

| Image Tag | Description | Base |
|-----------|-------------|------|
| `vx:latest` | Minimal image with just vx | Ubuntu 24.04 (Noble) |
| `vx:tools-latest` | Image with pre-installed tools (uv, ruff, node) | Ubuntu 24.04 (Noble) |

::: info Why Ubuntu 24.04?
We use Ubuntu 24.04 (glibc 2.39) because:
1. vx is compiled on Ubuntu 24.04 runners, requiring glibc 2.39
2. Most development tools provide glibc-compiled binaries
3. Alpine (musl) causes "No such file or directory" errors
4. Older Debian/Ubuntu versions have outdated glibc
:::

### Using the Tools Image in Container Jobs

The `vx:tools-latest` image comes with commonly used tools pre-installed, making it perfect for CI/CD workflows where you need fast startup times:

**Pre-installed tools:**
- **uv** - Fast Python package manager
- **ruff** (via uvx) - Python linter and formatter
- **Node.js** - JavaScript runtime (LTS version)

```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/loonghao/vx:tools-latest
    steps:
      - uses: actions/checkout@v6

      # Tools are already available - no installation needed!
      - name: Lint Python
        run: vx uvx ruff check .

      - name: Run tests
        run: |
          vx uv sync
          vx uv run pytest

      - name: Build frontend
        run: |
          vx npm ci
          vx npm run build
```

### Pulling the Images

```bash
# From GitHub Container Registry (recommended)
docker pull ghcr.io/loonghao/vx:latest
docker pull ghcr.io/loonghao/vx:tools-latest

# From Docker Hub
docker pull longhal/vx:latest
docker pull longhal/vx:tools-latest
```

### Using in Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  dev:
    image: ghcr.io/loonghao/vx:tools-latest
    working_dir: /app
    volumes:
      - .:/app
    command: bash -c "vx uv sync && vx uv run pytest"
```

### Building Custom Images

You can extend the vx images with your own tools:

```dockerfile
FROM ghcr.io/loonghao/vx:tools-latest

# Pre-install additional tools
RUN vx go version

# Add your project files
COPY . /app
WORKDIR /app

# Run your application
CMD ["vx", "uv", "run", "main.py"]
```

### Image Tags

Both Docker Hub and GHCR provide the following tags:

- `latest` - Latest stable base image
- `tools-latest` - Latest stable tools image
- `{version}` - Specific version (e.g., `0.6.5`)
- `tools-{version}` - Specific version with tools

::: tip When to use the tools image
Use `vx:tools-latest` when:
- Your workflow needs Python (uv/ruff) or Node.js
- You want faster CI startup times
- You're running multiple jobs that all need the same tools

Use `vx:latest` when:
- You only need specific tools not in the tools image
- You want the smallest possible image size
- You're building a custom image on top of vx
:::
