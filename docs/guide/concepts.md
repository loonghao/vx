# Core Concepts

Understanding these core concepts will help you get the most out of vx.

## Tools and Runtimes

In vx terminology:

- **Tool**: A development tool like Node.js, Python, Go, or Rust
- **Runtime**: A specific version of a tool (e.g., Node.js 20.0.0)
- **Provider**: The component that knows how to install and manage a specific tool

## Version Store

vx maintains a **version store** where all installed tool versions are kept:

```
~/.local/share/vx/
├── store/
�?  ├── node/
�?  �?  ├── 18.19.0/
�?  �?  └── 20.10.0/
�?  ├── go/
�?  �?  └── 1.21.5/
�?  └── uv/
�?      └── 0.1.24/
├── envs/
�?  ├── default/
�?  └── my-project/
└── cache/
```

Multiple versions can coexist without conflicts.

## Environments

An **environment** is a collection of tool versions that work together:

- **Default Environment**: Used when no project config is present
- **Project Environment**: Defined by `vx.toml` in a project
- **Named Environments**: Custom environments you create

```bash
# Create a named environment
vx env create my-env

# Add tools to it
vx env add node@20 --env my-env
vx env add go@1.21 --env my-env

# Use it
vx env use my-env
```

## Auto-Installation

When you run a tool through vx, it automatically:

1. Checks if the tool is installed
2. Installs it if missing (with user consent by default)
3. Runs the command

```bash
# First run - installs Node.js automatically
vx node --version
# Installing node@20.10.0...
# v20.10.0

# Subsequent runs - uses cached version
vx node --version
# v20.10.0
```

## Version Resolution

vx resolves tool versions in this order:

1. **Explicit version**: `vx node@18 --version`
2. **Project config**: `vx.toml` in current or parent directory
3. **Global config**: `~/.config/vx/config.toml`
4. **Latest stable**: If no version specified

### Version Specifiers

```toml
[tools]
node = "20"          # Latest 20.x.x
node = "20.10"       # Latest 20.10.x
node = "20.10.0"     # Exact version
node = "latest"      # Latest stable
node = "lts"         # Latest LTS (for Node.js)
node = "stable"      # Stable channel (for Rust)
```

## Shims vs Direct Execution

vx supports two execution modes:

### Direct Execution (Recommended)

Prefix commands with `vx`:

```bash
vx node script.js
vx npm install
vx go build
```

### Shim Mode

Install shims that intercept tool commands:

```bash
# Install shims
vx shell init bash >> ~/.bashrc

# Now you can run directly
node script.js  # Actually runs through vx
```

## Project Configuration

A `vx.toml` file defines project-specific tool requirements:

```toml
[project]
name = "my-project"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
test = "npm test"
```

When you enter a directory with `vx.toml`, vx automatically uses those tool versions.

## Dependency Resolution

Some tools depend on others. vx handles this automatically:

- `npm` requires `node`
- `cargo` requires `rust`
- `uvx` requires `uv`

When you run a dependent tool, vx ensures the parent tool is installed first.

## Caching

vx caches:

- **Downloaded archives**: Avoid re-downloading
- **Version lists**: Reduce API calls
- **Extracted binaries**: Fast startup

Cache location: `~/.local/share/vx/cache/`

Clear cache with:

```bash
vx clean --cache
```

## Next Steps

- [Direct Execution](/guide/direct-execution) - Using vx for quick tasks
- [Project Environments](/guide/project-environments) - Setting up project configurations
- [Environment Management](/guide/environment-management) - Managing multiple environments
