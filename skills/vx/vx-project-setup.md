# VX Project Setup Guide

## Quick Start

```bash
# Initialize a new project
vx init

# Or use a template
vx init --template node
vx init --template rust
vx init --template python
vx init --template electron       # Electron desktop app
vx init --template rust-python    # Rust + Python hybrid (maturin/PyO3)
vx init --template openclaw       # OpenClaw AI agent project
vx init --template fullstack      # Node.js + Python full-stack
```

## vx.toml Full Reference

```toml
# vx.toml — Project tool configuration

[tools]
# Version constraints
node = "22"                 # Major version (any 22.x.x)
go = "1.22"                 # Minor version (any 1.22.x)
uv = "latest"               # Always latest
rust = "1.80"               # Specific version
just = "*"                  # Any version

# Platform-specific tools
[tools.msvc]
version = "14.42"
os = ["windows"]

[tools.brew]
version = "latest"
os = ["macos", "linux"]

[scripts]
# Development
dev = "npm run dev"
test = "cargo test"
lint = "npm run lint && cargo clippy"
build = "just build"

# CI/CD
ci = "just ci"
release = "just release"

# Colon-separated variants
test:watch = "npm run test -- --watch"
test:coverage = "npm run test -- --coverage"
build:prod = "npm run build -- --mode production"

[hooks]
pre_commit = ["vx run lint"]
post_setup = ["npm install", "cargo fetch"]
post_checkout = ["vx sync"]

[env]
NODE_ENV = "development"
DATABASE_URL = "postgresql://localhost:5432/dev"
API_KEY = { env = "API_KEY", required = true }
PORT = { default = "3000" }
```

## Team Workflow

### Initial Setup

```bash
vx init                     # Create vx.toml
vx add node@22 go uv just   # Add required tools
vx lock                     # Generate lock file
git add vx.toml vx.lock     # Commit configuration
```

### Team Onboarding

```bash
git clone <repo> && cd <repo>
vx setup                    # One command — installs everything
```

### Updating Tools

```bash
vx outdated                 # Check for updates
vx update node              # Update specific tool
vx update --all             # Update all
vx lock                     # Regenerate lock file
git add vx.lock && git commit -m "chore: update tool versions"
```

## CI/CD Integration

### GitHub Actions

```yaml
steps:
  - uses: actions/checkout@v6
  - uses: loonghao/vx@main
    with:
      tools: 'node@22 uv'
      setup: 'true'
      cache: 'true'
  - run: vx npm test
  - run: vx cargo build --release
```

### Container Image

```dockerfile
FROM ghcr.io/loonghao/vx:latest
COPY . .
RUN vx sync && vx run build
```

## Monorepo Support

```toml
# Root vx.toml
[tools]
node = "22"
pnpm = "latest"

[scripts]
install = "pnpm install"
build = "pnpm -r build"
test = "pnpm -r test"
```

Subdirectories can have their own `vx.toml`:

```toml
# packages/backend/vx.toml
[tools]
go = "1.22"

[scripts]
dev = "go run ./cmd/server"
```
