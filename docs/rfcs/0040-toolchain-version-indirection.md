# RFC 0040: Toolchain Version Indirection — `version_info()` Provider Interface

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-04-04
> **目标版本**: v0.9.0

---

## 摘要

本 RFC 提出在 `provider.star` 接口中新增可选函数 `version_info()`，用于解决"版本管理器"型工具（如 Rust/rustup）中存在的**版本间接引用**问题。

当前 vx 对 Rust 的支持存在根本性的版本号错位：用户在 `vx.toml` 中记录 rustc 版本（如 `1.93.1`），但 vx store 按 rustup 版本（如 `1.28.1`）存储，导致 `vx check` 需要 O(n) 级别的扫描和运行时检测，`vx lock` 需要复杂的 passthrough 绕过逻辑。

通过在 provider.star 中声明版本映射关系，可以将复杂度从 Rust 代码下沉到 DSL 层，使 `vx check` 达到 O(1) 检测，并彻底简化 `vx lock` 的逻辑。

---

## 主流方案调研

在设计本方案之前，我们调研了以下主流版本管理工具的存储设计。

### 1. rustup (rust-lang/rustup)

**架构**: rustup 是 Rust 官方工具链管理器，按"通道-triple"命名存储工具链。

**存储布局**:
```
~/.rustup/
├── toolchains/
│   ├── stable-x86_64-unknown-linux-gnu/   # 按 toolchain 规格命名
│   │   ├── bin/
│   │   │   ├── rustc        # Rust 编译器
│   │   │   ├── cargo        # 包管理器
│   │   │   └── rustfmt
│   │   └── lib/
│   │       └── rustlib/
│   └── 1.70.0-x86_64-unknown-linux-gnu/  # 固定版本
├── settings.toml              # default_toolchain 配置
└── downloads/                 # 下载缓存
```

**关键设计**:
- 目录名 = `{channel}-{host-triple}`，如 `stable-x86_64-unknown-linux-gnu`
- **存储键是工具链规格**，不是 rustup 版本号
- 任何 rustup 版本都能管理任意 rustc 版本
- rustup 自身版本（1.28.1）与 rustc 版本（1.93.1）完全独立

**关键启示**: rustup 本身也按"用户面向版本"（如 `1.70.0`）命名目录，而不是按安装器版本。

---

### 2. pyenv (pyenv/pyenv)

**架构**: pyenv 按 Python 版本直接命名目录，是版本管理工具的标准模式。

**存储布局**:
```
~/.pyenv/
└── versions/
    ├── 3.11.13/     # 直接按 Python 版本号命名！
    │   ├── bin/
    │   │   ├── python3.11
    │   │   └── pip
    │   └── lib/
    ├── 3.12.9/
    └── 3.13.5/
```

**关键设计**:
- 目录名 = 用户关心的 Python 版本号
- O(1) 版本检测：`~/.pyenv/versions/3.11.13/` 是否存在？
- 安装器本身（pyenv 某个 commit）的版本与 Python 版本无关

---

### 3. nvm (nvm-sh/nvm)

**存储布局**:
```
~/.nvm/
└── versions/
    └── node/
        ├── v20.11.0/   # 按 Node.js 版本命名
        │   ├── bin/node
        │   └── bin/npm
        └── v22.3.0/
```

**关键设计**: 与 pyenv 相同——按**用户面向版本**（即管理的工具版本）命名，不按管理器版本命名。

---

### 4. uv (astral-sh/uv) 的 Python 管理

**存储布局** (uv 管理的 Python):
```
~/.local/share/uv/python/
├── cpython-3.11.9-linux-x86_64-gnu/   # 按 Python 版本 + 平台命名
└── cpython-3.12.4-linux-x86_64-gnu/
```

**关键设计**: 同样按"用户关心的 Python 版本"命名，安装器（uv）版本不影响目录名。

---

### 方案对比

