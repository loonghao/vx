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
│                          vx-core                                 │
│              (Core Engine, Traits, Tool Management)              │
└─────────────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐
│ vx-installer│ │  vx-config  │ │  vx-plugin  │ │ vx-version  │
│ (Downloads) │ │  (Config)   │ │  (Plugins)  │ │ (Versions)  │
└─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘
                                       │
                                       ▼
                              ┌─────────────┐
                              │   vx-sdk    │
                              │ (Tool SDK)  │
                              └─────────────┘
                                       │
         ┌─────────────────────────────┼─────────────────────────────┐
         ▼                             ▼                             ▼
┌─────────────────┐           ┌─────────────────┐           ┌─────────────────┐
│   vx-tools/*    │           │  vx-dependency  │           │    vx-paths     │
│ (Tool Plugins)  │           │ (Dep Resolution)│           │ (Path Handling) │
└─────────────────┘           └─────────────────┘           └─────────────────┘
```

## Crate Descriptions

### Core Crates

| Crate | Description | When to Use |
|-------|-------------|-------------|
| **vx-cli** | Command-line interface with beautiful UX | Entry point for users |
| **vx-core** | Core engine, traits, and tool management | Building core functionality |
| **vx-config** | Configuration management with TOML support | Managing user/project settings |

### Infrastructure Crates

| Crate | Description | When to Use |
|-------|-------------|-------------|
| **vx-installer** | Universal download and installation engine | Downloading and extracting tools |
| **vx-version** | Version parsing, fetching, and comparison | Managing tool versions |
| **vx-paths** | Cross-platform path management | Resolving tool installation paths |
| **vx-dependency** | Dependency resolution engine | Handling tool dependencies |

### Plugin System

| Crate | Description | When to Use |
|-------|-------------|-------------|
| **vx-plugin** | Plugin architecture (legacy API) | Internal plugin system |
| **vx-sdk** | Tool Development SDK (recommended) | Creating new tool plugins |

### Tool Plugins (vx-tools)

| Crate | Description |
|-------|-------------|
| **vx-tool-node** | Node.js runtime and npm/npx support |
| **vx-tool-go** | Go toolchain support |
| **vx-tool-rust** | Rust and Cargo support |
| **vx-tool-uv** | UV Python package manager |
| **vx-tool-pnpm** | pnpm package manager |
| **vx-tool-yarn** | Yarn package manager |
| **vx-tool-bun** | Bun JavaScript runtime |

## For Plugin Developers

If you want to create a new tool plugin for vx, use **vx-sdk**:

```rust
use vx_sdk::{Tool, VersionInfo, Result};
use async_trait::async_trait;

struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "mytool"
    }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        Ok(vec![VersionInfo::new("1.0.0")])
    }
}
```

See [vx-sdk/README.md](vx-sdk/README.md) for detailed documentation.

## For vx Contributors

### Adding a New Tool

1. Create a new crate under `vx-tools/`:

   ```text
   crates/vx-tools/mytool/
   ├── Cargo.toml
   ├── README.md
   └── src/
       ├── lib.rs
       ├── tool.rs
       └── config.rs
   ```

2. Implement the `Tool` trait from `vx-sdk`

3. Register the tool in `vx-core`

4. Add CLI commands in `vx-cli`

### Crate Dependencies

```text
vx-cli
  └── vx-core
        ├── vx-installer
        ├── vx-config
        ├── vx-plugin
        │     └── vx-sdk
        ├── vx-version
        ├── vx-paths
        └── vx-dependency

vx-tools/*
  └── vx-sdk
```

## Versioning

All crates share the same version number defined in the workspace `Cargo.toml`. This ensures compatibility across the ecosystem.

## License

All crates are licensed under the MIT License.
