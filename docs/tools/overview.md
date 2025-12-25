# Supported Tools Overview

vx supports a wide range of development tools across multiple ecosystems. All tools are automatically installed on first use.

## Tool Categories

### Language Runtimes

| Tool | Commands | Description | Auto-Install |
|------|----------|-------------|--------------|
| `node` | `node`, `npm`, `npx` | Node.js JavaScript runtime | ✅ |
| `bun` | `bun`, `bunx` | Fast all-in-one JavaScript runtime | ✅ |
| `deno` | `deno` | Secure JavaScript/TypeScript runtime | ✅ |
| `go` | `go` | Go programming language | ✅ |
| `rust` | `cargo`, `rustc`, `rustup` | Rust programming language | ✅ |
| `java` | `java`, `javac` | Java Development Kit | ✅ |
| `zig` | `zig` | Zig programming language | ✅ |

### Package Managers

| Tool | Commands | Description | Requires |
|------|----------|-------------|----------|
| `npm` | `npm` | Node.js package manager | node |
| `npx` | `npx` | Node.js package runner | node |
| `pnpm` | `pnpm`, `pnpx` | Fast, disk space efficient package manager | - |
| `yarn` | `yarn` | JavaScript package manager | - |
| `uv` | `uv` | Fast Python package manager | - |
| `uvx` | `uvx` | Python tool runner | uv |
| `cargo` | `cargo` | Rust package manager | rust |

### Build Tools

| Tool | Commands | Description | Auto-Install |
|------|----------|-------------|--------------|
| `vite` | `vite` | Next generation frontend tooling | ✅ |
| `just` | `just` | Command runner for project tasks | ✅ |
| `task` | `task` | Task runner / build tool (go-task) | ✅ |
| `cmake` | `cmake` | Cross-platform build system generator | ✅ |
| `ninja` | `ninja` | Small build system focused on speed | ✅ |
| `protoc` | `protoc` | Protocol Buffers compiler | ✅ |

### DevOps Tools

| Tool | Commands | Description | Auto-Install |
|------|----------|-------------|--------------|
| `docker` | `docker` | Container runtime and tooling | ✅ |
| `terraform` | `terraform` | Infrastructure as Code | ✅ |
| `kubectl` | `kubectl` | Kubernetes CLI | ✅ |
| `helm` | `helm` | Kubernetes package manager | ✅ |

### Cloud CLI Tools

| Tool | Commands | Description | Auto-Install |
|------|----------|-------------|--------------|
| `awscli` | `aws` | Amazon Web Services CLI | ✅ |
| `azcli` | `az` | Microsoft Azure CLI | ✅ |
| `gcloud` | `gcloud` | Google Cloud Platform CLI | ✅ |

### Code Quality Tools

| Tool | Commands | Description | Auto-Install |
|------|----------|-------------|--------------|
| `pre-commit` | `pre-commit` | Pre-commit hook framework | ✅ |

### Other Tools

| Tool | Commands | Description | Auto-Install |
|------|----------|-------------|--------------|
| `vscode` | `code` | Visual Studio Code | ✅ |
| `rez` | `rez` | Package management system | ✅ |
| `rcedit` | `rcedit` | Windows resource editor | ✅ |

## Checking Available Tools

```bash
# List all supported tools
vx list

# Show installation status
vx list --status

# Show details for a specific tool
vx list node
```

## Tool Dependencies

Some tools have dependencies on others:

```
npm, npx → node
cargo, rustc, rustup → rust
uvx → uv
```

vx automatically installs dependencies when needed.

## Version Support

Each tool supports different version specifiers:

```bash
vx install node 20          # Major version
vx install node 20.10       # Minor version
vx install node 20.10.0     # Exact version
vx install node latest      # Latest stable
vx install node lts         # LTS version (Node.js)
vx install rust stable      # Channel (Rust)
```

## Adding New Tools

vx uses a provider-based plugin system for tool support. See [Provider Development](/advanced/plugin-development) for information on adding new tools.
