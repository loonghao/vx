# RFC 0009: Unified Version Fetcher Abstraction Layer (vx-version-fetcher)

## Summary

Create a new crate `vx-version-fetcher` that provides a unified interface for fetching version information, encapsulating implementation details of different data sources (jsDelivr CDN, npm registry, PyPI, official APIs, etc.) to simplify new Provider development.

## Motivation

### Current Problems

1. **Severe Code Duplication**: 8+ Providers contain nearly identical jsDelivr parsing logic (~50 lines each), totaling ~350 lines of duplicate code
2. **Scattered Version Processing Logic**: semver sorting, prerelease filtering, prefix handling implemented independently in each Provider
3. **Difficult to Add New Providers**: Developers must copy large amounts of template code from existing Providers
4. **Hard to Switch Data Sources**: Switching from GitHub API to jsDelivr requires substantial code rewriting
5. **Inconsistent Error Handling**: Error message formats and handling vary across Providers

### Duplicate Code Statistics

| Pattern | Occurrences | Lines/Instance | Potential Savings |
|---------|-------------|----------------|-------------------|
| jsDelivr full parsing flow | 8 | ~50 | ~350 lines |
| Semver sorting closure | 12+ | ~12 | ~130 lines |
| Prerelease filtering | 15+ | ~6 | ~80 lines |
| npm registry parsing | 3 | ~30 | ~60 lines |
| Semver validation | 10+ | ~5 | ~40 lines |
| **Total** | | | **~660 lines** |

## Design

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Provider Runtime                          │
│                                                                  │
│  async fn fetch_versions(&self, ctx: &RuntimeContext)           │
│      -> Result<Vec<VersionInfo>>                                │
└───────────────────────────┬─────────────────────────────────────┘
                            │ calls
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    vx-version-fetcher                            │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                    VersionFetcher                            ││
│  │  - fetch(&self, ctx: &RuntimeContext) -> Vec<VersionInfo>   ││
│  └─────────────────────────────────────────────────────────────┘│
│                            │                                     │
│         ┌──────────────────┼──────────────────┐                 │
│         ▼                  ▼                  ▼                 │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐           │
│  │  jsDelivr   │   │    npm      │   │   GitHub    │           │
│  │  Fetcher    │   │  Registry   │   │   Releases  │           │
│  │             │   │  Fetcher    │   │   Fetcher   │           │
│  └─────────────┘   └─────────────┘   └─────────────┘           │
│                                                                  │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐           │
│  │    PyPI     │   │   Custom    │   │   Static    │           │
│  │   Fetcher   │   │    API      │   │   Versions  │           │
│  │             │   │  Fetcher    │   │             │           │
│  └─────────────┘   └─────────────┘   └─────────────┘           │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                  Version Utilities                           ││
│  │  - parse_semver()    - is_prerelease()    - sort_versions() ││
│  │  - strip_prefix()    - validate_semver()                    ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Core Trait Definition