| 特性 | rustup | pyenv | nvm | uv | vx 当前 | vx 理想 |
|------|--------|-------|-----|----|---------|---------|
| 存储键 | toolchain规格 | Python版本 | Node版本 | Python版本 | **rustup版本** ❌ | rustc版本 ✅ |
| 版本检测复杂度 | O(1) | O(1) | O(1) | O(1) | **O(n)** ❌ | O(1) ✅ |
| 多版本并存 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| 用户可读 | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |

### 设计启示

基于以上调研，本 RFC 应采用：

1. **按用户面向版本存储** — 所有成熟工具（pyenv/nvm/uv/rustup 自身）都按"用户关心的版本"命名目录
2. **安装器版本与工具版本解耦** — 通过 `version_info()` 声明映射关系，在 DSL 层解决复杂性
3. **O(1) 检测** — 版本目录存在即为已安装，无需运行可执行文件

---

## 动机

### 当前问题

vx 的 Rust provider 存在三个独立的版本号系统，相互间产生混乱：

```
用户视角 (vx.toml):     rust = "1.93.1"      ← rustc 版本
fetch_versions() 返回:  ["1.28.1", "1.27.0"]  ← rustup 版本
Store 目录:             ~/.vx/store/rust/1.28.1/ ← rustup 版本
                                      ↑
                           版本号完全不同！
```

这导致以下 workaround 代码（约 200 行）：

#### `vx check` 的问题：O(n) 扫描
```rust
// common.rs - 需要遍历所有 store 版本，运行 cargo --version
fn find_tool_in_store_by_detected_version(
    path_manager: &PathManager,
    tool: &str,
    requested_version: &str,
) -> Result<Option<(PathBuf, String)>> {
    // 遍历所有安装版本...
    for store_version in &installed_versions {
        // 运行 cargo --version 检测实际版本...
        // 版本匹配逻辑...
    }
}
```

#### `vx lock` 的问题：复杂 passthrough 逻辑
```rust
// lock.rs - 特殊处理 Rust 生态
let is_passthrough = ecosystem == Ecosystem::Rust
    || versions.iter().any(|v| v.metadata.get("passthrough") == Some(&"true".to_string()));

// ... passthrough 分支 ...
// ... try_lock_from_store() 回退 ...
// ... download_version 特殊处理 ...
```

### 根本原因

Rust 是**版本管理器**型工具的典型代表：vx 下载的是 **rustup**（工具管理器），但用户关心的是 **rustc/cargo** 版本。这种间接关系在 Rust 代码层处理会导致污染性扩散，正确的位置应该是 `provider.star` DSL 层。

### 影响范围

当前需要 workaround 的问题：
- `vx check rust` 报告 `system_fallback` 而非 `installed`（Issue #744）
- `vx lock` 无法持久化某些工具版本（Issue #745）
- 代码复杂度高，难以维护
- 潜在影响：未来任何"版本管理器"型工具都会遇到同样问题

---

## 设计方案

### 核心概念：`version_info()` 函数

在 `provider.star` 中新增可选函数 `version_info(ctx, user_version)`：

```starlark
def version_info(ctx, user_version):
    """Map user-facing version to store/install parameters.
    
    This function allows providers to declare a mapping between the version
    users specify in vx.toml and the actual storage/download strategy.
    
    For most tools, this function is NOT needed (1:1 mapping is assumed).
    
    Use this for "version manager" pattern tools where:
    - The user specifies the managed tool's version (e.g., rustc 1.93.1)
    - But vx must download a manager/installer (e.g., rustup 1.28.1)
    - And the store should be indexed by the USER-FACING version
    
    Args:
        ctx: Provider context (ctx.platform, ctx.install_dir, etc.)
        user_version: Version string from vx.toml (e.g., "1.93.1", "stable")
    
    Returns:
        None to use default behavior (store_version = user_version = download_version)
        
        Or a dict with:
            store_as (required):
                The version string to use as the store directory name.
                e.g., "1.93.1" → ~/.vx/store/rust/1.93.1/
                
            download_version (optional, default: None = latest available):
                The version to use for selecting the download URL.
                None means "use the latest available version from fetch_versions()".
                e.g., None → downloads latest rustup-init
                
            install_params (optional, default: {}):
                Extra parameters passed to post_extract as ctx.install_params.
                Use this to pass the user's version to the installation script.
                e.g., {"toolchain": "1.93.1"} → rustup-init --default-toolchain 1.93.1
    """
    # Default implementation (for most tools): 1:1 mapping
    return None
```

