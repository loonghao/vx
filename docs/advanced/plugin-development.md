# Provider Development Guide

This guide explains how to create a new tool provider for vx. Providers are the core extension mechanism that enables vx to support different development tools.

## Architecture Overview

vx uses a **Provider-Runtime** architecture:

- **Provider**: A container for related runtimes (e.g., `NodeProvider` provides `node`, `npm`, `npx`)
- **Runtime**: The actual tool implementation (version fetching, installation, execution)

```
┌─────────────────────────────────────────────────────────────┐
│                      ProviderRegistry                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │NodeProvider │  │ GoProvider  │  │ UVProvider  │   ...    │
│  │  - node     │  │  - go       │  │  - uv       │          │
│  │  - npm      │  │             │  │  - uvx      │          │
│  │  - npx      │  │             │  │             │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

## Provider Structure

Providers are Rust crates located in `crates/vx-providers/`:

```
crates/vx-providers/
├── node/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # Module exports
│       ├── provider.rs     # Provider implementation
│       ├── runtime.rs      # Runtime implementations
│       └── config.rs       # Configuration (optional)
├── go/
├── rust/
└── ...
```

## Step-by-Step Guide

### 1. Create the Crate

Create a new directory under `crates/vx-providers/`:

```bash
mkdir -p crates/vx-providers/mytool/src
```

Create `Cargo.toml`:

```toml
[package]
name = "vx-provider-mytool"
version.workspace = true
edition.workspace = true
license.workspace = true
description = "vx provider for MyTool"

[dependencies]
vx-core = { workspace = true }
vx-runtime = { workspace = true }
async-trait = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
tracing = { workspace = true }
```

### 2. Implement the Runtime Trait

The `Runtime` trait is the core abstraction. Only two methods are **required**:

```rust
// src/runtime.rs
use async_trait::async_trait;
use vx_runtime::{
    Runtime, RuntimeContext, VersionInfo, Ecosystem, Platform,
    ExecutionContext, ExecutionResult, InstallResult,
};
use anyhow::Result;

pub struct MyToolRuntime;

impl MyToolRuntime {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Runtime for MyToolRuntime {
    // ========== Required Methods ==========

    /// Runtime name - used as the command name
    fn name(&self) -> &str {
        "mytool"
    }

    /// Fetch available versions from official source
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Example: Fetch from GitHub releases API
        let url = "https://api.github.com/repos/org/mytool/releases";
        let response: serde_json::Value = ctx.http.get_json_value(url).await?;

        let versions = response
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|release| {
                let tag = release["tag_name"].as_str()?;
                let version = tag.strip_prefix('v').unwrap_or(tag);
                let prerelease = release["prerelease"].as_bool().unwrap_or(false);

                Some(VersionInfo {
                    version: version.to_string(),
                    prerelease,
                    lts: false,
                    release_date: release["published_at"].as_str().map(String::from),
                    ..Default::default()
                })
            })
            .collect();

        Ok(versions)
    }

    // ========== Optional Methods with Defaults ==========

    fn description(&self) -> &str {
        "MyTool - A fantastic development tool"
    }

    fn aliases(&self) -> &[&str] {
        &["mt", "my-tool"]  // Alternative names
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Unknown  // Or NodeJs, Go, Rust, Python, etc.
    }

    /// Get download URL for a specific version and platform
    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        let os = match platform.os.as_str() {
            "macos" => "darwin",
            "windows" => "windows",
            "linux" => "linux",
            _ => return Ok(None),
        };

        let arch = match platform.arch.as_str() {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            _ => return Ok(None),
        };

        let ext = if platform.os == "windows" { "zip" } else { "tar.gz" };

        Ok(Some(format!(
            "https://github.com/org/mytool/releases/download/v{}/mytool-{}-{}.{}",
            version, os, arch, ext
        )))
    }

    /// Customize executable path within the extracted archive
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.os == "windows" {
            "mytool.exe".to_string()
        } else {
            "mytool".to_string()
        }
    }
}
```

### 3. Implement the Provider Trait

The `Provider` groups related runtimes:

```rust
// src/provider.rs
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};
use crate::runtime::MyToolRuntime;

pub struct MyToolProvider;

impl Provider for MyToolProvider {
    fn name(&self) -> &str {
        "mytool"
    }

    fn description(&self) -> &str {
        "MyTool development tool"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MyToolRuntime::new())]
    }
}
```

### 4. Export from lib.rs

```rust
// src/lib.rs
mod provider;
mod runtime;

pub use provider::MyToolProvider;
pub use runtime::MyToolRuntime;
```

### 5. Register the Provider

Add your provider to `crates/vx-cli/src/registry.rs`:

```rust
use vx_provider_mytool::MyToolProvider;

