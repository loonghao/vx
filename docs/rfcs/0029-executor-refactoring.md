# RFC 0029: Executor 架构重构与版本策略统一

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-02-05
> **目标版本**: v0.6.0

## 摘要

本 RFC 提议对 `vx-resolver` 中的 `Executor` 进行架构重构，并统一版本解析策略。当前实现中存在以下核心问题：

1. **版本策略不一致**：`resolve_version` 中的 "latest" 语义与安装流程不一致
2. **Executor 职责过重**：混合了版本解析、安装、依赖注入、代理处理、命令执行等多个关注点
3. **ManifestRegistry 边界模糊**：同时承担清单加载、Provider 构建、元数据查询等职责
4. **Silent Failure**：缺失工厂时仅 warn，上层无法感知

本 RFC 将引入 **Pipeline 架构** 和 **统一版本策略**，提升代码可维护性、可测试性和错误可观测性。

## 主流方案调研

### 1. Cargo (rust-lang/cargo)

**架构**: Cargo 采用清晰的阶段分离架构

**核心设计**:
```rust
// cargo/src/cargo/core/resolver/mod.rs
pub struct Resolve {
    graph: Graph<PackageId>,
    replacements: HashMap<PackageId, PackageId>,
    features: HashMap<PackageId, HashSet<String>>,
}

// 解析与执行完全分离
pub fn resolve(
    ws: &Workspace<'_>,
    opts: &ResolveOpts,
) -> CargoResult<Resolve> {
    // 1. 收集依赖
    // 2. 构建依赖图
    // 3. 解析版本
    // 4. 返回解析结果（不执行）
}
```

**关键特性**:
- 解析（Resolve）与执行（Compile）完全分离
- 使用 `Resolve` 结构体作为中间表示
- 错误类型明确分层（`CargoResult<T>`）

**依赖库**:
- `semver` - 语义化版本处理
- `petgraph` - 依赖图管理

### 2. uv (astral-sh/uv)

**架构**: uv 采用 Pipeline + Context 模式

**核心设计**:
```rust
// uv/crates/uv-resolver/src/resolver/mod.rs
pub struct ResolverState {
    /// The packages that have been resolved.
    packages: FxHashMap<PackageName, ResolvedPackage>,
    /// The pending work queue.
    pending: VecDeque<PackageName>,
}

// 执行上下文
pub struct ResolverContext<'a> {
    client: &'a RegistryClient,
    index: &'a InMemoryIndex,
    config: &'a ResolverConfig,
}
```

**关键特性**:
- 状态与上下文分离
- 支持增量解析
- 使用 `tracing` 进行结构化日志

**依赖库**:
- `pep508_rs` - Python 版本规范解析
- `tracing` - 结构化日志

### 3. rustup (rust-lang/rustup)

**架构**: rustup 采用 Toolchain 抽象

**核心设计**:
```rust
// rustup/src/toolchain.rs
pub enum Toolchain {
    Installed(InstalledToolchain),
    NotInstalled(ToolchainDesc),
}

impl Toolchain {
    pub fn resolve(cfg: &Cfg, name: &str) -> Result<Self> {
        // 统一的版本解析逻辑
        if name == "stable" || name == "latest" {
            Self::resolve_stable(cfg)
        } else {
            Self::resolve_specific(cfg, name)
        }
    }
}
```

**关键特性**:
- 统一的版本别名处理（stable/latest/nightly）
- 明确的 Installed vs NotInstalled 状态
- 配置驱动的默认版本

### 4. Volta (volta-cli/volta) ⭐ 重点借鉴

**架构**: Volta 采用 **Shim + Project Pinning** 架构

**核心设计理念**:
```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Volta Architecture                               │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   User Command          Shim Layer           Tool Resolution            │
│   ┌─────────┐          ┌─────────┐          ┌─────────────────┐        │
│   │  node   │ ───────▶ │  shim   │ ───────▶ │ Project Pinning │        │
│   │  npm    │          │ (Rust)  │          │ (package.json)  │        │
│   │  yarn   │          │         │          │                 │        │
│   └─────────┘          └─────────┘          └────────┬────────┘        │
│                                                      │                  │
│                                                      ▼                  │
│                              ┌─────────────────────────────────┐       │
│                              │  Fallback Chain:                │       │
│                              │  1. Project (package.json)      │       │
│                              │  2. User Default (volta pin)    │       │
│                              │  3. System Default              │       │
│                              └─────────────────────────────────┘       │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

**关键特性**:

1. **零配置项目切换 (Seamless Project Switching)**
   ```json
   // package.json - 项目级版本固定
   {
     "volta": {
       "node": "20.10.0",
       "npm": "10.2.0"
     }
   }
   ```
   - 进入项目目录时自动切换版本
   - 无需手动执行 `nvm use` / `fnm use`
   - 版本信息与项目配置共存

2. **Rust 编写的高性能 Shim**
   - 所有工具调用经过统一的 shim
   - Shim 启动时间 < 5ms
   - 使用 Rust 编译为单一二进制

3. **工具链隔离 (Toolchain Isolation)**
   ```
   ~/.volta/
   ├── bin/              # Shims
   │   ├── node
   │   ├── npm
   │   └── yarn
   ├── tools/
   │   ├── node/
   │   │   └── 20.10.0/  # 完整 Node.js 安装
   │   └── yarn/
   │       └── 1.22.0/   # 独立的包管理器
   └── cache/            # 下载缓存
   ```

4. **错误处理与用户提示**
   ```rust
   // volta/crates/volta-core/src/error.rs
   pub enum ErrorKind {
       /// No matching version found
       VersionNotFound { tool: String, matching: String },
       /// Network error during download
       DownloadError { tool: String, from_url: String },
       /// Project has no pinned version
       NoProjectNodeVersion,
       // ... 精细的错误分类
   }
   ```

**vx 可借鉴点**:
- ✅ 项目级版本固定（已有 vx.toml）
- 🆕 **Shim 性能优化**：借鉴 Volta 的快速 shim 启动
- 🆕 **错误分类体系**：参考 Volta 的 ErrorKind 设计
- 🆕 **Fallback Chain**：统一的版本解析回退链

### 5. mise (jdx/mise)

**架构**: mise 采用 **多源配置 + 插件系统** 架构

**核心设计**:
```toml
# .mise.toml - 统一配置格式
[tools]
node = "20"
python = "3.12"
rust = "stable"

[env]
NODE_ENV = "development"