```rust
// crates/vx-version-fetcher/src/lib.rs

use async_trait::async_trait;
use vx_runtime::{RuntimeContext, VersionInfo};

/// Core version fetcher trait
#[async_trait]
pub trait VersionFetcher: Send + Sync {
    /// Fetch version list
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>>;
    
    /// Fetcher name (for debugging and logging)
    fn name(&self) -> &str;
    
    /// Data source URL (for error messages)
    fn source_url(&self) -> Option<String> { None }
}

/// Version fetcher builder
pub struct VersionFetcherBuilder {
    fetcher: Box<dyn VersionFetcher>,
    filters: Vec<Box<dyn VersionFilter>>,
    transformer: Option<Box<dyn VersionTransformer>>,
    sorter: Option<Box<dyn VersionSorter>>,
    limit: Option<usize>,
}

impl VersionFetcherBuilder {
    /// Create jsDelivr CDN fetcher
    pub fn jsdelivr(owner: &str, repo: &str) -> Self { ... }
    
    /// Create npm registry fetcher
    pub fn npm(package: &str) -> Self { ... }
    
    /// Create PyPI fetcher
    pub fn pypi(package: &str) -> Self { ... }
    
    /// Create GitHub Releases fetcher
    pub fn github_releases(owner: &str, repo: &str) -> Self { ... }
    
    /// Create custom API fetcher
    pub fn custom_api<F>(url: &str, parser: F) -> Self 
    where F: Fn(&serde_json::Value) -> Result<Vec<VersionInfo>> { ... }
    
    /// Create static version list
    pub fn static_versions(versions: Vec<&str>) -> Self { ... }
    
    // === Chained Configuration ===
    
    /// Set version prefix handling
    pub fn strip_prefix(self, prefix: &str) -> Self { ... }
    
    /// Set tag prefix (e.g., "bun-v")
    pub fn tag_prefix(self, prefix: &str) -> Self { ... }
    
    /// Skip prerelease versions
    pub fn skip_prereleases(self) -> Self { ... }
    
    /// Custom prerelease detector
    pub fn prerelease_markers(self, markers: &[&str]) -> Self { ... }
    
    /// Set LTS detector
    pub fn lts_detector<F>(self, detector: F) -> Self 
    where F: Fn(&str) -> bool { ... }
    
    /// Limit returned version count
    pub fn limit(self, max: usize) -> Self { ... }
    
    /// Custom filter
    pub fn filter<F>(self, filter: F) -> Self 
    where F: Fn(&VersionInfo) -> bool { ... }
    
    /// Build the fetcher
    pub fn build(self) -> Box<dyn VersionFetcher> { ... }
}
```

### Built-in Fetcher Implementations

#### 1. jsDelivr CDN Fetcher

```rust
// crates/vx-version-fetcher/src/fetchers/jsdelivr.rs

/// jsDelivr CDN version fetcher
/// 
/// Uses jsDelivr's API to get GitHub repository tag lists,
/// avoiding GitHub API rate limit issues.
pub struct JsDelivrFetcher {
    owner: String,
    repo: String,
    config: JsDelivrConfig,
}

#[derive(Default, Clone)]
pub struct JsDelivrConfig {
    /// Version prefix to strip (e.g., "v", "jq-", "bun-v")
    pub strip_prefix: Option<String>,
    /// Whether to skip prereleases
    pub skip_prereleases: bool,
    /// Prerelease marker list
    pub prerelease_markers: Vec<String>,
    /// Maximum versions to return
    pub max_versions: usize,
    /// LTS detection function
    pub lts_detector: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl JsDelivrFetcher {
    pub fn new(owner: &str, repo: &str) -> Self {
        Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
            config: JsDelivrConfig {
                max_versions: 50,
                prerelease_markers: vec![
                    "-alpha".to_string(),
                    "-beta".to_string(),
                    "-rc".to_string(),
                    "-dev".to_string(),
                    "canary".to_string(),
                ],
                ..Default::default()
            },
        }
    }
    
    pub fn with_config(mut self, config: JsDelivrConfig) -> Self {
        self.config = config;
        self
    }
}

#[async_trait]
impl VersionFetcher for JsDelivrFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = format!(
            "https://data.jsdelivr.com/v1/package/gh/{}/{}",
            self.owner, self.repo
        );
        
        let response = ctx.http().get_json(&url).await?;
        
        let versions_array = response
            .get("versions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Invalid jsDelivr response format"))?;
        
        let mut versions: Vec<VersionInfo> = versions_array
            .iter()
            .filter_map(|v| self.parse_version(v.as_str()?))
            .collect();
        
        version_utils::sort_versions_desc(&mut versions);
        versions.truncate(self.config.max_versions);
        
        Ok(versions)
    }
    
    fn name(&self) -> &str {
        "jsDelivr"
    }
    
    fn source_url(&self) -> Option<String> {
        Some(format!("https://github.com/{}/{}", self.owner, self.repo))
    }
}
```

#### 2. npm Registry Fetcher

