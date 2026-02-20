# RFC 0037: Provider.star Unified Facade

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-02-20
> **目标版本**: v0.15.0

## 摘要

将 `provider.star` 确立为所有 Provider 逻辑的**唯一权威来源**，通过引入 `ProviderHandle` 统一门面，使 CLI 命令、版本管理、路径查询、安装、执行等所有操作全部由 `provider.star` 中定义的函数驱动。Rust 代码退化为纯粹的"注册桩"和"执行桥"，不再包含任何业务逻辑。`provider.toml` 不再作为 Provider 逻辑的载体，所有 Provider 统一迁移到 `provider.star`。

## 动机

### 当前架构的问题

RFC 0036 引入了 `provider.star` 并实现了 `fetch_versions` 的 Starlark 化，但当前架构仍存在以下问题：

**1. 逻辑分散**

```
vx where 7zip
  → where_cmd.rs (硬编码路径扫描逻辑)
  → PathResolver::find_latest_executable_with_exe()
  → 硬编码路径扫描，完全不知道 provider.star 中定义的 system_paths
```

`provider.star` 中已经定义了 `system_paths`（在 `runtimes` 列表和 `prepare_execution` 中），但 `where` 命令完全不知道这些信息，而是通过硬编码路径扫描来实现。

**2. 功能重复**

| 功能 | provider.star 定义 | Rust 实现 |
|------|-------------------|-----------|
| 版本获取 | `fetch_versions(ctx)` | `ManifestDrivenRuntime::fetch_versions()` |
| 下载 URL | `download_url(ctx, version)` | `ManifestDrivenRuntime::download_url()` |
| 安装布局 | `install_layout(ctx, version)` | `ManifestDrivenRuntime::install_layout()` |
| 系统路径 | `prepare_execution(ctx, version)` | `where_cmd::find_via_detection_paths()` |
| 环境变量 | `environment(ctx, version, dir)` | 各 provider 的 Rust 实现 |
| 依赖关系 | `deps(ctx, version)` | `DependencyMap::with_defaults()` |

**3. 扩展困难**

添加新 Provider 需要同时维护 `provider.star` 和 Rust 代码，两者容易不同步。

### 目标

```
所有 Provider 逻辑 → provider.star（唯一权威）
所有 CLI 命令     → ProviderHandle（统一门面）
Rust 代码         → 注册桩 + 执行桥（无业务逻辑）
```

## 设计方案

### 核心概念：ProviderHandle

`ProviderHandle` 是 CLI 层与 `provider.star` 之间的统一门面，屏蔽 Starlark 执行细节：

```rust
/// CLI 层统一的 Provider 调用门面
/// 所有业务逻辑委托给 provider.star，Rust 只负责执行
pub struct ProviderHandle {
    /// Provider 名称（如 "7zip", "node"）
    name: String,
    /// Starlark provider 实例（懒加载）
    star: Arc<StarlarkProvider>,
    /// 路径管理
    paths: Arc<VxPaths>,
    /// 平台信息（注入到 ctx）
    platform: Platform,
}
```

### provider.star 完整 API 规范

每个 `provider.star` 必须实现以下函数（当前已有的保持不变，新增标注 `[NEW]`）：

```python
# ── 元数据（已有）──────────────────────────────────────────────
def name() -> str
def description() -> str
def homepage() -> str
def repository() -> str
def license() -> str
def ecosystem() -> str

# ── Runtime 定义（已有）────────────────────────────────────────
runtimes = [{ "name", "executable", "aliases", "system_paths", ... }]

# ── 版本管理（已有）────────────────────────────────────────────
def fetch_versions(ctx) -> [VersionInfo]
def download_url(ctx, version) -> str | None
def install_layout(ctx, version) -> InstallDescriptor | None

# ── 执行准备（已有）────────────────────────────────────────────
def prepare_execution(ctx, version) -> ExecutionDescriptor | None
def environment(ctx, version, install_dir) -> {str: str}
def deps(ctx, version) -> [str]

# ── 路径查询 [NEW] ─────────────────────────────────────────────
def store_root(ctx) -> str
    """返回该 provider 在 vx store 中的根目录路径模板。
    
    示例：
        return "{vx_home}/store/7zip"
    """

def get_execute_path(ctx, version) -> str | None
    """返回指定版本的可执行文件路径。
    
    示例：
        os = ctx["platform"]["os"]
        exe = "7z.exe" if os == "windows" else "7zz"
        return "{install_dir}/" + exe
    """

# ── 版本解析 [NEW] ─────────────────────────────────────────────
def resolve_version(ctx, spec) -> str | None
    """将版本规格（如 "latest", "^24"）解析为具体版本号。
    
    默认实现：返回 None（由 Rust 层处理 latest/range 解析）
    自定义实现：可以实现特殊的版本解析逻辑
    """

# ── 安装后处理 [NEW] ───────────────────────────────────────────
def post_install(ctx, version, install_dir) -> PostInstallDescriptor | None
    """安装完成后的处理步骤（如创建符号链接、设置权限等）。
    
    示例（macOS 7zip 需要创建 7z -> 7zz 符号链接）：
        if ctx["platform"]["os"] == "macos":
            return symlink_create("7zz", "7z", install_dir)
        return None
    """
```