### Rust Provider 实现

更新 `crates/vx-providers/rust/provider.star`：

```starlark
def version_info(ctx, user_version):
    """Rust version mapping: user specifies rustc version, we download rustup.
    
    Key insight:
    - User writes: rust = "1.93.1"  (rustc version)
    - We download: rustup-init-1.28.1-{triple}  (latest rustup installer)
    - We install to: ~/.vx/store/rust/1.93.1/   (stored by rustc version)
    - rustup installs: --default-toolchain 1.93.1
    
    Any rustup version can install any rustc version, so we always
    use the latest rustup as the installer.
    """
    return {
        "store_as": user_version,         # Store directory: ~/.vx/store/rust/1.93.1/
        "download_version": None,          # None = use latest rustup from fetch_versions()
        "install_params": {
            "toolchain": user_version,     # Passed to post_extract as ctx.install_params
        },
    }
```

更新 `post_extract` 使用 `install_params`：

```starlark
def post_extract(ctx, _version, install_dir):
    # Use toolchain from install_params (set by version_info), default to "stable"
    toolchain = ctx.install_params.get("toolchain", "stable") if hasattr(ctx, "install_params") else "stable"
    
    actions = []
    init_bin = "rustup-init.exe" if ctx.platform.os == "windows" else "rustup-init"
    if ctx.platform.os != "windows":
        actions.append(set_permissions("bin/" + init_bin, "755"))
    actions.append(run_command(
        install_dir + "/bin/" + init_bin,
        args = ["-y", "--no-modify-path", "--default-toolchain", toolchain],
        env = {
            "RUSTUP_HOME": install_dir + "/rustup",
            "CARGO_HOME":  install_dir + "/cargo",
        },
        on_failure = "error",
    ))
    return actions
```

### 安装流程变化

**Before（当前）**:
```
用户:    rust = "1.93.1"
Lock:    version = "1.93.1", download_version = "1.28.1" (passthrough)
Install: 下载 rustup-init-1.28.1-... 
         存储到 ~/.vx/store/rust/1.28.1/
         运行: rustup-init --default-toolchain stable  ← 丢失用户版本！

Check:   遍历 store/rust/1.28.1/... 运行 cargo --version → O(n)
```

**After（新设计）**:
```
用户:    rust = "1.93.1"
         ↓ version_info("1.93.1") 返回 store_as="1.93.1", download=None, params={toolchain:"1.93.1"}
Lock:    version = "1.93.1" (直接记录，无需 passthrough 特例)
Install: 下载 rustup-init-{latest}-...
         存储到 ~/.vx/store/rust/1.93.1/   ← 按 rustc 版本！
         运行: rustup-init --default-toolchain 1.93.1  ← 使用用户版本！

Check:   ~/.vx/store/rust/1.93.1/ 是否存在？ → O(1) ✅
```

### Starlark 引擎变更

在 `vx-starlark` 中新增对 `version_info` 函数的支持：

#### 1. `VersionInfo` 结构体（Rust）

```rust
// crates/vx-starlark/src/provider/types.rs

/// Version indirection information returned by provider.star's version_info()
#[derive(Debug, Clone, Default)]
pub struct VersionInfoResult {
    /// Version string to use as the store directory name.
    /// If None, the user-specified version is used directly.
    pub store_as: Option<String>,
    
    /// Version to use for download URL selection.
    /// If None, use the latest available version from fetch_versions().
    pub download_version: Option<String>,
    
    /// Extra parameters passed to post_extract as ctx.install_params.
    pub install_params: HashMap<String, String>,
}
```