pub fn create_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();

    // Register your provider
    registry.register(Box::new(MyToolProvider));

    // ... other providers
    registry
}
```

Add the dependency to `crates/vx-cli/Cargo.toml`:

```toml
[dependencies]
vx-provider-mytool = { path = "../vx-providers/mytool" }
```

## Lifecycle Hooks

The `Runtime` trait provides lifecycle hooks for customization:

### Installation Hooks

```rust
#[async_trait]
impl Runtime for MyToolRuntime {
    /// Called before installation - validate environment
    async fn pre_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        // Check system requirements
        Ok(())
    }

    /// Called after extraction - rename files, set permissions
    fn post_extract(&self, version: &str, install_path: &PathBuf) -> Result<()> {
        // Example: Rename platform-specific binary
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
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        // Run initialization, install bundled tools, etc.
        Ok(())
    }
}
```

### Execution Hooks

```rust
#[async_trait]
impl Runtime for MyToolRuntime {
    /// Called before command execution
    async fn pre_execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<()> {
        // Set up environment, validate args
        Ok(())
    }

    /// Called after command execution
    async fn post_execute(
        &self,
        args: &[String],
        result: &ExecutionResult,
        ctx: &ExecutionContext,
    ) -> Result<()> {
        // Log results, clean up temp files
        Ok(())
    }
}
```

### Version Switch Hooks

```rust
#[async_trait]
impl Runtime for MyToolRuntime {
    async fn pre_switch(
        &self,
        from_version: Option<&str>,
        to_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        // Validate target version
        Ok(())
    }

    async fn post_switch(
        &self,
        from_version: Option<&str>,
        to_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<()> {
        // Update symlinks, rehash commands
        Ok(())
    }
}
```

## Custom Installation Verification

Override `verify_installation` for custom verification logic:

```rust
fn verify_installation(
    &self,
    version: &str,
    install_path: &Path,
    platform: &Platform,
) -> VerificationResult {
    let exe_path = install_path.join(self.executable_relative_path(version, platform));

    if !exe_path.exists() {
        return VerificationResult::failure(
            vec![format!("Executable not found: {}", exe_path.display())],
            vec!["Check the download URL and archive structure".to_string()],
        );
    }

    // Additional checks (e.g., verify it's actually executable)
    VerificationResult::success(exe_path)
}
```

## Testing

Create tests in `tests/` directory (following project conventions):

```rust
// tests/runtime_tests.rs
use rstest::rstest;
use vx_provider_mytool::MyToolRuntime;
use vx_runtime::Runtime;

#[rstest]
#[tokio::test]
async fn test_fetch_versions() {
    let runtime = MyToolRuntime::new();
    // Note: This test requires network access
    // Consider mocking for CI
    assert_eq!(runtime.name(), "mytool");
}

#[rstest]
fn test_executable_path() {
    let runtime = MyToolRuntime::new();
    let platform = Platform::current();
    let path = runtime.executable_relative_path("1.0.0", &platform);
    assert!(!path.is_empty());
}
```

## Best Practices

### 1. Handle Platform Differences

```rust
async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
    // Map platform names to download URL format
    let (os, arch) = match (platform.os.as_str(), platform.arch.as_str()) {
        ("macos", "x86_64") => ("darwin", "amd64"),
        ("macos", "aarch64") => ("darwin", "arm64"),
        ("linux", "x86_64") => ("linux", "amd64"),
        ("linux", "aarch64") => ("linux", "arm64"),
        ("windows", "x86_64") => ("windows", "amd64"),
        _ => return Ok(None), // Unsupported platform
    };
    // ...
}
```

### 2. Verify Downloads

Use checksums when available:

```rust
async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
    // Verify checksum if available
    let checksum_url = format!("https://example.com/mytool/{}/SHA256SUMS", version);
    // ... verify
    Ok(())
}
```

### 3. Provide Good Error Messages

```rust
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    ctx.http
        .get_json_value(API_URL)
        .await
        .map_err(|e| anyhow::anyhow!(
            "Failed to fetch MyTool versions from {}: {}. \
             Check your network connection or try again later.",
            API_URL, e
        ))?;
    // ...
}
```

### 4. Support Version Specifiers

Handle common version formats:

```rust
async fn resolve_version(&self, version: &str, ctx: &RuntimeContext) -> Result<String> {
    match version {
        "latest" => {
            let versions = self.fetch_versions(ctx).await?;
            versions.into_iter()
                .filter(|v| !v.prerelease)
                .map(|v| v.version)
                .next()
                .ok_or_else(|| anyhow::anyhow!("No stable versions found"))
        }
        "lts" => {
            // Handle LTS if applicable
            self.resolve_version("latest", ctx).await
        }
        v => Ok(v.to_string()),
    }
}
```

## Example Providers

Study existing providers for reference:

| Provider | Features | Location |
|----------|----------|----------|
| `node` | Multiple runtimes (node, npm, npx), LTS support | `crates/vx-providers/node/` |
| `go` | Simple single runtime | `crates/vx-providers/go/` |
| `uv` | Python ecosystem, uvx runner | `crates/vx-providers/uv/` |
| `rust` | Multiple commands (cargo, rustc, rustup) | `crates/vx-providers/rust/` |
| `pnpm` | Post-extract file renaming | `crates/vx-providers/pnpm/` |

## Contributing

1. Fork the repository
2. Create your provider crate under `crates/vx-providers/`
3. Add comprehensive tests
4. Update documentation
5. Submit a pull request

See [Contributing Guide](contributing.md) for more details.
