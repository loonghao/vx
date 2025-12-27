# Provider 开发指南

本指南说明如何为 vx 创建新的工具 Provider。Provider 是核心扩展机制，使 vx 能够支持不同的开发工具。

## 架构概览

vx 使用 **Provider-Runtime** 架构：

- **Provider**: 相关运行时的容器（例如 `NodeProvider` 提供 `node`、`npm`、`npx`）
- **Runtime**: 实际的工具实现（版本获取、安装、执行）

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

## Provider 结构

Provider 是位于 `crates/vx-providers/` 下的 Rust crate：

```
crates/vx-providers/
├── node/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # 模块导出
│       ├── provider.rs     # Provider 实现
│       ├── runtime.rs      # Runtime 实现
│       └── config.rs       # 配置（可选）
├── go/
├── rust/
└── ...
```

## 分步指南

### 1. 创建 Crate

在 `crates/vx-providers/` 下创建新目录：

```bash
mkdir -p crates/vx-providers/mytool/src
```

创建 `Cargo.toml`：

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

### 2. 实现 Runtime Trait

`Runtime` trait 是核心抽象。只有两个方法是**必需**的：

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
    // ========== 必需方法 ==========

    /// 运行时名称 - 用作命令名
    fn name(&self) -> &str {
        "mytool"
    }

    /// 从官方源获取可用版本
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // 示例：从 GitHub releases API 获取
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

    // ========== 可选方法（有默认实现） ==========

    fn description(&self) -> &str {
        "MyTool - 一个很棒的开发工具"
    }

    fn aliases(&self) -> &[&str] {
        &["mt", "my-tool"]  // 别名
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Unknown  // 或 NodeJs, Go, Rust, Python 等
    }

    /// 获取特定版本和平台的下载 URL
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

    /// 自定义解压后的可执行文件路径
    fn executable_relative_path(&self, _version: &str, platform: &Platform) -> String {
        if platform.os == "windows" {
            "mytool.exe".to_string()
        } else {
            "mytool".to_string()
        }
    }
}
```

### 3. 实现 Provider Trait

`Provider` 将相关运行时分组：

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
        "MyTool 开发工具"
    }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MyToolRuntime::new())]
    }
}
```

### 4. 从 lib.rs 导出

```rust
// src/lib.rs
mod provider;
mod runtime;

pub use provider::MyToolProvider;
pub use runtime::MyToolRuntime;
```

### 5. 注册 Provider

在 `crates/vx-cli/src/registry.rs` 中添加你的 provider：

```rust
use vx_provider_mytool::MyToolProvider;

pub fn create_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();

    // 注册你的 provider
    registry.register(Box::new(MyToolProvider));

    // ... 其他 providers
    registry
}
```

在 `crates/vx-cli/Cargo.toml` 中添加依赖：

```toml
[dependencies]
vx-provider-mytool = { path = "../vx-providers/mytool" }
```

## 生命周期钩子

`Runtime` trait 提供生命周期钩子用于自定义：

### 安装钩子

```rust
#[async_trait]
impl Runtime for MyToolRuntime {
    /// 安装前调用 - 验证环境
    async fn pre_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        // 检查系统要求
        Ok(())
    }

    /// 解压后调用 - 重命名文件、设置权限
    fn post_extract(&self, version: &str, install_path: &PathBuf) -> Result<()> {
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

    /// 安装成功后调用
    async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
        // 运行初始化、安装捆绑工具等
        Ok(())
    }
}
```

### 执行钩子

```rust
#[async_trait]
impl Runtime for MyToolRuntime {
    /// 命令执行前调用
    async fn pre_execute(&self, args: &[String], ctx: &ExecutionContext) -> Result<()> {
        // 设置环境、验证参数
        Ok(())
    }

    /// 命令执行后调用
    async fn post_execute(
        &self,
        args: &[String],
        result: &ExecutionResult,
        ctx: &ExecutionContext,
    ) -> Result<()> {
        // 记录结果、清理临时文件
        Ok(())
    }
}
```

## 测试

在 `tests/` 目录创建测试（遵循项目约定）：

```rust
// tests/runtime_tests.rs
use rstest::rstest;
use vx_provider_mytool::MyToolRuntime;
use vx_runtime::Runtime;

#[rstest]
#[tokio::test]
async fn test_fetch_versions() {
    let runtime = MyToolRuntime::new();
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

## 最佳实践

### 1. 处理平台差异

```rust
async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
    let (os, arch) = match (platform.os.as_str(), platform.arch.as_str()) {
        ("macos", "x86_64") => ("darwin", "amd64"),
        ("macos", "aarch64") => ("darwin", "arm64"),
        ("linux", "x86_64") => ("linux", "amd64"),
        ("linux", "aarch64") => ("linux", "arm64"),
        ("windows", "x86_64") => ("windows", "amd64"),
        _ => return Ok(None), // 不支持的平台
    };
    // ...
}
```

### 2. 提供良好的错误消息

```rust
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    ctx.http
        .get_json_value(API_URL)
        .await
        .map_err(|e| anyhow::anyhow!(
            "从 {} 获取 MyTool 版本失败: {}。\
             请检查网络连接或稍后重试。",
            API_URL, e
        ))?;
    // ...
}
```

## 示例 Provider

参考现有 provider：

| Provider | 特性 | 位置 |
|----------|------|------|
| `node` | 多运行时（node、npm、npx）、LTS 支持 | `crates/vx-providers/node/` |
| `go` | 简单的单运行时 | `crates/vx-providers/go/` |
| `uv` | Python 生态、uvx 运行器 | `crates/vx-providers/uv/` |
| `rust` | 多命令（cargo、rustc、rustup） | `crates/vx-providers/rust/` |
| `pnpm` | 解压后文件重命名 | `crates/vx-providers/pnpm/` |

## 参见

- [CLI 命令开发](./cli-development) - 添加新 CLI 命令
- [Extension 开发](./extension-development) - 添加脚本扩展
- [架构概览](./architecture) - 系统架构
- [贡献指南](./contributing) - 如何贡献