#### 2. Provider Handle 新方法

```rust
// crates/vx-starlark/src/handle.rs

impl StarlarkProviderHandle {
    /// Call version_info(ctx, user_version) if the provider defines it.
    /// Returns None if the provider doesn't define version_info (default: 1:1 mapping).
    pub fn version_info(
        &self,
        ctx: &StarlarkContext,
        user_version: &str,
    ) -> Result<Option<VersionInfoResult>> {
        // Check if function exists in provider.star
        if !self.has_function("version_info") {
            return Ok(None);
        }
        // Execute Starlark function and parse result
        let result = self.call_function("version_info", ctx, &[user_version])?;
        Ok(Some(VersionInfoResult::from_starlark(result)?))
    }
}
```

#### 3. Runtime trait 扩展

```rust
// crates/vx-runtime/src/runtime/mod.rs

pub trait Runtime: Send + Sync {
    // ... existing methods ...
    
    /// Resolve version indirection for this runtime.
    ///
    /// For most tools, returns None (1:1 mapping: store_version = user_version).
    /// For toolchain-managed tools (like Rust/rustup), returns a VersionInfoResult
    /// describing how to map user version → store version, download version, etc.
    ///
    /// This is the key hook for the "version manager" pattern.
    async fn version_info(
        &self,
        user_version: &str,
        ctx: &RuntimeContext,
    ) -> Result<Option<VersionInfoResult>> {
        Ok(None)  // Default: 1:1 mapping
    }
}
```

### `vx check` 简化

**After**（`common.rs` 变更）：

```rust
pub fn check_tool_status(
    path_manager: &PathManager,
    tool: &str,
    version: &str,
    runtime: Option<&dyn Runtime>,
    ctx: &RuntimeContext,
) -> Result<(ToolStatus, Option<PathBuf>, Option<String>)> {
    // Resolve store version via version_info() if provider supports it
    let store_version = if let Some(rt) = runtime {
        if let Ok(Some(info)) = rt.version_info(version, ctx).await {
            info.store_as.unwrap_or_else(|| version.to_string())
        } else {
            version.to_string()
        }
    } else {
        version.to_string()
    };

    // O(1) check: does the store directory exist?
    let store_dir = path_manager.version_store_dir(tool, &store_version);
    if store_dir.exists() {
        let bin_path = find_tool_bin_dir(&store_dir, tool);
        return Ok((ToolStatus::Installed, Some(bin_path), Some(store_version)));
    }

    // ... rest of system fallback logic (unchanged) ...
}
```

**可删除的代码**（~150 行）：
- `find_tool_in_store_by_detected_version()` — 整个函数
- `get_tool_store_bin_subdirs()` — 不再需要
- Rust 特例的版本命令映射

### `vx lock` 简化

**After**（`lock.rs` 变更）：