```rust
// crates/vx-version-fetcher/src/fetchers/npm.rs

/// npm Registry version fetcher
pub struct NpmFetcher {
    package: String,
    config: NpmConfig,
}

#[derive(Default, Clone)]
pub struct NpmConfig {
    /// Whether to skip prereleases (versions with -)
    pub skip_prereleases: bool,
    /// Maximum versions to return
    pub max_versions: usize,
    /// Whether to include release dates
    pub include_release_date: bool,
    /// LTS detection function
    pub lts_detector: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
}

impl NpmFetcher {
    pub fn new(package: &str) -> Self {
        Self {
            package: package.to_string(),
            config: NpmConfig {
                max_versions: 100,
                include_release_date: true,
                ..Default::default()
            },
        }
    }
}

#[async_trait]
impl VersionFetcher for NpmFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = format!("https://registry.npmjs.org/{}", self.package);
        let response = ctx.http().get_json(&url).await?;
        
        let versions_obj = response
            .get("versions")
            .and_then(|v| v.as_object())
            .ok_or_else(|| anyhow!("Invalid npm registry response"))?;
        
        let time_obj = response.get("time").and_then(|t| t.as_object());
        
        let mut versions: Vec<VersionInfo> = versions_obj
            .keys()
            .filter_map(|version| self.parse_version(version, time_obj))
            .collect();
        
        version_utils::sort_versions_desc(&mut versions);
        versions.truncate(self.config.max_versions);
        
        Ok(versions)
    }
    
    fn name(&self) -> &str { "npm" }
    
    fn source_url(&self) -> Option<String> {
        Some(format!("https://www.npmjs.com/package/{}", self.package))
    }
}
```

#### 3. PyPI Fetcher

```rust
// crates/vx-version-fetcher/src/fetchers/pypi.rs

/// PyPI version fetcher
pub struct PyPiFetcher {
    package: String,
    config: PyPiConfig,
}

#[async_trait]
impl VersionFetcher for PyPiFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let url = format!("https://pypi.org/pypi/{}/json", self.package);
        let response = ctx.http().get_json(&url).await?;
        
        let releases = response
            .get("releases")
            .and_then(|r| r.as_object())
            .ok_or_else(|| anyhow!("Invalid PyPI response"))?;
        
        // ... parsing logic
    }
    
    fn name(&self) -> &str { "PyPI" }
}
```

#### 4. GitHub Releases Fetcher (Preserving Existing Features)

```rust
// crates/vx-version-fetcher/src/fetchers/github.rs

/// GitHub Releases version fetcher
/// 
/// Uses GitHub API directly, supports:
/// - GITHUB_TOKEN authentication
/// - Automatic jsDelivr fallback
/// - Rich release info (description, assets, etc.)
pub struct GitHubReleasesFetcher {
    owner: String,
    repo: String,
    options: GitHubReleaseOptions,
}

#[derive(Default, Clone)]
pub struct GitHubReleaseOptions {
    pub strip_v_prefix: bool,
    pub tag_prefix: Option<String>,
    pub skip_prereleases: bool,
    pub per_page: usize,
    pub lts_detector: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    /// Whether to automatically fallback to jsDelivr when GitHub API fails
    pub jsdelivr_fallback: bool,
}

#[async_trait]
impl VersionFetcher for GitHubReleasesFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // Try GitHub API first
        match self.fetch_from_github(ctx).await {
            Ok(versions) => Ok(versions),
            Err(e) if self.options.jsdelivr_fallback => {
                tracing::warn!("GitHub API failed, falling back to jsDelivr: {}", e);
                self.fetch_from_jsdelivr(ctx).await
            }
            Err(e) => Err(e),
        }
    }
    
    fn name(&self) -> &str { "GitHub Releases" }
}
```

#### 5. Custom API Fetcher

```rust
// crates/vx-version-fetcher/src/fetchers/custom.rs

/// Custom API version fetcher
/// 
/// Supports any JSON API with custom parsing logic via closure
pub struct CustomApiFetcher<F> {
    url: String,
    parser: F,
    name: String,
}

impl<F> CustomApiFetcher<F>
where
    F: Fn(&serde_json::Value) -> Result<Vec<VersionInfo>> + Send + Sync,
{
    pub fn new(url: &str, parser: F) -> Self {
        Self {
            url: url.to_string(),
            parser,
            name: "Custom API".to_string(),
        }
    }
    
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
}

#[async_trait]
impl<F> VersionFetcher for CustomApiFetcher<F>
where
    F: Fn(&serde_json::Value) -> Result<Vec<VersionInfo>> + Send + Sync,
{
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let response = ctx.http().get_json(&self.url).await?;
        (self.parser)(&response)
    }
    
    fn name(&self) -> &str { &self.name }
}
```

