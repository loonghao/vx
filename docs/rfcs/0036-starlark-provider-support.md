# RFC 0036: Starlark Provider Support

> **状态**: Draft (v0.4)
> **作者**: vx team
> **创建日期**: 2026-02-19
> **目标版本**: v0.14.0

## 摘要

引入 [Starlark](https://github.com/bazelbuild/starlark) 语言作为 Provider 的脚本配置语言，与现有的 TOML 格式并存。Starlark 是一种 Python 方言，被 Bazel、Buck2 等构建系统广泛使用，具有表达能力强、安全沙箱、易于嵌入等优点。

本 RFC 设计：
1. **混合格式支持** - 同时支持 `provider.toml` 和 `provider.star`
2. **Starlark API 设计** - 为 Provider 开发提供安全的脚本 API
3. **沙箱安全模型** - 限制文件系统、网络访问，确保安全性
4. **Buck2 借鉴** - 引入 Frozen Provider、两阶段执行、Provider 组合等设计
5. **MSVC Provider 迁移示例** - 展示复杂 Provider 的 Starlark 实现

## 动机

### 当前 TOML 的局限性

经过对 62 个现有 Provider 的分析，以下场景 TOML 无法优雅处理：

| 场景 | TOML 表达能力 | 实际例子 |
|------|--------------|----------|
| 动态 URL 构建 | ❌ 无逻辑 | MSVC 需要根据架构/组件动态选择下载包 |
| 多步骤安装流程 | ❌ 无流程控制 | vcpkg 需要 git clone + sparse checkout |
| 复杂检测逻辑 | ❌ 无条件组合 | winget 需要检查 where + registry + env |
| 环境变量构建 | ❌ 无字符串操作 | MSVC 需要构建复杂的 INCLUDE/LIB/PATH |
| 组件存在性检查 | ❌ 无文件操作 | MSVC 需要检查 Spectre/MFC 组件是否存在 |

### Provider 复杂度分析

```
┌─────────────────────────────────────────────────────────────────┐
│                    Provider 代码行数分布                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  MSVC      ████████████████████████████████████████  1077 行    │
│  vcpkg     ███████████████████████████████████        809 行    │
│  ffmpeg    ███████████████████████████                 413 行    │
│  winget    ████████████████████                        215 行    │
│  brew      ██████████████                              139 行    │
│  docker    ████████████                                111 行    │
│  node      ████████████████████████                    ~200 行   │
│  go        ████████████████████                        ~150 行   │
│                                                                 │
│  ─────────────────────────────────────────────────────────────  │
│  简单 Provider (<200 行): TOML 足够          ~70%              │
│  复杂 Provider (>200 行): 需要脚本能力       ~30%              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 主流方案调研

### Buck2 插件设计

Meta 的 Buck2 构建系统是目前最成熟的 Starlark 嵌入实践，其设计对 vx 有重要参考价值。

#### Buck2 Provider 核心概念

Buck2 的 Provider 是**不可变（Frozen）的数据结构**，在分析阶段创建后即被冻结，在执行阶段只读消费：

```python
# Buck2 中的 Provider 定义
def my_rule_impl(ctx: "context") -> ["provider"]:
    binary = ctx.actions.declare_output(ctx.attrs.out)
    ctx.actions.run(["compiler", "-o", binary.as_output()])
    # 返回 Frozen Provider 列表
    return [
        DefaultInfo(default_output = binary),
        RunInfo(args = cmd_args(binary)),
    ]
```

**关键设计原则：**
1. **两阶段分离**：分析阶段（Analysis）声明意图，执行阶段（Execution）实施操作
2. **Frozen Values**：Provider 创建后不可变，保证线程安全和确定性
3. **Provider 组合**：规则通过返回 Provider 列表向上游传递信息
4. **显式依赖**：所有依赖必须在 `deps` 中声明，无隐式依赖

#### Buck2 Context 设计

Buck2 的 `ctx` 对象是扁平化的，而非嵌套对象：

```python
# Buck2 风格：扁平化 ctx
ctx.attrs.deps          # 依赖列表
ctx.actions.run(...)    # 声明执行动作
ctx.label               # 目标标识符
```

这与 vx 当前 RFC 设计的嵌套 `ctx.fs.exists()` 风格不同。Buck2 的扁平化设计更易于 IDE 自动补全和类型检查。

#### Buck2 Toolchain Provider 模式

Buck2 通过 Toolchain Provider 解耦工具链配置与规则实现：

```python
# 工具链 Provider（类似 vx 的 RuntimeInfo）
RuntimeInfo = provider(fields = {
    "executable": provider_field(Artifact),
    "version": provider_field(str),
    "env": provider_field(dict[str, str]),
})

# 规则通过 toolchain 获取工具链信息
def compile_impl(ctx):
    runtime = ctx.attrs.toolchain[RuntimeInfo]
    ctx.actions.run([runtime.executable, ctx.attrs.src])
```

**vx 可借鉴的点：**
- 将 `RuntimeInfo` 作为 Starlark Provider 类型，而非仅仅是 Rust struct
- 允许 Starlark Provider 声明自己提供的 `RuntimeInfo`，供其他 Provider 消费

#### Buck2 动态依赖（Dynamic Output）

Buck2 支持在运行时动态解析依赖，这对 vx 的版本解析很有参考价值：

```python
# Buck2 动态依赖模式
ctx.actions.dynamic_output(
    dynamic = [dep_file],
    inputs = [src],
    outputs = [out],
    f = lambda ctx, artifacts, outputs: resolve_deps(artifacts[dep_file])
)
```

**vx 对应场景**：MSVC 安装时需要先下载 manifest，再根据 manifest 动态决定下载哪些包。

#### Buck2 Typed Provider Fields（强类型 Provider）

Buck2 使用 `provider(fields = {...})` 定义强类型 Provider，而非无类型 dict：

```python
# Buck2 强类型 Provider 定义
RuntimeInfo = provider(
    doc = "Information about an installed runtime",
    fields = {
        "executable": provider_field(Artifact, doc = "Path to the executable"),
        "version":    provider_field(str,      doc = "Installed version string"),
        "env":        provider_field(dict[str, str], default = {}, doc = "Environment variables"),
    },
)

# 消费方通过类型安全的字段访问
def compile_impl(ctx):
    runtime = ctx.attrs.toolchain[RuntimeInfo]
    # runtime.executable, runtime.version 都有类型检查
    ctx.actions.run([runtime.executable, ctx.attrs.src])
```

**vx 借鉴**：将 `ProviderInfo` 从无类型 dict 升级为强类型 Starlark record，在分析阶段即可捕获字段错误：

```python
# vx provider.star 中的强类型 ProviderInfo（借鉴 Buck2 typed provider_field）
ProviderInfo = record(
    versions_url    = field(str),
    download_url_fn = field(typing.Callable),   # 函数引用
    env_template    = field(dict[str, str], default = {}),
    metadata        = field(dict[str, typing.Any], default = {}),
)

def analyze(ctx) -> ProviderInfo:
    return ProviderInfo(
        versions_url = "https://api.github.com/repos/...",
        download_url_fn = download_url,
        env_template = {"VCPKG_ROOT": "{install_dir}"},
    )
```

#### Buck2 `load()` 模块系统（跨 Provider 代码共享）

Buck2 通过 `load()` 语句实现跨文件代码共享，这是 Starlark 的标准模块机制（注意：`load()` 是 Starlark 的合法语句，与 Python 的 `import` 不同）：

```python
# Buck2 中的 load() 用法
load("@prelude//toolchains:cxx.bzl", "cxx_toolchain")
load("@prelude//utils:utils.bzl", "flatten", "dedupe")
```

**vx 借鉴**：提供 `@vx//stdlib` 标准库，允许 Provider 通过 `load()` 共享工具函数：

```python
# provider.star 中使用 vx 标准库（借鉴 Buck2 load() 模块系统）
load("@vx//stdlib:semver.star", "semver_compare", "semver_strip_v")
load("@vx//stdlib:platform.star", "platform_triple", "is_windows")
load("@vx//stdlib:http.star", "github_releases", "parse_github_tag")

def fetch_versions(ctx):
    releases = github_releases(ctx, "microsoft", "vcpkg-tool")
    return [
        {"version": semver_strip_v(r["tag_name"]), "lts": not r["prerelease"]}
        for r in releases
        if not r["draft"]
    ]
```

Rust 侧实现 `@vx//stdlib` 虚拟文件系统，将内置工具函数以 `.star` 文件形式暴露：

```rust
// crates/vx-starlark/src/loader.rs
pub struct VxModuleLoader {
    /// 内置模块映射：模块路径 -> Starlark 源码
    builtins: HashMap<String, &'static str>,
}

impl VxModuleLoader {
    pub fn new() -> Self {
        let mut builtins = HashMap::new();
        builtins.insert("@vx//stdlib:semver.star",   include_str!("../stdlib/semver.star"));
        builtins.insert("@vx//stdlib:platform.star", include_str!("../stdlib/platform.star"));
        builtins.insert("@vx//stdlib:http.star",     include_str!("../stdlib/http.star"));
        Self { builtins }
    }
}
```

#### Buck2 增量分析缓存（Incremental Analysis）

Buck2 的核心优化之一是**增量分析**：对未变更的目标复用上次分析结果，避免重复执行 Starlark。

**vx 借鉴**：对 `provider.star` 的分析结果（`ProviderInfo`）进行内容哈希缓存：

```rust
// crates/vx-starlark/src/provider.rs
/// 分析结果缓存条目
struct AnalysisCacheEntry {
    /// provider.star 文件内容的 SHA256 哈希
    script_hash: [u8; 32],
    /// 冻结的 ProviderInfo（分析阶段输出）
    frozen_info: FrozenProviderInfo,
    /// 缓存时间
    cached_at: std::time::SystemTime,
}

impl StarlarkProvider {
    /// 获取分析结果（带缓存）
    async fn get_analysis(&self, ctx: &ProviderContext) -> Result<FrozenProviderInfo> {
        let script_hash = sha256_file(&self.script_path)?;

        // 检查缓存
        if let Some(entry) = self.analysis_cache.get(&script_hash) {
            tracing::debug!(provider = %self.name, "Using cached analysis result");
            return Ok(entry.frozen_info.clone());
        }

        // 重新分析
        let info = self.run_analysis_phase(ctx).await?;
        self.analysis_cache.insert(script_hash, AnalysisCacheEntry {
            script_hash,
            frozen_info: info.clone(),
            cached_at: std::time::SystemTime::now(),
        });

        Ok(info)
    }
}
```

#### Buck2 `ctx.actions` 声明式动作模式

Buck2 的分析阶段只**声明**动作（`ctx.actions.run()`），不立即执行。执行引擎在分析完成后统一调度。

**vx 借鉴**：在 `install()` 函数中引入声明式动作 API，让 Starlark 脚本描述"做什么"而非"怎么做"：

```python
# provider.star 中的声明式安装动作（借鉴 Buck2 ctx.actions 模式）
def install(ctx, version) -> list:
    """
    返回安装动作列表（声明式），由 Rust 核心执行
    """
    install_dir = ctx.paths.install_dir("msvc", version)
    url = download_url(ctx, version)

    return [
        # 动作 1：下载归档
        ctx.actions.download(
            url = url,
            dest = ctx.paths.cache_dir("msvc-{}.zip".format(version)),
            checksum = None,  # 可选 SHA256
        ),
        # 动作 2：解压
        ctx.actions.extract(
            src = ctx.paths.cache_dir("msvc-{}.zip".format(version)),
            dest = install_dir,
            strip_prefix = "msvc-{}".format(version),
        ),
        # 动作 3：自定义脚本（可选）
        ctx.actions.run_hook(
            name = "post_install",
            args = [install_dir],
        ),
    ]
```

这种声明式模式的优势：
- Rust 核心可以**并行执行**无依赖的动作（如同时下载多个包）
- 动作列表可以被**序列化和缓存**，避免重复分析
- 与 Buck2 的执行模型保持一致，降低概念负担

### Bazel 方案对比

Bazel 与 Buck2 类似，但有以下差异：

| 特性 | Bazel | Buck2 | vx 选择 |
|------|-------|-------|---------|
| Starlark 实现 | Java | Rust (starlark-rust) | Rust ✓ |
| Provider 不可变性 | 强制 | 强制 | 强制 ✓ |
| 两阶段执行 | 有 | 有 | 简化版 ✓ |
| 工具链抽象 | 复杂 | 中等 | 轻量 ✓ |
| 沙箱模型 | 文件系统级 | 文件系统级 | API 级 ✓ |
| Typed Provider Fields | 弱 | 强（`provider_field`） | `record` 类型 ✓ |
| 模块系统 | `load()` | `load()` | `@vx//stdlib` ✓ |
| 增量分析缓存 | 有 | 有（内容哈希） | 内容哈希 ✓ |
| 声明式动作 | `ctx.actions` | `ctx.actions` | 简化版 ✓ |
| 扩展语言（BXL） | Starlark | BXL（Starlark 超集） | `vx provider debug` |

### Deno 插件方案

Deno 使用 JavaScript/TypeScript 作为插件语言，其沙箱模型值得参考：

- **权限声明式**：`--allow-read=/tmp --allow-net=api.github.com`
- **细粒度控制**：每个权限独立授予，而非全有全无
- **运行时检查**：权限在运行时动态检查，而非编译时

**vx 借鉴**：在 `provider.star` 头部声明所需权限：

```python
# 声明式权限（借鉴 Deno）
permissions = {
    "fs": ["~/.vx/store", "C:\\Program Files\\Microsoft Visual Studio"],
    "http": ["api.github.com", "aka.ms"],
    "exec": ["where", "powershell"],
}
```

### 为什么选择 Starlark

| 特性 | Starlark | Lua | JavaScript | Python |
|------|----------|-----|------------|--------|
| 学习曲线 | ⭐⭐⭐⭐ (类 Python) | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| 安全沙箱 | ⭐⭐⭐⭐⭐ 内置 | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ |
| 表达能力 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 嵌入难度 | ⭐⭐⭐⭐⭐ 简单 | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| 生态成熟度 | ⭐⭐⭐⭐⭐ Bazel/Buck2 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Rust 实现 | ⭐⭐⭐⭐⭐ starlark-rust | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

**选择 Starlark 的理由：**
1. **Buck2 同款** - Meta 的 Buck2 使用 starlark-rust，经过大规模生产验证
2. **Python 语法** - 对开发者友好，学习成本低
3. **内置沙箱** - 无 I/O、无全局状态、无副作用，天然安全
4. **Rust 原生支持** - `starlark-rust` crate 提供完整的 Rust 实现
5. **无 `import` 语句** - 语言层面杜绝了模块系统滥用

## 替代方案

### 方案 A：Lua 脚本

**优点**：轻量、嵌入简单、有 `mlua` Rust crate
**缺点**：语法与 Python 差异大，团队学习成本高；沙箱需要手动实现；生态不如 Starlark 成熟

**放弃原因**：Starlark 的 Python 语法更符合目标用户习惯，且 Buck2 的生产验证更有说服力。

### 方案 B：JavaScript/TypeScript (Deno)

**优点**：生态最丰富、TypeScript 类型安全、开发者熟悉
**缺点**：运行时体积大（Deno ~100MB）；沙箱模型复杂；与 vx 的 Rust 集成成本高

**放弃原因**：引入 JS 运行时会显著增加 vx 的二进制体积，与"零依赖"目标冲突。

### 方案 C：WASM 插件

**优点**：语言无关、强沙箱、可移植
**缺点**：开发复杂度极高；调试困难；Provider 开发者需要了解 WASM

**放弃原因**：Provider 开发者门槛过高，不符合"易于扩展"的设计目标。

### 方案 D：扩展 TOML（模板语言）

**优点**：无需引入新语言；向后兼容性最好
**缺点**：模板语言（如 Tera）表达能力有限；复杂逻辑仍然难以表达；调试困难

**放弃原因**：TOML + 模板语言的组合会产生一种"四不像"的 DSL，不如直接使用成熟的脚本语言。

## 设计

### 1. 混合格式架构

#### 1.1 文件选择优先级

```
~/.vx/providers/myprovider/
├── provider.star    # 优先级 1: Starlark 脚本
├── provider.toml    # 优先级 2: TOML 配置
└── README.md
```

**加载逻辑：**

```rust
impl ProviderLoader {
    fn load_provider(path: &Path) -> Result<Box<dyn Provider>> {
        let star_path = path.join("provider.star");
        let toml_path = path.join("provider.toml");

        if star_path.exists() {
            // 优先使用 Starlark
            StarlarkProvider::load(&star_path)
        } else if toml_path.exists() {
            // 回退到 TOML
            self.load_toml_provider(&toml_path)
        } else {
            Err(anyhow!("No provider.star or provider.toml found"))
        }
    }
}
```

#### 1.2 格式对比

| 特性 | provider.toml | provider.star |
|------|--------------|---------------|
| 声明能力 | 静态配置 | 完全可编程 |
| 学习成本 | 极低 | 中等 (类 Python) |
| 适用场景 | 简单 Provider | 复杂 Provider |
| 安全性 | 无风险 | 需要沙箱 |
| 调试支持 | 无需调试 | 需要 debug 工具 |

### 2. Buck2 借鉴：两阶段执行模型

受 Buck2 启发，vx 的 Starlark Provider 采用**两阶段执行**：

```
┌─────────────────────────────────────────────────────────────────┐
│                    两阶段执行模型                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Phase 1: Analysis（分析阶段）                                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Starlark 脚本执行                                       │   │
│  │  • 调用 fetch_versions() → 返回版本列表                  │   │
│  │  │  调用 download_url() → 返回 URL 字符串               │   │
│  │  • 调用 prepare_environment() → 返回环境变量字典         │   │
│  │  • 所有返回值被"冻结"（Frozen），不可变                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                          │                                      │
│                          ▼ Frozen ProviderInfo                  │
│  Phase 2: Execution（执行阶段）                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Rust 核心执行                                           │   │
│  │  • 使用冻结的 URL 下载文件                               │   │
│  │  • 使用冻结的环境变量配置执行环境                        │   │
│  │  • 调用 install() 钩子（可选，复杂安装逻辑）             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**好处：**
- 分析阶段可以并行执行（Starlark 无副作用）
- 执行阶段由 Rust 控制，保证安全性
- 与 TOML Provider 的执行路径统一

### 3. Buck2 借鉴：ProviderInfo 数据结构

受 Buck2 的 `DefaultInfo`/`RunInfo` 启发，vx 引入 `ProviderInfo` 作为 Starlark Provider 的标准输出格式：

```python
# provider.star 中返回 ProviderInfo（借鉴 Buck2 的 Provider 列表模式）

def analyze(ctx) -> dict:
    """
    分析阶段：返回 ProviderInfo（不可变）

    这是 Buck2 风格的两阶段设计：
    - analyze() 在分析阶段调用，返回值被冻结
    - install() 在执行阶段调用，可以有副作用

    Returns:
        ProviderInfo 字典，包含：
        - versions_url: 版本列表 API URL
        - download_template: 下载 URL 模板
        - env_template: 环境变量模板
        - metadata: 额外元数据
    """
    arch = ctx.platform.arch

    return {
        "versions_url": "https://api.github.com/repos/microsoft/vcpkg-tool/releases",
        "download_template": f"https://github.com/microsoft/vcpkg-tool/releases/download/v{{version}}/vcpkg-{arch}.zip",
        "env_template": {
            "VCPKG_ROOT": "{install_dir}",
        },
        "metadata": {
            "ecosystem": "system",
            "aliases": ["cl", "nmake"],
        },
    }
```

### 4. Starlark Provider API

#### 4.1 核心 API 设计

```python
# provider.star - Starlark Provider API

# ============== 元数据 ==============

def name() -> str:
    """Provider 名称"""
    return "msvc"

def description() -> str:
    """Provider 描述"""
    return "MSVC Build Tools - Microsoft Visual C++ compiler"

def version() -> str:
    """Provider API 版本"""
    return "1.0"

def ecosystem() -> str:
    """生态系统: nodejs, python, rust, go, system, custom"""
    return "system"

def aliases() -> list:
    """Runtime 别名"""
    return ["cl", "nmake"]

def supported_platforms() -> list:
    """支持的平台列表"""
    return [
        {"os": "windows", "arch": "x64"},
        {"os": "windows", "arch": "arm64"},
    ]

# ============== 版本管理 ==============

def fetch_versions(ctx) -> list:
    """
    获取可用版本列表

    Args:
        ctx: 执行上下文，包含平台信息、HTTP 客户端等

    Returns:
        版本信息列表，每个版本是一个字典：
        {"version": "14.42", "lts": True, "prerelease": False}
    """
    releases = ctx.http.get_json(
        "https://api.github.com/repos/microsoft/vcpkg-tool/releases"
    )

    versions = []
    for release in releases:
        if not release.get("draft"):
            tag = release["tag_name"]
            # 使用字符串操作而非正则（Starlark 无 re 模块）
            v = tag[1:] if tag.startswith("v") else tag
            versions.append({
                "version": v,
                "lts": not release.get("prerelease"),
                "prerelease": release.get("prerelease", False),
            })

    return versions

# ============== 下载 URL ==============

def download_url(ctx, version) -> str:
    """
    构建下载 URL

    Args:
        ctx: 执行上下文
        version: 目标版本

    Returns:
        下载 URL，如果平台不支持则返回 None
    """
    if ctx.platform.os != "windows":
        return None

    arch = ctx.platform.arch  # "x64" or "arm64"
    return "https://github.com/microsoft/vcpkg-tool/releases/download/v{}/vcpkg-{}.zip".format(version, arch)

# ============== 安装流程 ==============

def install(ctx, version) -> dict:
    """
    安装指定版本（执行阶段钩子）

    Args:
        ctx: 执行上下文
        version: 目标版本

    Returns:
        安装结果：
        {"success": True, "path": "/path/to/executable"}
        或
        {"success": False, "error": "错误信息"}
    """
    install_path = ctx.paths.install_dir("msvc", version)

    if ctx.fs.exists(ctx.fs.join(install_path, "cl.exe")):
        return {"success": True, "path": install_path, "already_installed": True}

    ctx.fs.mkdir(install_path)

    ctx.progress("Downloading MSVC packages...")
    result = _install_with_msvc_kit(ctx, version, install_path)

    if not result.get("success"):
        return result

    ctx.progress("Deploying MSBuild bridge...")
    _deploy_msbuild_bridge(ctx, install_path)

    return {"success": True, "path": install_path}

# ============== 系统检测 ==============

def detect_system_installation(ctx) -> list:
    """
    检测系统已安装的版本

    Returns:
        检测结果列表，按优先级排序
    """
    results = []

    if ctx.platform.os != "windows":
        return results

    # 方式 1: 检查 Visual Studio 安装
    vs_editions = ["Community", "Professional", "Enterprise"]
    vs_root = "C:\\Program Files\\Microsoft Visual Studio\\2022"

    for edition in vs_editions:
        vs_path = ctx.fs.join(vs_root, edition)
        if ctx.fs.exists(vs_path):
            cl_exes = ctx.fs.glob(ctx.fs.join(vs_path, "VC", "Tools", "MSVC", "*", "bin", "Host*", "cl.exe"))
            if cl_exes:
                version = _extract_version_from_path(cl_exes[0])
                results.append({
                    "type": "visual_studio_2022",
                    "path": cl_exes[0],
                    "version": version,
                    "edition": edition,
                    "priority": 100,
                })

    # 方式 2: 使用 where 命令
    where_result = ctx.execute("where", ["cl.exe"])
    if where_result["success"]:
        existing_paths = [r["path"] for r in results]
        for path in where_result["stdout"].strip().split("\n"):
            path = path.strip()
            if path and ctx.fs.exists(path) and path not in existing_paths:
                version = _detect_cl_version(ctx, path)
                results.append({
                    "type": "path",
                    "path": path,
                    "version": version,
                    "priority": 90,
                })

    # 方式 3: 检查环境变量
    vc_dir = ctx.env.get("VCINSTALLDIR", "")
    if vc_dir:
        cl_exes = ctx.fs.glob(ctx.fs.join(vc_dir, "Tools", "MSVC", "*", "bin", "Host*", "cl.exe"))
        if cl_exes:
            existing_paths = [r["path"] for r in results]
            if cl_exes[0] not in existing_paths:
                results.append({
                    "type": "env",
                    "path": cl_exes[0],
                    "version": _extract_version_from_path(cl_exes[0]),
                    "priority": 80,
                })

    return sorted(results, key=lambda x: x["priority"], reverse=True)

# ============== 环境变量 ==============

def prepare_environment(ctx, version) -> dict:
    """
    准备执行环境变量

    Args:
        ctx: 执行上下文
        version: 目标版本

    Returns:
        环境变量字典
    """
    env = {}
    install_path = ctx.paths.install_dir("msvc", version)

    tools_dirs = ctx.fs.glob(ctx.fs.join(install_path, "VC", "Tools", "MSVC", "*"))
    if not tools_dirs:
        return env

    msvc_version = ctx.fs.basename(tools_dirs[0])
    arch = ctx.platform.arch

    include_paths = _build_include_paths(ctx, install_path, msvc_version, arch)
    if include_paths:
        env["INCLUDE"] = ";".join(include_paths)

    lib_paths = _build_lib_paths(ctx, install_path, msvc_version, arch)
    if lib_paths:
        env["LIB"] = ";".join(lib_paths)

    vc_dir = ctx.fs.join(install_path, "VC")
    if ctx.fs.exists(vc_dir):
        env["VCINSTALLDIR"] = vc_dir + "\\"
        env["VCToolsInstallDir"] = ctx.fs.join(vc_dir, "Tools", "MSVC", msvc_version) + "\\"
        env["VSCMD_VER"] = "17.0"
        env["GYP_MSVS_VERSION"] = "2022"

    sdk_version = _detect_windows_sdk_version(ctx)
    if sdk_version:
        env["WindowsSDKVersion"] = sdk_version + "\\"

    return env

# ============== 验证 ==============

def verify_installation(ctx, version) -> dict:
    """
    验证安装

    Returns:
        {"valid": True, "executable": "/path/to/cl.exe"}
        或
        {"valid": False, "errors": ["..."], "suggestions": ["..."]}
    """
    install_path = ctx.paths.install_dir("msvc", version)
    arch = ctx.platform.arch

    # 搜索 cl.exe
    cl_exes = ctx.fs.glob(ctx.fs.join(install_path, "**", "cl.exe"))
    if cl_exes:
        return {"valid": True, "executable": cl_exes[0]}

    return {
        "valid": False,
        "errors": ["MSVC compiler (cl.exe) not found in {}".format(install_path)],
        "suggestions": [
            "Try reinstalling: vx install msvc",
            "Ensure the installation completed successfully",
        ]
    }

# ============== 组件管理 ==============

def check_missing_components(ctx, version, components) -> list:
    """
    检查缺失的 MSVC 组件

    Args:
        ctx: 执行上下文
        version: MSVC 版本
        components: 请求的组件列表 (如 ["spectre", "mfc", "atl"])

    Returns:
        缺失的组件列表
    """
    install_path = ctx.paths.install_dir("msvc", version)
    arch = ctx.platform.arch
    missing = []

    tools_dirs = ctx.fs.glob(ctx.fs.join(install_path, "VC", "Tools", "MSVC", "*"))
    if not tools_dirs:
        return list(components)

    msvc_dir = tools_dirs[0]

    for component in components:
        if component == "spectre":
            spectre_dir = ctx.fs.join(msvc_dir, "lib", arch, "spectre")
            if not ctx.fs.exists(spectre_dir) or not ctx.fs.list_dir(spectre_dir):
                missing.append("spectre")

        elif component in ["mfc", "atl"]:
            atlmfc_dir = ctx.fs.join(msvc_dir, "atlmfc", "include")
            if not ctx.fs.exists(atlmfc_dir):
                missing.append(component)

        elif component == "asan":
            lib_dir = ctx.fs.join(msvc_dir, "lib", arch)
            asan_libs = ctx.fs.glob(ctx.fs.join(lib_dir, "clang_rt.asan*.lib"))
            if not asan_libs:
                missing.append("asan")

    return missing

# ============== 内部辅助函数 ==============

def _extract_version_from_path(path) -> str:
    """从路径提取版本号，返回 str"""
    # path like: .../VC/Tools/MSVC/14.42.34433/bin/...
    parts = path.replace("/", "\\").split("\\")
    for i, part in enumerate(parts):
        if part == "MSVC" and i + 1 < len(parts):
            version_parts = parts[i + 1].split(".")
            # 返回 "14.42" 格式的字符串
            if len(version_parts) >= 2:
                return version_parts[0] + "." + version_parts[1]
    return "unknown"


def _detect_cl_version(ctx, cl_path) -> str:
    """通过执行 cl.exe 检测版本（使用字符串操作，不依赖正则）"""
    result = ctx.execute(cl_path, [])
    if result["success"] or result.get("stderr"):
        # cl.exe 输出格式: "Microsoft (R) C/C++ Optimizing Compiler Version 19.42.34433"
        stderr = result.get("stderr", "")
        for line in stderr.split("\n"):
            if "Version" in line:
                # 找到 "Version " 后的数字
                idx = line.find("Version ")
                if idx >= 0:
                    rest = line[idx + 8:].strip()
                    # 取第一个空格前的内容作为版本号
                    parts = rest.split(" ")
                    if parts:
                        return parts[0]
    return "unknown"


def _detect_windows_sdk_version(ctx) -> str:
    """检测 Windows SDK 版本"""
    sdk_roots = [
        "C:\\Program Files (x86)\\Windows Kits\\10\\Include",
        "C:\\Program Files\\Windows Kits\\10\\Include",
    ]

    for sdk_root in sdk_roots:
        if ctx.fs.exists(sdk_root):
            versions = ctx.fs.list_dir(sdk_root)
            sdk_versions = [v for v in versions if v.startswith("10.0.")]
            if sdk_versions:
                return sorted(sdk_versions)[-1]

    return None


def _build_include_paths(ctx, install_path, msvc_version, arch) -> list:
    """构建 INCLUDE 路径"""
    paths = []

    msvc_inc = ctx.fs.join(install_path, "VC", "Tools", "MSVC", msvc_version, "include")
    if ctx.fs.exists(msvc_inc):
        paths.append(msvc_inc)

    sdk_version = _detect_windows_sdk_version(ctx)
    if sdk_version:
        for sdk_root in ["C:\\Program Files (x86)\\Windows Kits\\10", "C:\\Program Files\\Windows Kits\\10"]:
            inc_base = ctx.fs.join(sdk_root, "Include", sdk_version)
            for subdir in ["ucrt", "shared", "um", "winrt"]:
                path = ctx.fs.join(inc_base, subdir)
                if ctx.fs.exists(path):
                    paths.append(path)

    return paths


def _build_lib_paths(ctx, install_path, msvc_version, arch) -> list:
    """构建 LIB 路径"""
    paths = []

    msvc_lib = ctx.fs.join(install_path, "VC", "Tools", "MSVC", msvc_version, "lib", arch)
    if ctx.fs.exists(msvc_lib):
        paths.append(msvc_lib)

    sdk_version = _detect_windows_sdk_version(ctx)
    if sdk_version:
        for sdk_root in ["C:\\Program Files (x86)\\Windows Kits\\10", "C:\\Program Files\\Windows Kits\\10"]:
            lib_base = ctx.fs.join(sdk_root, "Lib", sdk_version)
            for subdir in ["ucrt", "um"]:
                path = ctx.fs.join(lib_base, subdir, arch)
                if ctx.fs.exists(path):
                    paths.append(path)

    return paths


def _deploy_msbuild_bridge(ctx, install_path) -> None:
    """部署 MSBuild bridge（通过 ctx 调用 Rust 实现）"""
    ctx.deploy_msbuild_bridge(install_path)


def _install_with_msvc_kit(ctx, version, install_path) -> dict:
    """使用 msvc-kit 安装（通过 ctx 调用 Rust 实现）"""
    components_str = ctx.env.get("VX_MSVC_COMPONENTS", "")
    components = [c.strip() for c in components_str.split(",") if c.strip()] if components_str else []
    return ctx.install_msvc_kit(version, install_path, components)
```

#### 4.2 ProviderContext API（Rust 侧）

```rust
// crates/vx-starlark/src/context.rs

/// Provider 执行上下文（注入到 Starlark 脚本）
/// 命名为 ProviderContext 以区分 vx-runtime 中的 RuntimeContext
pub struct ProviderContext {
    /// 平台信息
    pub platform: PlatformInfo,

    /// 环境变量（只读）
    pub env: HashMap<String, String>,

    /// 路径管理器
    pub paths: Arc<dyn PathProvider>,

    /// 沙箱文件系统
    pub fs: Arc<SandboxFileSystem>,

    /// 沙箱 HTTP 客户端
    pub http: Arc<SandboxHttpClient>,

    /// 命令执行器（受沙箱限制）
    pub executor: Arc<SandboxCommandExecutor>,

    /// 进度报告回调
    pub progress_reporter: Arc<dyn ProgressReporter>,
}

/// 平台信息（暴露给 Starlark）
#[derive(Clone)]
pub struct PlatformInfo {
    pub os: String,    // "windows", "macos", "linux"
    pub arch: String,  // "x64", "arm64", "x86"
}
```

**注意**：Starlark 脚本中通过 `ctx.fs.join()`、`ctx.fs.exists()` 等**扁平方法**访问，而非嵌套对象，这与 Buck2 的扁平化 `ctx` 设计一致，有利于 IDE 自动补全。

### 5. 沙箱安全模型

#### 5.1 Starlark 内置安全特性

Starlark 语言本身的设计就考虑了安全性：

```python
# ❌ Starlark 不支持的操作（语言层面禁止）
import os          # SyntaxError: import not allowed
open("/etc/passwd")  # NameError: open not defined
eval("code")       # SyntaxError: eval not allowed
exec("code")       # SyntaxError: exec not allowed

# ❌ 无副作用（数据结构默认不可变）
x = [1, 2, 3]
x.append(4)  # Error: cannot mutate frozen list
```

**内置限制：**
- 无 `import` 语句（这是 Starlark 的核心设计，与 Python 的最大区别）
- 无文件 I/O（除非通过注入的 `ctx.fs` API）
- 无网络访问（除非通过注入的 `ctx.http` API）
- 无全局可变状态
- 无无限循环（可配置超时）

#### 5.2 声明式权限（借鉴 Deno）

受 Deno 权限模型启发，`provider.star` 在头部声明所需权限：

```python
# provider.star 头部声明权限（借鉴 Deno 的显式权限模型）
permissions = {
    # 文件系统访问白名单（仅允许访问这些路径前缀）
    "fs": [
        "~/.vx/store",
        "C:\\Program Files\\Microsoft Visual Studio",
        "C:\\Program Files (x86)\\Windows Kits",
    ],
    # HTTP 访问白名单
    "http": [
        "api.github.com",
        "aka.ms",
    ],
    # 允许执行的命令白名单
    "exec": [
        "where",
        "powershell",
    ],
}
```

Rust 侧在加载 `provider.star` 时读取 `permissions` 变量，并据此构建 `SandboxConfig`：

```rust
// 从 provider.star 的 permissions 变量构建沙箱配置
let sandbox = SandboxConfig::from_permissions(&permissions_value)?;
```

#### 5.3 SandboxConfig

```rust
// crates/vx-starlark/src/sandbox.rs

/// Starlark 沙箱配置
pub struct SandboxConfig {
    /// 文件系统访问白名单
    pub fs_allowed_paths: Vec<PathBuf>,

    /// HTTP 请求域名白名单
    pub http_allowed_hosts: Vec<String>,

    /// 执行超时时间
    pub execution_timeout: Duration,

    /// 内存限制
    pub memory_limit: usize,

    /// 允许执行的命令白名单（空表示禁止所有命令执行）
    pub allowed_commands: Vec<String>,
}

impl SandboxConfig {
    /// 最严格的沙箱配置（默认）
    pub fn restrictive() -> Self {
        Self {
            fs_allowed_paths: vec![],
            http_allowed_hosts: vec![
                "api.github.com".to_string(),
                "github.com".to_string(),
                "nodejs.org".to_string(),
                "go.dev".to_string(),
                "pypi.org".to_string(),
                "static.rust-lang.org".to_string(),
            ],
            execution_timeout: Duration::from_secs(60),
            memory_limit: 64 * 1024 * 1024, // 64MB
            allowed_commands: vec![],
        }
    }

    /// 从 provider.star 的 permissions 声明构建
    pub fn from_permissions(permissions: &PermissionsDecl) -> Result<Self> {
        let mut config = Self::restrictive();

        // 解析文件系统权限
        for path_str in &permissions.fs {
            let path = expand_home(path_str)?;
            config.fs_allowed_paths.push(path);
        }

        // 解析 HTTP 权限
        config.http_allowed_hosts.extend(permissions.http.clone());

        // 解析命令执行权限
        config.allowed_commands.extend(permissions.exec.clone());

        Ok(config)
    }
}
```

#### 5.4 文件系统沙箱

```rust
// crates/vx-starlark/src/sandbox.rs（续）

/// 沙箱文件系统
pub struct SandboxFileSystem {
    /// 允许访问的路径前缀
    allowed_prefixes: Vec<PathBuf>,
}

impl SandboxFileSystem {
    /// 检查路径是否在白名单内
    fn check_path(&self, path: &Path) -> Result<()> {
        // 始终允许访问 vx 自己的目录
        let vx_home = dirs::home_dir()
            .map(|h| h.join(".vx"))
            .unwrap_or_default();

        if path.starts_with(&vx_home) {
            return Ok(());
        }

        for prefix in &self.allowed_prefixes {
            if path.starts_with(prefix) {
                return Ok(());
            }
        }

        Err(anyhow!(
            "Sandbox violation: access to '{}' is not permitted. \
             Declare required paths in the 'permissions.fs' field of provider.star",
            path.display()
        ))
    }
}
```

### 6. Rust 实现

#### 6.1 Cargo.toml

```toml
# crates/vx-starlark/Cargo.toml

[package]
name = "vx-starlark"
version.workspace = true
edition.workspace = true
description = "Starlark scripting support for vx providers"

[dependencies]
# Starlark runtime (starlark-rust by Meta/Facebook)
starlark = { version = "0.13" }
starlark_derive = { version = "0.13" }

# vx crates
vx-core = { path = "../vx-core" }
vx-paths = { path = "../vx-paths" }

# Async
tokio = { workspace = true }
async-trait = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }

# Utilities
anyhow = { workspace = true }
tracing = { workspace = true }
once_cell = { workspace = true }
```

#### 6.2 模块结构

```rust
// crates/vx-starlark/src/lib.rs

//! Starlark scripting support for vx providers.
//!
//! This crate enables writing vx providers in Starlark (a Python dialect
//! used by Bazel and Buck2), providing a safe sandboxed execution environment.
//!
//! # Architecture
//!
//! Inspired by Buck2's two-phase execution model:
//! - **Analysis phase**: Starlark scripts run to produce frozen ProviderInfo
//! - **Execution phase**: Rust core uses frozen ProviderInfo to perform I/O
//!
//! # Example
//!
//! ```rust
//! use vx_starlark::StarlarkProvider;
//!
//! let provider = StarlarkProvider::load(Path::new("provider.star")).await?;
//! let versions = provider.fetch_versions(&ctx).await?;
//! ```

pub mod context;
pub mod error;
pub mod provider;
pub mod sandbox;
pub mod stdlib;

pub use provider::StarlarkProvider;
pub use context::ProviderContext;
pub use sandbox::SandboxConfig;
pub use error::StarlarkError;
```

#### 6.3 StarlarkProvider

```rust
// crates/vx-starlark/src/provider.rs

use std::path::Path;
use anyhow::Result;
use async_trait::async_trait;