### ProviderHandle API

```rust
impl ProviderHandle {
    // ── 构造 ──────────────────────────────────────────────────
    
    /// 从 provider 名称加载（自动查找 provider.star）
    pub async fn load(name: &str) -> Result<Self>;
    
    /// 从内嵌的 provider.star 内容加载（用于内置 provider）
    pub async fn from_content(name: &str, content: &'static str) -> Result<Self>;

    // ── 元数据（零成本，解析时缓存）──────────────────────────
    
    pub fn meta(&self) -> &StarMetadata;
    pub fn name(&self) -> &str;
    pub fn description(&self) -> &str;
    pub fn homepage(&self) -> Option<&str>;
    pub fn runtimes(&self) -> &[StarRuntimeMeta];

    // ── 版本管理 ──────────────────────────────────────────────
    
    /// 获取可用版本列表（委托给 provider.star::fetch_versions）
    /// 对应 `vx versions <tool>`
    pub async fn versions(&self, filter: VersionFilter) -> Result<Vec<VersionInfo>>;
    
    /// 获取已安装版本列表（扫描 store 目录）
    pub async fn installed_versions(&self) -> Result<Vec<String>>;
    
    /// 检查某版本是否已安装
    pub fn is_installed(&self, version: &str) -> bool;

    // ── 路径查询 ──────────────────────────────────────────────
    
    /// 获取 store 根目录（委托给 provider.star::store_root 或默认实现）
    /// 对应 `vx where <tool>`（无版本）
    pub fn store_root(&self) -> PathBuf;
    
    /// 获取指定版本的可执行文件路径（委托给 provider.star::get_execute_path）
    /// 对应 `vx where <tool>@<version>`
    pub fn get_execute_path(&self, version: &str) -> Option<PathBuf>;
    
    /// 获取最新已安装版本的可执行文件路径
    /// 对应 `vx where <tool>`（有安装版本时）
    pub fn get_latest_execute_path(&self) -> Option<PathBuf>;

    // ── 安装 ──────────────────────────────────────────────────
    
    /// 获取下载 URL（委托给 provider.star::download_url）
    pub async fn download_url(&self, version: &str) -> Result<Option<String>>;
    
    /// 获取安装布局（委托给 provider.star::install_layout）
    pub async fn install_layout(&self, version: &str) -> Result<Option<InstallLayout>>;
    
    /// 获取安装后处理步骤（委托给 provider.star::post_install）
    pub async fn post_install(&self, version: &str, install_dir: &Path) -> Result<Option<PostInstallOps>>;

    // ── 执行 ──────────────────────────────────────────────────
    
    /// 执行前准备（委托给 provider.star::prepare_execution）
    /// 包含系统工具查找、路径解析等
    pub async fn prepare_execution(&self, version: &str) -> Result<ExecutionPrep>;
    
    /// 获取环境变量（委托给 provider.star::environment）
    pub async fn environment(&self, version: &str, install_dir: &Path) -> Result<HashMap<String, String>>;
    
    /// 获取依赖（委托给 provider.star::deps）
    pub async fn deps(&self, version: &str) -> Result<Vec<String>>;
}
```

### CLI 命令改造

#### `vx versions <tool>` 命令

**改造前**：
```rust
// versions_cmd.rs - 直接调用 runtime trait
let runtime = registry.get_runtime(tool)?;
let versions = runtime.fetch_versions(&ctx).await?;
```

**改造后**：
```rust
// versions_cmd.rs - 通过 ProviderHandle
let handle = ProviderHandle::load(tool).await?;
let versions = handle.versions(filter).await?;
```

#### `vx where <tool>` 命令

**改造前**：
```rust
// where_cmd.rs - 硬编码路径扫描，不知道 provider.star
let resolver = PathResolver::new(path_manager);
let path = resolver.find_latest_executable_with_exe(&canonical_name, &exe_name);
// 还需要 find_via_detection_paths() 硬编码扫描系统路径
```

