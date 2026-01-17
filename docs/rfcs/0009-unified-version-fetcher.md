# RFC 0009: 统一版本获取器抽象层 (vx-version-fetcher)

## 摘要

创建一个新的 crate `vx-version-fetcher`，提供统一的版本信息获取接口，封装不同数据源（jsDelivr CDN、npm registry、PyPI、官方 API 等）的实现细节，简化新 Provider 的开发。

## 动机

### 现状问题

1. **代码重复严重**：当前 8+ 个 Provider 中存在几乎相同的 jsDelivr 解析逻辑（约 50 行/处），总计 ~350 行重复代码
2. **版本处理逻辑分散**：semver 排序、prerelease 过滤、前缀处理等逻辑在各 Provider 中独立实现
3. **新增 Provider 困难**：开发者需要从其他 Provider 复制大量模板代码
4. **数据源切换困难**：当需要从 GitHub API 切换到 jsDelivr 时，需要大量重写代码
5. **错误处理不一致**：各 Provider 的错误消息格式和处理方式不统一

### 重复代码统计

| 模式 | 重复次数 | 代码行数/处 | 潜在节省 |
|------|---------|-------------|----------|
| jsDelivr 解析完整流程 | 8 | ~50 | ~350 行 |
| Semver 排序 closure | 12+ | ~12 | ~130 行 |
| Prerelease 过滤 | 15+ | ~6 | ~80 行 |
| npm registry 解析 | 3 | ~30 | ~60 行 |
| Semver 验证 | 10+ | ~5 | ~40 行 |
| **总计** | | | **~660 行** |

## 设计

### 架构概览

```
┌─────────────────────────────────────────────────────────────────┐
│                        Provider Runtime                          │
│                                                                  │
│  async fn fetch_versions(&self, ctx: &RuntimeContext)           │
│      -> Result<Vec<VersionInfo>>                                │
└───────────────────────────┬─────────────────────────────────────┘
                            │ 调用
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

### 核心 Trait 定义

```rust
// crates/vx-version-fetcher/src/lib.rs

use async_trait::async_trait;
use vx_runtime::{RuntimeContext, VersionInfo};

/// 版本获取器核心 trait
#[async_trait]
pub trait VersionFetcher: Send + Sync {
    /// 获取版本列表
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>>;
    
    /// 获取器名称（用于调试和日志）
    fn name(&self) -> &str;
    
    /// 获取数据源 URL（用于错误提示）
    fn source_url(&self) -> Option<String> { None }
}

/// 版本获取器构建器
pub struct VersionFetcherBuilder {
    fetcher: Box<dyn VersionFetcher>,
    filters: Vec<Box<dyn VersionFilter>>,
    transformer: Option<Box<dyn VersionTransformer>>,
    sorter: Option<Box<dyn VersionSorter>>,
    limit: Option<usize>,
}

impl VersionFetcherBuilder {
    /// 创建 jsDelivr CDN 获取器
    pub fn jsdelivr(owner: &str, repo: &str) -> Self { ... }
    
    /// 创建 npm registry 获取器
    pub fn npm(package: &str) -> Self { ... }
    
    /// 创建 PyPI 获取器
    pub fn pypi(package: &str) -> Self { ... }
    
    /// 创建 GitHub Releases 获取器
    pub fn github_releases(owner: &str, repo: &str) -> Self { ... }
    
    /// 创建自定义 API 获取器
    pub fn custom_api<F>(url: &str, parser: F) -> Self 
    where F: Fn(&serde_json::Value) -> Result<Vec<VersionInfo>> { ... }
    
    /// 创建静态版本列表
    pub fn static_versions(versions: Vec<&str>) -> Self { ... }
    
    // === 链式配置 ===
    
    /// 设置版本前缀处理
    pub fn strip_prefix(self, prefix: &str) -> Self { ... }
    
    /// 设置 tag 前缀（如 "bun-v"）
    pub fn tag_prefix(self, prefix: &str) -> Self { ... }
    
    /// 跳过 prerelease 版本
    pub fn skip_prereleases(self) -> Self { ... }
    
    /// 自定义 prerelease 检测器
    pub fn prerelease_markers(self, markers: &[&str]) -> Self { ... }
    
    /// 设置 LTS 检测器
    pub fn lts_detector<F>(self, detector: F) -> Self 
    where F: Fn(&str) -> bool { ... }
    
    /// 限制返回版本数量
    pub fn limit(self, max: usize) -> Self { ... }
    
    /// 自定义过滤器
    pub fn filter<F>(self, filter: F) -> Self 
    where F: Fn(&VersionInfo) -> bool { ... }
    
    /// 构建获取器
    pub fn build(self) -> Box<dyn VersionFetcher> { ... }
}
```

### 内置获取器实现

#### 1. jsDelivr CDN 获取器

```rust
// crates/vx-version-fetcher/src/fetchers/jsdelivr.rs