### Version Utilities Module

```rust
// crates/vx-version-fetcher/src/utils.rs

/// Version processing utility functions
pub mod version_utils {
    use vx_runtime::VersionInfo;

    /// Parse semver to tuple (major, minor, patch)
    pub fn parse_semver_tuple(v: &str) -> (u64, u64, u64) {
        let parts: Vec<&str> = v.split('.').collect();
        let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts
            .get(2)
            .and_then(|s| s.split('-').next())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        (major, minor, patch)
    }

    /// Sort versions descending
    pub fn sort_versions_desc(versions: &mut [VersionInfo]) {
        versions.sort_by(|a, b| {
            parse_semver_tuple(&b.version).cmp(&parse_semver_tuple(&a.version))
        });
    }

    /// Check if version is prerelease
    pub fn is_prerelease(version: &str) -> bool {
        let lower = version.to_lowercase();
        lower.contains("-alpha")
            || lower.contains("-beta")
            || lower.contains("-rc")
            || lower.contains("-dev")
            || lower.contains("canary")
    }
    
    /// Check if prerelease with custom markers
    pub fn is_prerelease_with_markers(version: &str, markers: &[&str]) -> bool {
        let lower = version.to_lowercase();
        markers.iter().any(|m| lower.contains(m))
    }

    /// Validate basic semver format
    pub fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() >= 2 && parts[0].parse::<u32>().is_ok()
    }

    /// Strip version prefix
    pub fn strip_version_prefix<'a>(version: &'a str, prefix: &str) -> Option<&'a str> {
        if prefix.is_empty() {
            Some(version.trim_start_matches('v'))
        } else {
            version.strip_prefix(prefix)
        }
    }
    
    /// Compare two version strings
    pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        parse_semver_tuple(a).cmp(&parse_semver_tuple(b))
    }
}
```

### Usage Examples

#### Before Refactoring (Current Code)

```rust
// crates/vx-providers/helm/src/runtime.rs (60+ lines)

async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    let url = "https://data.jsdelivr.com/v1/package/gh/helm/helm";

    ctx.fetch_json_versions("helm", url, |response| {
        let versions_array = response
            .get("versions")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid jsDelivr response format"))?;

        let mut versions: Vec<VersionInfo> = versions_array
            .iter()
            .filter_map(|v| {
                let version_str = v.as_str()?;
                let version = version_str.trim_start_matches('v').to_string();

                let lower = version.to_lowercase();
                if lower.contains("-alpha")
                    || lower.contains("-beta")
                    || lower.contains("-rc")
                    || lower.contains("-dev")
                {
                    return None;
                }

                let parts: Vec<&str> = version.split('.').collect();
                if parts.len() < 2 {
                    return None;
                }
                if parts[0].parse::<u32>().is_err() {
                    return None;
                }

                Some(VersionInfo::new(&version).with_prerelease(false))
            })
            .collect();

        versions.sort_by(|a, b| {
            let parse_semver = |v: &str| -> (u64, u64, u64) {
                let parts: Vec<&str> = v.split('.').collect();
                let major = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
                let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let patch = parts
                    .get(2)
                    .and_then(|s| s.split('-').next())
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
                (major, minor, patch)
            };
            let a_ver = parse_semver(&a.version);
            let b_ver = parse_semver(&b.version);
            b_ver.cmp(&a_ver)
        });

        versions.truncate(50);
        Ok(versions)
    })
    .await
}
```

#### After Refactoring (Using New API)

```rust
// crates/vx-providers/helm/src/runtime.rs (5 lines)

use vx_version_fetcher::VersionFetcherBuilder;

async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    VersionFetcherBuilder::jsdelivr("helm", "helm")
        .strip_prefix("v")
        .skip_prereleases()
        .limit(50)
        .build()
        .fetch(ctx)
        .await
}
```