[tasks]
dev = "npm run dev"
test = "npm test"
```

**关键特性**:

1. **多配置文件支持 (Polyglot Config)**
   ```
   配置优先级（从高到低）：
   1. .mise.toml (本目录)
   2. .mise.toml (父目录，递归向上)
   3. .tool-versions (asdf 兼容)
   4. .nvmrc / .node-version (Node.js 专用)
   5. .python-version (Python 专用)
   6. ~/.config/mise/config.toml (全局)
   ```

2. **任务系统 (Task Runner)**
   ```bash
   mise run test      # 运行项目任务
   mise run dev       # 启动开发服务器
   mise tasks         # 列出所有任务
   ```

3. **环境变量管理**
   ```toml
   [env]
   DATABASE_URL = "postgres://localhost/dev"
   # 支持文件引用
   _.file = ".env.local"
   ```

4. **信任机制 (Trust)**
   ```bash
   mise trust          # 信任当前目录的配置
   mise trust --all    # 信任所有配置
   ```
   - 防止恶意配置执行任意代码
   - 首次进入目录提示用户确认

**vx 可借鉴点**:
- 🆕 **多配置文件兼容**：支持 .nvmrc、.node-version 等
- 🆕 **任务系统集成**：vx.toml 中的 [scripts] 增强
- 🆕 **环境变量管理**：项目级环境变量
- 🆕 **配置信任机制**：安全执行用户配置

### 6. proto (moonrepo/proto)

**架构**: proto 采用 **WASM 插件 + 统一版本文件** 架构

**核心设计**:
```toml
# .prototools - 统一版本文件
node = "20.10.0"
npm = "10.2.0"
pnpm = "8.10.0"

[plugins]
custom-tool = "source:https://example.com/plugin.wasm"
```

**关键特性**:

1. **WASM 插件系统**
   ```rust
   // 插件 trait
   #[extism_pdk::plugin_fn]
   pub fn download_prebuilt(input: Json<DownloadPrebuiltInput>) 
       -> FnResult<Json<DownloadPrebuiltOutput>> {
       // 完全自定义的下载逻辑
   }
   ```
   - 插件使用 WASM 编写，跨平台
   - 沙箱执行，安全隔离
   - 支持远程插件加载

2. **版本检测与自动升级**
   ```bash
   proto outdated       # 检查过时版本
   proto upgrade        # 升级到最新
   proto pin node 21    # 固定到新版本
   ```

3. **工具链钩子 (Hooks)**
   ```toml
   [tools.node.hooks]
   pre_install = "echo Installing Node.js..."
   post_install = "npm install -g pnpm"
   ```

**vx 可借鉴点**:
- 🆕 **版本过时检测**：`vx outdated` 命令
- 🆕 **钩子系统**：安装前后执行自定义脚本
- 🆕 **升级辅助**：`vx upgrade` 批量升级

### 7. fnm (Schniz/fnm)

**架构**: fnm 专注于 **极速启动 + Shell 集成**

**关键特性**:

1. **超快启动时间**
   ```
   启动时间对比（ms）：
   ┌────────┬──────────┐
   │ Tool   │ Time     │
   ├────────┼──────────┤
   │ fnm    │ < 1ms    │
   │ nvm    │ ~200ms   │
   │ volta  │ < 5ms    │
   │ vx     │ ~10ms    │ ← 目标优化
   └────────┴──────────┘
   ```

2. **自动版本切换 (Auto-switch)**
   ```bash
   # .bashrc / .zshrc
   eval "$(fnm env --use-on-cd)"
   
   # 进入目录时自动切换
   cd my-project  # 自动读取 .nvmrc 并切换
   ```

3. **多 Shell 支持**
   ```bash
   fnm env --shell bash
   fnm env --shell zsh
   fnm env --shell fish
   fnm env --shell powershell
   ```

**vx 可借鉴点**:
- 🆕 **启动性能优化**：目标 < 5ms
- 🆕 **自动版本切换**：`vx env --use-on-cd`
- 🆕 **多 Shell 集成**：完善的 shell 初始化脚本

### 方案对比（扩展版）

| 特性 | Cargo | uv | Volta | mise | proto | fnm | vx (当前) | vx (目标) |
|------|-------|-----|-------|------|-------|-----|-----------|------------|
| 解析/执行分离 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| 统一版本策略 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| 项目级版本固定 | N/A | ✓ | ✓ | ✓ | ✓ | ✓ | 部分 | ✓ |
| 自动版本切换 | N/A | - | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| 多配置文件支持 | N/A | ✓ | ✗ | ✓ | ✗ | ✓ | ✗ | ✓ |
| 任务系统 | ✓ | - | ✗ | ✓ | ✗ | ✗ | 部分 | ✓ |
| 错误分类 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✗ | ✓ |
| 启动性能 (ms) | N/A | <5 | <5 | <10 | <10 | <1 | ~10 | <5 |
| 跨语言支持 | ✗ | ✗ | ✗ | ✓ | ✓ | ✗ | ✓ | ✓ |

### 设计启示

基于以上调研，本 RFC 应采用：

1. **Pipeline 架构** - 参考 Cargo/uv 的阶段分离设计，将 Executor 拆分为独立阶段
2. **ExecutionPlan 中间表示** - 参考 Cargo 的 `Resolve` 结构，引入执行计划作为中间层
3. **统一版本策略** - 参考 rustup 的版本别名处理，统一 "latest" 语义
4. **结构化错误** - 参考 Volta 的 ErrorKind，引入精细的错误分类
5. **Fallback Chain** - 参考 Volta 的版本解析回退链，实现项目 → 用户 → 系统的版本解析
6. **多配置文件支持** - 参考 mise，兼容 .nvmrc/.node-version/.python-version 等
7. **自动版本切换** - 参考 fnm，实现 `--use-on-cd` 进入目录自动切换
8. **启动性能优化** - 参考 fnm/Volta，目标 shim 启动时间 < 5ms
9. **任务系统增强** - 参考 mise 的任务系统，增强 vx.toml [scripts] 功能

## 动机

### 当前状态分析

#### 问题 1: 版本策略不一致

```rust
// executor.rs - resolve_version
fn resolve_version(&self, runtime_name: &str, version: &str) -> Option<String> {
    if version == "latest" {
        // 使用已安装的最新版本
        self.get_latest_installed_version(runtime_name)
    }
}

