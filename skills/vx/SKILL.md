---
name: vx
description: |
  Teaches AI agents how to use vx, the universal development tool manager.
  Use when the project has vx.toml or .vx/ directory, or when the user mentions
  vx, tool version management, or cross-platform development setup.
  vx transparently manages Node.js, Python, Go, Rust, and 78 tools
  with zero configuration.
---

# VX — Universal Development Tool Manager

vx is a **zero-config universal development tool manager**. Users prefix any command with `vx` and it automatically installs, manages, and forwards to the correct tool version.

## Core Concept

Instead of manually installing tools, just prefix any command with `vx`:

```bash
vx node --version      # Auto-installs Node.js if needed
vx uv pip install x    # Auto-installs uv if needed
vx go build .          # Auto-installs Go if needed
vx cargo build         # Auto-installs Rust if needed
vx just test           # Auto-installs just if needed
```

vx is fully transparent — same commands, same arguments, just add `vx` prefix.

## Essential Commands

### Tool Execution (Most Common)

```bash
vx <tool> [args...]           # Run any tool (auto-installs if missing)
vx node app.js                # Run Node.js
vx python script.py           # Run Python (via uv)
vx npm install                # Run npm
vx npx create-react-app app   # Run npx
vx cargo test                 # Run cargo
vx just build                 # Run just (task runner)
vx git status                 # Run git
```

### Tool Management

```bash
vx install node@22            # Install specific version
vx install uv go rust         # Install multiple tools at once
vx list                       # List all available tools
vx list --installed           # List installed tools only
vx versions node              # Show available versions
vx switch node@20             # Switch active version
vx uninstall go@1.21          # Remove a version
```

### Project Management

```bash
vx init                       # Initialize vx.toml for project
vx sync                       # Install all tools from vx.toml
vx setup                      # Full project setup (sync + hooks)
vx dev                        # Enter dev environment with all tools
vx run test                   # Run project scripts from vx.toml
vx check                      # Verify tool constraints
vx lock                       # Generate vx.lock for reproducibility
```

### Environment & Config

```bash
vx env list                   # List environments
vx config show                # Show configuration
vx cache info                 # Show cache usage
vx search <query>             # Search available tools
vx info                       # System info and capabilities
```

## Project Configuration (vx.toml)

Projects use `vx.toml` in the root directory:

```toml
[tools]
node = "22"         # Major version
go = "1.22"         # Minor version
uv = "latest"       # Always latest
rust = "1.80"       # Specific version
just = "*"          # Any version

[scripts]
dev = "npm run dev"
test = "cargo test"
lint = "npm run lint && cargo clippy"
build = "just build"

[hooks]
pre_commit = ["vx run lint"]
post_setup = ["npm install"]
```

### Version Constraints

| Constraint | Example | Meaning |
|------------|---------|---------|
| Exact | `"1.2.3"` | Only version 1.2.3 |
| Major | `"1"` | Any 1.x.x |
| Minor | `"1.2"` | Any 1.2.x |
| Latest | `"latest"` | Always latest |
| Any | `"*"` | Any available version |
| Range | `">=1.0.0 <2.0.0"` | Range constraint |
| LTS | `"lts"` | Latest LTS version |

### Platform-Specific Tools

```toml
[tools]
node = "22"
uv = "latest"

[tools.msvc]
version = "14.42"
os = ["windows"]

[tools.brew]
version = "latest"
os = ["macos", "linux"]
```

## Using `--with` for Multi-Runtime

When a command needs additional runtimes available:

```bash
vx --with bun node app.js     # Node.js + Bun in PATH
vx --with deno npm test        # npm + Deno available
```

## Package Aliases

vx supports **package aliases** — short commands that automatically route to ecosystem packages:

```bash
# These are equivalent:
vx vite              # Same as: vx npm:vite
vx vite@5.0          # Same as: vx npm:vite@5.0
vx rez               # Same as: vx uv:rez
vx pre-commit        # Same as: vx uv:pre-commit
vx meson             # Same as: vx uv:meson
```

## Supported Tools (78 Providers)

