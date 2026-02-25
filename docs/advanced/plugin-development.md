# Provider Development Guide

This guide explains how to create a new provider for vx. The recommended approach is
**Starlark-first**: write a `provider.star` file and let vx handle the rest. For advanced
use cases that require custom Rust logic, see the [Custom Rust Provider](#custom-rust-provider) section.

## Two Approaches

| Approach | When to Use | Effort |
|----------|-------------|--------|
| **`provider.star`** (recommended) | GitHub releases, archive/binary downloads, PyPI/npm tools, system package manager fallback | Minutes |
| **Custom Rust Provider** | Custom install logic, complex version parsing, non-standard protocols | Hours |

---

## Approach 1: provider.star (Recommended)

For the vast majority of tools, a `provider.star` file is all you need.
See the [Manifest-Driven Providers Guide](../guide/manifest-driven-providers.md) for the
complete reference. Here is a condensed walkthrough.

### Minimal Example

```python
# crates/vx-providers/mytool/provider.star
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# --- Metadata ---
name        = "mytool"
description = "My awesome tool"
homepage    = "https://github.com/myorg/mytool"
repository  = "https://github.com/myorg/mytool"
license     = "MIT"
ecosystem   = "devtools"

runtimes = [
    {
        "name":        "mytool",
        "executable":  "mytool",
        "description": "My tool runtime",
        "priority":    100,
        "test_commands": [
            {"command": "{executable} --version", "name": "version_check"},
        ],
    },
]

permissions = {
    "http": ["api.github.com", "github.com"],
    "fs":   [],
    "exec": [],
}

# --- Logic ---
fetch_versions = make_fetch_versions("myorg", "mytool")

def download_url(ctx, version):
    os   = ctx.platform.os
    arch = ctx.platform.arch
    triples = {
        "windows/x64":  "x86_64-pc-windows-msvc",
        "macos/x64":    "x86_64-apple-darwin",
        "macos/arm64":  "aarch64-apple-darwin",
        "linux/x64":    "x86_64-unknown-linux-musl",
        "linux/arm64":  "aarch64-unknown-linux-gnu",
    }
    triple = triples.get("{}/{}".format(os, arch))
    if not triple:
        return None
    ext   = "zip" if os == "windows" else "tar.gz"
    asset = "mytool-{}-{}.{}".format(version, triple, ext)
    return github_asset_url("myorg", "mytool", "v" + version, asset)

def install_layout(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return {
        "type":             "archive",
        "strip_prefix":     "mytool-{}".format(version),
        "executable_paths": [exe, "mytool"],
    }

def store_root(ctx):
    return ctx.vx_home + "/store/mytool"

def get_execute_path(ctx, version):
    os  = ctx.platform.os
    exe = "mytool.exe" if os == "windows" else "mytool"
    return ctx.install_dir + "/" + exe

def post_install(_ctx, _version):
    return None

def environment(ctx, _version):
    return [env_prepend("PATH", ctx.install_dir)]
```

### Required Files

```
crates/vx-providers/mytool/
├── provider.star     # All logic and metadata (required)
├── Cargo.toml        # Package configuration
├── src/lib.rs        # Minimal Rust shim
└── build.rs          # Rebuild trigger for provider.star changes
```

**`provider.star`** — metadata and logic combined:

```python
# provider.star
name        = "mytool"
description = "My awesome tool"
homepage    = "https://github.com/myorg/mytool"
repository  = "https://github.com/myorg/mytool"
ecosystem   = "devtools"
license     = "MIT"

runtimes = [
    runtime_def("mytool"),
]

permissions = github_permissions()

def fetch_versions(ctx):
    return fetch_versions_from_github(ctx, "myorg", "mytool")

def download_url(ctx, version, platform):
    return github_asset_url(ctx, "myorg", "mytool", version, platform)
```

### Required Functions Checklist

Every `provider.star` must implement:

| Function | Signature | Notes |
|----------|-----------|-------|
| `fetch_versions` | `fetch_versions(ctx)` or `make_fetch_versions(...)` | Returns version list |
| `download_url` | `download_url(ctx, version) -> str\|None` | Returns download URL |
| `install_layout` | `install_layout(ctx, version) -> dict\|None` | Returns install descriptor |
| `store_root` | `store_root(ctx) -> str` | Returns store path |
| `get_execute_path` | `get_execute_path(ctx, version) -> str` | Returns executable path |
| `post_install` | `post_install(ctx, version) -> None` | Post-install hook |
| `environment` | `environment(ctx, version) -> list` | Returns env operations |

Optional functions:

| Function | Signature | Notes |
|----------|-----------|-------|
| `system_install` | `system_install(ctx) -> dict` | Package manager fallback |
| `deps` | `deps(ctx, version) -> list` | Runtime dependencies |
| `uninstall` | `uninstall(ctx, version) -> None` | Custom uninstall logic |

### Registering a Built-in Provider

After creating `provider.star` and `provider.toml`, register the provider in the Rust
registry so it is loaded at startup:

**`crates/vx-starlark/src/registry.rs`** (or equivalent registry file):

```rust
// Add the provider directory name to the built-in list
pub const BUILTIN_PROVIDERS: &[&str] = &[
    "node",
    "go",
    // ... existing providers ...
    "mytool",   // ← add here
];
```

No Rust code is needed beyond this registration line.

---

## Approach 2: Custom Rust Provider

Use this approach only when `provider.star` is insufficient — for example:
- Custom authentication flows
- Non-HTTP install sources (e.g., S3, internal registries)
- Complex post-install logic that Starlark cannot express
- Providers that wrap other Rust crates

### Directory Structure

```
crates/vx-providers/mytool/
├── Cargo.toml
├── provider.star     # Still recommended even with Rust code
├── provider.toml
└── src/
    ├── lib.rs        # Module exports
    ├── provider.rs   # Provider implementation
    └── runtime.rs    # Runtime implementation
```

### Cargo.toml

```toml
[package]
name        = "vx-provider-mytool"
version.workspace   = true
edition.workspace   = true
license.workspace   = true
description = "vx provider for MyTool"

[dependencies]
vx-core    = { workspace = true }
vx-runtime = { workspace = true }
async-trait = { workspace = true }
anyhow      = { workspace = true }
serde_json  = { workspace = true }
tracing     = { workspace = true }
```

### Implement the Runtime Trait

```rust
// src/runtime.rs
use async_trait::async_trait;
use vx_runtime::{Runtime, RuntimeContext, VersionInfo, Ecosystem, Platform};
use anyhow::Result;

pub struct MyToolRuntime;

impl MyToolRuntime {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Runtime for MyToolRuntime {
    // ── Required ──────────────────────────────────────────────────────────

    fn name(&self) -> &str { "mytool" }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = "https://api.github.com/repos/myorg/mytool/releases";
        let response: serde_json::Value = ctx.http.get_json_value(url).await?;

        let versions = response
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|r| {
                let tag = r["tag_name"].as_str()?;
                let version = tag.strip_prefix('v').unwrap_or(tag);
                Some(VersionInfo {
                    version:    version.to_string(),
                    prerelease: r["prerelease"].as_bool().unwrap_or(false),
                    ..Default::default()
                })
            })
            .collect();

        Ok(versions)
    }

    // ── Optional ──────────────────────────────────────────────────────────

    fn description(&self) -> &str { "MyTool - A fantastic development tool" }

    fn aliases(&self) -> &[&str] { &["mt"] }

    fn ecosystem(&self) -> Ecosystem { Ecosystem::Unknown }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        let triple = match (platform.os.as_str(), platform.arch.as_str()) {
            ("windows", "x86_64") => "x86_64-pc-windows-msvc",
            ("macos",   "x86_64") => "x86_64-apple-darwin",
            ("macos",   "aarch64") => "aarch64-apple-darwin",
            ("linux",   "x86_64") => "x86_64-unknown-linux-musl",
            ("linux",   "aarch64") => "aarch64-unknown-linux-gnu",
            _ => return Ok(None),
        };
        let ext   = if platform.os == "windows" { "zip" } else { "tar.gz" };
        let asset = format!("mytool-{}-{}.{}", version, triple, ext);
        Ok(Some(format!(
            "https://github.com/myorg/mytool/releases/download/v{}/{}",
            version, asset
        )))
    }
}
```

### Implement the Provider Trait

```rust
// src/provider.rs
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};
use crate::runtime::MyToolRuntime;

pub struct MyToolProvider;

impl Provider for MyToolProvider {
    fn name(&self) -> &str { "mytool" }

    fn description(&self) -> &str { "MyTool development tool" }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MyToolRuntime::new())]
    }
}
```

### Export from lib.rs

```rust
// src/lib.rs
mod provider;
mod runtime;

pub use provider::MyToolProvider;
pub use runtime::MyToolRuntime;
```

### Register the Provider

Add to `crates/vx-cli/Cargo.toml`:

```toml
[dependencies]
vx-provider-mytool = { path = "../vx-providers/mytool" }
```

Add to `crates/vx-cli/src/registry.rs`:

```rust
use vx_provider_mytool::MyToolProvider;

pub fn create_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MyToolProvider));
    // ... other providers
    registry
}
```

### Lifecycle Hooks

```rust
#[async_trait]
impl Runtime for MyToolRuntime {
    /// Called after extraction — rename files, set permissions
    fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let exe = install_path.join("mytool");
            if exe.exists() {
                let mut perms = std::fs::metadata(&exe)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&exe, perms)?;
            }
        }
        Ok(())
    }

    /// Called after successful installation
    async fn post_install(&self, _version: &str, _ctx: &RuntimeContext) -> Result<()> {
        // Run initialization, install bundled tools, etc.
        Ok(())
    }
}
```

---

## Testing

### Unit Tests

Place tests in `tests/` (not inline `#[cfg(test)]` modules):

```
crates/vx-providers/mytool/tests/
├── provider_tests.rs
└── runtime_tests.rs
```

```rust
// tests/runtime_tests.rs
use rstest::rstest;
use vx_provider_mytool::MyToolRuntime;
use vx_runtime::Runtime;

#[rstest]
fn test_runtime_name() {
    let runtime = MyToolRuntime::new();
    assert_eq!(runtime.name(), "mytool");
}

#[rstest]
fn test_aliases() {
    let runtime = MyToolRuntime::new();
    assert!(runtime.aliases().contains(&"mt"));
}

#[tokio::test]
async fn test_download_url_linux() {
    let runtime = MyToolRuntime::new();
    let platform = Platform { os: "linux".into(), arch: "x86_64".into() };
    let url = runtime.download_url("1.0.0", &platform).await.unwrap();
    assert!(url.is_some());
    assert!(url.unwrap().contains("1.0.0"));
}
```

### Testing provider.star

For Starlark providers, test by running vx commands against a temporary `VX_HOME`:

```bash
# Set a temp home and test
VX_HOME=/tmp/vx-test vx mytool --version
```

---

## Checklist

### provider.star Provider

- [ ] `provider.star` created with all required functions
- [ ] `provider.toml` created (metadata only, no layout fields)
- [ ] `license` field set to SPDX identifier
- [ ] `runtimes` list includes `test_commands`
- [ ] `download_url()` covers all major platforms (windows/x64, macos/x64, macos/arm64, linux/x64, linux/arm64)
- [ ] `install_layout()` returns correct `strip_prefix` and `executable_paths`
- [ ] `environment()` returns a **list** (not a dict)
- [ ] `system_install()` added if tool is available via brew/winget/choco
- [ ] Provider registered in built-in registry
- [ ] Tested on at least one platform

### Custom Rust Provider (additional)

- [ ] `Cargo.toml` created with workspace dependencies
- [ ] `Runtime` trait implemented (`name()` + `fetch_versions()` required)
- [ ] `Provider` trait implemented
- [ ] Tests in `tests/` directory (not inline)
- [ ] Added to `vx-cli/Cargo.toml` dependencies
- [ ] Registered in `create_registry()`

---

## Reference Providers

Study these built-in providers as examples:

| Provider | Pattern | Location |
|----------|---------|----------|
| `ripgrep` | Standard GitHub binary, archive layout | `crates/vx-providers/ripgrep/` |
| `meson` | PyPI package alias (`uvx`) | `crates/vx-providers/meson/` |
| `imagemagick` | Hybrid: direct download (Linux) + system pkg (Win/Mac) | `crates/vx-providers/imagemagick/` |
| `node` | Custom Rust provider, multiple runtimes, LTS support | `crates/vx-providers/node/` |
| `go` | Custom Rust provider, official API version fetching | `crates/vx-providers/go/` |
| `uv` | Custom Rust provider, Python ecosystem | `crates/vx-providers/uv/` |

## See Also

- [Manifest-Driven Providers](../guide/manifest-driven-providers.md) — Complete `provider.star` reference
- [Extension Development](./extension-development.md) — Script-based extensions
- [Contributing Guide](./contributing.md) — How to submit a provider
