# vx global - Global Package Management

Manage globally installed packages with complete isolation across different ecosystems.

## Overview

The `vx global` command provides a unified interface for installing, managing, and using global packages from multiple ecosystems (npm, pip, cargo, go, gem) without polluting your runtime installations.

**Key Features:**
- 🔒 **Complete Isolation**: Global packages never pollute runtime installations
- 🌍 **Cross-Language Support**: Unified experience across npm, pip, cargo, go, and gem
- 🔗 **Shim-Based Access**: Automatic shim creation for seamless command execution
- 📦 **Version Coexistence**: Multiple versions of the same package can coexist

## Syntax

```bash
vx global <subcommand> [options]
```

## Subcommands

| Subcommand | Alias | Description |
|------------|-------|-------------|
| `install` | - | Install a package globally (isolated) |
| `list` | `ls` | List globally installed packages |
| `uninstall` | `rm` | Uninstall a global package |
| `info` | - | Show information about a global package |
| `shim-update` | - | Update shims after manual changes |

---

## vx global install

Install a package globally with complete isolation.

### Syntax

```bash
vx global install <package-spec> [options]
```

### Package Specification Formats

| Format | Description | Example |
|--------|-------------|---------|
| `package` | Auto-detect ecosystem, latest version | `typescript` |
| `package@version` | Auto-detect ecosystem, specific version | `typescript@5.3` |
| `ecosystem:package` | Explicit ecosystem, latest version | `npm:typescript` |
| `ecosystem:package@version` | Explicit ecosystem and version | `npm:typescript@5.3.3` |

### Supported Ecosystems

| Ecosystem | Aliases | Package Manager | Example |
|-----------|---------|-----------------|---------|
| `npm` | `node` | npm, yarn, pnpm, bun | `npm:typescript@5.3` |
| `pip` | `python`, `pypi`, `uv` | pip, uv | `pip:black@24.1` |
| `cargo` | `rust`, `crates` | cargo | `cargo:ripgrep@14` |
| `go` | `golang` | go install | `go:golangci-lint@1.55` |
| `gem` | `ruby`, `rubygems` | gem | `gem:bundler@2.5` |

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--force` | `-f` | Force reinstallation even if already installed |
| `--verbose` | `-v` | Show detailed installation progress |
| `--` | - | Pass extra arguments to package manager |

### Examples

```bash
# Install npm packages
vx global install typescript@5.3
vx global install npm:eslint
vx global install npm:@biomejs/biome@1.5

# Install Python tools
vx global install pip:black@24.1
vx global install pip:ruff
vx global install uv:pytest  # Uses uv as installer

# Install Rust CLI tools
vx global install cargo:ripgrep@14
vx global install cargo:fd-find
vx global install cargo:bat

# Install Go tools
vx global install go:golangci-lint@1.55
vx global install go:gopls

# Install Ruby gems
vx global install gem:bundler@2.5
vx global install gem:rubocop

# Force reinstall
vx global install typescript@5.3 --force

# Verbose output
vx global install pip:black -v

# Pass extra arguments to package manager
vx global install npm:some-package -- --legacy-peer-deps
```

### Auto-Detection

When ecosystem is not specified, vx automatically detects it based on common package names:

```bash
# These are equivalent:
vx global install typescript@5.3
vx global install npm:typescript@5.3

# These are equivalent:
vx global install black@24.1
vx global install pip:black@24.1

# For unknown packages, specify explicitly:
vx global install npm:my-custom-package
```

---

## vx global list

List all globally installed packages.

### Syntax

```bash
vx global list [options]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--ecosystem <name>` | - | Filter by ecosystem (npm, pip, cargo, go, gem) |
| `--format <format>` | - | Output format: `table` (default), `json`, `plain` |
| `--verbose` | `-v` | Show detailed information including paths |

### Examples

```bash
# List all packages
vx global list
vx global ls

# Filter by ecosystem
vx global list --ecosystem npm
vx global list --ecosystem pip

# Different output formats
vx global list --format json
vx global list --format plain

# Verbose output
vx global list -v
```

### Output Example

```
ECOSYSTEM    PACKAGE                  VERSION      EXECUTABLES
----------------------------------------------------------------------
npm          typescript               5.3.3        tsc, tsserver
npm          eslint                   8.56.0       eslint
pip          black                    24.1.0       black
pip          ruff                     0.3.0        ruff
cargo        ripgrep                  14.0.0       rg
cargo        fd-find                  9.0.0        fd
go           golangci-lint            1.55.0       golangci-lint

Total: 7 package(s)
```

---

## vx global uninstall

Remove a globally installed package.

### Syntax

```bash
vx global uninstall <package-spec> [options]
```

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--force` | `-f` | Skip confirmation prompt |
| `--verbose` | `-v` | Show detailed removal progress |