// installation.rs - install_runtime
async fn install_runtime(&self, runtime_name: &str) -> Result<()> {
    // 安装远端最新版本
    let latest = runtime.fetch_versions().await?.first();
}
```

**影响**：
- 用户指定 `"latest"` 时行为不可预测
- CI/CD 环境复现困难
- 调试时版本追踪困难

#### 问题 2: Executor 职责过重

当前 `Executor::execute_with_with_deps` 混合了 8+ 个关注点：

```
┌─────────────────────────────────────────────────────┐
│                execute_with_with_deps               │
├─────────────────────────────────────────────────────┤
│ 1. 版本解析                                          │
│ 2. 离线 bundle 处理                                  │
│ 3. 平台约束检查                                      │
│ 4. 安装 + 重新解析                                   │
│ 5. 依赖注入 (--with)                                │
│ 6. Proxy runtime 处理 (RFC 0028)                    │
│ 7. 环境变量准备                                      │
│ 8. 命令执行                                          │
└─────────────────────────────────────────────────────┘
```

**影响**：
- 单元测试困难，需要 mock 大量依赖
- 修改一处逻辑可能影响其他流程
- 代码复用困难

#### 问题 3: ManifestRegistry 边界模糊

```rust
impl ManifestRegistry {
    // 职责 1: 清单加载
    pub fn load_all() -> Self { ... }
    
    // 职责 2: 构造 ProviderRegistry
    pub fn build_registry(&self) -> ProviderRegistry { ... }
    
    // 职责 3: 元数据查询
    pub fn get_runtime_metadata(&self, name: &str) -> Option<...> { ... }
    
    // 职责 4: 平台约束处理
    fn merge_platform_constraint(&self) -> Option<...> { ... }
}
```

**影响**：
- 缺失工厂时仅 warn，上层无感知
- 元数据与实际 Provider 可能不一致
- 难以独立测试各职责

### 需求分析

1. **统一版本语义** - "latest" 应有明确、一致的定义
2. **关注点分离** - 每个模块只负责单一职责
3. **可观测性** - 错误信息完整，包含上下文链路
4. **可测试性** - 各阶段可独立测试
5. **向后兼容** - 不改变用户可见行为（除 bug fix）

## 设计方案

### 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         ExecutionPipeline                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐     │
│  │ Resolve  │ → │  Ensure  │ → │ Prepare  │ → │ Execute  │     │
│  │  Stage   │   │  Stage   │   │  Stage   │   │  Stage   │     │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘     │
│       │              │              │              │             │
│       ▼              ▼              ▼              ▼             │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐     │
│  │Resolution│   │Installed │   │Execution │   │  Exit    │     │
│  │  Result  │   │ Versions │   │  Context │   │  Code    │     │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 模块设计

#### 1. ExecutionPlan (中间表示)

```rust
// crates/vx-resolver/src/executor/plan.rs

/// 执行计划 - 解析阶段的输出，执行阶段的输入
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// 主运行时
    pub primary: ResolvedRuntime,
    /// 依赖运行时（按拓扑排序）
    pub dependencies: Vec<ResolvedRuntime>,
    /// 额外注入的运行时 (--with)
    pub injected: Vec<ResolvedRuntime>,
    /// Proxy 运行时（如果需要）
    pub proxy: Option<ProxyRuntime>,
    /// 执行配置
    pub config: ExecutionConfig,
}

