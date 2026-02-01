# RFC 0026: Unified Runtime Provider Relationships

## Status

- **RFC**: 0026
- **Status**: Draft
- **Created**: 2026-01-31
- **Updated**: 2026-02-01
- **Author**: vx team

## Summary

This RFC documents and unifies the runtime provider relationships model in `provider.toml`. After analyzing existing providers, we found that the **current syntax is already well-designed** - this RFC focuses on:

1. **Documenting** the semantic differences between existing syntaxes
2. **Eliminating redundancy** in provider definitions
3. **Adding automatic inference rules** to reduce boilerplate
4. **Preserving version-ranged constraints** (already working in yarn/pnpm)

## Current State Analysis

### Three Existing Syntaxes

After analyzing all 40+ providers, we identified three syntaxes with distinct semantics:

```toml
# Syntax 1: bundled_with - Executable distributed together
[[runtimes]]
name = "npm"
bundled_with = "node"     # npm.exe is inside node's installation

# Syntax 2: runtime_dependency - Installed/managed by another tool
[[runtimes]]
name = "cargo"
[runtimes.runtime_dependency]
runtime = "rustup"        # cargo is installed via `rustup component add`

# Syntax 3: constraints.requires - Runtime dependency with version ranges
[[runtimes.constraints]]
when = ">=4"
requires = [{ runtime = "node", version = ">=18" }]  # yarn 4.x needs node 18+
```

### Existing Usage Patterns

| Syntax | Used By | Semantics |
|--------|---------|-----------|
| `bundled_with` | node(npm,npx,corepack), go(gofmt), bun(bunx), uv(uvx), python(pip), rez(rez-build), systemctl(journalctl) | Executable is **physically bundled** with parent runtime; version inherits automatically |
| `runtime_dependency` | rust(cargo,rustc→rustup) | Tool is **managed/installed** by another tool; has independent versioning |
| `constraints.requires` | yarn, pnpm, pip, gofmt | **Runtime dependency** with version constraints; supports version ranges |

### Key Insight: Version-Ranged Constraints Already Work

The yarn and pnpm providers already implement sophisticated version-ranged constraints:

```toml
# yarn/provider.toml - Different node requirements per yarn version
[[runtimes.constraints]]
when = "^1"
requires = [{ runtime = "node", version = ">=12, <23", recommended = "20" }]

[[runtimes.constraints]]
when = ">=2, <4"
requires = [{ runtime = "node", version = ">=16", recommended = "20" }]

[[runtimes.constraints]]
when = ">=4"
requires = [{ runtime = "node", version = ">=18", recommended = "22" }]
```

This pattern should be the **canonical way** to express version-dependent runtime requirements.

## Design Decisions

### Decision 1: Keep Existing Syntax

The current syntax is:
- **Concise**: `bundled_with = "node"` is clear and minimal
- **Semantic**: Each syntax has distinct meaning
- **Already implemented**: No code changes needed for basic functionality

**No syntax migration required.**

### Decision 2: Eliminate Redundant Declarations

Currently, some providers have redundant declarations:

```toml
# go/provider.toml - CURRENT (redundant)
[[runtimes]]
name = "gofmt"
bundled_with = "go"              # ← This implies...

[[runtimes.constraints]]
when = "*"
requires = [{ runtime = "go" }]  # ← ...this, so it's redundant
```

**New Rule**: If a runtime has `bundled_with = "X"`, the constraint `requires = [{ runtime = "X" }]` is **automatically inferred** and should not be declared.

```toml
# go/provider.toml - SIMPLIFIED
[[runtimes]]
name = "gofmt"
bundled_with = "go"
# constraints.requires is automatically inferred
```

### Decision 3: Automatic Inference Rules

The runtime resolver should automatically infer:

| If Has | Automatically Implies |
|--------|----------------------|
| `bundled_with = "X"` | `requires = [{ runtime = "X", version = "*" }]` |
| `runtime_dependency.runtime = "X"` | `requires = [{ runtime = "X", version = "*" }]` |

**Benefits**:
1. Reduces boilerplate in provider.toml
2. Prevents inconsistency between declaration and constraint
3. Simplifies provider authoring