**改造后**：
```rust
// where_cmd.rs - 通过 ProviderHandle，provider.star 提供所有路径信息
let handle = ProviderHandle::load(tool).await?;
match version {
    Some(v) => {
        // vx where 7zip@24.09
        let path = handle.get_execute_path(v);
        render_path(path);
    }
    None => {
        // vx where 7zip → 最新已安装版本路径，或 store 根目录
        let path = handle.get_latest_execute_path()
            .or_else(|| Some(handle.store_root()));
        render_path(path);
    }
}
```

#### `vx install <tool>@<version>` 命令

**改造后**：
```rust
let handle = ProviderHandle::load(tool).await?;
let url = handle.download_url(version).await?;
let layout = handle.install_layout(version).await?;
// 安装...
let post_ops = handle.post_install(version, &install_dir).await?;
// 执行后处理...
```

### provider.star 新增函数示例（7zip）

```python
# ---------------------------------------------------------------------------
# 路径查询（新增）
# ---------------------------------------------------------------------------

def store_root(ctx):
    """7zip 在 vx store 中的根目录。"""
    # 返回路径模板，{vx_home} 由 Rust 层替换
    return "{vx_home}/store/7zip"

def get_execute_path(ctx, version):
    """返回指定版本的 7z 可执行文件路径。"""
    os = ctx["platform"]["os"]
    # 路径模板，{install_dir} 由 Rust 层替换为实际安装目录
    if os == "windows":
        return "{install_dir}/7z.exe"
    elif os == "macos":
        return "{install_dir}/7zz"
    else:
        return "{install_dir}/7zz"

def post_install(ctx, version, install_dir):
    """macOS 上创建 7z -> 7zz 的符号链接。"""
    os = ctx["platform"]["os"]
    if os == "macos":
        return {
            "type": "symlink",
            "source": install_dir + "/7zz",
            "target": install_dir + "/7z",
        }
    return None
```

### Rust 代码桩（最终形态）

改造完成后，每个 provider 的 Rust 代码只需要：

```rust
// crates/vx-providers/7zip/src/provider.rs
//! 7zip provider - 纯注册桩，所有逻辑由 provider.star 提供

use std::sync::Arc;
use vx_runtime::{provider::Provider, Runtime};

pub const PROVIDER_STAR: &str = include_str!("../provider.star");

/// 7zip provider（Starlark 驱动）
#[derive(Debug, Default)]
pub struct SevenZipProvider;

impl Provider for SevenZipProvider {
    fn name(&self) -> &str { "7zip" }
    fn description(&self) -> &str { "7-Zip file archiver" }
    
    fn runtimes(&self) -> Vec<Arc<dyn Runtime>> {
        // 所有逻辑由 provider.star 提供
        // Rust 只负责将 provider.star 注册到运行时系统
        vx_starlark::make_runtimes_from_star("7zip", PROVIDER_STAR)
    }
}
```

### ProviderHandle 注册表

```rust
/// 全局 ProviderHandle 注册表
/// 支持按名称或别名查找
pub struct ProviderHandleRegistry {
    handles: HashMap<String, Arc<ProviderHandle>>,
    aliases: HashMap<String, String>,  // alias -> canonical name
}

impl ProviderHandleRegistry {
    /// 注册一个 provider（从内嵌 star 内容）
    pub async fn register_builtin(&mut self, name: &str, star_content: &'static str) -> Result<()>;
    
    /// 注册一个 provider（从文件路径，用于用户自定义 provider）
    pub async fn register_from_file(&mut self, path: &Path) -> Result<()>;
    
    /// 按名称或别名获取 ProviderHandle
    pub fn get(&self, name: &str) -> Option<Arc<ProviderHandle>>;
    
    /// 列出所有已注册的 provider 名称
    pub fn names(&self) -> Vec<&str>;
}
```

## 实现计划

### Phase 1：ProviderHandle 基础实现（v0.15.0）

- [ ] 在 `vx-starlark` 中实现 `ProviderHandle` 结构体
- [ ] 实现 `store_root()` 和 `get_execute_path()` 的默认逻辑（基于 VxPaths）
- [ ] 为 `provider.star` 添加可选的 `store_root` 和 `get_execute_path` 函数支持
- [ ] 实现 `ProviderHandleRegistry`
- [ ] 为所有内置 provider 注册到 `ProviderHandleRegistry`

### Phase 2：CLI 命令迁移（v0.15.0）

