# Implicit Package Execution

Execute globally installed packages or run packages on-demand without explicit installation, similar to `npx` and `uvx` but with cross-language support.

## Overview

The implicit package execution feature allows you to run packages directly using a unified syntax. Unlike `npx` (Node.js only) or `uvx` (Python only), vx supports multiple ecosystems with a consistent interface.

**Key Benefits:**
- 🚀 **One-Command Execution**: Run packages without prior installation
- 🌍 **Cross-Language**: Works with npm, pip, cargo, go, and more
- 📦 **Auto-Install**: Automatically installs packages on first run
- 🔒 **Isolated**: Each package is installed in its own isolated environment
- 🎯 **Version Control**: Specify exact versions for reproducibility

## Syntax

```
vx <ecosystem>[@runtime_version]:<package>[@version][::executable] [args...]
```

### Syntax Components

| Component | Description | Example |
|-----------|-------------|---------|
| `ecosystem` | Package ecosystem (npm, pip, cargo, go, etc.) | `npm`, `pip` |
| `@runtime_version` | (Optional) Runtime version to use | `@20`, `@3.11` |
| `package` | Package name | `typescript`, `ruff` |
| `@version` | (Optional) Package version | `@5.3`, `@0.3` |
| `::executable` | (Optional) Executable name if different from package | `::tsc`, `::rg` |

## Basic Usage

### Running Installed Tools

Once a package is installed via `vx global install`, you can run it directly:

```bash
# Run installed tool by executable name
vx tsc --version
vx black --check .
vx rg "pattern" ./src
```

### Explicit Package Syntax

Use the full syntax when the package name differs from the executable:

```bash
# Package name ≠ executable name
vx npm:typescript::tsc --version      # typescript package, tsc executable
vx pip:httpie::http GET example.com   # httpie package, http executable
vx cargo:ripgrep::rg "pattern"        # ripgrep package, rg executable
```

### Auto-Detection and Installation

If a package is not installed, vx automatically downloads and installs it:

```bash
# First run - automatically installs typescript
vx npm:typescript --version

# First run - automatically installs ruff
vx pip:ruff check .

# The package is cached for subsequent runs
```

## Supported Ecosystems

| Ecosystem | Aliases | Runtime | Example Package |
|-----------|---------|---------|-----------------|
| `npm` | `node` | Node.js | `npm:typescript` |
| `pip` | `python`, `pypi` | Python | `pip:black` |
| `uv` | - | Python (via uv) | `uv:ruff` |
| `cargo` | `rust`, `crates` | Rust | `cargo:ripgrep` |
| `go` | `golang` | Go | `go:golangci-lint` |
| `bun` | - | Bun | `bun:typescript` |
| `yarn` | - | Node.js | `yarn:typescript` |
| `pnpm` | - | Node.js | `pnpm:typescript` |

## Common Use Cases

### TypeScript/Node.js

```bash
# Compile TypeScript (auto-installs if needed)
vx npm:typescript::tsc --version

# Run ESLint
vx npm:eslint .

# Create React App with specific Node version
vx npm@18:create-react-app my-app

# Run scoped package (@biomejs/biome)
vx npm:@biomejs/biome::biome check .

# Run TypeScript with specific version
vx npm:typescript@5.3::tsc --version
```

### Python

```bash
# Format code with black
vx pip:black .

# Lint with ruff (specific version)
vx pip:ruff@0.3 check .

# Run pytest
vx pip:pytest -v

# Use specific Python version
vx pip@3.11:black .

# Using uv (faster)
vx uv:ruff check .

# HTTP client
vx pip:httpie::http GET example.com
```

### Rust

```bash
# Search with ripgrep
vx cargo:ripgrep::rg "pattern" ./src

# Find files with fd
vx cargo:fd-find::fd ".rs$" .

# Syntax highlighting with bat
vx cargo:bat::bat src/main.rs
```

### Go

```bash
# Run linter
vx go:golangci-lint run

# Run language server
vx go:gopls version
```

## The `::` Separator Explained

Many packages provide executables with different names than the package itself. The `::` separator lets you specify the exact executable:

| Package | Executable | Full Command | Shorthand (if installed) |
|---------|------------|--------------|--------------------------|
| `typescript` | `tsc` | `vx npm:typescript::tsc` | `vx tsc` |
| `typescript` | `tsserver` | `vx npm:typescript::tsserver` | `vx tsserver` |
| `httpie` | `http` | `vx pip:httpie::http` | `vx http` |
| `ripgrep` | `rg` | `vx cargo:ripgrep::rg` | `vx rg` |
| `fd-find` | `fd` | `vx cargo:fd-find::fd` | `vx fd` |
| `bat` | `bat` | `vx cargo:bat::bat` | `vx bat` |

### When to Use `::`

**Use `::` when:**
- Package name differs from executable name (e.g., `typescript` → `tsc`)
- Package has multiple executables (e.g., `typescript` has `tsc` and `tsserver`)
- You want to be explicit about which executable to run

**Skip `::` when:**
- Package name equals executable name (e.g., `eslint`, `ruff`)
- Running via shorthand after installation

## Version Specifications

### Package Versions

```bash
# Latest version (default)
vx npm:typescript --version

# Specific version
vx npm:typescript@5.3 --version

# Version range
vx npm:typescript@^5.0 --version
```

### Runtime Versions

```bash
# Use specific Node.js version
vx npm@20:typescript::tsc --version
vx npm@18:eslint .

# Use specific Python version
vx pip@3.11:black .
vx pip@3.12:ruff check .

# Use latest runtime (default)
vx npm:typescript --version
```

### Combined Specification

```bash
# Full specification: ecosystem@runtime:package@version::executable
vx npm@20:typescript@5.3::tsc --version
# │    │  │          │   │  │
# │    │  │          │   │  └── executable
# │    │  │          │   └───── package version
# │    │  │          └───────── package name
# │    │  └──────────────────── runtime version
# │    └─────────────────────── runtime
# └──────────────────────────── ecosystem
```

## Scoped npm Packages

For npm packages with scopes (@org/package):

```bash
# Biome (JavaScript toolchain)
vx npm:@biomejs/biome::biome check .

# OpenAI Codex
vx npm:@openai/codex::codex

# TypeScript Go implementation
vx npm:@aspect-build/tsgo::tsgo check .
```

## Comparison with Existing Tools

### vx vs npx

| Scenario | npx | vx |
|----------|-----|-----|
| Basic execution | `npx eslint` | `vx npm:eslint` or `vx eslint` (installed) |
| Different executable | `npx -p typescript tsc` | `vx npm:typescript::tsc` |
| Specific version | `npx eslint@8` | `vx npm:eslint@8` |
| Runtime version | ❌ Not supported | `vx npm@20:eslint` |
| Other ecosystems | ❌ Not supported | ✅ pip, cargo, go, etc. |

### vx vs uvx

| Scenario | uvx | vx |
|----------|-----|-----|
| Basic execution | `uvx ruff` | `vx pip:ruff` or `vx ruff` (installed) |
| Different executable | `uvx --from httpie http` | `vx pip:httpie::http` |
| Specific version | `uvx ruff@0.3` | `vx pip:ruff@0.3` |
| Runtime version | `uvx --python 3.11 ruff` | `vx pip@3.11:ruff` |
| Other ecosystems | ❌ Not supported | ✅ npm, cargo, go, etc. |

## Project-Level Configuration

For projects, you can declare commonly used packages in `vx.toml`:

```toml
[tools.global]
typescript = "5.3"
eslint = "8"
black = "24.1"
ruff = "0.3"
```

Then use them directly:

```bash
vx sync    # Install all declared global tools

vx tsc --version    # Uses project's typescript version
vx eslint .
vx black .
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `VX_AUTO_INSTALL` | Enable/disable auto-install (default: `true`) |
| `VX_GLOBAL_CACHE` | Override global packages cache directory |

## Troubleshooting

### "Package not found"

```bash
# Ensure correct ecosystem
vx npm:my-package      # For npm packages
vx pip:my-package      # For Python packages

# Check if package exists
vx global list
```

### "Runtime not installed"

```bash
# Install required runtime first
vx install node        # For npm packages
vx install python      # For pip packages
vx install rust        # For cargo packages
```

### Command Conflicts

If a command conflicts with a runtime name:

```bash
# Use explicit syntax
vx npm:node             # Run 'node' package, not node runtime