```rust
async fn resolve_tool_version(
    registry: &ProviderRegistry,
    ctx: &RuntimeContext,
    solver: &VersionSolver,
    tool_name: &str,
    version_str: &str,
    verbose: bool,
) -> Result<LockedTool> {
    let provider = registry.get_provider(tool_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown tool: {}", tool_name))?;
    let runtime = provider.get_runtime(tool_name)
        .ok_or_else(|| anyhow::anyhow!("No runtime found for: {}", tool_name))?;

    // Check version_info for version mapping
    let version_info = runtime.version_info(version_str, ctx).await?;
    
    if let Some(ref info) = version_info {
        // Provider declares explicit version mapping (e.g., Rust)
        let locked_version = info.store_as.as_deref().unwrap_or(version_str);
        let download_version = info.download_version.as_deref();
        
        // Get download URL using download_version (or latest if None)
        let dl_version = if let Some(dv) = download_version {
            dv.to_string()
        } else {
            // Use latest available version for download
            let versions = runtime.fetch_versions(ctx).await?;
            versions.first()
                .map(|v| v.version.clone())
                .unwrap_or_else(|| version_str.to_string())
        };
        
        let current_platform = vx_runtime::Platform::current();
        let download_url = runtime.download_url(&dl_version, &current_platform).await.ok().flatten();
        
        let mut locked = LockedTool::new(locked_version.to_string(), "provider".to_string())
            .with_resolved_from(version_str)
            .with_ecosystem(Ecosystem::Generic);
        if let Some(url) = download_url {
            locked = locked.with_download_url(url);
        }
        return Ok(locked);
    }

    // Standard path for tools without version_info (the majority)
    let versions = runtime.fetch_versions(ctx).await?;
    let ecosystem = convert_ecosystem(runtime.ecosystem());
    let request = VersionRequest::parse(version_str);
    
    let resolved = solver.resolve(tool_name, &request, &versions, &ecosystem)
        .map_err(|e| {
            // Fallback: check local store
            try_lock_from_store(tool_name, version_str, &ecosystem, verbose)
                .ok().flatten()
                .map(Ok)
                .unwrap_or_else(|| Err(anyhow::anyhow!("{}", e)))
        })??;

    // ... rest unchanged ...
}
```

**可删除/简化的代码**（~100 行）：
- `is_passthrough` 整个逻辑分支
- Rust 生态的 passthrough 特殊处理
- `runtime/mod.rs` 中的 "Rust ecosystem passthrough" 逻辑

---

## Store 目录布局变化

### 当前布局（Before）

```
~/.vx/store/
└── rust/
    └── 1.28.1/           ← rustup 版本
        └── {platform}/
            ├── bin/
            │   └── rustup-init
            ├── cargo/
            │   └── bin/
            │       ├── rustc    (v1.93.1 实际版本)
            │       └── cargo
            └── rustup/
```

### 新布局（After）

```
~/.vx/store/
└── rust/
    ├── 1.93.1/           ← rustc 版本（与 vx.toml 一致！）
    │   └── {platform}/
    │       ├── cargo/
    │       │   └── bin/
    │       │       ├── rustc
    │       │       └── cargo
    │       └── rustup/
    └── stable/           ← 通道名也可以作为版本
        └── {platform}/
```

### 并存兼容策略

- 旧目录（按 rustup 版本命名）继续工作，check 时系统 PATH 回退仍然有效
- 新安装统一使用新布局
- `vx store gc` 可迁移旧布局（可选，不强制）

---

## 向后兼容性

### 兼容策略

1. **`version_info()` 是可选函数** — 不实现该函数的 provider 行为完全不变
2. **现有安装不受影响** — 旧 store 目录（如 `store/rust/1.28.1/`）继续可用
3. **系统 PATH 回退保留** — 即使 store 中不存在，仍能通过系统 PATH 找到工具
4. **Lock 文件向后兼容** — 现有 `vx.lock` 文件格式不变

### 迁移路径

```bash
# 检查当前 Rust 安装状态
vx check rust

# 使用新布局重新安装（可选）
vx install rust@1.93.1 --force

# 清理旧的 rustup 版本目录
vx store gc --prune-orphaned
```

---

## 实现计划

### Phase 1: provider.star DSL 支持（v0.9.0）✅

- [x] 在 `vx-starlark/stdlib/` 中文档化 `version_info()` 函数签名
- [x] 在 `crates/vx-starlark/src/provider/types.rs` 中添加 `VersionInfoResult`
- [x] 在 `StarlarkProviderHandle` 中添加 `version_info()` 调用支持
- [x] 在 `Runtime` trait 中添加默认 `version_info()` 方法
- [x] 在 `ManifestDrivenRuntime` 中通过 provider handle 实现该方法

### Phase 2: Rust Provider 更新（v0.9.0）✅

- [x] 更新 `crates/vx-providers/rust/provider.star`：
  - 添加 `version_info()` 函数
  - 更新 `post_extract()` 使用 `ctx.install_params`
- [x] 添加 Rust provider 的 starlark 单元测试