### Examples

```bash
# Uninstall by name (auto-detect ecosystem from registry)
vx global uninstall typescript
vx global rm eslint

# Explicit ecosystem
vx global uninstall npm:typescript
vx global uninstall pip:black

# Force remove without confirmation
vx global uninstall typescript --force
```

---

## vx global info

Show detailed information about an installed package.

### Syntax

```bash
vx global info <package-or-executable> [options]
```

### Options

| Option | Description |
|--------|-------------|
| `--json` | Output as JSON |

### Examples

```bash
# Query by package name
vx global info typescript
vx global info npm:typescript

# Query by executable name
vx global info tsc
vx global info rg

# JSON output
vx global info typescript --json
```

### Output Example

```
Package: typescript
Version: 5.3.3
Ecosystem: npm
Installed at: 2024-01-15T10:30:00Z
Location: ~/.vx/packages/npm/typescript/5.3.3
Executables: tsc, tsserver
```

---

## vx global shim-update

Manually synchronize shims with the package registry. This is usually not needed as shims are automatically created/removed during install/uninstall.

### Syntax

```bash
vx global shim-update
```

### When to Use

- After manually modifying package directories
- If shims become out of sync
- After system recovery or restoration

---

## Installation Directory Structure

Packages are installed in isolated directories:

```
~/.vx/
├── packages/                    # Global packages
│   ├── npm/
│   │   └── typescript/
│   │       └── 5.3.3/
│   │           ├── node_modules/
│   │           └── bin/
│   │               ├── tsc
│   │               └── tsserver
│   ├── pip/
│   │   └── black/
│   │       └── 24.1.0/
│   │           ├── venv/
│   │           └── bin/
│   │               └── black
│   └── cargo/
│       └── ripgrep/
│           └── 14.0.0/
│               └── bin/
│                   └── rg
│
└── shims/                       # Global shims
    ├── tsc -> ../packages/npm/typescript/5.3.3/bin/tsc
    ├── black -> ../packages/pip/black/24.1.0/bin/black
    └── rg -> ../packages/cargo/ripgrep/14.0.0/bin/rg
```

## Using Installed Tools

After installation, tools are available via shims:

```bash
# Add shims directory to PATH (recommended in shell config)
export PATH="$HOME/.vx/shims:$PATH"

# Now use tools directly
tsc --version
black --check .
rg "pattern" ./src
```

Or run through vx:

```bash
vx tsc --version
vx black --check .
```

## Best Practices

### 1. Specify Ecosystem for Unknown Packages

```bash
# Good: Explicit ecosystem
vx global install npm:my-internal-package

# May fail: Unknown package
vx global install my-internal-package
```

### 2. Pin Versions for Reproducibility

```bash
# Good: Specific version
vx global install typescript@5.3.3

# Less predictable: Latest version
vx global install typescript
```

### 3. Use Preferred Package Managers

```bash
# Python: uv is faster than pip
vx global install uv:black@24.1

# Node.js: npm is default, but you can specify
vx global install npm:typescript
```

### 4. Keep PATH Updated

Add to your shell configuration (`~/.bashrc`, `~/.zshrc`, etc.):

```bash
# Add vx shims to PATH
export PATH="$HOME/.vx/shims:$PATH"
```

## Comparison with Native Package Managers

| Feature | vx global | npm -g | pip | cargo install |
|---------|-----------|--------|-----|---------------|
| Isolation | ✅ Complete | ❌ Pollutes node | ❌ Pollutes Python | ❌ Pollutes ~/.cargo |
| Cross-language | ✅ Unified | ❌ npm only | ❌ pip only | ❌ cargo only |
| Version coexistence | ✅ Multiple versions | ❌ One version | ❌ One version | ❌ One version |
| Shim management | ✅ Automatic | ❌ Manual | ❌ Manual | ❌ Manual |
| Cleanup | ✅ Clean uninstall | ⚠️ May leave files | ⚠️ May leave files | ⚠️ May leave files |

## Troubleshooting

### Shims Not Working

```bash
# Check if shims directory is in PATH
echo $PATH | grep -q ".vx/shims" && echo "OK" || echo "Missing"

# Rebuild shims
vx global shim-update
```

### Package Manager Not Found

```bash
# Ensure runtime is installed
vx install node    # For npm packages
vx install python  # For pip packages
vx install rust    # For cargo packages
```

### Permission Issues

```bash
# Check directory permissions
ls -la ~/.vx/packages/

# Re-create with correct permissions
chmod -R u+rwX ~/.vx/packages/
```

## See Also

- [install](./install) - Install runtime versions
- [list](./list) - List available runtimes
- [env](./env) - Manage environments