/// Starlark Provider 实现
///
/// 通过加载 provider.star 文件创建，实现 vx-core 的 Provider trait。
/// 采用 Buck2 风格的两阶段执行：
/// 1. 分析阶段：调用 Starlark 函数获取元数据（无副作用）
/// 2. 执行阶段：Rust 核心执行实际 I/O 操作
pub struct StarlarkProvider {
    /// Provider 名称（从 name() 函数获取）
    name: String,

    /// Provider 描述
    description: String,

    /// 沙箱配置（从 permissions 变量构建）
    sandbox: SandboxConfig,

    /// provider.star 文件路径（用于重新加载）
    source_path: PathBuf,
}

impl StarlarkProvider {
    /// 异步加载 provider.star 文件
    pub async fn load(path: &Path) -> Result<Self> {
        let source = tokio::fs::read_to_string(path).await?;

        // 解析元数据（不需要完整执行）
        let metadata = parse_metadata(&source)?;

        // 从 permissions 变量构建沙箱配置
        let sandbox = if let Some(perms) = metadata.permissions {
            SandboxConfig::from_permissions(&perms)?
        } else {
            SandboxConfig::restrictive()
        };

        Ok(Self {
            name: metadata.name,
            description: metadata.description,
            sandbox,
            source_path: path.to_path_buf(),
        })
    }