### Phase 3: 命令简化（v0.9.0）🔧 部分完成

- [ ] 更新 `vx-cli/src/commands/check.rs`（调用 `version_info`）
- [ ] 更新 `vx-cli/src/commands/common.rs`（移除 O(n) store 扫描）：
  - 删除 `find_tool_in_store_by_detected_version()`
  - 删除 `get_tool_store_bin_subdirs()`
  - 简化 `check_tool_status()`
- [x] 更新 `vx-cli/src/commands/lock.rs`（集成 `version_info` 优先调用）：
  - `resolve_tool_version()` 优先调用 `version_info()`
  - 旧 `is_passthrough` 逻辑保留以向后兼容（待后续清理）
- [ ] 更新 `vx-runtime/src/runtime/mod.rs`（移除 Rust passthrough 回退）

### Phase 4: 测试（v0.9.0）✅

- [x] `crates/vx-starlark/tests/version_info_tests.rs` — 26 个测试
  - `VersionInfoResult::from_json()` 解析（null、有效、部分字段、边界情况）
  - `StarlarkProvider::version_info()` 集成（未定义、返回 None、返回完整结构）
- [x] `crates/vx-runtime/tests/version_info_tests.rs` — 23 个测试
  - 构造器和 builder 模式
  - `effective_store_version()` 逻辑
  - Rust/Python 场景、channel 名称、日期版本等边界情况
- [ ] `crates/vx-cli/tests/` 添加 check/lock 集成测试（使用 mock runtime）
- [ ] 更新 `crates/vx-cli/tests/cmd/` 快照测试

---

## 替代方案

### 方案 A：在 `provider.toml`（已废弃）中添加 `version_scheme` 字段

```toml
[version_scheme]
type = "manager"  # manager | direct
# type=manager: user version → managed tool version
# type=direct:  user version = installer version (default)
```

**不选择原因**: vx 已迁移到纯 Starlark DSL，TOML 方案已废弃（RFC 0038）。

### 方案 B：在 Rust 代码中维护工具类型映射表

```rust
fn is_toolchain_manager(tool: &str) -> bool {
    matches!(tool, "rust" | "rustup")
}
```

**不选择原因**: 硬编码不可扩展，新增工具需要修改 Rust 代码，违背 provider.star 的设计初衷。

### 方案 C：按 rustup 版本存储，添加版本索引文件

维护 `store/rust/index.json`：
```json
{"1.28.1": "1.93.1", "1.27.0": "1.85.0"}
```

**不选择原因**: 增加额外的 index 文件，并发修改时有竞争风险，且每次 check 仍需读取 index 文件（非 O(1) 常量时间）。

---

## 参考资料

### 主流项目实现

- [rustup toolchains 目录设计](https://github.com/rust-lang/rustup/tree/master/src) — 按 toolchain 规格命名存储
- [pyenv versions 布局](https://github.com/pyenv/pyenv) — 按 Python 版本直接命名
- [nvm 存储设计](https://github.com/nvm-sh/nvm) — 按 Node.js 版本命名
- [uv Python 管理](https://github.com/astral-sh/uv) — 按 CPython 版本命名

### 相关 RFC

- [RFC 0036: Starlark Provider Support](./0036-starlark-provider-support.md)
- [RFC 0038: provider.star Replaces TOML](./0038-provider-star-replaces-toml.md)
- [RFC 0039: Star-Only Providers and Dynamic Registry](./0039-star-only-providers-and-dynamic-registry.md)

### 相关 Issues

- Issue #744: `vx check` reports Rust as `system_fallback` instead of `installed`
- Issue #745: `vx lock` fails to persist Python version in lock file
- PR #746: Partial fix for #744 and #745 (workaround, to be superseded by this RFC)

---

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-04-04 | Draft | 初始草案 |
| 2026-04-04 | Impl | Phase 1-2 实现：DSL 支持 + Rust provider 更新 + lock.rs 集成 + 49 个测试 |