/// jsDelivr CDN 版本获取器
/// 
/// 使用 jsDelivr 的 API 来获取 GitHub 仓库的 tag 列表，
/// 避免 GitHub API 的速率限制问题。
pub struct JsDelivrFetcher {
    owner: String,
    repo: String,
    config: JsDelivrConfig,
}

#[derive(Default, Clone)]
pub struct JsDelivrConfig {
    /// 要去除的版本前缀（如 "v", "jq-", "bun-v"）
    pub strip_prefix: Option<String>,
    /// 是否跳过 prerelease
    pub skip_prereleases: bool,
    /// prerelease 标记列表
    pub prerelease_markers: Vec<String>,
    /// 最大返回版本数
    pub max_versions: usize,
    /// LTS 检测函数
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

#### 2. npm Registry 获取器

```rust
// crates/vx-version-fetcher/src/fetchers/npm.rs

/// npm Registry 版本获取器
pub struct NpmFetcher {
    package: String,
    config: NpmConfig,
}

#[derive(Default, Clone)]
pub struct NpmConfig {
    /// 是否跳过 prerelease（带 - 的版本）
    pub skip_prereleases: bool,
    /// 最大返回版本数
    pub max_versions: usize,
    /// 是否包含发布日期
    pub include_release_date: bool,
    /// LTS 检测函数
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

#### 3. PyPI 获取器

```rust
// crates/vx-version-fetcher/src/fetchers/pypi.rs

/// PyPI 版本获取器
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
        
        // ... 解析逻辑
    }
    
    fn name(&self) -> &str { "PyPI" }
}
```

#### 4. GitHub Releases 获取器（保留现有功能）

```rust
// crates/vx-version-fetcher/src/fetchers/github.rs

/// GitHub Releases 版本获取器
/// 
/// 直接使用 GitHub API，支持：
/// - GITHUB_TOKEN 认证
/// - jsDelivr 自动 fallback
/// - 丰富的 release 信息（描述、资产等）
pub struct GitHubReleasesFetcher {
    owner: String,
    repo: String,
    options: GitHubReleaseOptions,
}

// 复用现有的 GitHubReleaseOptions 设计
#[derive(Default, Clone)]
pub struct GitHubReleaseOptions {
    pub strip_v_prefix: bool,
    pub tag_prefix: Option<String>,
    pub skip_prereleases: bool,
    pub per_page: usize,
    pub lts_detector: Option<Box<dyn Fn(&str) -> bool + Send + Sync>>,
    /// 当 GitHub API 失败时是否自动 fallback 到 jsDelivr
    pub jsdelivr_fallback: bool,
}

#[async_trait]
impl VersionFetcher for GitHubReleasesFetcher {
    async fn fetch(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // 先尝试 GitHub API
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

#### 5. 自定义 API 获取器

```rust
// crates/vx-version-fetcher/src/fetchers/custom.rs

/// 自定义 API 版本获取器
/// 
/// 支持任意 JSON API，通过闭包自定义解析逻辑
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

### 版本工具模块

```rust
// crates/vx-version-fetcher/src/utils.rs

/// 版本处理工具函数
pub mod version_utils {
    use vx_runtime::VersionInfo;

    /// 解析 semver 为元组 (major, minor, patch)
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

    /// 按版本号降序排序
    pub fn sort_versions_desc(versions: &mut [VersionInfo]) {
        versions.sort_by(|a, b| {
            parse_semver_tuple(&b.version).cmp(&parse_semver_tuple(&a.version))
        });
    }

    /// 检查是否为 prerelease 版本
    pub fn is_prerelease(version: &str) -> bool {
        let lower = version.to_lowercase();
        lower.contains("-alpha")
            || lower.contains("-beta")
            || lower.contains("-rc")
            || lower.contains("-dev")
            || lower.contains("canary")
    }
    
    /// 检查是否为 prerelease（使用自定义标记）
    pub fn is_prerelease_with_markers(version: &str, markers: &[&str]) -> bool {
        let lower = version.to_lowercase();
        markers.iter().any(|m| lower.contains(m))
    }

    /// 验证基本 semver 格式
    pub fn is_valid_semver(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        parts.len() >= 2 && parts[0].parse::<u32>().is_ok()
    }

    /// 去除版本前缀
    pub fn strip_version_prefix<'a>(version: &'a str, prefix: &str) -> Option<&'a str> {
        if prefix.is_empty() {
            Some(version.trim_start_matches('v'))
        } else {
            version.strip_prefix(prefix)
        }
    }
    
    /// 比较两个版本号
    pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
        parse_semver_tuple(a).cmp(&parse_semver_tuple(b))
    }
}
```

### 使用示例

#### 重构前 (当前代码)

```rust
// crates/vx-providers/helm/src/runtime.rs (60+ 行)

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

#### 重构后 (使用新 API)

