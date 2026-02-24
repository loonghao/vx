# Provider 开发指南

本指南说明如何为 vx 创建新的 Provider。推荐方式是 **Starlark 优先**：编写一个 `provider.star`
文件，让 vx 处理其余的一切。对于需要自定义 Rust 逻辑的高级场景，请参阅
[自定义 Rust Provider](#自定义-rust-provider) 章节。

## 两种方式对比

| 方式 | 适用场景 | 工作量 |
|------|---------|--------|
| **`provider.star`**（推荐） | GitHub releases、归档/二进制下载、PyPI/npm 工具、系统包管理器回退 | 分钟级 |
| **自定义 Rust Provider** | 自定义安装逻辑、复杂版本解析、非标准协议 | 小时级 |

---

## 方式一：provider.star（推荐）

对于绝大多数工具，一个 `provider.star` 文件就足够了。
完整参考请查阅[清单驱动 Provider 指南](../guide/manifest-driven-providers.md)。
以下是简明流程。

### 最小示例

```python
# crates/vx-providers/mytool/provider.star
load("@vx//stdlib:github.star", "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:env.star",    "env_prepend")

# --- 元数据 ---
name        = "mytool"
description = "我的超棒工具"
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

# --- 逻辑 ---
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

### 必需文件

```
crates/vx-providers/mytool/
├── provider.star     # 所有逻辑（必需）
└── provider.toml     # 仅元数据（内置 provider 必需）
```

**`provider.toml`** — 仅元数据，无 layout 字段：

```toml
[provider]
name        = "mytool"
description = "我的超棒工具"
homepage    = "https://github.com/myorg/mytool"
repository  = "https://github.com/myorg/mytool"
ecosystem   = "devtools"
license     = "MIT"
```

### 必需函数清单

每个 `provider.star` 必须实现：

| 函数 | 签名 | 说明 |
|------|------|------|
| `fetch_versions` | `fetch_versions(ctx)` 或 `make_fetch_versions(...)` | 返回版本列表 |
| `download_url` | `download_url(ctx, version) -> str\|None` | 返回下载 URL |
| `install_layout` | `install_layout(ctx, version) -> dict\|None` | 返回安装描述符 |
| `store_root` | `store_root(ctx) -> str` | 返回存储路径 |
| `get_execute_path` | `get_execute_path(ctx, version) -> str` | 返回可执行文件路径 |
| `post_install` | `post_install(ctx, version) -> None` | 安装后钩子 |
| `environment` | `environment(ctx, version) -> list` | 返回环境变量操作列表 |

可选函数：

| 函数 | 签名 | 说明 |
|------|------|------|
| `system_install` | `system_install(ctx) -> dict` | 系统包管理器回退 |
| `deps` | `deps(ctx, version) -> list` | 运行时依赖 |
| `uninstall` | `uninstall(ctx, version) -> None` | 自定义卸载逻辑 |

### 注册内置 Provider

创建 `provider.star` 和 `provider.toml` 后，在 Rust 注册表中注册 provider，
使其在启动时加载：

**`crates/vx-starlark/src/registry.rs`**（或对应的注册文件）：

```rust
// 将 provider 目录名添加到内置列表
pub const BUILTIN_PROVIDERS: &[&str] = &[
    "node",
    "go",
    // ... 现有 providers ...
    "mytool",   // ← 在此添加
];
```

除此注册行外，无需任何 Rust 代码。

---

## 方式二：自定义 Rust Provider

仅在 `provider.star` 不够用时才使用此方式，例如：
- 自定义认证流程
- 非 HTTP 安装源（如 S3、内部注册表）
- Starlark 无法表达的复杂安装后逻辑
- 需要封装其他 Rust crate 的 provider

### 目录结构

```
crates/vx-providers/mytool/
├── Cargo.toml
├── provider.star     # 即使有 Rust 代码也推荐保留
├── provider.toml
└── src/
    ├── lib.rs        # 模块导出
    ├── provider.rs   # Provider 实现
    └── runtime.rs    # Runtime 实现
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

### 实现 Runtime Trait

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
    // ── 必需 ──────────────────────────────────────────────────────────────

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

    // ── 可选 ──────────────────────────────────────────────────────────────

    fn description(&self) -> &str { "MyTool - 一个很棒的开发工具" }

    fn aliases(&self) -> &[&str] { &["mt"] }

    fn ecosystem(&self) -> Ecosystem { Ecosystem::Unknown }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        let triple = match (platform.os.as_str(), platform.arch.as_str()) {
            ("windows", "x86_64")  => "x86_64-pc-windows-msvc",
            ("macos",   "x86_64")  => "x86_64-apple-darwin",
            ("macos",   "aarch64") => "aarch64-apple-darwin",
            ("linux",   "x86_64")  => "x86_64-unknown-linux-musl",
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

### 实现 Provider Trait

```rust
// src/provider.rs
use std::sync::Arc;
use vx_runtime::{Provider, Runtime};
use crate::runtime::MyToolRuntime;

pub struct MyToolProvider;

impl Provider for MyToolProvider {
    fn name(&self) -> &str { "mytool" }

    fn description(&self) -> &str { "MyTool 开发工具" }

    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        vec![Arc::new(MyToolRuntime::new())]
    }
}
```

### 从 lib.rs 导出

```rust
// src/lib.rs
mod provider;
mod runtime;

pub use provider::MyToolProvider;
pub use runtime::MyToolRuntime;
```

### 注册 Provider

在 `crates/vx-cli/Cargo.toml` 中添加：

```toml
[dependencies]
vx-provider-mytool = { path = "../vx-providers/mytool" }
```

在 `crates/vx-cli/src/registry.rs` 中添加：

```rust
use vx_provider_mytool::MyToolProvider;

pub fn create_registry() -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();
    registry.register(Box::new(MyToolProvider));
    // ... 其他 providers
    registry
}
```

### 生命周期钩子

```rust
#[async_trait]
impl Runtime for MyToolRuntime {
    /// 解压后调用 — 重命名文件、设置权限
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

    /// 安装成功后调用
    async fn post_install(&self, _version: &str, _ctx: &RuntimeContext) -> Result<()> {
        // 运行初始化、安装捆绑工具等
        Ok(())
    }
}
```

---

## 测试

### 单元测试

测试放在 `tests/` 目录（不要内联 `#[cfg(test)]` 模块）：

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

### 测试 provider.star

对于 Starlark provider，通过临时 `VX_HOME` 运行 vx 命令来测试：

```bash
VX_HOME=/tmp/vx-test vx mytool --version
```

---

## 检查清单

### provider.star Provider

- [ ] 创建 `provider.star`，包含所有必需函数
- [ ] 创建 `provider.toml`（仅元数据，无 layout 字段）
- [ ] `license` 字段设置为 SPDX 标识符
- [ ] `runtimes` 列表包含 `test_commands`
- [ ] `download_url()` 覆盖所有主要平台（windows/x64、macos/x64、macos/arm64、linux/x64、linux/arm64）
- [ ] `install_layout()` 返回正确的 `strip_prefix` 和 `executable_paths`
- [ ] `environment()` 返回**列表**（不是字典）
- [ ] 如果工具可通过 brew/winget/choco 安装，添加 `system_install()`
- [ ] 在内置注册表中注册 provider
- [ ] 在至少一个平台上测试通过

### 自定义 Rust Provider（额外项）

- [ ] 创建 `Cargo.toml`，使用 workspace 依赖
- [ ] 实现 `Runtime` trait（`name()` + `fetch_versions()` 必需）
- [ ] 实现 `Provider` trait
- [ ] 测试放在 `tests/` 目录（不要内联）
- [ ] 添加到 `vx-cli/Cargo.toml` 依赖
- [ ] 在 `create_registry()` 中注册

---

## 参考 Provider

参考这些内置 provider 作为示例：

| Provider | 模式 | 位置 |
|----------|------|------|
| `ripgrep` | 标准 GitHub 二进制，归档布局 | `crates/vx-providers/ripgrep/` |
| `meson` | PyPI 包别名（`uvx`） | `crates/vx-providers/meson/` |
| `imagemagick` | 混合：直接下载（Linux）+ 系统包（Win/Mac） | `crates/vx-providers/imagemagick/` |
| `node` | 自定义 Rust provider，多运行时，LTS 支持 | `crates/vx-providers/node/` |
| `go` | 自定义 Rust provider，官方 API 版本获取 | `crates/vx-providers/go/` |
| `uv` | 自定义 Rust provider，Python 生态 | `crates/vx-providers/uv/` |

## 参见

- [清单驱动 Provider](../guide/manifest-driven-providers.md) — 完整 `provider.star` 参考
- [Extension 开发](./extension-development.md) — 脚本扩展
- [贡献指南](./contributing.md) — 如何提交 provider