# Or use global command
vx global install npm:node
vx node                 # Now runs the package
```

## Best Practices

### 1. Pin Versions for Reproducibility

```bash
# Good: Specific version
vx npm:typescript@5.3.3 --version

# Less predictable: Latest version
vx npm:typescript --version
```

### 2. Use Explicit Syntax in Scripts

```bash
# In CI/CD or shared scripts, be explicit
vx npm:typescript@5.3::tsc --project tsconfig.json
```

### 3. Prefer `vx global install` for Frequently Used Tools

```bash
# Install once, use many times
vx global install npm:typescript@5.3

# Then use shorthand
vx tsc --version
```

### 4. Use `vx dev` for Project Isolation

```bash
# Enter project environment
vx dev

# All tools are available without prefix
tsc --version
black .
ruff check .
```

## Implementation Details

### vx-shim Crate

The `vx-shim` crate implements the RFC 0027 parsing and execution logic:

```rust
// Parse RFC 0027 syntax
let request = PackageRequest::parse("npm@20:typescript@5.3::tsc")?;
// request.ecosystem = "npm"
// request.package = "typescript"
// request.version = Some("5.3")
// request.executable = Some("tsc")
// request.runtime_spec = Some(RuntimeSpec { runtime: "node", version: "20" })
```

**Architecture:**

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         vx-shim Architecture                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  PackageRequest                                                  │   │
│  │  ├── parse(input: &str) -> Result<Self>                         │   │
│  │  ├── is_package_request(input: &str) -> bool                    │   │
│  │  └── executable_name() -> &str                                  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  ShimExecutor                                                    │   │
│  │  ├── execute_request(req, args) -> Result<ExitCode>             │   │
│  │  ├── find_package(ecosystem, package) -> Option<GlobalPackage>  │   │
│  │  └── resolve_executable(package, exe_name) -> PathBuf           │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │  Execution Flow                                                  │   │
│  │  1. Parse request (ecosystem:package@version::executable)        │   │
│  │  2. Check if package is installed in PackageRegistry             │   │
│  │  3. If not installed: return PackageNotInstalled error           │   │
│  │  4. If installed: resolve executable path                        │   │
│  │  5. Execute with runtime in PATH                                 │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Auto-Install Mechanism

When a package is not found, the CLI triggers automatic installation:

```rust
// In vx-cli/src/lib.rs
async fn execute_package_request(ctx, spec, args) {
    match executor.execute_request(&pkg_request, args).await {
        Ok(exit_code) => Ok(()),
        Err(ShimError::PackageNotInstalled { ecosystem, package }) => {
            // Auto-install the package
            auto_install_package(ctx, &pkg_request).await?;
            // Retry execution
            executor.execute_request(&pkg_request, args).await
        }
    }
}
```

This provides a seamless `uvx`/`npx`-like experience:
- First run: Auto-installs and executes
- Subsequent runs: Executes from cache

### Supported Syntax Patterns

| Pattern | Example | Description |
|---------|---------|-------------|
| Simple | `npm:typescript` | Package name = executable |
| With version | `npm:typescript@5.3` | Specific package version |
| Different executable | `npm:typescript::tsc` | Explicit executable name |
| Full syntax | `npm@20:typescript@5.3::tsc` | Runtime + package version + executable |
| Scoped npm | `npm:@biomejs/biome::biome` | Scoped package with executable |
| Runtime version | `pip@3.11:black` | Specific runtime version |

### Parser Implementation

The parser handles edge cases like scoped npm packages:

```rust
// Scoped packages: @org/package@version
if part.starts_with('@') {
    // Handle @types/node or @types/node@1.0
    if let Some(slash_pos) = part.find('/') {
        // Parse scope and package name
        // Handle version after package name
    }
}
```

## See Also

- [`vx global`](./global) - Manage global packages
- [`vx install`](./install) - Install runtime versions
- [RFC 0027: Implicit Package Execution](../rfcs/0027-implicit-package-execution.md)
- [RFC 0025: Cross-Language Package Isolation](../rfcs/0025-cross-language-package-isolation.md)