### Decision 4: Semantic Clarification

| Concept | Syntax | Version Inheritance | Installation |
|---------|--------|---------------------|--------------|
| Bundled | `bundled_with` | Yes (same as parent) | No (comes with parent) |
| Managed | `runtime_dependency` | No (independent) | Yes (via manager) |
| Required | `constraints.requires` | No (independent) | Yes (separately) |

## Implementation

### Phase 1: Add Automatic Inference

Update `RuntimeMap::runtime_def_to_spec()` to infer constraints:

```rust
// In vx-resolver/src/runtime_map.rs
impl RuntimeMap {
    fn runtime_def_to_spec(&self, runtime: &RuntimeDef) -> RuntimeSpec {
        let mut spec = RuntimeSpec::new(runtime.name.clone());

        // Infer constraints from bundled_with
        if let Some(ref parent) = runtime.bundled_with {
            spec.effective_runtime = Some(parent.clone());
            spec.version_inherit = true;

            // Auto-add implied constraint (no need to declare in TOML)
            spec.add_implied_constraint(RuntimeConstraint {
                runtime: parent.clone(),
                version: "*".into(),
                implied: true,  // Mark as auto-inferred
            });
        }

        // Infer constraints from runtime_dependency
        if let Some(ref dep) = runtime.runtime_dependency {
            spec.effective_runtime = Some(dep.runtime.clone());
            spec.version_inherit = false;

            spec.add_implied_constraint(RuntimeConstraint {
                runtime: dep.runtime.clone(),
                version: "*".into(),
                implied: true,
            });
        }

        spec
    }
}
```

### Phase 2: Clean Up Redundant Declarations

Remove redundant `constraints.requires` from providers that have `bundled_with`:

**Files to update:**
- `go/provider.toml`: Remove gofmt's redundant constraint
- `rez/provider.toml`: Remove rez-build's redundant constraint
- `bun/provider.toml`: Remove bunx's redundant constraint
- `uv/provider.toml`: Remove uvx's redundant constraint

### Phase 3: Documentation

Update `docs/guide/manifest-driven-providers.md` with:
1. Clear explanation of when to use each syntax
2. Automatic inference rules
3. Version-ranged constraint examples

## Version Semantics for Bundled Tools

### Important: Version Inheritance

When a tool has `bundled_with = "X"`, the version specifier refers to the **parent runtime**, not the tool itself.

**Example: `vx npm@20`**

```
User runs:     vx npm@20 --version
Output:        10.8.2

Explanation:
- @20 refers to node version 20, not npm version 20
- npm 10.8.2 is the version bundled with node 20.20.0
- npm does NOT have version 20.x
```

This is the **correct behavior** because:
1. npm is physically bundled inside the node installation
2. Users typically want "the npm that comes with node 20"
3. npm's own version (10.x) is an implementation detail

### Version Matrix for Bundled Tools

| Command | Meaning | Actual Versions |
|---------|---------|-----------------|
| `vx npm@20` | npm from node 20 | npm 10.8.2 (from node 20.20.0) |
| `vx npm@18` | npm from node 18 | npm 10.2.3 (from node 18.x) |
| `vx npx@20` | npx from node 20 | npx 10.8.2 (same as npm) |
| `vx bunx@1` | bunx from bun 1 | bun 1.x with "x" subcommand |
| `vx gofmt@1.21` | gofmt from go 1.21 | gofmt bundled with go 1.21.x |

### Independent vs Inherited Versions

| Pattern | Version Behavior | Example |
|---------|-----------------|---------|
| `bundled_with` | **Inherits** parent version | `vx npm@20` → npm from node 20 |
| `runtime_dependency` | **Independent** version | `vx cargo@1.75` → cargo 1.75 |
| standalone | **Own** version | `vx yarn@4` → yarn 4.x |

### Command Prefix Pattern (bunx, uvx)

Some bundled tools are **command aliases** rather than separate executables:

```toml
# bunx is NOT a separate executable
# It runs: bun x <args>
[[runtimes]]
name = "bunx"
executable = "bun"           # Uses bun executable
bundled_with = "bun"
command_prefix = ["x"]       # Prepends "x" to arguments
```

