# RFC 0027: Implicit Package Execution

- **Status**: Implemented
- **Created**: 2026-02-01
- **Updated**: 2026-02-01
- **Related**: RFC 0025 (Cross-Language Package Isolation)

## Summary

Enable implicit package execution similar to `uvx` and `npx`, allowing users to run packages directly without explicit `vx global install` commands. This RFC introduces the `::` separator syntax for specifying executables when the package name differs from the executable name.

## Motivation

Currently, to run a global package, users must:

```bash
# Step 1: Install the package
vx global install npm:typescript@5.3

# Step 2: Run the executable
vx tsc --version
```

This is verbose compared to tools like `uvx` and `npx`:

```bash
# uvx - one command
uvx ruff check .

# npx - one command
npx create-react-app my-app
```

We want to provide a similar experience:

```bash
# Proposed: one command
vx npm:typescript --version
vx pip:ruff check .
```

## Design

### Complete Syntax

```
vx <ecosystem>[@runtime_version]:<package>[@version][::executable] [args...]
```

### Syntax Overview

| Syntax | Description | Example |
|--------|-------------|---------|
| `vx <ecosystem>:<package>` | Run package (default executable) | `vx npm:eslint` |
| `vx <ecosystem>:<package>@<version>` | Run package with specific version | `vx npm:typescript@5.3` |
| `vx <ecosystem>:<package>::<exe>` | Run specific executable from package | `vx npm:typescript::tsc` |
| `vx <ecosystem>@<runtime>:<package>` | Run with specific runtime version | `vx npm@20:typescript` |
| `vx <ecosystem>@<runtime>:<package>::<exe>` | Full specification | `vx npm@20:typescript::tsc` |
| `vx <executable>` | Run installed shim directly | `vx tsc` |

### The `::` Separator

Many packages provide executables with different names than the package itself. The `::` separator allows specifying the exact executable to run:

| Package | Executable | Command |
|---------|------------|---------|
| `typescript` | `tsc` | `vx npm:typescript::tsc` |
| `httpie` | `http` | `vx pip:httpie::http` |
| `@biomejs/biome` | `biome` | `vx npm:@biomejs/biome::biome` |

### Comparison with uvx/npx

| Scenario | uvx | npx | vx |
|----------|-----|-----|-----|
| Package = Executable | `uvx ruff` | `npx eslint` | `vx pip:ruff` |
| Package ≠ Executable | `uvx --from httpie http` | `npx -p typescript tsc` | `vx pip:httpie::http` |
| With version | `uvx ruff@0.3` | `npx eslint@8` | `vx pip:ruff@0.3` |
| Version + different cmd | `uvx --from 'httpie==2.0' http` | - | `vx pip:httpie@2.0::http` |
| Runtime version | `uvx --python 3.11 ruff` | - | `vx pip@3.11:ruff` |

### Detailed Syntax

#### 1. Basic Package Execution

```bash
vx <ecosystem>:<package>[@version] [args...]
```

Examples:
```bash
vx npm:eslint .                   # Run eslint (package name = executable)
vx npm:@biomejs/biome check .     # Scoped npm package
vx pip:black .                    # Python package
vx pip:ruff@0.3 check .           # With version
vx cargo:ripgrep "pattern" ./src  # Rust package
vx go:golangci-lint run           # Go package
```

#### 2. Package with Different Executable Name

```bash
vx <ecosystem>:<package>::<executable> [args...]
```

Examples:
```bash
vx npm:typescript::tsc --version  # Run tsc from typescript package
vx pip:httpie::http GET example.com
vx npm:@typescript/native-preview::tsgo check .
```

#### 3. With Package Version

```bash
vx <ecosystem>:<package>@<version>::<executable> [args...]
```

Examples:
```bash
vx npm:typescript@5.3::tsc --version
vx pip:httpie@3.0::http GET example.com
```

#### 4. Runtime Version Specification

```bash
vx <ecosystem>@<runtime-version>:<package>[@version][::executable] [args...]
```

Examples:
```bash
vx npm@20:typescript::tsc --version  # Use Node.js 20
vx npm@18:create-react-app my-app    # Use Node.js 18
vx pip@3.11:black .                  # Use Python 3.11
vx pip@3.12:ruff check .             # Use Python 3.12
```

#### 3. Executable Alias (Shorthand)

For commonly used packages, support direct executable names:

```bash
vx tsgo                           # → vx npm:@aspect-build/tsgo
vx ruff check .                   # → vx pip:ruff check .
vx rg "pattern"                   # → vx cargo:ripgrep "pattern"
```

This requires a mapping registry (see "Executable Aliases" section).

### Parsing Logic

When vx receives a command:

```
vx <first-arg> [rest...]
```

1. **Check for `:` separator** → Package execution mode
   - Parse `ecosystem[@runtime]:package[@version]`
   - Auto-install if needed, then execute

2. **Check if known runtime** → Runtime execution mode
   - Existing behavior: `vx node@20 --version`

3. **Check if installed shim** → Shim execution mode
   - Execute from `~/.vx/shims/<executable>`

