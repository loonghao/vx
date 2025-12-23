# vx Crates

This directory contains all the Rust crates that make up the vx universal development tool manager.

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                           vx-cli                                 │
│                    (Command Line Interface)                      │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         vx-runtime                               │
│           (Provider Registry, Runtime Context, Executor)         │
└─────────────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ vx-installer│ │  vx-core    │ │ vx-resolver │ │  vx-paths   │
│ (Downloads) │ │  (Types)    │ │ (Versions)  │ │  (Paths)    │
└─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         ▼                             ▼                             ▼
┌─────────────────┐           ┌─────────────────┐           ┌─────────────────┐
│  vx-providers   │           │  vx-providers   │           │  vx-providers   │
│     /node       │           │      /go        │           │      /uv        │
│ (Node.js, npm)  │           │  (Go language)  │           │ (UV, Python)    │
└─────────────────┘           └─────────────────┘           └─────────────────┘
```

## Crate Descriptions

### Core Crates

| Crate | Description | When to Use |
|-------|-------------|-------------|
| **vx-cli** | Command-line interface with beautiful UX | Entry point for users |
| **vx-runtime** | Runtime management, provider registry, execution context | Core runtime operations |
| **vx-core** | Core types, traits, and abstractions | Shared types across crates |

### Infrastructure Crates

| Crate | Description | When to Use |
|-------|-------------|-------------|
| **vx-installer** | Universal download and installation engine | Downloading and extracting tools |
| **vx-resolver** | Version parsing, resolution, and comparison | Managing tool versions |
| **vx-paths** | Cross-platform path management | Resolving tool installation paths |

### Tool Providers (vx-providers)

All tool providers are located in `vx-providers/` directory:

| Provider | Tools | Description |
|----------|-------|-------------|
| **node** | `node`, `npm`, `npx` | Node.js JavaScript runtime |
| **bun** | `bun`, `bunx` | Fast JavaScript runtime |
| **deno** | `deno` | Secure JavaScript/TypeScript runtime |
| **go** | `go` | Go programming language |
| **rust** | `cargo`, `rustc`, `rustup` | Rust toolchain |
| **java** | `java`, `javac` | Java Development Kit |
| **zig** | `zig` | Zig programming language |
| **uv** | `uv`, `uvx` | Fast Python package manager |
| **pnpm** | `pnpm`, `pnpx` | Fast, disk-efficient package manager |
| **yarn** | `yarn` | JavaScript package manager |
| **vite** | `vite` | Next generation frontend tooling |
| **just** | `just` | Command runner |
| **terraform** | `terraform` | Infrastructure as Code |
| **kubectl** | `kubectl` | Kubernetes CLI |
| **helm** | `helm` | Kubernetes package manager |
| **vscode** | `code` | Visual Studio Code |
| **rez** | `rez` | Package management system |
| **rcedit** | `rcedit` | Windows resource editor |

## For Provider Developers

If you want to create a new tool provider for vx, implement the `Provider` and `Runtime` traits from `vx-runtime`:

```rust
use vx_runtime::{Provider, Runtime, RuntimeContext, VersionInfo};
use async_trait::async_trait;
use std::sync::Arc;

// 1. Implement the Runtime trait
pub struct MyToolRuntime;

#[async_trait]
impl Runtime for MyToolRuntime {
    fn name(&self) -> &str {
        "mytool"
    }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> anyhow::Result<Vec<VersionInfo>> {
        // Fetch versions from official API
        Ok(vec![VersionInfo::new("1.0.0")])
    }
}

// 2. Implement the Provider trait
pub struct MyToolProvider;

impl Provider for MyToolProvider {
    fn name(&self) -> &str {
        "mytool"
    }

    fn description(&self) -> &str {
        "My Tool description"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MyToolRuntime)]
    }
}
```

See [Provider Development Guide](../docs/advanced/plugin-development.md) for detailed documentation.

## For vx Contributors

### Adding a New Tool

1. Create a new crate under `vx-providers/`:

   ```text
   crates/vx-providers/mytool/
   ├── Cargo.toml
   └── src/
       ├── lib.rs
       ├── provider.rs
       └── runtime.rs
   ```

2. Implement the `Runtime` trait (required methods: `name()`, `fetch_versions()`)

3. Implement the `Provider` trait

4. Register the provider in `vx-cli/src/registry.rs`

5. Add tests in `tests/` directory

### Crate Dependencies

```text
vx-cli
  └── vx-runtime
        ├── vx-installer
        ├── vx-core
        ├── vx-resolver
        └── vx-paths

vx-providers/*
  └── vx-runtime
```

## Versioning

All crates share the same version number defined in the workspace `Cargo.toml`. This ensures compatibility across the ecosystem.

## License

All crates are licensed under the MIT License.