```rust
// crates/vx-providers/helm/src/runtime.rs (5 行)

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

#### 更多示例

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

// node - 官方 API
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    VersionFetcherBuilder::custom_api(
        "https://nodejs.org/dist/index.json",
        |response| {
            // 自定义解析逻辑
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

### Crate 结构

```
crates/vx-version-fetcher/
├── Cargo.toml
├── src/
│   ├── lib.rs              # 公共导出
│   ├── fetcher.rs          # VersionFetcher trait
│   ├── builder.rs          # VersionFetcherBuilder
│   ├── utils.rs            # version_utils 模块
│   ├── fetchers/
│   │   ├── mod.rs
│   │   ├── jsdelivr.rs     # JsDelivrFetcher
│   │   ├── npm.rs          # NpmFetcher
│   │   ├── pypi.rs         # PyPiFetcher
│   │   ├── github.rs       # GitHubReleasesFetcher
│   │   └── custom.rs       # CustomApiFetcher
│   └── error.rs            # 错误类型
└── tests/
    ├── jsdelivr_tests.rs
    ├── npm_tests.rs
    └── utils_tests.rs
```

### 依赖关系

```
vx-version-fetcher
├── vx-runtime (RuntimeContext, VersionInfo)
├── async-trait
├── serde_json
├── anyhow
└── tracing
```

## 迁移计划

### Phase 1: 创建 crate 和基础设施 (1-2 天)

1. 创建 `vx-version-fetcher` crate
2. 实现 `VersionFetcher` trait 和 `VersionFetcherBuilder`
3. 提取 `version_utils` 模块
4. 添加单元测试

### Phase 2: 实现内置获取器 (2-3 天)

1. 实现 `JsDelivrFetcher`
2. 实现 `NpmFetcher`
3. 实现 `PyPiFetcher`
4. 迁移 `GitHubReleasesFetcher`（从 vx-runtime）
5. 实现 `CustomApiFetcher`

### Phase 3: 迁移现有 Provider (2-3 天)

**优先级高**（使用 jsDelivr，代码最重复）：
- [ ] `helm`
- [ ] `kubectl`
- [ ] `bun`
- [ ] `deno`
- [ ] `uv`
- [ ] `ninja`
- [ ] `just`
- [ ] `jq`

**优先级中**（使用 npm registry）：
- [ ] `pnpm`
- [ ] `yarn`

**优先级低**（使用官方 API）：
- [ ] `node` (保持现有实现，但可用 custom_api 简化)
- [ ] `go`
- [ ] `python`
- [ ] `java`

### Phase 4: 清理和文档 (1 天)

1. 移除 `vx-runtime` 中的 `fetch_github_releases` 等冗余代码
2. 更新 Provider 开发文档
3. 添加使用示例

## 兼容性

### 向后兼容

- `RuntimeContext.fetch_json_versions()` 保留但标记为 deprecated
- `GitHubReleaseOptions` 保留，内部实现移至新 crate
- 现有 Provider 无需立即迁移

### API 稳定性

- `VersionFetcher` trait 保持稳定
- `VersionFetcherBuilder` 可以扩展新方法
- `version_utils` 模块函数签名稳定

## 测试策略

### 单元测试

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

### 集成测试

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

## 未来扩展

### 可能的新获取器

- **Homebrew Formula**: `https://formulae.brew.sh/api/formula/{name}.json`
- **Chocolatey**: `https://community.chocolatey.org/api/v2/package-versions/{name}`
- **APT Repository**: 解析 `Packages` 文件
- **Cargo Registry**: `https://crates.io/api/v1/crates/{name}`

### 缓存层

```rust
// 未来可能添加
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

### 镜像支持

```rust
// 未来可能添加
pub struct MirroredFetcher<F: VersionFetcher> {
    primary: F,
    mirrors: Vec<Box<dyn VersionFetcher>>,
}
```

## 决策记录

### 为什么使用 Builder 模式？

1. **类型安全**：编译时检查配置有效性
2. **可读性**：链式调用清晰表达意图
3. **可扩展**：易于添加新配置选项
4. **IDE 友好**：自动补全支持

### 为什么保留 `fetch_json_versions`？

1. **向后兼容**：不破坏现有代码
2. **灵活性**：某些场景仍需完全自定义解析
3. **渐进迁移**：允许 Provider 逐步迁移

### 为什么 jsDelivr 优先于 GitHub API？

1. **无速率限制**：jsDelivr 是 CDN，无 API 限制
2. **更快响应**：CDN 缓存，响应更快
3. **无需认证**：不需要 GITHUB_TOKEN
4. **更稳定**：不受 GitHub API 故障影响

## 参考

- [jsDelivr API 文档](https://www.jsdelivr.com/docs/data.jsdelivr.com)
- [npm Registry API](https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md)
- [PyPI JSON API](https://warehouse.pypa.io/api-reference/json.html)
- [GitHub REST API - Releases](https://docs.github.com/en/rest/releases)
