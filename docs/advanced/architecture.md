# Architecture

Overview of vx's internal architecture.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────�?
�?                        vx CLI                               �?
├─────────────────────────────────────────────────────────────�?
�? Commands  �? Config  �? UI  �? Shell Integration           �?
├─────────────────────────────────────────────────────────────�?
�?                     vx-resolver                             �?
�? Version Resolution �?Dependency Graph �?Executor           �?
├─────────────────────────────────────────────────────────────�?
�?                     vx-runtime                              �?
�? Provider Registry �?Runtime Context �?Environment          �?
├─────────────────────────────────────────────────────────────�?
�?                     vx-providers                            �?
�? Node �?Go �?Rust �?UV �?Deno �?... (pluggable)            �?
├─────────────────────────────────────────────────────────────�?
�?                      vx-core                                �?
�? Types �?Traits �?Utilities �?Platform Abstraction          �?
├─────────────────────────────────────────────────────────────�?
�?                      vx-paths                               �?
�? Path Management �?Store �?Environments �?Cache             �?
└─────────────────────────────────────────────────────────────�?
```

## Crate Structure

### vx-core

Core types and traits shared across all crates.

```
vx-core/
├── src/
�?  ├── lib.rs
�?  ├── types.rs      # Common types
�?  ├── traits.rs     # Core traits
�?  ├── error.rs      # Error types
�?  └── platform.rs   # Platform detection
```

### vx-paths

Path management and directory structure.

```
vx-paths/
├── src/
�?  ├── lib.rs
�?  ├── manager.rs    # PathManager
�?  ├── store.rs      # Version store
�?  ├── envs.rs       # Environments
�?  └── cache.rs      # Cache management
```

### vx-runtime

Runtime management and provider registry.

```
vx-runtime/
├── src/
�?  ├── lib.rs
�?  ├── registry.rs   # ProviderRegistry
�?  ├── context.rs    # RuntimeContext
�?  ├── provider.rs   # Provider trait
�?  └── runtime.rs    # Runtime info
```

### vx-resolver

Version resolution and execution.

```
vx-resolver/
├── src/
�?  ├── lib.rs
�?  ├── resolver.rs   # Version resolver
�?  ├── executor.rs   # Command executor
�?  ├── deps.rs       # Dependency resolution
�?  └── spec.rs       # Runtime specifications
```

### vx-cli

Command-line interface.

```
vx-cli/
├── src/
�?  ├── lib.rs
�?  ├── cli.rs        # Clap definitions
�?  ├── commands/     # Command implementations
�?  ├── config.rs     # Configuration parsing
�?  ├── registry.rs   # Provider registration
�?  └── ui.rs         # User interface
```

### vx-providers

Tool providers (one crate per tool).

```
vx-providers/
├── node/
├── go/
├── rust/
├── uv/
├── deno/
└── ...
```

## Data Flow

### Command Execution

```
User Input
    �?
    �?
┌─────────�?
�?  CLI   �?Parse arguments
└────┬────�?
     �?
     �?
┌─────────�?
�?Resolver�?Resolve version, check dependencies
└────┬────�?
     �?
     �?
┌─────────�?
│Provider �?Install if needed
└────┬────�?
     �?
     �?
┌─────────�?
│Executor �?Run command with correct PATH
└────┬────�?
     �?
     �?
  Output
```

### Version Resolution

```
Version Spec (e.g., "node@20")
    �?
    �?
┌──────────────�?
�?Parse Spec   �?Extract tool name and version constraint
└──────┬───────�?
       �?
       �?
┌──────────────�?
�?Check Store  �?Is version already installed?
└──────┬───────�?
       �?
       �?(if not installed)
┌──────────────�?
�?Fetch List   �?Get available versions from provider
└──────┬───────�?
       �?
       �?
┌──────────────�?
�?Match Version�?Find best matching version
└──────┬───────�?
       �?
       �?
┌──────────────�?
�?Install      �?Download and install
└──────────────�?
```

## Directory Structure

```
~/.local/share/vx/
├── store/              # Installed tool versions
�?  ├── node/
�?  �?  ├── 18.19.0/
�?  �?  └── 20.10.0/
�?  ├── go/
�?  �?  └── 1.21.5/
�?  └── uv/
�?      └── 0.1.24/
├── envs/               # Named environments
�?  ├── default/        # Default environment (symlinks)
�?  └── my-project/     # Project environment
├── cache/              # Downloaded archives, version lists
└── tmp/                # Temporary files
```

## Key Abstractions

### Provider Trait

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    fn info(&self) -> ProviderInfo;
    async fn list_versions(&self) -> Result<Vec<String>>;
    async fn install(&self, version: &str) -> Result<()>;
    fn get_runtime(&self, version: &str) -> Result<RuntimeInfo>;
}
```

### RuntimeSpec

```rust
pub struct RuntimeSpec {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub dependencies: Vec<RuntimeDependency>,
    pub executable: Option<String>,
    pub command_prefix: Vec<String>,
    pub ecosystem: Ecosystem,
}
```

### PathManager

```rust
impl PathManager {
    pub fn version_store_dir(&self, tool: &str, version: &str) -> PathBuf;
    pub fn env_dir(&self, name: &str) -> PathBuf;
    pub fn cache_dir(&self) -> PathBuf;
    pub fn list_store_versions(&self, tool: &str) -> Result<Vec<String>>;
}
```

## Concurrency

- Parallel tool installation using `tokio`
- Async version fetching
- Thread-safe provider registry

## Error Handling

- `anyhow` for error propagation
- Contextual error messages
- User-friendly error display
