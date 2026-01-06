# Architecture

Overview of vx's internal architecture.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                        vx CLI                               │
├─────────────────────────────────────────────────────────────┤
│ Commands  │ Config  │ UI  │ Shell Integration               │
├─────────────────────────────────────────────────────────────┤
│                     vx-resolver                             │
│ Version Resolution │ Dependency Graph │ Executor            │
├─────────────────────────────────────────────────────────────┤
│                     vx-runtime                              │
│ Provider Registry │ Manifest Registry │ Runtime Context     │
├─────────────────────────────────────────────────────────────┤
│                     vx-providers                            │
│ Node │ Go │ Rust │ UV │ Deno │ ... (pluggable)              │
├─────────────────────────────────────────────────────────────┤
│                      vx-core                                │
│ Types │ Traits │ Utilities │ Platform Abstraction           │
├─────────────────────────────────────────────────────────────┤
│                      vx-paths                               │
│ Path Management │ Store │ Environments │ Cache              │
└─────────────────────────────────────────────────────────────┘
```

## Crate Structure

### vx-core

Core types and traits shared across all crates.

```
vx-core/
├── src/
│   ├── lib.rs
│   ├── types.rs      # Common types
│   ├── traits.rs     # Core traits
│   ├── error.rs      # Error types
│   └── platform.rs   # Platform detection
```

### vx-paths

Path management and directory structure.

```
vx-paths/
├── src/
│   ├── lib.rs
│   ├── manager.rs    # PathManager
│   ├── store.rs      # Version store
│   ├── envs.rs       # Environments
│   └── cache.rs      # Cache management
```

### vx-runtime

Runtime management and provider registry.

```
vx-runtime/
├── src/
│   ├── lib.rs
│   ├── registry.rs          # ProviderRegistry
│   ├── manifest_registry.rs # ManifestRegistry (manifest-driven)
│   ├── context.rs           # RuntimeContext
│   ├── provider.rs          # Provider trait
│   └── runtime.rs           # Runtime info
```

#### ManifestRegistry

The `ManifestRegistry` provides manifest-driven provider registration, enabling lazy loading and metadata queries:

```rust
// Create registry with factory functions
let mut registry = ManifestRegistry::new();
registry.register_factory("node", || create_node_provider());
registry.register_factory("go", || create_go_provider());

// Build ProviderRegistry from factories
let provider_registry = registry.build_registry_from_factories()?;

// Query metadata without loading provider
if let Some(metadata) = registry.get_runtime_metadata("npm") {
    println!("Provider: {}", metadata.provider_name);
    println!("Ecosystem: {:?}", metadata.ecosystem);
}
```

Benefits:
- **Lazy loading**: Providers created only when needed
- **Metadata access**: Query runtime info without loading providers
- **Extensibility**: Add new providers via manifest files

### vx-resolver

Version resolution and execution with observability.

```
vx-resolver/
├── src/
│   ├── lib.rs
│   ├── resolver.rs        # Version resolver
│   ├── executor.rs        # Command executor (with tracing spans)
│   ├── resolution_cache.rs # Cache with structured logging
│   ├── deps.rs            # Dependency resolution
│   └── spec.rs            # Runtime specifications
```

#### Observability

The executor includes structured tracing for debugging and monitoring:

```rust
// Execution span with structured fields
info_span!("vx_execute",
    runtime = %runtime_name,
    version = version.unwrap_or("latest"),
    args_count = args.len()
)

// Cache logging with structured fields
debug!(
    runtime = %runtime,
    cache_hit = true,
    "Resolution cache hit"
);
```

### vx-cli

Command-line interface.

```
vx-cli/
├── src/
│   ├── lib.rs
│   ├── cli.rs        # Clap definitions
│   ├── commands/     # Command implementations
│   ├── config.rs     # Configuration (re-exports vx-config)
│   ├── registry.rs   # Provider registration
│   └── ui.rs         # User interface
```

### vx-config

Configuration management with security features.

```
vx-config/
├── src/
│   ├── lib.rs
│   ├── parser.rs      # TOML parsing
│   ├── inheritance.rs # Preset inheritance with SHA256 verification
│   └── types.rs       # Configuration types
```

#### Security Features

Remote preset verification:

```rust
// PresetSource with SHA256 verification
impl PresetSource {
    pub fn warn_if_unverified(&self);
    pub fn verify_content(&self, content: &str) -> Result<()>;
    pub fn has_hash_verification(&self) -> bool;
}
```

### vx-extension

Extension system with trust model.

```
vx-extension/
├── src/
│   ├── lib.rs
│   ├── discovery.rs  # Extension discovery with warnings
│   └── loader.rs     # Extension loading
```

#### Extension Trust Model

```rust
impl Extension {
    /// Get extension source information
    pub fn source_info(&self) -> String;
    
    /// Check if extension is from potentially untrusted source
    pub fn is_potentially_untrusted(&self) -> bool;
    
    /// Display warning for untrusted extensions
    pub fn warn_if_untrusted(&self);
}
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
└── ... (34+ providers)
```

Each provider includes a `provider.toml` manifest:

```toml
[provider]
name = "node"
description = "Node.js runtime"

[[runtimes]]
name = "node"
executable = "node"
ecosystem = "nodejs"

[[runtimes]]
name = "npm"
executable = "npm"
ecosystem = "nodejs"
```

## Data Flow

### Command Execution

```
User Input
    │
    ▼
┌─────────┐
│  CLI    │ Parse arguments
└────┬────┘
     │
     ▼
┌─────────┐
│Resolver │ Resolve version, check dependencies
└────┬────┘
     │
     ▼
┌─────────┐
│Provider │ Install if needed
└────┬────┘
     │
     ▼
┌─────────┐
│Executor │ Run command with correct PATH
└────┬────┘
     │
     ▼
  Output
```

### Version Resolution

```
Version Spec (e.g., "node@20")
    │
    ▼
┌──────────────┐
│Parse Spec    │ Extract tool name and version constraint
└──────┬───────┘
       │
       ▼
┌──────────────┐
│Check Store   │ Is version already installed?
└──────┬───────┘
       │
       ▼ (if not installed)
┌──────────────┐
│Fetch List    │ Get available versions from provider
└──────┬───────┘
       │
       ▼
┌──────────────┐
│Match Version │ Find best matching version
└──────┬───────┘
       │
       ▼
┌──────────────┐
│Install       │ Download and install
└──────────────┘
```

## Directory Structure

```
~/.local/share/vx/
├── store/              # Installed tool versions
│   ├── node/
│   │   ├── 18.19.0/
│   │   └── 20.10.0/
│   ├── go/
│   │   └── 1.21.5/
│   └── uv/
│       └── 0.1.24/
├── envs/               # Named environments
│   ├── default/        # Default environment (symlinks)
│   └── my-project/     # Project environment
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

### ConfigView

A flattened view of configuration for simple key-value operations:

```rust
pub struct ConfigView {
    pub tools: HashMap<String, String>,
    pub settings: HashMap<String, String>,
    pub env: HashMap<String, String>,
    pub scripts: HashMap<String, String>,
}

impl From<VxConfig> for ConfigView {
    fn from(config: VxConfig) -> Self { ... }
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

## Security

See [Security](/advanced/security) for details on:
- Remote preset SHA256 verification
- Extension trust model
- Structured logging for auditing