#### More Examples

```rust
// pnpm - npm registry
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    VersionFetcherBuilder::npm("pnpm")
        .skip_prereleases()
        .limit(100)
        .build()
        .fetch(ctx)
        .await
}

// jq - jsDelivr with custom prefix
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    VersionFetcherBuilder::jsdelivr("jqlang", "jq")
        .tag_prefix("jq-")
        .skip_prereleases()
        .build()
        .fetch(ctx)
        .await
}

// bun - jsDelivr with complex prefix
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    VersionFetcherBuilder::jsdelivr("oven-sh", "bun")
        .tag_prefix("bun-v")
        .prerelease_markers(&["canary", "-alpha", "-beta", "-rc"])
        .build()
        .fetch(ctx)
        .await
}

// yarn - npm with LTS detection
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    VersionFetcherBuilder::npm("yarn")
        .skip_prereleases()
        .lts_detector(|v| v.starts_with("1.22."))
        .build()
        .fetch(ctx)
        .await
}

// node - official API
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    VersionFetcherBuilder::custom_api(
        "https://nodejs.org/dist/index.json",
        |response| {
            // Custom parsing logic
            let array = response.as_array()?;
            // ...
        }
    )
    .with_name("Node.js Official")
    .build()
    .fetch(ctx)
    .await
}
```

### Crate Structure

```
crates/vx-version-fetcher/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public exports
│   ├── fetcher.rs          # VersionFetcher trait
│   ├── builder.rs          # VersionFetcherBuilder
│   ├── utils.rs            # version_utils module
│   ├── fetchers/
│   │   ├── mod.rs
│   │   ├── jsdelivr.rs     # JsDelivrFetcher
│   │   ├── npm.rs          # NpmFetcher
│   │   ├── pypi.rs         # PyPiFetcher
│   │   ├── github.rs       # GitHubReleasesFetcher
│   │   └── custom.rs       # CustomApiFetcher
│   └── error.rs            # Error types
└── tests/
    ├── jsdelivr_tests.rs
    ├── npm_tests.rs
    └── utils_tests.rs
```

### Dependencies

```
vx-version-fetcher
├── vx-runtime (RuntimeContext, VersionInfo)
├── async-trait
├── serde_json
├── anyhow
└── tracing
```

## Migration Plan

### Phase 1: Create Crate and Infrastructure (1-2 days)

1. Create `vx-version-fetcher` crate
2. Implement `VersionFetcher` trait and `VersionFetcherBuilder`
3. Extract `version_utils` module
4. Add unit tests

### Phase 2: Implement Built-in Fetchers (2-3 days)

1. Implement `JsDelivrFetcher`
2. Implement `NpmFetcher`
3. Implement `PyPiFetcher`
4. Migrate `GitHubReleasesFetcher` (from vx-runtime)
5. Implement `CustomApiFetcher`

### Phase 3: Migrate Existing Providers (2-3 days)

**High Priority** (using jsDelivr, most code duplication):
- [ ] `helm`
- [ ] `kubectl`
- [ ] `bun`
- [ ] `deno`
- [ ] `uv`
- [ ] `ninja`
- [ ] `just`
- [ ] `jq`

**Medium Priority** (using npm registry):
- [ ] `pnpm`
- [ ] `yarn`

**Low Priority** (using official APIs):
- [ ] `node` (keep existing implementation, but can simplify with custom_api)
- [ ] `go`
- [ ] `python`
- [ ] `java`

### Phase 4: Cleanup and Documentation (1 day)

1. Remove redundant code like `fetch_github_releases` from `vx-runtime`
2. Update Provider development documentation
3. Add usage examples

## Compatibility

### Backward Compatibility

- `RuntimeContext.fetch_json_versions()` retained but marked deprecated
- `GitHubReleaseOptions` retained, internal implementation moved to new crate
- Existing Providers don't need immediate migration

### API Stability