#[derive(Debug, Clone)]
pub struct ResolvedRuntime {
    /// 运行时名称
    pub name: String,
    /// 解析后的版本
    pub version: ResolvedVersion,
    /// 安装状态
    pub status: InstallStatus,
    /// 可执行文件路径（如果已安装）
    pub executable: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum ResolvedVersion {
    /// 具体版本号
    Specific(String),
    /// 已安装的最新版本
    LatestInstalled(String),
    /// 远端最新版本（需要安装）
    LatestRemote(String),
    /// 范围版本解析结果
    Range { spec: String, resolved: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallStatus {
    /// 已安装
    Installed,
    /// 需要安装
    NeedsInstall,
    /// 需要先安装依赖
    NeedsDependency(String),
    /// 平台不支持
    PlatformUnsupported(String),
}
```

#### 2. Pipeline Stages

```rust
// crates/vx-resolver/src/executor/stages/mod.rs

pub mod resolve;
pub mod ensure;
pub mod prepare;
pub mod execute;

/// Pipeline Stage trait
#[async_trait]
pub trait Stage<Input, Output> {
    type Error;
    
    async fn execute(&self, input: Input, ctx: &ExecutorContext) -> Result<Output, Self::Error>;
}
```

##### Stage 1: Resolve (解析阶段)

```rust
// crates/vx-resolver/src/executor/stages/resolve.rs

pub struct ResolveStage {
    resolver: Arc<Resolver>,
    version_strategy: VersionStrategy,
}

/// 版本策略配置
#[derive(Debug, Clone, Default)]
pub struct VersionStrategy {
    /// "latest" 的语义
    pub latest_behavior: LatestBehavior,
    /// 是否允许自动升级
    pub allow_auto_upgrade: bool,
}

#[derive(Debug, Clone, Default)]
pub enum LatestBehavior {
    /// 使用已安装的最新版本（默认，快速）
    #[default]
    InstalledLatest,
    /// 检查远端最新版本（需要网络）
    RemoteLatest,
    /// 使用锁文件版本（CI 推荐）
    Locked,
}

#[async_trait]
impl Stage<ResolveRequest, ExecutionPlan> for ResolveStage {
    type Error = ResolverError;
    
    async fn execute(
        &self, 
        request: ResolveRequest, 
        ctx: &ExecutorContext
    ) -> Result<ExecutionPlan, Self::Error> {
        // 1. 解析主运行时版本
        let primary = self.resolve_runtime(&request.runtime, &request.version, ctx).await?;
        
        // 2. 解析依赖
        let dependencies = self.resolve_dependencies(&primary, ctx).await?;
        
        // 3. 解析 --with 注入
        let injected = self.resolve_injected(&request.with_runtimes, ctx).await?;
        
        // 4. 检查 proxy 需求
        let proxy = self.resolve_proxy(&primary, ctx).await?;
        
        Ok(ExecutionPlan {
            primary,
            dependencies,
            injected,
            proxy,
            config: request.config,
        })
    }
}

impl ResolveStage {
    /// 统一的版本解析逻辑
    async fn resolve_version(
        &self,
        runtime_name: &str,
        version_spec: &str,
        ctx: &ExecutorContext,
    ) -> Result<ResolvedVersion, ResolverError> {
        match version_spec {
            "latest" => self.resolve_latest(runtime_name, ctx).await,
            spec if spec.starts_with('^') || spec.starts_with('~') => {
                self.resolve_range(runtime_name, spec, ctx).await
            }
            specific => Ok(ResolvedVersion::Specific(specific.to_string())),
        }
    }
    
    async fn resolve_latest(
        &self,
        runtime_name: &str,
        ctx: &ExecutorContext,
    ) -> Result<ResolvedVersion, ResolverError> {
        match self.version_strategy.latest_behavior {
            LatestBehavior::InstalledLatest => {
                // 优先使用已安装版本
                if let Some(version) = ctx.get_latest_installed(runtime_name).await? {
                    return Ok(ResolvedVersion::LatestInstalled(version));
                }
                // fallback 到远端
                let version = ctx.fetch_latest_remote(runtime_name).await?;
                Ok(ResolvedVersion::LatestRemote(version))
            }
            LatestBehavior::RemoteLatest => {
                let version = ctx.fetch_latest_remote(runtime_name).await?;
                Ok(ResolvedVersion::LatestRemote(version))
            }
            LatestBehavior::Locked => {
                ctx.get_locked_version(runtime_name)
                    .ok_or_else(|| ResolverError::NoLockedVersion(runtime_name.to_string()))
            }
        }
    }
}
```

##### Stage 2: Ensure (安装阶段)

```rust
// crates/vx-resolver/src/executor/stages/ensure.rs

pub struct EnsureStage {
    installer: Arc<InstallationManager>,
}

#[async_trait]
impl Stage<ExecutionPlan, ExecutionPlan> for EnsureStage {
    type Error = InstallError;
    
    async fn execute(
        &self,
        mut plan: ExecutionPlan,
        ctx: &ExecutorContext,
    ) -> Result<ExecutionPlan, Self::Error> {
        // 1. 按拓扑顺序安装依赖
        for dep in &mut plan.dependencies {
            if dep.status == InstallStatus::NeedsInstall {
                self.install_runtime(dep, ctx).await?;
            }
        }
        
        // 2. 安装主运行时
        if plan.primary.status == InstallStatus::NeedsInstall {
            self.install_runtime(&mut plan.primary, ctx).await?;
        }
        
        // 3. 安装注入运行时
        for injected in &mut plan.injected {
            if injected.status == InstallStatus::NeedsInstall {
                self.install_runtime(injected, ctx).await?;
            }
        }
        
        Ok(plan)
    }
}
```

##### Stage 3: Prepare (环境准备阶段)

```rust
// crates/vx-resolver/src/executor/stages/prepare.rs

pub struct PrepareStage;

/// 执行上下文
#[derive(Debug)]
pub struct PreparedContext {
    /// 可执行文件路径
    pub executable: PathBuf,
    /// 工作目录
    pub working_dir: PathBuf,
    /// 环境变量
    pub env: HashMap<String, String>,
    /// 命令参数
    pub args: Vec<String>,
}

#[async_trait]
impl Stage<ExecutionPlan, PreparedContext> for PrepareStage {
    type Error = PrepareError;
    
    async fn execute(
        &self,
        plan: ExecutionPlan,
        ctx: &ExecutorContext,
    ) -> Result<PreparedContext, Self::Error> {
        let mut env = ctx.base_env.clone();
        
        // 1. 注入依赖的 PATH
        let path_entries: Vec<PathBuf> = plan.dependencies
            .iter()
            .chain(plan.injected.iter())
            .filter_map(|r| r.executable.as_ref().map(|e| e.parent().unwrap().to_path_buf()))
            .collect();
        
        self.prepend_path(&mut env, &path_entries)?;
        
        // 2. 处理 proxy runtime
        let executable = if let Some(proxy) = &plan.proxy {
            self.prepare_proxy(proxy, &plan.primary, &mut env)?
        } else {
            plan.primary.executable.clone()
                .ok_or(PrepareError::NoExecutable(plan.primary.name.clone()))?
        };
        
        Ok(PreparedContext {
            executable,
            working_dir: ctx.working_dir.clone(),
            env,
            args: plan.config.args.clone(),
        })
    }
}
```

##### Stage 4: Execute (执行阶段)

```rust
// crates/vx-resolver/src/executor/stages/execute.rs

pub struct ExecuteStage;

#[async_trait]
impl Stage<PreparedContext, i32> for ExecuteStage {
    type Error = ExecuteError;
    
    async fn execute(
        &self,
        prepared: PreparedContext,
        _ctx: &ExecutorContext,
    ) -> Result<i32, Self::Error> {
        let mut cmd = tokio::process::Command::new(&prepared.executable);
        cmd.args(&prepared.args)
            .current_dir(&prepared.working_dir)
            .envs(&prepared.env);
        
        let status = cmd.status().await
            .map_err(|e| ExecuteError::SpawnFailed(e))?;
        
        Ok(status.code().unwrap_or(-1))
    }
}
```

#### 3. ExecutionPipeline (编排器)

```rust
// crates/vx-resolver/src/executor/pipeline.rs

pub struct ExecutionPipeline {
    resolve: ResolveStage,
    ensure: EnsureStage,
    prepare: PrepareStage,
    execute: ExecuteStage,
}

impl ExecutionPipeline {
    pub async fn run(
        &self,
        request: ResolveRequest,
        ctx: &ExecutorContext,
    ) -> Result<i32, PipelineError> {
        // Stage 1: Resolve
        let plan = self.resolve.execute(request, ctx).await
            .map_err(PipelineError::Resolve)?;
        
        // 提前检查平台支持
        self.check_platform_support(&plan)?;
        
        // Stage 2: Ensure (if auto_install enabled)
        let plan = if ctx.config.auto_install {
            self.ensure.execute(plan, ctx).await
                .map_err(PipelineError::Install)?
        } else {
            self.verify_all_installed(&plan)?;
            plan
        };
        
        // Stage 3: Prepare
        let prepared = self.prepare.execute(plan, ctx).await
            .map_err(PipelineError::Prepare)?;
        
        // Stage 4: Execute
        self.execute.execute(prepared, ctx).await
            .map_err(PipelineError::Execute)
    }
    
    fn check_platform_support(&self, plan: &ExecutionPlan) -> Result<(), PipelineError> {
        let unsupported: Vec<_> = std::iter::once(&plan.primary)
            .chain(plan.dependencies.iter())
            .filter(|r| matches!(r.status, InstallStatus::PlatformUnsupported(_)))
            .collect();
        
        if !unsupported.is_empty() {
            return Err(PipelineError::PlatformUnsupported(
                unsupported.iter()
                    .map(|r| format!("{}: {}", r.name, match &r.status {
                        InstallStatus::PlatformUnsupported(reason) => reason.clone(),
                        _ => unreachable!(),
                    }))
                    .collect()
            ));
        }
        Ok(())
    }
}
```

#### 4. 结构化错误类型

```rust
// crates/vx-resolver/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ResolverError {
    #[error("Runtime not found: {0}")]
    RuntimeNotFound(String),
    
    #[error("Version not found: {runtime}@{version}")]
    VersionNotFound { runtime: String, version: String },
    
    #[error("No locked version for {0}, run 'vx lock' first")]
    NoLockedVersion(String),
    
    #[error("Dependency cycle detected: {}", .0.join(" -> "))]
    DependencyCycle(Vec<String>),
    
    #[error("Platform not supported: {runtime} requires {required}, current: {current}")]
    PlatformNotSupported {
        runtime: String,
        required: String,
        current: String,
    },
}

#[derive(Error, Debug)]
pub enum InstallError {
    #[error("Failed to install {runtime}@{version}: {reason}")]
    InstallFailed {
        runtime: String,
        version: String,
        reason: String,
    },
    
    #[error("Dependency {dep} required by {runtime} failed to install: {reason}")]
    DependencyFailed {
        runtime: String,
        dep: String,
        reason: String,
    },
    
    #[error("Download failed for {url}: {reason}")]
    DownloadFailed { url: String, reason: String },
}

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Resolution failed: {0}")]
    Resolve(#[from] ResolverError),
    
    #[error("Installation failed: {0}")]
    Install(#[from] InstallError),
    
    #[error("Environment preparation failed: {0}")]
    Prepare(#[from] PrepareError),
    
    #[error("Execution failed: {0}")]
    Execute(#[from] ExecuteError),
    
    #[error("Platform not supported:\n{}", .0.join("\n"))]
    PlatformUnsupported(Vec<String>),
}
```

#### 5. ManifestRegistry 拆分

```rust
// crates/vx-runtime/src/manifest/mod.rs

pub mod loader;
pub mod index;
pub mod builder;

// 职责 1: 清单加载
pub use loader::ManifestLoader;

// 职责 2: 元数据索引
pub use index::ManifestIndex;

// 职责 3: Provider 构建
pub use builder::ProviderBuilder;
```

```rust
// crates/vx-runtime/src/manifest/loader.rs

pub struct ManifestLoader {
    paths: Vec<PathBuf>,
}

impl ManifestLoader {
    pub fn load_all(&self) -> Result<Vec<ProviderManifest>, LoadError> {
        // 仅负责加载和解析清单文件
    }
}
```

```rust
// crates/vx-runtime/src/manifest/index.rs

/// 元数据索引 - 用于快速查询运行时信息
pub struct ManifestIndex {
    runtimes: HashMap<String, RuntimeMetadata>,
    aliases: HashMap<String, String>,
    providers: HashMap<String, ProviderMetadata>,
}

impl ManifestIndex {
    pub fn from_manifests(manifests: &[ProviderManifest]) -> Self {
        // 构建索引
    }
    
    pub fn get_runtime(&self, name: &str) -> Option<&RuntimeMetadata> {
        let name = self.resolve_alias(name);
        self.runtimes.get(name)
    }
    
    pub fn get_platform_constraint(&self, runtime: &str) -> Option<&PlatformConstraint> {
        // 合并 provider + runtime 级别约束（取交集）
        let runtime_meta = self.get_runtime(runtime)?;
        let provider_meta = self.providers.get(&runtime_meta.provider)?;
        
        match (&provider_meta.platform_constraint, &runtime_meta.platform_constraint) {
            (Some(p), Some(r)) => Some(p.intersect(r)),
            (Some(p), None) => Some(p),
            (None, Some(r)) => Some(r),
            (None, None) => None,
        }
    }
}
```

```rust
// crates/vx-runtime/src/manifest/builder.rs

/// Provider 构建结果
pub struct BuildResult {
    pub registry: ProviderRegistry,
    pub warnings: Vec<BuildWarning>,
    pub errors: Vec<BuildError>,
}

#[derive(Debug)]
pub struct BuildWarning {
    pub provider: String,
    pub message: String,
}

#[derive(Debug)]
pub struct BuildError {
    pub provider: String,
    pub runtime: Option<String>,
    pub reason: String,
}

pub struct ProviderBuilder {
    factories: HashMap<String, Box<dyn Fn(&ProviderManifest) -> Arc<dyn Provider>>>,
}

impl ProviderBuilder {
    pub fn build(&self, manifests: &[ProviderManifest]) -> BuildResult {
        let mut registry = ProviderRegistry::new();
        let mut warnings = Vec::new();
        let mut errors = Vec::new();
        
        for manifest in manifests {
            match self.factories.get(&manifest.provider.name) {
                Some(factory) => {
                    let provider = factory(manifest);
                    registry.register(provider);
                }
                None => {
                    // 记录错误而非仅 warn
                    errors.push(BuildError {
                        provider: manifest.provider.name.clone(),
                        runtime: None,
                        reason: "No factory registered".to_string(),
                    });
                }
            }
        }
        
        BuildResult { registry, warnings, errors }
    }
}
```

### 6. 版本解析 Fallback Chain（借鉴 Volta）

```rust
// crates/vx-resolver/src/version/chain.rs

/// 版本解析 Fallback Chain
/// 参考 Volta 的版本解析策略，实现从项目到系统的回退链
pub struct VersionFallbackChain {
    /// 解析器列表（按优先级排序）
    resolvers: Vec<Box<dyn VersionResolver>>,
}

/// 版本解析器 trait
#[async_trait]
pub trait VersionResolver: Send + Sync {
    /// 解析器名称（用于日志）
    fn name(&self) -> &str;
    
    /// 尝试解析版本
    async fn resolve(&self, runtime: &str, ctx: &ResolveContext) -> Option<ResolvedVersion>;
}

impl VersionFallbackChain {
    pub fn new() -> Self {
        Self {
            resolvers: vec![
                // 1. 显式指定（命令行参数）
                Box::new(ExplicitVersionResolver),
                // 2. 项目配置（vx.toml）
                Box::new(ProjectConfigResolver),
                // 3. 传统配置文件（.nvmrc, .node-version 等）
                Box::new(LegacyConfigResolver),
                // 4. 用户默认（~/.vx/defaults.toml）
                Box::new(UserDefaultResolver),
                // 5. 已安装的最新版本
                Box::new(InstalledLatestResolver),
                // 6. 远端最新稳定版
                Box::new(RemoteLatestResolver),
            ],
        }
    }
    
    pub async fn resolve(
        &self,
        runtime: &str,
        ctx: &ResolveContext,
    ) -> Result<(ResolvedVersion, &str), ResolverError> {
        for resolver in &self.resolvers {
            if let Some(version) = resolver.resolve(runtime, ctx).await {
                tracing::debug!(
                    "Version for {} resolved by {}: {:?}",
                    runtime, resolver.name(), version
                );
                return Ok((version, resolver.name()));
            }
        }
        Err(ResolverError::NoVersionFound(runtime.to_string()))
    }
}
```

#### 传统配置文件支持（借鉴 mise/fnm）

```rust
// crates/vx-resolver/src/version/legacy.rs

/// 传统配置文件解析器
/// 支持其他工具的配置文件格式，便于用户迁移
pub struct LegacyConfigResolver;

impl LegacyConfigResolver {
    /// 支持的传统配置文件（按优先级）
    const LEGACY_FILES: &'static [LegacyConfig] = &[
        // Node.js
        LegacyConfig { runtime: "node", file: ".nvmrc", parser: Parser::SingleLine },
        LegacyConfig { runtime: "node", file: ".node-version", parser: Parser::SingleLine },
        // Python
        LegacyConfig { runtime: "python", file: ".python-version", parser: Parser::SingleLine },
        // Ruby
        LegacyConfig { runtime: "ruby", file: ".ruby-version", parser: Parser::SingleLine },
        // Go
        LegacyConfig { runtime: "go", file: ".go-version", parser: Parser::SingleLine },
        // Rust
        LegacyConfig { runtime: "rust", file: "rust-toolchain.toml", parser: Parser::RustToolchain },
        LegacyConfig { runtime: "rust", file: "rust-toolchain", parser: Parser::SingleLine },
        // Java
        LegacyConfig { runtime: "java", file: ".java-version", parser: Parser::SingleLine },
        LegacyConfig { runtime: "java", file: ".sdkmanrc", parser: Parser::Sdkman },
        // Volta 兼容
        LegacyConfig { runtime: "node", file: "package.json", parser: Parser::PackageJsonVolta },
        // asdf 兼容
        LegacyConfig { runtime: "*", file: ".tool-versions", parser: Parser::ToolVersions },
    ];
}

#[async_trait]
impl VersionResolver for LegacyConfigResolver {
    fn name(&self) -> &str { "legacy-config" }
    
    async fn resolve(&self, runtime: &str, ctx: &ResolveContext) -> Option<ResolvedVersion> {
        // 从当前目录向上查找
        let mut dir = ctx.working_dir.clone();
        loop {
            for config in Self::LEGACY_FILES {
                if config.runtime != "*" && config.runtime != runtime {
                    continue;
                }
                let file_path = dir.join(config.file);
                if file_path.exists() {
                    if let Some(version) = config.parser.parse(&file_path, runtime).await {
                        return Some(ResolvedVersion::LegacyConfig {
                            version,
                            source: file_path,
                        });
                    }
                }
            }
            if !dir.pop() {
                break;
            }
        }
        None
    }
}
```

### 7. 自动版本切换（借鉴 fnm）

```rust
// crates/vx-shell/src/hooks.rs

/// Shell 集成钩子
/// 实现进入目录时自动切换版本（类似 fnm env --use-on-cd）
pub struct ShellHooks;

impl ShellHooks {
    /// 生成 shell 初始化脚本
    pub fn generate_init_script(shell: Shell, options: &HookOptions) -> String {
        match shell {
            Shell::Bash => Self::bash_init(options),
            Shell::Zsh => Self::zsh_init(options),
            Shell::Fish => Self::fish_init(options),
            Shell::PowerShell => Self::powershell_init(options),
        }
    }
    
    fn bash_init(options: &HookOptions) -> String {
        let mut script = String::from(r#"
# VX Shell Integration
export VX_SHELL="bash"

__vx_use() {
    local vx_output
    vx_output="$(vx env --shell bash 2>/dev/null)"
    if [ -n "$vx_output" ]; then
        eval "$vx_output"
    fi
}
"#);

        if options.use_on_cd {
            script.push_str(r#"
# Auto-switch on directory change (like fnm --use-on-cd)
__vx_cd() {
    \cd "$@" || return $?
    __vx_use
}
alias cd='__vx_cd'

# Trigger on shell start
__vx_use
"#);
        }
        
        script
    }
    
    fn zsh_init(options: &HookOptions) -> String {
        let mut script = String::from(r#"
# VX Shell Integration
export VX_SHELL="zsh"

__vx_use() {
    local vx_output
    vx_output="$(vx env --shell zsh 2>/dev/null)"
    if [[ -n "$vx_output" ]]; then
        eval "$vx_output"
    fi
}
"#);

        if options.use_on_cd {
            script.push_str(r#"
# Auto-switch on directory change
autoload -U add-zsh-hook
add-zsh-hook chpwd __vx_use

# Trigger on shell start
__vx_use
"#);
        }
        
        script
    }
}

#[derive(Debug, Clone)]
pub struct HookOptions {
    /// 进入目录时自动切换版本
    pub use_on_cd: bool,
    /// 显示版本切换信息
    pub log_level: LogLevel,
    /// 版本未找到时的行为
    pub version_not_found: VersionNotFoundBehavior,
}

#[derive(Debug, Clone)]
pub enum VersionNotFoundBehavior {
    /// 静默使用默认版本
    Silent,
    /// 显示警告
    Warn,
    /// 报错
    Error,
}
```

### 8. 版本过期检测（借鉴 proto）

```rust
// crates/vx-resolver/src/outdated.rs

/// 版本过期检测器
pub struct OutdatedChecker {
    version_fetcher: Arc<dyn VersionFetcher>,
    cache: Arc<VersionCache>,
}

#[derive(Debug)]
pub struct OutdatedReport {
    pub runtime: String,
    pub current: String,
    pub latest: String,
    pub latest_lts: Option<String>,
    pub security_update: bool,
}

impl OutdatedChecker {
    /// 检查单个运行时是否过期
    pub async fn check(&self, runtime: &str, current: &str) -> Result<Option<OutdatedReport>> {
        let versions = self.version_fetcher.fetch(runtime).await?;
        
        let latest = versions.iter()
            .filter(|v| !v.prerelease)
            .max_by(|a, b| a.semver().cmp(&b.semver()));
        
        let latest_lts = versions.iter()
            .filter(|v| v.lts.is_some())
            .max_by(|a, b| a.semver().cmp(&b.semver()));
        
        let current_semver = semver::Version::parse(current)?;
        
        if let Some(latest) = latest {
            if latest.semver() > &current_semver {
                // 检查是否有安全更新
                let security_update = versions.iter()
                    .filter(|v| v.semver() > &current_semver && v.semver() <= latest.semver())
                    .any(|v| v.security_release);
                
                return Ok(Some(OutdatedReport {
                    runtime: runtime.to_string(),
                    current: current.to_string(),
                    latest: latest.version.clone(),
                    latest_lts: latest_lts.map(|v| v.version.clone()),
                    security_update,
                }));
            }
        }
        
        Ok(None)
    }
    
    /// 检查所有已安装的运行时
    pub async fn check_all(&self) -> Result<Vec<OutdatedReport>> {
        let installed = self.get_all_installed().await?;
        let mut reports = Vec::new();
        
        // 并行检查
        let futures: Vec<_> = installed.iter()
            .map(|(runtime, version)| self.check(runtime, version))
            .collect();
        
        let results = futures::future::join_all(futures).await;
        
        for result in results {
            if let Ok(Some(report)) = result {
                reports.push(report);
            }
        }
        
        Ok(reports)
    }
}
```

### 9. 安装钩子（借鉴 proto）

```rust
// crates/vx-installer/src/hooks.rs

/// 安装钩子系统
/// 支持在安装前后执行自定义脚本
#[derive(Debug, Clone, Deserialize)]
pub struct InstallHooks {
    /// 安装前执行
    pub pre_install: Option<Vec<String>>,
    /// 安装后执行
    pub post_install: Option<Vec<String>>,
    /// 卸载前执行
    pub pre_uninstall: Option<Vec<String>>,
    /// 卸载后执行
    pub post_uninstall: Option<Vec<String>>,
}

impl InstallHooks {
    pub async fn run_pre_install(&self, ctx: &HookContext) -> Result<()> {
        if let Some(commands) = &self.pre_install {
            for cmd in commands {
                self.run_hook(cmd, ctx).await?;
            }
        }
        Ok(())
    }
    
    pub async fn run_post_install(&self, ctx: &HookContext) -> Result<()> {
        if let Some(commands) = &self.post_install {
            for cmd in commands {
                self.run_hook(cmd, ctx).await?;
            }
        }
        Ok(())
    }
    
    async fn run_hook(&self, cmd: &str, ctx: &HookContext) -> Result<()> {
        let expanded = self.expand_variables(cmd, ctx);
        
        tracing::info!("Running hook: {}", expanded);
        
        let status = tokio::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
            .args(if cfg!(windows) { vec!["/C", &expanded] } else { vec!["-c", &expanded] })
            .env("VX_RUNTIME", &ctx.runtime)
            .env("VX_VERSION", &ctx.version)
            .env("VX_INSTALL_DIR", ctx.install_dir.to_str().unwrap())
            .status()
            .await?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("Hook failed: {}", expanded));
        }
        
        Ok(())
    }
    
    fn expand_variables(&self, cmd: &str, ctx: &HookContext) -> String {
        cmd.replace("$RUNTIME", &ctx.runtime)
           .replace("$VERSION", &ctx.version)
           .replace("$INSTALL_DIR", ctx.install_dir.to_str().unwrap())
    }
}

#[derive(Debug)]
pub struct HookContext {
    pub runtime: String,
    pub version: String,
    pub install_dir: PathBuf,
}
```

### 配置示例（增强版）

```toml
# ~/.vx/config.toml

[resolver]
# "latest" 版本的默认行为
# - "installed" : 使用已安装的最新版本（默认，快速）
# - "remote"    : 检查远端最新版本（需要网络）
# - "locked"    : 使用锁文件版本（CI 推荐）
latest_behavior = "installed"

# 是否允许自动安装缺失的运行时
auto_install = true

# 安装超时（秒）
install_timeout = 300

# 传统配置文件支持（借鉴 mise/fnm）
[resolver.legacy]
# 是否读取传统配置文件（.nvmrc, .node-version 等）
enabled = true
# 支持的文件列表（可自定义禁用某些）
# files = [".nvmrc", ".node-version", ".tool-versions"]

[resolver.ci]
# CI 模式下的配置覆盖
latest_behavior = "locked"
auto_install = false

# Shell 集成配置（借鉴 fnm）
[shell]
# 进入目录时自动切换版本
use_on_cd = true
# 版本切换日志级别: "silent", "info", "verbose"
log_level = "info"
# 版本未找到时的行为: "silent", "warn", "error"
version_not_found = "warn"

# 安装钩子配置（借鉴 proto）
[hooks.node]
post_install = ["npm install -g pnpm", "npm install -g yarn"]

[hooks.python]
post_install = ["pip install pipx"]

[hooks.rust]
post_install = ["rustup component add clippy rustfmt"]

# 任务系统配置（借鉴 mise）
[tasks]
# 默认 shell
shell = "bash"
# 任务执行目录
dir = "."
# 环境变量
[tasks.env]
NODE_ENV = "development"
```

### 项目配置增强（借鉴 mise）

```toml
# vx.toml - 项目配置（增强版）

[project]
name = "my-awesome-project"
description = "A sample project using vx"

# 运行时版本固定
[tools]
node = "20"           # 使用 20.x.x 最新
python = "3.12"       # 使用 3.12.x 最新
go = "1.21.0"         # 精确版本
rust = "stable"       # 频道版本

# 环境变量（借鉴 mise）
[env]
NODE_ENV = "development"
DATABASE_URL = "postgres://localhost/dev"
# 从文件加载（支持 .env 格式）
_.file = [".env.local", ".env"]

# 任务定义（增强版，借鉴 mise）
[tasks]
# 简单任务
dev = "npm run dev"
test = "npm test"
build = "npm run build"

# 复杂任务（带依赖）
[tasks.ci]
depends = ["lint", "test", "build"]
run = "echo CI passed!"

[tasks.lint]
run = "npm run lint"
# 条件执行
sources = ["src/**/*.ts", "src/**/*.tsx"]

# 带环境变量的任务
[tasks.deploy]
run = "npm run deploy"
env = { NODE_ENV = "production" }

# 安装钩子（项目级）
[hooks]
pre_install = "echo Installing tools for $PROJECT..."
post_install = "npm install"
```

## 向后兼容性

### 兼容策略

1. **默认行为不变**: `latest_behavior = "installed"` 保持当前行为
2. **渐进式迁移**: 用户可选择启用新策略
3. **配置优先**: 命令行参数 > 项目配置 > 用户配置 > 默认值

### 迁移路径

```bash
# 1. 检查当前配置
vx config show resolver

# 2. 切换到推荐的 CI 模式
vx config set resolver.latest_behavior locked

# 3. 生成锁文件
vx lock

# 4. 验证
vx run node --version  # 使用锁定版本
```

### 弃用计划

| 版本 | 变更 |
|------|------|
| v0.6.0 | 引入新配置项，默认值保持兼容 |
| v0.7.0 | CLI 提示推荐使用新配置 |
| v0.8.0 | CI 环境默认 `latest_behavior = "locked"` |

## 实现计划

### Phase 1: 核心重构 (v0.6.0)

- [ ] 定义 `ExecutionPlan` 和相关类型
- [ ] 实现 `ResolveStage`，统一版本解析逻辑
- [ ] 实现 `EnsureStage`，分离安装逻辑
- [ ] 实现 `PrepareStage` 和 `ExecuteStage`
- [ ] 实现 `ExecutionPipeline` 编排器
- [ ] 添加 `VersionStrategy` 配置
- [ ] 迁移 `Executor::execute_with_with_deps` 到 pipeline
- [ ] 添加单元测试

### Phase 2: ManifestRegistry 拆分 (v0.6.0)

- [ ] 拆分 `ManifestLoader`
- [ ] 实现 `ManifestIndex`
- [ ] 实现 `ProviderBuilder`，返回 `BuildResult`
- [ ] 在 CLI 中处理 `BuildResult.errors`
- [ ] 添加 `vx info --warnings` 命令

### Phase 3: 错误处理改进 (v0.6.1)

- [ ] 定义 `ResolverError`/`InstallError`/`PipelineError`
- [ ] 迁移现有 `anyhow` 错误到类型化错误
- [ ] 改进 CLI 错误输出格式
- [ ] 添加依赖链上下文到错误消息

### Phase 4: Fallback Chain 与传统配置支持 (v0.6.1) 🆕

*借鉴 Volta/mise/fnm 的版本解析策略*

- [ ] 实现 `VersionFallbackChain` 版本解析回退链
- [ ] 实现 `LegacyConfigResolver` 传统配置文件解析
  - [ ] .nvmrc / .node-version (Node.js)
  - [ ] .python-version (Python)
  - [ ] .ruby-version (Ruby)
  - [ ] .go-version (Go)
  - [ ] rust-toolchain.toml (Rust)
  - [ ] .tool-versions (asdf 兼容)
  - [ ] package.json volta 字段 (Volta 兼容)
- [ ] 实现 `UserDefaultResolver` 用户默认版本
- [ ] 添加 `vx config set default.node 20` 命令
- [ ] 添加配置项 `[resolver.legacy]`

### Phase 5: Shell 集成与自动切换 (v0.7.0) 🆕

*借鉴 fnm 的极速启动和自动切换*

- [ ] 实现 `ShellHooks` shell 集成模块
- [ ] 实现 `--use-on-cd` 进入目录自动切换版本
- [ ] 支持 Bash/Zsh/Fish/PowerShell
- [ ] 优化 shim 启动性能，目标 < 5ms
- [ ] 实现 `vx env` 命令输出环境变量
- [ ] 添加 `vx shell init <shell>` 命令
- [ ] 添加配置项 `[shell]`

### Phase 6: 版本管理增强 (v0.7.0) 🆕

*借鉴 proto 的版本管理功能*

- [ ] 实现 `OutdatedChecker` 版本过期检测
- [ ] 添加 `vx outdated` 命令
- [ ] 实现安全更新检测
- [ ] 添加 `vx upgrade [runtime]` 批量升级命令
- [ ] 实现 `InstallHooks` 安装钩子系统
- [ ] 添加配置项 `[hooks.<runtime>]`

### Phase 7: 任务系统增强 (v0.7.0) 🆕

*借鉴 mise 的任务系统*

- [ ] 增强 vx.toml `[tasks]` 语法
- [ ] 支持任务依赖 `depends = ["lint", "test"]`
- [ ] 支持任务条件 `sources = ["src/**"]`
- [ ] 支持任务环境变量 `env = { NODE_ENV = "production" }`
- [ ] 支持环境变量文件加载 `_.file = [".env"]`
- [ ] 添加 `vx task <name>` 命令
- [ ] 添加 `vx tasks` 列出所有任务

### Phase 8: 高级特性 (v0.8.0)

- [ ] 实现 `LatestBehavior::Locked` 模式
- [ ] CI 环境自动检测
- [ ] `vx lock` 命令增强
- [ ] 性能优化（并行解析、缓存）
- [ ] Shim 性能基准测试
- [ ] 配置信任机制（借鉴 mise）

## 替代方案

### 方案 A: 仅修复 Bug，不重构

**优点**: 改动小，风险低
**缺点**: 技术债务累积，长期维护成本高
**结论**: 不推荐，问题会持续恶化

### 方案 B: 使用 Actor 模型

**优点**: 更好的并发控制
**缺点**: 复杂度高，学习曲线陡峭
**结论**: 过度设计，不适合当前规模

### 方案 C: Pipeline + 中间表示（本方案）

**优点**: 
- 清晰的阶段分离
- 各阶段可独立测试
- 中间表示便于调试和缓存
- 与主流工具设计一致

**缺点**: 需要较大改动

**结论**: 推荐，长期收益明显

## 参考资料

### 主流项目源码
- [Cargo Resolver](https://github.com/rust-lang/cargo/tree/master/src/cargo/core/resolver) - Pipeline 架构参考
- [uv Resolver](https://github.com/astral-sh/uv/tree/main/crates/uv-resolver) - 状态管理参考
- [rustup Toolchain](https://github.com/rust-lang/rustup/blob/master/src/toolchain.rs) - 版本策略参考
- [Volta](https://github.com/volta-cli/volta) - Shim 架构、项目版本固定、错误处理 ⭐
- [mise](https://github.com/jdx/mise) - 多配置文件支持、任务系统、环境变量管理 ⭐
- [proto](https://github.com/moonrepo/proto) - 版本过期检测、安装钩子、WASM 插件 ⭐
- [fnm](https://github.com/Schniz/fnm) - 极速启动、自动版本切换、Shell 集成 ⭐

### 设计文档
- [Volta Architecture](https://docs.volta.sh/advanced/architecture) - Volta 架构设计
- [mise Configuration](https://mise.jdx.dev/configuration.html) - mise 配置系统
- [proto Hooks](https://moonrepo.dev/docs/proto/config#hooks) - proto 钩子系统

### 相关 RFC
- [RFC 0028: Proxy Managed Runtimes](./0028-proxy-managed-runtimes.md) - Proxy 运行时设计

### 设计模式
- [Pipeline Pattern](https://docs.microsoft.com/en-us/azure/architecture/patterns/pipes-and-filters) - 管道过滤器模式
- [Chain of Responsibility](https://refactoring.guru/design-patterns/chain-of-responsibility) - 责任链模式（Fallback Chain）

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-05 | Draft | 初始草案 |
