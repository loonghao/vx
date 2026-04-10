# VX Architecture Overview

> This document is the **source of truth** for vx's architecture.
> All architectural decisions should be documented here or in [`docs/rfcs/`](../rfcs/).

## System Architecture

```
                          User Command
                              │
                              ▼
                    ┌──────────────────┐
                    │     vx-cli       │  CLI parsing, routing
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
                    │   vx-resolver    │  Resolve runtime, check deps,
                    │   + Executor     │  auto-install, forward command
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼───────┐ ┌───▼──────┐ ┌────▼─────────┐
     │  vx-starlark   │ │vx-runtime│ │ vx-installer  │
     │ (DSL engine)   │ │(registry)│ │ (download &   │
     │                │ │          │ │  extract)     │
     └────────┬───────┘ └──────────┘ └──────────────┘
              │
     ┌────────▼───────┐
<<<<<<< HEAD
     │ provider.star   │  122 Provider definitions
=======
     │ provider.star   │  114 Provider definitions
>>>>>>> origin/main
     │ files           │  (Starlark DSL)
     └────────────────┘
```

## Crate Dependency Graph

### Layer 0: Foundation (no internal deps)

| Crate | Purpose |
|-------|---------|
| `vx-core` | Core traits: `Runtime`, `Provider`, `PackageManager` |
| `vx-paths` | Cross-platform path management (`~/.vx/` structure) |
| `vx-cache` | Caching layer (HTTP responses, version lists) |
| `vx-versions` | Semver parsing and comparison |
| `vx-manifest` | Provider manifest parsing (provider.star metadata) |
| `vx-args` | Argument parsing utilities |

### Layer 1: Infrastructure (depends on Layer 0)

| Crate | Purpose | Key Dependencies |
|-------|---------|-----------------|
| `vx-runtime-core` | Runtime trait extensions | vx-core |
| `vx-runtime-archive` | Archive extraction (tar, zip, xz) | vx-core |
| `vx-runtime-http` | HTTP client wrapper | vx-core, vx-cache |
| `vx-config` | Layered config (built-in → user → project → env) | vx-paths |
| `vx-env` | Environment variable management | vx-paths |
| `vx-console` | Unified output, progress bars, structured logging | — |
| `vx-metrics` | OpenTelemetry tracing & metrics | — |

### Layer 2: Services (depends on Layer 0-1)

| Crate | Purpose | Key Dependencies |
|-------|---------|-----------------|
| `vx-runtime` | Runtime management, `ManifestDrivenRuntime`, `ProviderRegistry` | vx-core, vx-runtime-*, vx-paths |
| `vx-starlark` | Starlark DSL engine, loads `provider.star` | vx-runtime, vx-paths |
| `vx-installer` | Download, verify checksum, extract | vx-runtime-archive, vx-runtime-http |
| `vx-version-fetcher` | Fetch available versions from GitHub/npm/PyPI | vx-cache, vx-runtime-http |
| `vx-system-pm` | System package manager integration (apt, brew, winget) | vx-core |
| `vx-ecosystem-pm` | Ecosystem package managers (npm, pip, cargo) | vx-core |
| `vx-shim` | Shim binary generation | vx-paths |

### Layer 3: Orchestration (depends on Layer 0-2)

| Crate | Purpose | Key Dependencies |
|-------|---------|-----------------|
| `vx-resolver` | Dependency resolution, topological sort, command execution | vx-runtime, vx-installer |
| `vx-setup` | `vx setup` command — install all tools from vx.toml | vx-resolver, vx-config |
| `vx-migration` | Version migration between vx versions | vx-paths, vx-config |
| `vx-extension` | Extension system | vx-runtime, vx-args |
| `vx-project-analyzer` | Detect project type (React, Python, Rust, etc.) | vx-config |

### Layer 4: Application (depends on everything)

| Crate | Purpose |
|-------|---------|
| `vx-cli` | CLI entry point, command routing, user interaction |

### Provider Layer (isolated, parallel to Layer 2-3)

| Directory | Purpose |
|-----------|---------|
<<<<<<< HEAD
| `crates/vx-providers/*` | 122 Provider definitions using `provider.star` Starlark DSL |
=======
| `crates/vx-providers/*` | 114 Provider definitions using `provider.star` Starlark DSL |
>>>>>>> origin/main
| `vx-bridge` | Generic command bridge framework for providers |

## Data Flow: `vx node --version`

```
1. CLI Parse
   vx-cli receives ["node", "--version"]

2. Runtime Lookup
   vx-resolver → ProviderRegistry.find("node")
   → Found: NodeProvider (via provider.star)

3. Dependency Check
   vx-resolver checks node has no unmet dependencies
   (npm/npx are bundled with node, not the other way)

4. Version Resolution
   vx-starlark calls fetch_versions(ctx) from provider.star
   → Returns available versions list
   vx-config resolves which version to use (vx.toml, .vxrc, default)

5. Installation Check
   Check ~/.vx/store/node/<version>/ exists
   If not: download_url(ctx, version) → vx-installer → extract

6. Environment Setup
   vx-starlark calls environment(ctx, version)
   → Returns [env_prepend("PATH", ".../bin")]

7. Command Forwarding
   Execute: /path/to/node --version
   Forward exit code to caller
```

## Storage Layout

```
~/.vx/
├── store/           # Installed tool versions (content-addressable)
│   ├── node/
│   │   ├── 20.0.0/
│   │   └── 22.0.0/
│   └── go/
│       └── 1.23.0/
├── cache/           # Download cache, version lists
│   ├── downloads/
│   └── versions/
├── bin/             # Global shims
├── config/          # User configuration
└── metrics/         # Telemetry data (JSON files)
```

## Key Design Decisions

| Decision | Rationale | RFC |
|----------|-----------|-----|
| Starlark DSL for providers | Zero-compile, declarative, type-safe | [RFC-0036](../rfcs/0036-starlark-provider-support.md) |
| provider.star replaces TOML | Single source of truth, more expressive | [RFC-0038](../rfcs/0038-provider-star-replaces-toml.md) |
| Manifest-driven registration | No Rust code needed for new providers | [RFC-0013](../rfcs/0013-manifest-driven-registration.md) |
| cargo-nextest for testing | 3x faster parallel test execution | — |
| sccache in CI | Reduce compilation time by 40-60% | — |
| Crate-level change detection | Only test affected code in PRs | — |
| Pure Rust TLS (rustls) | No OpenSSL dependency, easier cross-compilation | — |
| workspace-hack (cargo-hakari) | Reduce duplicate dependency compilation | — |

## Cross-Platform Strategy

| Platform | Build Target | TLS | Notes |
|----------|-------------|-----|-------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | rustls | Primary |
| Linux x86_64 (static) | `x86_64-unknown-linux-musl` | rustls | Alpine/Docker |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | rustls | Raspberry Pi, ARM servers |
| Linux ARM64 (static) | `aarch64-unknown-linux-musl` | rustls | Alpine ARM |
| macOS x86_64 | `x86_64-apple-darwin` | rustls | Intel Macs |
| macOS ARM64 | `aarch64-apple-darwin` | rustls | Apple Silicon |
| Windows x86_64 | `x86_64-pc-windows-msvc` | rustls | Primary Windows |
