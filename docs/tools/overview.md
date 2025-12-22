# Supported Tools Overview

vx supports a wide range of development tools across multiple ecosystems.

## Tool Categories

### Language Runtimes

| Tool | Description | Auto-Install |
|------|-------------|--------------|
| `node` | Node.js JavaScript runtime | âœ?|
| `go` | Go programming language | âœ?|
| `rust` | Rust programming language | âœ?|
| `deno` | Secure JavaScript/TypeScript runtime | âœ?|
| `bun` | All-in-one JavaScript runtime | âœ?|
| `java` | Java Development Kit | âœ?|
| `zig` | Zig programming language | âœ?|

### Package Managers

| Tool | Description | Requires |
|------|-------------|----------|
| `npm` | Node.js package manager | node |
| `npx` | Node.js package runner | node |
| `pnpm` | Fast, disk space efficient package manager | - |
| `yarn` | JavaScript package manager | - |
| `uv` | Fast Python package manager | - |
| `uvx` | Python tool runner | uv |
| `cargo` | Rust package manager | rust |

### Build Tools

| Tool | Description | Auto-Install |
|------|-------------|--------------|
| `vite` | Next generation frontend tooling | âœ?|
| `just` | Command runner | âœ?|

### DevOps Tools

| Tool | Description | Auto-Install |
|------|-------------|--------------|
| `terraform` | Infrastructure as Code | âœ?|
| `kubectl` | Kubernetes CLI | âœ?|
| `helm` | Kubernetes package manager | âœ?|

### Other Tools

| Tool | Description | Auto-Install |
|------|-------------|--------------|
| `vscode` | Visual Studio Code | âœ?|
| `rez` | Package management system | âœ?|
| `rcedit` | Windows resource editor | âœ?|

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
npm, npx â†?node
cargo, rustc, rustup â†?rust
uvx â†?uv
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

vx uses a plugin system for tool support. See [Plugin Development](/advanced/plugin-development) for information on adding new tools.