4. **Check executable aliases** → Alias resolution
   - Map `tsgo` → `npm:@aspect-build/tsgo`

5. **Otherwise** → Error with suggestions

### Execution Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    vx npm:typescript@5.3 --version                      │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  1. Parse: ecosystem=npm, package=typescript, version=5.3               │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  2. Check if already installed in ~/.vx/packages/npm/typescript/5.3     │
│     └── If not: auto-install (same as `vx global install`)             │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  3. Resolve executable (tsc from typescript package)                    │
│     └── Use package metadata or detect from bin directory              │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  4. Execute with proper environment                                     │
│     └── Ensure runtime (node) is in PATH, run executable               │
└─────────────────────────────────────────────────────────────────────────┘
```

### Comparison with Existing Tools

| Feature | vx (proposed) | uvx | npx |
|---------|---------------|-----|-----|
| Syntax | `vx pip:ruff` | `uvx ruff` | `npx pkg` |
| Cross-language | ✅ All ecosystems | ❌ Python only | ❌ Node only |
| Runtime version | `vx pip@3.11:ruff` | `uvx --python 3.11 ruff` | N/A |
| Package version | `vx pip:ruff@0.3` | `uvx ruff@0.3` | `npx pkg@ver` |
| Isolation | ✅ Complete | ✅ Complete | ⚠️ Partial |
| Caching | ✅ Persistent | ✅ Cached | ⚠️ Temp |

## Executable Aliases

To support shorthand like `vx tsgo`, we need an alias registry:

```toml
# ~/.vx/config/aliases.toml or built-in
[aliases]
tsgo = "npm:@aspect-build/tsgo"
ruff = "pip:ruff"
black = "pip:black"
rg = "cargo:ripgrep"
fd = "cargo:fd-find"
bat = "cargo:bat"
```

### Auto-Detection

For packages not in the alias registry, vx can:
1. Check if it's an installed shim
2. Search known package registries (npm, PyPI, crates.io)
3. Prompt user to specify ecosystem

## Implementation Plan

### Phase 1: Core Parsing ✅
- [x] Add `PackageRequest` type to parse `ecosystem:package@version::executable`
- [x] Create `vx-shim` crate for shim execution
- [x] Modify CLI entry point to detect package execution syntax

### Phase 2: Execution ✅
- [x] Execute globally installed package shims via `vx <executable>`
- [x] Execute via RFC 0027 syntax `vx ecosystem:package::executable`
- [x] Handle executable resolution from package registry

### Phase 3: Auto-Install ✅
- [x] Auto-install packages on first run (uvx/npx behavior)
- [x] Execute immediately after installation

### Phase 4: Runtime Version Support (Planned)
- [ ] Ensure correct runtime version is used for installation and execution

### Phase 5: Aliases (Planned)
- [ ] Implement alias registry
- [ ] Add built-in aliases for common tools
- [ ] Support user-defined aliases

### Implementation Details

The implementation uses the `vx-shim` crate which provides:

1. **`PackageRequest`**: Parser for the RFC 0027 syntax
   ```rust
   // Parses: npm@20:typescript@5.3::tsc
   let req = PackageRequest::parse("npm@20:typescript@5.3::tsc")?;
   // req.ecosystem = "npm"
   // req.package = "typescript"
   // req.version = Some("5.3")
   // req.executable = Some("tsc")
   // req.runtime_spec = Some(RuntimeSpec { runtime: "node", version: "20" })
   ```

2. **`ShimExecutor`**: Handles shim lookup and execution
   - Looks up executables in the package registry
   - Executes the corresponding shim with proper environment

## Examples

### TypeScript/Node.js

```bash
# Install and run typescript
vx npm:typescript --version

# Run with specific Node.js version
vx npm@20:typescript --version

# Scoped packages
vx npm:@biomejs/biome check .

# Create React App
vx npm:create-react-app my-app
```

### Python

```bash
# Run ruff
vx pip:ruff check .

# Run black with Python 3.11
vx pip@3.11:black .

# Run pytest
vx pip:pytest -v
```

### Rust

```bash
# Run ripgrep
vx cargo:ripgrep "pattern" ./src

# Run with specific version
vx cargo:ripgrep@14 "pattern"
```

### Go

```bash
# Run golangci-lint
vx go:golangci-lint run

# Run gopls
vx go:gopls version
```

## Backward Compatibility

This is purely additive. Existing commands continue to work:
- `vx node@20 --version` → Runtime execution (unchanged)
- `vx global install npm:typescript` → Explicit install (unchanged)
- `vx tsc --version` → Shim execution (unchanged)

## Open Questions

1. **Caching strategy**: Should packages be cached permanently or with TTL?
2. **Default executable**: When a package has multiple executables, which one to run?
3. **Alias conflicts**: How to handle when an alias conflicts with a runtime name?

## References

- [uvx documentation](https://docs.astral.sh/uv/guides/tools/)
- [npx documentation](https://docs.npmjs.com/cli/v10/commands/npx)
- RFC 0025: Cross-Language Package Isolation