    /// 在沙箱中执行 Starlark 函数
    fn eval_function(&self, func_name: &str, ctx: &ProviderContext) -> Result<serde_json::Value> {
        // TODO: Phase 2 实现完整的 Starlark 执行引擎
        // 当前为 placeholder，返回空结果
        tracing::warn!(
            provider = %self.name,
            func = %func_name,
            "Starlark execution not yet implemented (Phase 2)"
        );
        Ok(serde_json::Value::Null)
    }
}
```

#### 6.4 stdlib（标准库注入）

```rust
// crates/vx-starlark/src/stdlib.rs

/// 注册 vx 标准库到 Starlark 环境
///
/// 提供以下内置函数（无需 import）：
/// - semver_compare(a, b) -> int  版本比较
/// - str_contains(s, sub) -> bool  字符串包含检查
/// - str_split_first(s, sep) -> list  分割并取第一个
/// - path_join(*parts) -> str  路径拼接（跨平台）
pub fn register_stdlib(env: &mut GlobalsBuilder) {
    // 版本比较（避免在 Starlark 中手写版本解析逻辑）
    env.set("semver_compare", semver_compare_fn);

    // 字符串工具（补充 Starlark 内置字符串方法）
    env.set("str_contains", str_contains_fn);

    // 路径工具（跨平台路径处理）
    env.set("path_join", path_join_fn);
    env.set("path_basename", path_basename_fn);
    env.set("path_dirname", path_dirname_fn);
}
```

### 7. 测试策略

#### 7.1 单元测试（放在 `tests/` 目录）

```
crates/vx-starlark/tests/
├── sandbox_tests.rs      # 沙箱安全测试
├── provider_tests.rs     # Provider 加载测试
├── context_tests.rs      # ProviderContext 测试
└── stdlib_tests.rs       # 标准库函数测试
```

```rust
// crates/vx-starlark/tests/sandbox_tests.rs