**Execution flow:**
```
User runs:      vx bunx@1 create-react-app my-app
vx translates:  bun x create-react-app my-app
```

**Other examples:**
- `uvx` could use `command_prefix = ["run"]` → `uv run <args>` (if designed this way)
- Future: `pip-run` could use `command_prefix = ["run"]` → `pip run <args>`

## Examples

### Bundled Tool (Simple)

```toml
# npm is bundled with node - no constraints needed
[[runtimes]]
name = "npm"
executable = "npm"
bundled_with = "node"
# Automatically infers: requires node, version inherits from node
```

### Managed Tool

```toml
# cargo is managed by rustup - no constraints needed
[[runtimes]]
name = "cargo"
executable = "cargo"
[runtimes.runtime_dependency]
runtime = "rustup"
# Automatically infers: requires rustup
```

### Standalone Tool with Version-Ranged Dependencies

```toml
# yarn requires node, with different versions per yarn version
[[runtimes]]
name = "yarn"
executable = "yarn"

[[runtimes.constraints]]
when = "^1"
requires = [{ runtime = "node", version = ">=12, <23" }]

[[runtimes.constraints]]
when = ">=4"
requires = [{ runtime = "node", version = ">=18" }]
```

### Bundled Tool with Command Prefix

```toml
# bunx uses bun executable with "x" prefix
[[runtimes]]
name = "bunx"
executable = "bun"
bundled_with = "bun"
command_prefix = ["x"]  # bunx args -> bun x args
```

## Migration Guide

### For Existing Providers

1. **Remove redundant constraints** if you have both `bundled_with` and `constraints.requires` for the same runtime:

```diff
 [[runtimes]]
 name = "gofmt"
 bundled_with = "go"

-[[runtimes.constraints]]
-when = "*"
-requires = [{ runtime = "go" }]
```

2. **Keep version-ranged constraints** - they are not redundant:

```toml
# This is NOT redundant - it specifies version ranges
[[runtimes.constraints]]
when = ">=4"
requires = [{ runtime = "node", version = ">=18" }]
```

### For New Providers

Choose syntax based on relationship type:

| Relationship | Use |
|--------------|-----|
| Executable shipped together | `bundled_with = "parent"` |
| Installed by another tool | `[runtimes.runtime_dependency]` |
| Needs runtime with version constraints | `[[runtimes.constraints]]` |

## Alternatives Considered

### Alternative 1: New `provided_by` Unified Syntax

```toml
[runtimes.provided_by]
runtime = "node"
type = "bundled"
version_inherit = true
```

**Rejected**: More verbose than current `bundled_with = "node"`, with no additional expressiveness for current use cases.

### Alternative 2: Merge All into `constraints.requires`

**Rejected**: Loses semantic distinction between "bundled" (no separate installation) and "requires" (separate installation needed).

## Extended Vision: Additional Software Forms and Combinations

This section explores software forms and combinations that are not yet well-supported but should be considered for future extensibility. Examples use the **current syntax** with potential extensions.

### 1. Container and Virtual Environment Runtimes

| Tool | Form | Relationship | Notes |
|------|------|--------------|-------|
| Docker | Container runtime | Self-contained | Runs isolated containers |
| Podman | Container runtime | Alias of Docker | CLI-compatible with Docker |
| nerdctl | Container runtime | Bundled with containerd | CLI for containerd |
| devcontainer | Dev environment | Requires Docker/Podman | VS Code devcontainers |

```toml
# Future: Container-based tools using constraints
[[runtimes]]
name = "devcontainer"
executable = "devcontainer"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "docker", version = "*", alternatives = ["podman"] }
]

# Future extension: container execution context
[runtimes.container]
image = "mcr.microsoft.com/devcontainers/base"
```

### 2. Language Version Managers (Meta-Managers)

These tools manage multiple versions of language runtimes, creating a hierarchy of relationships:

| Manager | Manages | Pattern |
|---------|---------|---------|
| pyenv | python versions | Installs multiple Python versions |
| nvm | node versions | Installs multiple Node.js versions |
| rbenv | ruby versions | Installs multiple Ruby versions |
| jenv | java versions | Switches between installed JDKs |
| goenv | go versions | Installs multiple Go versions |
| sdkman | JVM tools | Manages Java, Kotlin, Gradle, etc. |