- [ ] `vx where` 命令改用 `ProviderHandle::get_execute_path` / `get_latest_execute_path`
- [ ] `vx versions` 命令改用 `ProviderHandle::versions`
- [ ] `vx install` 命令改用 `ProviderHandle::download_url` / `install_layout` / `post_install`
- [ ] 删除 `where_cmd.rs` 中的 `find_via_detection_paths` 硬编码逻辑

### Phase 3：provider.star 补全（v0.15.0）

- [ ] 为所有内置 provider 的 `provider.star` 添加 `store_root` 函数
- [ ] 为所有内置 provider 的 `provider.star` 添加 `get_execute_path` 函数
- [ ] 为需要安装后处理的 provider 添加 `post_install` 函数

### Phase 4：Rust 代码桩精简（v0.16.0）

- [ ] 实现 `vx_starlark::make_runtimes_from_star()` 工厂函数
- [ ] 将所有 provider 的 `runtimes()` 实现替换为 `make_runtimes_from_star()`
- [ ] 删除各 provider 中重复的 Rust 业务逻辑
- [ ] 验证所有 provider 功能正常

### Phase 5：用户自定义 provider 支持（v0.16.0）

- [ ] 支持从 `~/.vx/providers/*/provider.star` 加载用户自定义 provider
- [ ] 支持从项目目录 `.vx/providers/*/provider.star` 加载项目级 provider
- [ ] 实现 provider 热重载（开发模式）

## 迁移策略

### provider.toml 迁移

现有使用 `provider.toml` 的内置 Provider 需要全部迁移到 `provider.star`。迁移原则：

- `provider.toml` 中的静态字段（`name`、`description`、`homepage` 等）→ `provider.star` 中的对应函数
- `runtimes[].versions` 配置 → `fetch_versions(ctx)` 函数
- `runtimes[].layout` 配置 → `install_layout(ctx, version)` 函数
- `runtimes[].detection` 配置 → `prepare_execution(ctx, version)` 函数
- `runtimes[].env` 配置 → `environment(ctx, version, install_dir)` 函数

迁移完成后，`provider.toml` 文件将被删除，不再作为 Provider 逻辑的载体。

### 新增函数的约定默认实现

`store_root`、`get_execute_path`、`post_install` 等新增函数在 `provider.star` 中是**必须实现**的。为降低迁移成本，`ProviderHandle` 在 `provider.star` 未定义这些函数时提供基于约定的默认实现（Convention over Configuration）：

```rust
impl ProviderHandle {
    pub fn get_execute_path(&self, version: &str) -> Option<PathBuf> {
        // 优先调用 provider.star::get_execute_path
        if let Ok(Some(path)) = self.star.call_get_execute_path(version) {
            return Some(path);
        }
        // 约定默认实现：{store}/{name}/{version}/bin/{name}[.exe]
        // 迁移期间的临时兜底，迁移完成后移除
        self.convention_execute_path(version)
    }
}
```

## 替代方案

### 方案 A：保持现状，仅扩展 StarlarkProvider API

只在 `StarlarkProvider` 上添加新方法，不引入 `ProviderHandle`。

**缺点**：CLI 命令仍需直接依赖 `StarlarkProvider`，调用方式不统一，无法形成清晰的分层架构。

### 方案 B：provider.toml + provider.star 双轨并存

保留 `provider.toml` 作为简单 Provider 的描述格式，`provider.star` 只用于复杂逻辑。

**缺点**：两套格式长期并存，维护成本高，新增 Provider 时需要决策使用哪种格式，增加认知负担。

### 选择方案（本 RFC）：provider.star 唯一驱动 + ProviderHandle 统一门面

`provider.star` 是所有 Provider 逻辑的唯一来源，`ProviderHandle` 是 CLI 层的统一调用门面。这样的架构：
- **最大化 Starlark 的灵活性**：所有逻辑都可以用 Python 语法表达，包括复杂的平台差异处理
- **最小化 Rust 代码**：Rust 只做注册和执行桥接，不包含任何业务逻辑
- **统一扩展点**：内置 Provider、用户自定义 Provider、项目级 Provider 使用完全相同的机制

## 参考资料

- [RFC 0036: Starlark Provider Support](./0036-starlark-provider-support.md) - Starlark 基础设施
- [RFC 0015: System Tool Discovery](./0015-system-tool-discovery.md) - 系统工具发现
- [Buck2 Provider Design](https://buck2.build/docs/rule_authors/writing_rules/) - 两阶段执行模式参考
- [7zip provider.star](../../crates/vx-providers/7zip/provider.star) - 当前实现参考

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-20 | Draft | 初始草案 |
| 2026-02-20 | Draft v2 | 移除 provider.toml 兼容性，确立 provider.star 为唯一驱动 |