- `VersionFetcher` trait remains stable
- `VersionFetcherBuilder` can be extended with new methods
- `version_utils` module function signatures are stable

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_semver_tuple() {
        assert_eq!(parse_semver_tuple("1.2.3"), (1, 2, 3));
        assert_eq!(parse_semver_tuple("1.2"), (1, 2, 0));
        assert_eq!(parse_semver_tuple("1.2.3-alpha"), (1, 2, 3));
    }
    
    #[test]
    fn test_is_prerelease() {
        assert!(is_prerelease("1.0.0-alpha"));
        assert!(is_prerelease("1.0.0-beta.1"));
        assert!(is_prerelease("1.0.0-rc.1"));
        assert!(!is_prerelease("1.0.0"));
    }
    
    #[test]
    fn test_sort_versions_desc() {
        let mut versions = vec![
            VersionInfo::new("1.0.0"),
            VersionInfo::new("2.0.0"),
            VersionInfo::new("1.5.0"),
        ];
        sort_versions_desc(&mut versions);
        assert_eq!(versions[0].version, "2.0.0");
        assert_eq!(versions[1].version, "1.5.0");
        assert_eq!(versions[2].version, "1.0.0");
    }
}
```

### Integration Tests

```rust
// tests/jsdelivr_tests.rs

#[tokio::test]
async fn test_jsdelivr_fetcher() {
    let fetcher = VersionFetcherBuilder::jsdelivr("helm", "helm")
        .strip_prefix("v")
        .skip_prereleases()
        .limit(10)
        .build();
    
    let ctx = mock_runtime_context();
    ctx.http().mock_response(
        "https://data.jsdelivr.com/v1/package/gh/helm/helm",
        json!({"versions": ["v3.14.0", "v3.13.0", "v3.12.0-rc.1"]})
    );
    
    let versions = fetcher.fetch(&ctx).await.unwrap();
    
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].version, "3.14.0");
    assert_eq!(versions[1].version, "3.13.0");
}
```

## Future Extensions

### Possible New Fetchers

- **Homebrew Formula**: `https://formulae.brew.sh/api/formula/{name}.json`
- **Chocolatey**: `https://community.chocolatey.org/api/v2/package-versions/{name}`
- **APT Repository**: Parse `Packages` file
- **Cargo Registry**: `https://crates.io/api/v1/crates/{name}`

### Caching Layer

```rust
// May add in future
pub struct CachedFetcher<F: VersionFetcher> {
    inner: F,
    cache: Arc<VersionCache>,
    ttl: Duration,
}

impl<F: VersionFetcher> VersionFetcher for CachedFetcher<F> {
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        if let Some(cached) = self.cache.get(self.inner.name()).await {
            if cached.is_valid(self.ttl) {
                return Ok(cached.versions);
            }
        }
        
        let versions = self.inner.fetch(ctx).await?;
        self.cache.set(self.inner.name(), &versions).await;
        Ok(versions)
    }
}
```

### Mirror Support

```rust
// May add in future
pub struct MirroredFetcher<F: VersionFetcher> {
    primary: F,
    mirrors: Vec<Box<dyn VersionFetcher>>,
}
```

## Decision Records

### Why Builder Pattern?

1. **Type Safety**: Compile-time configuration validation
2. **Readability**: Chained calls clearly express intent
3. **Extensibility**: Easy to add new configuration options
4. **IDE Friendly**: Autocomplete support

### Why Keep `fetch_json_versions`?

1. **Backward Compatibility**: Don't break existing code
2. **Flexibility**: Some scenarios still need fully custom parsing
3. **Gradual Migration**: Allow Providers to migrate incrementally

### Why jsDelivr Over GitHub API?

1. **No Rate Limits**: jsDelivr is a CDN with no API limits
2. **Faster Response**: CDN caching, faster responses
3. **No Auth Required**: Doesn't need GITHUB_TOKEN
4. **More Stable**: Not affected by GitHub API outages

## References

- [jsDelivr API Documentation](https://www.jsdelivr.com/docs/data.jsdelivr.com)
- [npm Registry API](https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md)
- [PyPI JSON API](https://warehouse.pypa.io/api-reference/json.html)
- [GitHub REST API - Releases](https://docs.github.com/en/rest/releases)
