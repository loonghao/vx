---
outline: deep
---

# Using vx in GitHub Actions

vx provides an official GitHub Action that makes it easy to use vx in your CI/CD workflows. This allows you to have consistent development tool versions across local development and CI environments.

## Quick Start

Add the following to your GitHub Actions workflow:

```yaml
- uses: loonghao/vx@v1
  with:
    github-token: ${{secrets.GITHUB_TOKEN}}
```

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
      - uses: actions/checkout@v4

      # Setup vx with caching
      - uses: loonghao/vx@v1
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

### Node.js Project

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: loonghao/vx@v1
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'node'

      - run: vx npm ci
      - run: vx npm test
      - run: vx npm run build
```

### Python Project with UV

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: loonghao/vx@v1
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'uv'

      - run: vx uv sync
      - run: vx uv run pytest
      - run: vx uvx ruff check .
```

### Go Project

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: loonghao/vx@v1
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'go'

      - run: vx go build ./...
      - run: vx go test ./...
```

### Multi-Language Project

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: loonghao/vx@v1
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
      - uses: actions/checkout@v4

      - uses: loonghao/vx@v1
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}

      # Same commands work on all platforms!
      - run: vx node --version
      - run: vx npm ci
      - run: vx npm test
```

### Using with Version Pinning

If your project has a `.vx.toml` configuration file, vx will automatically use the versions specified there:

```toml
# .vx.toml
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
      - uses: actions/checkout@v4

      - uses: loonghao/vx@v1
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}

      # vx will use versions from .vx.toml
      - run: vx node --version  # Uses 20.10.0
      - run: vx uv --version    # Uses 0.4.0
```

### One-Click Setup with vx setup

The most powerful way to use vx in CI is with `vx setup`. This command reads your `.vx.toml` and automatically:

1. Installs all required tools with pinned versions
2. Sets up Python virtual environment (if configured)
3. Installs Python dependencies
4. Verifies environment variables

**Example `.vx.toml`:**

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
      - uses: actions/checkout@v4

      - uses: loonghao/vx@v1
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
- **Reproducibility**: Just copy `.vx.toml` to any project for the same setup

::: tip Share Your Configuration
Commit `.vx.toml` to your repository. New team members can run `vx setup` to get the exact same development environment in seconds.
:::

## Caching

The action automatically caches the vx tools directory (`~/.vx`) to speed up subsequent runs. You can customize the cache behavior:

```yaml
- uses: loonghao/vx@v1
  with:
    cache: 'true'
    cache-key-prefix: 'my-project-vx'
```

To disable caching:

```yaml
- uses: loonghao/vx@v1
  with:
    cache: 'false'
```

## Troubleshooting

### Rate Limiting

If you encounter GitHub API rate limiting, make sure to provide a GitHub token:

```yaml
- uses: loonghao/vx@v1
  with:
    github-token: ${{secrets.GITHUB_TOKEN}}
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
- uses: loonghao/vx@v1
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
- uses: loonghao/vx@v1
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
- uses: loonghao/vx@v1
- run: vx go build ./...
```

## Benefits of Using vx

1. **Unified Tool Management**: One action to manage all development tools
2. **Version Consistency**: Same tool versions in CI as local development
3. **Cross-Platform**: Works on Linux, macOS, and Windows
4. **Caching**: Automatic caching for faster CI runs
5. **Simplicity**: No need to configure multiple setup actions