```toml
# Future: Version manager using runtime_dependency
[[runtimes]]
name = "python"
executable = "python"

[runtimes.runtime_dependency]
runtime = "pyenv"
version_selection = "local"     # Use .python-version file
fallback = "system"             # Fall back to system Python
```

### 3. Build Tool Chains

Complex tools that orchestrate multiple other tools:

| Tool | Dependencies | Pattern |
|------|--------------|---------|
| CMake | Compiler (gcc/clang/msvc), make/ninja | Build system generator |
| Meson | Python, ninja, compiler | Modern build system |
| Bazel | Java, various compilers | Polyglot build system |
| Buck2 | Rust-based, various compilers | Meta build system |
| xmake | Lua embedded | Cross-platform build |

```toml
# Future: Build chain with optional backends using constraints
[[runtimes]]
name = "cmake"
executable = "cmake"

[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "ninja", version = "*", reason = "Fast parallel builds" }
]

# Platform-specific optional requirements
[[runtimes.constraints]]
when = "*"
platforms = ["windows"]
recommends = [
    { runtime = "msvc", version = "*", reason = "Native Windows compilation" }
]
```

### 4. IDE and Editor Plugins/Extensions

| Tool | Host | Pattern |
|------|------|---------|
| rust-analyzer | VS Code, Neovim, etc. | Language server |
| pylsp | Any LSP client | Language server |
| gopls | Any LSP client | Language server |
| clangd | Any LSP client | C/C++ language server |
| ESLint | Node.js + editor | Linting service |

```toml
# Future: Language server as optional component
[[runtimes]]
name = "rust-analyzer"
executable = "rust-analyzer"

[runtimes.runtime_dependency]
runtime = "rustup"
component = "rust-analyzer"     # Future: component installation
install_command = "rustup component add rust-analyzer"
```

### 5. Package Managers with Multiple Backends

| Tool | Backends | Pattern |
|------|----------|---------|
| pip | PyPI | Python packages |
| pipx | pip + venv | Isolated Python apps |
| uv | Custom Rust resolver | Fast Python package management |
| conda | Anaconda channels | Cross-language packages |
| mamba | conda-compatible | Fast conda alternative |
| pixi | conda-compatible | Modern conda replacement |

```toml
# Future: Alternative implementations using constraints
[[runtimes]]
name = "pip"
executable = "pip"
bundled_with = "python"

# Future extension: alternatives field
[runtimes.alternatives]
preferred = "uv"                # Use uv if available
fallback = "pip"                # Original implementation
```

### 6. Wasm and Cross-Compilation Targets

| Tool | Host | Target | Pattern |
|------|------|--------|---------|
| wasm-pack | Rust | WebAssembly | Bundler for Rust→Wasm |
| emscripten | C/C++ | WebAssembly | Compiler to Wasm |
| wasmtime | Host | Wasm runtime | Wasm execution |
| wasmer | Host | Wasm runtime | Wasm execution |
| cargo-wasi | Rust | WASI | WASI target |

```toml
# Future: Cargo plugin using constraints
[[runtimes]]
name = "wasm-pack"
executable = "wasm-pack"

[[runtimes.constraints]]
when = "*"
requires = [{ runtime = "cargo", version = "*" }]

# Future extension: installation method
[runtimes.install]
method = "cargo"
command = "cargo install wasm-pack"
```

### 7. Database and Service Managers

| Tool | Type | Pattern |
|------|------|---------|
| pg_ctl | PostgreSQL control | Database manager |
| mysql_safe | MySQL control | Database manager |
| redis-server | Redis | In-memory store |
| mongod | MongoDB | Document database |

```toml
# Future: Service runtime using bundled_with
[[runtimes]]
name = "psql"
executable = "psql"
bundled_with = "postgresql"

# Future extension: service management
[runtimes.service]
start_command = "pg_ctl start"
stop_command = "pg_ctl stop"
port = 5432
```

### 8. AI/ML Frameworks with Hardware Dependencies