| Category | Tools |
|----------|-------|
| **JavaScript** | node, npm, npx, bun, deno, pnpm, yarn, vite, nx, turbo |
| **JS Tooling** | oxlint |
| **Python** | uv, uvx, python, pip, ruff, maturin, pre-commit |
| **Rust** | cargo, rustc, rustup |
| **Go** | go, gofmt, gws |
| **System/CLI** | git, bash, curl, pwsh, jq, yq, fd, bat, ripgrep, fzf, starship, jj |
| **Build Tools** | just, task, cmake, ninja, make, meson, xmake, protoc, conan, vcpkg, spack |
| **DevOps** | kubectl, helm, podman, terraform, hadolint, dagu |
| **Cloud CLI** | awscli, azcli, gcloud |
| **.NET** | dotnet, msbuild, nuget |
| **C/C++** | msvc, llvm, nasm, ccache, buildcache, sccache, rcedit |
| **Media** | ffmpeg, imagemagick |
| **Java** | java |
| **AI** | ollama, openclaw |
| **Other Langs** | zig |
| **Package Managers** | brew, choco, winget |
| **Misc** | gh, prek, actrun, wix, vscode, xcodebuild, systemctl, release-please, rez, 7zip |

## AI-Optimized Output

All commands support structured output for AI consumption:

```bash
vx list --json              # JSON output
vx list --format toon       # Token-optimized (saves 40-60% tokens)
vx analyze --json           # Analyze project structure
vx check --json             # Verify tool constraints
vx ai context --json        # AI-friendly project context
```

Set default output format:
```bash
export VX_OUTPUT=json       # Default to JSON
export VX_OUTPUT=toon       # Default to TOON (recommended for AI agents)
```

## GitHub Actions Integration

```yaml
name: CI
on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          tools: 'node@22 uv'
          setup: 'true'
          cache: 'true'

      - run: vx node --version
      - run: vx npm test
```

### Action Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `version` | `latest` | vx version to install |
| `github-token` | `${{ github.token }}` | GitHub token for API requests |
| `tools` | `''` | Space-separated tools to pre-install |
| `cache` | `true` | Enable caching of ~/.vx directory |
| `setup` | `false` | Run `vx setup --ci` for vx.toml projects |

## Container Image Support

```dockerfile
FROM ghcr.io/loonghao/vx:latest
RUN vx node --version
RUN vx uv pip install mypackage
```

## Best Practices

### 1. Always Use `vx` Prefix

In vx-managed projects, always prefix tool commands with `vx`:

```bash
# ✅ Correct
vx npm install
vx cargo build
vx just test

# ❌ Wrong (might use system tools with different versions)
npm install
cargo build
just test
```

### 2. Prefer Project-Level Configuration

Use `vx.toml` to ensure consistency across team members:

```bash
# ✅ Define in vx.toml, then sync
vx sync

# ❌ Manual installation per developer
vx install node@22
```

### 3. Commit Lock Files

Always commit `vx.lock` for reproducible builds:

```bash
git add vx.toml vx.lock
git commit -m "chore: pin tool versions"
```

### 4. Version Selection Strategy

| Scenario | Constraint | Example |
|----------|------------|---------|
| Development | Major version | `node = "22"` |
| CI/CD | Exact version | `node = "22.0.0"` |
| Library | Range | `node = ">=18 <23"` |
| Latest features | `"latest"` | `uv = "latest"` |

### 5. Team Onboarding

New team members only need:

```bash
git clone <repo>
cd <repo>
vx setup       # One command installs everything
```

## Troubleshooting

### Quick Diagnostics

```bash
vx doctor                   # Run diagnostics
vx info                     # System info
vx --debug node --version   # Debug mode
```

### Common Solutions

| Issue | Solution |
|-------|----------|
| Download failed | `vx cache clean && vx install <tool>` |
| Wrong version | `vx which <tool>` to check, `vx switch <tool>@<version>` |
| Checksum mismatch | `vx cache clean && vx install <tool> --force` |
| vx.toml not loading | `vx check` to validate configuration |
| Lock file conflicts | `rm vx.lock && vx lock` |

### Complete Reset

```bash
cp -r ~/.vx ~/.vx.backup   # Backup
rm -rf ~/.vx                # Remove
vx setup                    # Reinstall from vx.toml
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Tool not found |
| 3 | Installation failed |
| 4 | Version not found |

## Important Rules for AI Agents

1. **Always use `vx` prefix** when running tools in vx-managed projects
2. **Check `vx.toml` first** to understand project tool requirements
3. **Use `vx run <script>`** for project-defined scripts
4. **Never suggest manual tool installation** — vx handles it automatically
5. **Use `--json` or `--format toon`** for structured output parsing
6. Commands like `vx install` are for explicit pre-installation; normal usage auto-installs
7. **Use `vx just`** instead of `just` for task runner commands
8. **Use `vx npm`/`vx cargo`** instead of bare `npm`/`cargo`

## Links

- GitHub: https://github.com/loonghao/vx
- Documentation: https://github.com/loonghao/vx#readme
- Issues: https://github.com/loonghao/vx/issues