#[test]
fn test_sandbox_blocks_unauthorized_path() {
    let sandbox = SandboxFileSystem::new(vec![
        PathBuf::from("/tmp/vx-test"),
    ]);

    // 允许访问白名单路径
    assert!(sandbox.exists("/tmp/vx-test/file.txt").is_ok());

    // 拒绝访问非白名单路径
    let result = sandbox.exists("/etc/passwd");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Sandbox violation"));
}

#[test]
fn test_sandbox_always_allows_vx_home() {
    let sandbox = SandboxFileSystem::new(vec![]);
    let vx_home = dirs::home_dir().unwrap().join(".vx/store/node/20.0.0");

    // vx 自己的目录始终允许
    assert!(sandbox.check_path(&vx_home).is_ok());
}

#[tokio::test]
async fn test_provider_load_metadata() {
    let temp = tempfile::TempDir::new().unwrap();
    let star_path = temp.path().join("provider.star");

    std::fs::write(&star_path, r#"
def name():
    return "test-provider"

def description():
    return "A test provider"
"#).unwrap();

    let provider = StarlarkProvider::load(&star_path).await.unwrap();
    assert_eq!(provider.name(), "test-provider");
}
```

#### 7.2 Starlark 脚本测试

```python
# crates/vx-starlark/tests/fixtures/test_provider.star
# 用于测试的最小 Provider

def name():
    return "test"

def description():
    return "Test provider"

def fetch_versions(ctx):
    return [
        {"version": "1.0.0", "lts": True},
        {"version": "0.9.0", "lts": False},
    ]

def download_url(ctx, version):
    return "https://example.com/test-{}.tar.gz".format(version)
```

## 实现计划

### Phase 1: 基础设施（✅ 已完成）

- [x] 创建 `vx-starlark` crate
- [x] 集成 `starlark-rust` 依赖
- [x] 实现基础沙箱配置（`SandboxConfig::restrictive()`）
- [x] 实现 `ProviderContext` 结构
- [x] 实现 `StarlarkProvider::load()` 元数据解析
- [x] 实现 `SandboxConfig::from_permissions()` 权限解析
- [x] 实现 `SandboxConfig::is_path_allowed()` / `is_host_allowed()` / `is_command_allowed()`
- [x] 实现 `ProviderContext` 文件系统 API（`file_exists`, `create_dir`, `read_file` 等）
- [x] 实现 `ProviderFormat::detect()` 混合格式检测
- [x] 编写 `tests/sandbox_tests.rs`
- [x] 编写 `tests/stdlib_tests.rs`
- [x] 实现 `permissions` 变量从 Starlark 脚本中解析（`SandboxConfig::from_permissions()`）
- [x] 实现 `VxModuleLoader`（`@vx//stdlib` 虚拟文件系统，借鉴 Buck2 `load()` 模块系统）

### Phase 2: Starlark 执行引擎（✅ 已完成）

- [x] 集成 `starlark-rust` 完整执行引擎（`AstModule` + `Evaluator`）
- [x] 实现 `ProviderContext` 到 Starlark `Value` 的转换（JSON bridge via `context_to_json`）
- [x] 实现 `eval_function()` 完整逻辑（`StarlarkEngine::call_function()`）
- [x] 注册 `stdlib` 标准库函数到 Starlark `GlobalsBuilder`
- [x] 实现两阶段执行（Analysis → Execution，`StarlarkEngine` + `StarlarkProvider`）
- [x] 实现 `FrozenProviderInfo` 不可变分析结果（借鉴 Buck2 Frozen Values）
- [x] 实现增量分析缓存（内容哈希 `sha256_bytes`，借鉴 Buck2 增量分析）
- [x] 实现 `@vx//stdlib` 模块加载器（`VxModuleLoader`，`loader.rs`）
- [x] 编写 `tests/provider_tests.rs`

### Phase 3: Provider 迁移（✅ 已完成）

- [x] 创建 `@vx//stdlib:github.star` — GitHub provider 通用基类（`make_fetch_versions`、`make_download_url`、`make_github_provider`）
- [x] 创建 `@vx//stdlib:platform.star` — 平台检测工具函数
- [x] 创建 `@vx//stdlib:http.star` — HTTP 工具函数（`github_releases`、`releases_to_versions`）
- [x] 创建 `@vx//stdlib:semver.star` — 语义版本工具函数
- [x] **engine.rs 实现 `load()` 支持** — `VxFileLoader` 实现 `FileLoader` trait，支持 `@vx//stdlib` 模块递归加载
- [x] **loader.rs 注册 `github.star`** — 所有 4 个 stdlib 模块均已注册
- [x] **jj provider 迁移** — `crates/vx-providers/jj/provider.star`（首个 Starlark provider 示例）
- [x] **批量迁移 20 个 GitHub provider** — 全部完成，覆盖三种继承模式：

  | Provider | 模式 | 特点 |
  |----------|------|------|
  | `fzf` | Level 2 | Go 风格平台后缀 |
  | `ripgrep` | Level 2 | Rust triple，Linux musl |
  | `fd` | Level 2 | Rust triple，asset 带 v 前缀 |
  | `bat` | Level 2 | sharkdp 出品，v-prefix asset |
  | `yq` | Level 3 | 直接二进制，无归档 |
  | `starship` | Level 2 | asset 名不含版本号 |
  | `just` | Level 2 | tag 无 v 前缀 |
  | `deno` | Level 2 | 全平台 zip |
  | `zig` | Level 3 | ziglang.org 自定义域名 |
  | `hadolint` | Level 3 | 直接二进制，os-arch 命名 |
  | `kubectl` | Level 3 | dl.k8s.io 自定义域名 |
  | `helm` | Level 2 | get.helm.sh，tar.gz |
  | `terraform` | Level 3 | releases.hashicorp.com |
  | `dagu` | Level 2 | Rust triple，tar.gz |
  | `ollama` | Level 3 | 直接二进制 |
  | `task` | Level 2 | Go 风格平台 |
  | `ninja` | Level 2 | zip，平台简称 |
  | `protoc` | Level 2 | zip，os-arch |
  | `gh` | Level 2 | Linux tar.gz / Windows zip |
  | `rcedit` | Level 3 | Windows-only，直接二进制 |

- [ ] 迁移 MSVC provider 到 Starlark（最复杂，1077 行 → 预计 ~200 行 Starlark）
- [ ] 迁移 vcpkg provider 到 Starlark（git clone 多步骤安装）
- [ ] 添加混合格式支持（`provider.star` 优先于 `provider.toml`，`ProviderFormat::detect()` 已实现）
- [ ] 实现声明式动作 API（`ctx.actions.download`、`ctx.actions.extract`，借鉴 Buck2 `ctx.actions`）
- [ ] 添加调试工具（`vx provider debug <name>`，借鉴 Buck2 BXL 查询能力）
- [ ] 编写集成测试

### Phase 4: 生态完善（Week 7-8）

- [ ] 迁移 MSVC provider（最复杂，需要 Windows SDK 检测）
- [ ] 迁移 vcpkg provider（git clone 多步骤安装）
- [ ] 更新用户文档
- [ ] 发布 v0.14.0

## 继承复用模式（`load()` 工厂函数）

Starlark 的 `load()` + 函数作为一等公民，天然支持"继承基类、只重写需要定制的部分"的模式。
这是 vx Starlark provider 的核心设计理念，比 Rust trait 更轻量，比 TOML 模板更强大。

### 三层复用粒度

```
┌─────────────────────────────────────────────────────────────────┐
│              Starlark Provider 继承复用层次                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Level 3: 完全复用（零自定义代码）                               │
│  ─────────────────────────────────────────────────────────────  │
│  load("@vx//stdlib:github.star", "make_github_provider")        │
│  _p = make_github_provider("owner", "repo",                     │
│           "{name}-{vversion}-{triple}.{ext}")                   │
│  fetch_versions = _p.fetch_versions                             │
│  download_url   = _p.download_url                               │
│                                                                 │
│  Level 2: 部分重写（只重写 download_url）← jj 示例              │
│  ─────────────────────────────────────────────────────────────  │
│  fetch_versions = make_fetch_versions("jj-vcs", "jj")  # 继承  │
│  def download_url(ctx, version):                        # 重写  │
│      triple = _jj_triple(ctx)   # musl instead of gnu          │
│      ...                                                        │
│                                                                 │
│  Level 1: 完全自定义（复杂 Provider，如 MSVC）                   │
│  ─────────────────────────────────────────────────────────────  │
│  def fetch_versions(ctx): ...   # 完全自定义                    │
│  def download_url(ctx, version): ...                            │
│  def install(ctx, version): ...                                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### `@vx//stdlib:github.star` 工厂函数

`github.star` 提供三个工厂函数，实现不同粒度的复用：

```python
# 工厂 1：只复用 fetch_versions
fetch_versions = make_fetch_versions("jj-vcs", "jj")
# → 等价于 Rust: ctx.fetch_github_releases("jj", "jj-vcs", "jj", ...)

# 工厂 2：只复用 download_url（标准 Rust triple 命名）
download_url = make_download_url(
    "cli", "cli",
    "gh_{version}_{os}_{arch}.{ext}"   # GitHub CLI 命名格式
)

# 工厂 3：完整 provider（fetch_versions + download_url 一起）
_p = make_github_provider(
    "BurntSushi", "ripgrep",
    "ripgrep-{version}-{triple}.{ext}"
)
fetch_versions = _p.fetch_versions
download_url   = _p.download_url
```

### jj provider.star 实现示例

`crates/vx-providers/jj/provider.star` 是首个 Starlark provider 迁移示例，
展示了 Level 2 复用（继承 `fetch_versions`，重写 `download_url`）：

```python
load("@vx//stdlib:github.star",   "make_fetch_versions", "github_asset_url")
load("@vx//stdlib:platform.star", "is_windows")

# ✅ fetch_versions 完全继承，零自定义代码
# jj tags 是 "v0.38.0"，parse_github_tag() 自动去掉 v 前缀
fetch_versions = make_fetch_versions("jj-vcs", "jj")

# ✅ download_url 重写：因为 jj Linux 用 musl（不是 gnu）
def _jj_triple(ctx):
    triples = {
        "linux/x64":  "x86_64-unknown-linux-musl",   # musl!
        "linux/arm64": "aarch64-unknown-linux-musl",
        "windows/x64": "x86_64-pc-windows-msvc",
        "macos/arm64": "aarch64-apple-darwin",
        # ...
    }
    return triples.get("{}/{}".format(ctx["platform"]["os"],
                                      ctx["platform"]["arch"]))

def download_url(ctx, version):
    triple = _jj_triple(ctx)
    if not triple:
        return None
    ext   = "zip" if ctx["platform"]["os"] == "windows" else "tar.gz"
    asset = "jj-v{}-{}.{}".format(version, triple, ext)
    return github_asset_url("jj-vcs", "jj", "v" + version, asset)
```

**对比 Rust 实现**：原 `JjUrlBuilder`（117 行 Rust）→ `provider.star`（~30 行 Starlark），
代码量减少 **74%**，且逻辑更直观。

### 与 TOML 的对比

| 能力 | TOML `provider.toml` | Starlark `provider.star` |
|------|---------------------|--------------------------|
| 静态 URL 模板 | ✅ `{version}` 占位符 | ✅ 字符串 format |
| 动态 URL 构建 | ❌ 无逻辑 | ✅ 完整 Python 逻辑 |
| 跨 provider 复用 | ❌ 无法共享 | ✅ `load()` 导入 |
| 继承并重写部分方法 | ❌ 无法继承 | ✅ 工厂函数 + 覆盖 |
| 条件逻辑（if/for） | ❌ | ✅ |
| 多步骤安装流程 | ❌ | ✅ `install()` 函数 |
| 沙箱安全 | N/A | ✅ 声明式权限 |

## 向后兼容性

1. **TOML 格式完全保留** - 所有现有 `provider.toml` 继续工作，无需修改
2. **优先级明确** - `provider.star` > `provider.toml`，共存时 Starlark 优先
3. **迁移路径清晰** - 可渐进式迁移，无需一次性全部转换
4. **API 版本化** - `provider.star` 中的 `version()` 函数支持未来扩展

## 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| Starlark 学习曲线 | 中 | 提供详细文档和示例，TOML 仍可用 |
| 沙箱绕过 | 高 | 严格审计 API，限制权限，编写安全测试 |
| 性能开销 | 低 | Starlark 执行很快，主要时间在 I/O |
| 维护复杂度 | 中 | 混合格式增加测试负担，需要 CI 覆盖 |
| starlark-rust API 变更 | 中 | 封装 starlark-rust，隔离变更影响 |

## 参考资料

- [Starlark Language Specification](https://github.com/bazelbuild/starlark/blob/master/spec.md)
- [starlark-rust (Meta/Facebook)](https://github.com/facebook/starlark-rust)
- [Buck2 Rule Authors Guide](https://buck2.build/docs/rule_authors/writing_rules/)
- [Buck2 Provider Design](https://buck2.build/docs/concepts/providers/)
- [Bazel Starlark Rules](https://bazel.build/extending/rules)
- [Deno Permission Model](https://docs.deno.com/runtime/fundamentals/security/)

## 更新记录

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2026-02-19 | v0.1 | 初始草稿 |
| 2026-02-19 | v0.2 | 加入 Buck2 借鉴内容：两阶段执行模型、Frozen Provider、声明式权限；修复 Starlark 示例中的非法 `import re` 语法；修正 Cargo.toml 和模块结构以匹配实际实现；修正 `SandboxConfig::restrictive()`（原 `secure()`）和内存限制（64MB）；修正 `_extract_version_from_path` 返回类型为 `str`；补充主流方案调研、替代方案章节 |
| 2026-02-19 | v0.3 | 深化 Buck2 借鉴：补充 Typed Provider Fields（`record` 类型替代无类型 dict）、`load()` 模块系统（`@vx//stdlib` 虚拟文件系统）、增量分析缓存（内容哈希）、声明式动作 API（`ctx.actions`）、BXL 调试工具对应设计；更新 Bazel 对比表格；更新实现计划（Phase 1 已完成项打勾，Phase 2-3 补充新任务） |
| 2026-02-19 | v0.4 | 实现进展更新：Phase 1/2 全部完成；新增 `@vx//stdlib:github.star`（`make_fetch_versions`、`make_download_url`、`make_github_provider` 工厂函数，实现「继承复用」模式）；完成首个 Starlark provider 迁移示例（`jj/provider.star`）；修复 jj `strip_v_prefix(false)` 导致的 `vv0.38.0` 双重前缀 bug；优化 `registry.rs` 合并重复的 provider 列表宏调用 |
| 2026-02-19 | v0.5 | Phase 3 全部完成：批量迁移 20 个 GitHub provider（fzf/ripgrep/fd/bat/yq/starship/just/deno/zig/hadolint/kubectl/helm/terraform/dagu/ollama/task/ninja/protoc/gh/rcedit）；`engine.rs` 实现 `VxFileLoader`（`FileLoader` trait）支持 `load()` 语句；`loader.rs` 注册 `github.star` 并实现 `RecursiveVxLoader` 支持 stdlib 模块间递归加载；三种继承模式（Level 1/2/3）均有实际案例 |
