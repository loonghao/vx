# provider.star — Core API Reference

> ← [Back to Overview](./provider-star-reference.md)

This document covers the execution model, file structure, top-level variables, provider functions, and the `ctx` context object.

---

## Table of Contents

- [1. Execution Model](#1-execution-model)
- [2. File Structure](#2-file-structure)
- [3. Top-Level Variables](#3-top-level-variables)
- [4. Provider Functions](#4-provider-functions)
- [5. Context Object (`ctx`)](#5-context-object-ctx)

---

## 1. Execution Model

vx uses a **two-phase execution model** (inspired by Buck2):

```
┌──────────────────────────────────┐     ┌──────────────────────────────────┐
│  Phase 1 — Analysis (Starlark)   │     │  Phase 2 — Execution (Rust)      │
│                                  │     │                                  │
│  provider.star runs and produces │────▶│  Rust interprets descriptors     │
│  descriptor dicts (pure compute, │     │  and performs real I/O:          │
│  no I/O, no network)            │     │  HTTP, filesystem, processes     │
└──────────────────────────────────┘     └──────────────────────────────────┘
```

Key implications:

| Principle | Detail |
|-----------|--------|
| **No side effects** | Starlark functions return descriptor dicts, they never call the network or filesystem directly |
| **Deterministic** | Given the same `ctx`, a function always returns the same result |
| **JSON round-trip** | All values pass through JSON serialization between Starlark and Rust |
| **`None` = unsupported** | Returning `None` from `download_url()` means "not available on this platform" |

---

## 2. File Structure

Every provider lives in a single directory:

```
crates/vx-providers/<name>/
├── provider.star     # All logic (REQUIRED)
├── provider.toml     # Optional metadata supplement
└── README.md         # Optional documentation
```

User-defined providers follow the same structure under `~/.vx/providers/<name>/`.

---

## 3. Top-Level Variables

These are declared as module-level assignments, **not** functions.

| Variable | Type | Required | Description |
|----------|------|----------|-------------|
| `name` | `string` | **Yes** | Provider name (must match directory name) |
| `description` | `string` | **Yes** | Human-readable description |
| `runtimes` | `list[dict]` | **Yes** | Runtime definitions (see [runtime.star](./provider-star-stdlib.md#62-runtimestar--runtime-definitions)) |
| `permissions` | `dict` | No | Permission declarations (see [permissions.star](./provider-star-stdlib.md#69-permissionsstar--permission-declarations)) |
| `homepage` | `string` | No | Project homepage URL |
| `repository` | `string` | No | Source repository URL |
| `license` | `string` | No | SPDX license identifier (e.g. `"MIT"`, `"Apache-2.0"`) |
| `ecosystem` | `string` | No | Category: `nodejs`, `python`, `rust`, `go`, `devtools`, `system`, `custom`, etc. |
| `package_alias` | `dict` | No | Route to ecosystem package runner (e.g. `{"ecosystem": "uvx", "package": "ruff"}`) |
| `package_prefixes` | `list[string]` | No | Prefixes for package execution (e.g. `["bun", "bunx"]`) |
| `vx_version` | `string` | No | Minimum vx version requirement (e.g. `">=0.7.0"`) |

```python
# Example
name        = "ripgrep"
description = "ripgrep — recursively search directories for a regex pattern"
homepage    = "https://github.com/BurntSushi/ripgrep"
repository  = "https://github.com/BurntSushi/ripgrep"
license     = "MIT OR Unlicense"
ecosystem   = "devtools"
```

---

## 4. Provider Functions

These are module-level functions called by the Rust runtime.

| Function | Signature | Required | Returns |
|----------|-----------|----------|---------|
| `fetch_versions` | `(ctx) → descriptor` | **Yes** | Version list or fetch descriptor |
| `download_url` | `(ctx, version) → string \| None` | **Yes** | Download URL, or `None` if unsupported |
| `install_layout` | `(ctx, version) → dict \| None` | **Yes** | Install layout descriptor |
| `environment` | `(ctx, version) → list[EnvOp]` | **Yes** | Environment variable operations |
| `store_root` | `(ctx) → string` | No | Store root directory path |
| `get_execute_path` | `(ctx, version) → string` | No | Full path to executable |
| `post_install` | `(ctx, version) → dict \| None` | No | Post-install actions |
| `post_extract` | `(ctx, version, install_dir) → list` | No | Post-extract hook actions |
| `pre_run` | `(ctx, args, executable) → list` | No | Pre-run hook actions |
| `deps` | `(ctx, version) → list[DepDef]` | No | Runtime dependency declarations |
| `system_install` | `(ctx) → dict` | No | System package manager strategies |
| `script_install` | `(ctx) → dict` | No | Script-based install commands |
| `uninstall` | `(ctx, version) → bool` | No | Custom uninstall logic |

### Minimal provider skeleton

```python
load("@vx//stdlib:provider.star",
     "runtime_def", "github_permissions",
     "github_rust_provider")

name        = "mytool"
description = "My awesome tool"
ecosystem   = "devtools"

runtimes    = [runtime_def("mytool")]
permissions = github_permissions()

_p = github_rust_provider("owner", "repo",
    asset = "mytool-{vversion}-{triple}.{ext}")

fetch_versions   = _p["fetch_versions"]
download_url     = _p["download_url"]
install_layout   = _p["install_layout"]
store_root       = _p["store_root"]
get_execute_path = _p["get_execute_path"]
environment      = _p["environment"]
```

---

## 5. Context Object (`ctx`)

The `ctx` object is injected by the Rust runtime. It uses Starlark `struct` syntax (dot access).

| Field | Type | Description |
|-------|------|-------------|
| `ctx.name` | `string` | Provider name |
| `ctx.description` | `string` | Provider description |
| `ctx.version` | `string` | Version being processed |
| `ctx.runtime_name` | `string` | Runtime name (for multi-runtime providers) |
| `ctx.version_date` | `string` | Build tag or date of the version |
| `ctx.vx_home` | `string` | vx home directory (`~/.vx`) |
| `ctx.install_dir` | `string` | Version-specific install directory |
| `ctx.platform.os` | `string` | `"windows"` \| `"macos"` \| `"linux"` |
| `ctx.platform.arch` | `string` | `"x64"` \| `"arm64"` \| `"x86"` |
| `ctx.platform.target` | `string` | Rust target triple (e.g. `"x86_64-pc-windows-msvc"`) |
| `ctx.env` | `dict` | Current environment variables |
| `ctx.paths.install_dir` | `string` | Same as `ctx.install_dir` |
| `ctx.paths.vx_home` | `string` | Same as `ctx.vx_home` |
| `ctx.paths.store_dir` | `string` | Global store directory |
| `ctx.paths.cache_dir` | `string` | Cache directory |
| `ctx.paths.download_cache` | `string` | Download cache directory |

### Usage

```python
def download_url(ctx, version):
    key = "{}/{}".format(ctx.platform.os, ctx.platform.arch)
    # ...
```

---

## See Also

- [Standard Library](./provider-star-stdlib.md) — All 14 stdlib modules
- [Layouts & Strategies](./provider-star-layouts.md) — Install layouts, version fetching, hooks
- [Language & Conventions](./provider-star-language.md) — Starlark subset, coding style, new provider checklist
- [Back to Overview](./provider-star-reference.md)