| Tool | Dependencies | Pattern |
|------|--------------|---------|
| PyTorch | Python, CUDA/ROCm/MPS | GPU-accelerated ML |
| TensorFlow | Python, CUDA | GPU-accelerated ML |
| JAX | Python, CUDA/TPU | Composable transforms |
| ONNX Runtime | Multiple backends | Inference runtime |

```toml
# Future: Hardware-dependent runtime using constraints
[[runtimes]]
name = "pytorch"
executable = "python -c 'import torch'"

[[runtimes.constraints]]
when = "*"
requires = [{ runtime = "python", version = ">=3.8" }]

# Future extension: hardware requirements
[runtimes.hardware]
gpu = "optional"                # CUDA, ROCm, or MPS
cuda_versions = ["11.8", "12.1"]
```

### 9. Monorepo and Workspace Tools

| Tool | Ecosystem | Pattern |
|------|-----------|---------|
| Turborepo | Node.js | Monorepo build orchestrator |
| Nx | Node.js | Monorepo dev toolkit |
| Lerna | Node.js | Package publishing |
| Rush | Node.js | Enterprise monorepo |
| cargo workspace | Rust | Built-in workspace |
| uv workspace | Python | Poetry-like workspaces |

```toml
# Future: Workspace-aware tools using constraints
[[runtimes]]
name = "turbo"
executable = "turbo"

[[runtimes.constraints]]
when = "*"
requires = [{ runtime = "node", version = ">=18" }]

# Future extension: workspace configuration
[runtimes.workspace]
config_file = "turbo.json"
monorepo_aware = true
```

### 10. Shell and Script Interpreters

| Tool | Type | Pattern |
|------|------|---------|
| bash | Shell | Built-in on Unix |
| zsh | Shell | macOS default |
| fish | Shell | Modern shell |
| PowerShell | Shell | Cross-platform |
| nushell | Shell | Structured data shell |

```toml
# Future: Shell interpreter (standalone runtime)
[[runtimes]]
name = "nushell"
executable = "nu"
aliases = ["nu"]

# Future extension: script execution
[runtimes.script]
shebang_pattern = "#!/usr/bin/env nu"
file_extensions = [".nu"]
```

### 11. Remote Execution and Distributed Systems

| Tool | Type | Pattern |
|------|------|---------|
| SSH | Remote execution | Execute on remote host |
| kubectl exec | Kubernetes | Execute in pod |
| docker exec | Container | Execute in container |
| Ansible | Automation | Remote orchestration |
| Dagger | CI/CD | Container-based pipelines |

```toml
# Future: Remote execution context (standalone runtime)
[[runtimes]]
name = "kubectl"
executable = "kubectl"

# Future extension: remote context
[runtimes.context]
type = "kubernetes"
namespace_env = "KUBECTL_NAMESPACE"
config_env = "KUBECONFIG"
```

### 12. Future Extension Points Summary

Based on the analysis above, the following extension points may be added to provider.toml in the future:

| Extension | Purpose | Example |
|-----------|---------|---------|
| `[runtimes.alternatives]` | Drop-in replacements | uv for pip |
| `[runtimes.container]` | Container execution | devcontainer |
| `[runtimes.hardware]` | Hardware requirements | CUDA for pytorch |
| `[runtimes.workspace]` | Monorepo awareness | turbo.json |
| `[runtimes.script]` | Script execution | shebang patterns |
| `[runtimes.context]` | Execution context | Kubernetes namespaces |
| `[runtimes.service]` | Service management | Database start/stop |
| `[runtimes.install]` | Custom installation | cargo install |

These extensions would **complement** the existing `bundled_with`, `runtime_dependency`, and `constraints` fields, not replace them.

## References

- RFC 0017: Declarative Runtime Map
- RFC 0018: Extended Provider Schema
- RFC 0012: Provider Manifest
- [mise tool documentation](https://mise.jdx.dev/)
- [asdf plugin structure](https://asdf-vm.com/)
- [Nix derivations](https://nixos.org/manual/nix/stable/)
- [devcontainer specification](https://containers.dev/)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org/)
- [pixi documentation](https://pixi.sh/)

