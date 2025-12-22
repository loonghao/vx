# Plugin Development

vx uses a provider-based plugin system for tool support.

## Architecture

Each tool is supported by a **provider** that implements:

- Version discovery
- Download and installation
- Execution

## Provider Structure

Providers are Rust crates in `crates/vx-providers/`:

```
crates/vx-providers/
├── node/
�?  ├── Cargo.toml
�?  └── src/
�?      ├── lib.rs
�?      ├── provider.rs
�?      ├── runtime.rs
�?      └── config.rs
├── go/
├── rust/
└── ...
```

## Implementing a Provider

### 1. Create the Crate

```toml
# Cargo.toml
[package]
name = "vx-provider-mytool"
version.workspace = true
edition.workspace = true

[dependencies]
vx-core = { workspace = true }
vx-runtime = { workspace = true }
async-trait = { workspace = true }
anyhow = { workspace = true }
```

### 2. Implement the Provider Trait

```rust
// src/provider.rs
use async_trait::async_trait;
use vx_runtime::{Provider, ProviderInfo, RuntimeInfo};

pub struct MyToolProvider;

#[async_trait]
impl Provider for MyToolProvider {
    fn info(&self) -> ProviderInfo {
        ProviderInfo {
            name: "mytool".to_string(),
            display_name: "My Tool".to_string(),
            description: "Description of my tool".to_string(),
            homepage: "https://mytool.dev".to_string(),
        }
    }

    async fn list_versions(&self) -> anyhow::Result<Vec<String>> {
        // Fetch available versions from API or releases
        Ok(vec!["1.0.0".to_string(), "1.1.0".to_string()])
    }

    async fn install(&self, version: &str) -> anyhow::Result<()> {
        // Download and install the tool
        Ok(())
    }

    fn get_runtime(&self, version: &str) -> anyhow::Result<RuntimeInfo> {
        // Return runtime information
        Ok(RuntimeInfo {
            name: "mytool".to_string(),
            version: version.to_string(),
            // ...
        })
    }
}
```

### 3. Implement Version Resolution

```rust
impl MyToolProvider {
    async fn resolve_version(&self, spec: &str) -> anyhow::Result<String> {
        match spec {
            "latest" => self.get_latest_version().await,
            version => Ok(version.to_string()),
        }
    }

    async fn get_latest_version(&self) -> anyhow::Result<String> {
        let versions = self.list_versions().await?;
        versions.last()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No versions available"))
    }
}
```

### 4. Implement Installation

```rust
impl MyToolProvider {
    async fn download_and_install(&self, version: &str) -> anyhow::Result<()> {
        let url = self.get_download_url(version)?;
        let archive = self.download(&url).await?;
        self.extract(&archive)?;
        self.verify_installation(version)?;
        Ok(())
    }

    fn get_download_url(&self, version: &str) -> anyhow::Result<String> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        Ok(format!(
            "https://releases.mytool.dev/{}/mytool-{}-{}.tar.gz",
            version, os, arch
        ))
    }
}
```

### 5. Register the Provider

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

## Testing

Create tests in `tests/`:

```rust
// tests/runtime_tests.rs
use rstest::rstest;
use vx_provider_mytool::MyToolProvider;

#[rstest]
#[tokio::test]
async fn test_list_versions() {
    let provider = MyToolProvider;
    let versions = provider.list_versions().await.unwrap();
    assert!(!versions.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_resolve_latest() {
    let provider = MyToolProvider;
    let version = provider.resolve_version("latest").await.unwrap();
    assert!(!version.is_empty());
}
```

## Best Practices

1. **Handle platform differences**: Check OS and architecture
2. **Verify downloads**: Use checksums when available
3. **Cache version lists**: Reduce API calls
4. **Provide good error messages**: Help users debug issues
5. **Support version specifiers**: latest, major, minor, exact

## Example Providers

Study existing providers for reference:

- `vx-provider-node` - Node.js with npm/npx
- `vx-provider-go` - Go language
- `vx-provider-uv` - UV Python package manager
- `vx-provider-rust` - Rust toolchain

## Contributing

1. Fork the repository
2. Create your provider crate
3. Add tests
4. Submit a pull request

See [Contributing](contributing) for more details.
