# RFC 0008: 通用版本解析器设计

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2025-12-30
> **目标版本**: v0.7.0

## 摘要

本 RFC 提出在 `vx-resolver` crate 中实现一个通用的版本解析器（Version Solver），借鉴 [rez](https://rez.readthedocs.io/en/3.2.0/api/rez.solver.html) 的设计理念，支持：

1. **部分版本匹配** - `3.11` → `3.11.11`
2. **版本约束表达式** - `>=3.9,<3.12`
3. **锁文件机制** - `vx.lock` 确保可复现的环境
4. **多生态系统支持** - 不同语言的版本语义差异

## 动机

### 当前问题分析

1. **版本解析不一致**
   ```bash
   # vx.toml 配置
   python = "3.11"

   # vx sync 尝试下载
   # 错误: cpython-3.11+20251217-... (缺少 patch 版本)
   # 期望: cpython-3.11.11+20251217-...
   ```

2. **缺乏锁文件机制**
   - `vx init` 生成的版本在 `vx setup`/`vx sync` 时可能解析为不同版本
   - 团队成员环境不一致
   - CI/CD 构建不可复现

3. **版本约束表达能力有限**
   - 只支持精确版本或 `latest`
   - 无法表达 `>=3.9,<3.12` 等约束

4. **各 Provider 重复实现版本解析**
   - `PythonRuntime.resolve_version()`
   - `NodeRuntime.resolve_version()`
   - 逻辑分散，难以维护

### 行业对比

| 工具 | 版本解析 | 锁文件 | 约束表达式 |
|------|---------|--------|-----------|
| **rez** | ✅ 完整 Solver | ✅ resolved_packages | ✅ 丰富语法 |
| **uv/pip** | ✅ PEP 440 | ✅ uv.lock | ✅ PEP 440 |
| **npm/yarn** | ✅ semver | ✅ package-lock.json | ✅ semver ranges |
| **cargo** | ✅ semver | ✅ Cargo.lock | ✅ semver |
| **mise** | ✅ 部分匹配 | ❌ | ⚠️ 有限 |
| **vx (当前)** | ⚠️ 基础 | ❌ | ❌ |

## 设计方案

### 1. 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                      vx-resolver                             │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌─────────────────┐    ┌─────────────────┐                 │
│  │  VersionSolver  │───▶│  SolverStatus   │                 │
│  └────────┬────────┘    └─────────────────┘                 │
│           │                                                  │
│           ▼                                                  │
│  ┌─────────────────┐    ┌─────────────────┐                 │
│  │ VersionRequest  │───▶│ ResolvedVersion │                 │
│  └────────┬────────┘    └─────────────────┘                 │
│           │                                                  │
│           ▼                                                  │
│  ┌─────────────────┐    ┌─────────────────┐                 │
│  │VersionStrategy  │───▶│   Ecosystem     │                 │
│  │  (per ecosystem)│    │  (Node/Python/  │                 │
│  └─────────────────┘    │   Go/Rust/...)  │                 │
│                         └─────────────────┘                 │
│                                                              │
│  ┌─────────────────┐    ┌─────────────────┐                 │
│  │   LockFile      │◀──▶│  vx.lock        │                 │
│  └─────────────────┘    └─────────────────┘                 │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### 2. 核心类型定义

```rust
// crates/vx-resolver/src/version/mod.rs

/// 版本请求 - 用户在 vx.toml 中指定的版本
#[derive(Debug, Clone)]
pub struct VersionRequest {
    /// 原始版本字符串 (如 "3.11", ">=3.9,<3.12", "latest")
    pub raw: String,
    /// 解析后的约束
    pub constraint: VersionConstraint,
}

/// 版本约束类型
#[derive(Debug, Clone)]
pub enum VersionConstraint {
    /// 精确版本: "3.11.11"
    Exact(Version),
    /// 部分版本: "3.11" (匹配 3.11.x 最新)
    Partial { major: u32, minor: u32 },
    /// 主版本: "3" (匹配 3.x.x 最新)
    Major(u32),
    /// 最新稳定版
    Latest,
    /// 最新预发布版
    LatestPrerelease,
    /// 范围约束: ">=3.9,<3.12"
    Range(Vec<RangeConstraint>),
    /// 通配符: "3.11.*"
    Wildcard { major: u32, minor: u32 },
}

/// 范围约束
#[derive(Debug, Clone)]
pub struct RangeConstraint {
    pub op: RangeOp,
    pub version: Version,
}

#[derive(Debug, Clone, Copy)]
pub enum RangeOp {
    Eq,      // =
    Ne,      // !=
    Gt,      // >
    Ge,      // >=
    Lt,      // <
    Le,      // <=
    Tilde,   // ~= (compatible release)
    Caret,   // ^  (compatible with)
}

/// 解析后的版本
#[derive(Debug, Clone)]
pub struct ResolvedVersion {
    /// 完整版本号
    pub version: Version,
    /// 来源 (GitHub release, npm registry, etc.)
    pub source: String,
    /// 额外元数据 (如 python-build-standalone 的 release_date)
    pub metadata: HashMap<String, String>,
}

/// 求解器状态 (借鉴 rez)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverStatus {
    /// 尚未开始
    Pending,
    /// 解析成功
    Solved,
    /// 无法满足约束
    Failed,
    /// 存在循环依赖
    Cyclic,
    /// 解析中
    InProgress,
}
```

### 3. 版本策略 trait (支持多生态系统)

```rust
// crates/vx-resolver/src/version/strategy.rs

/// 版本解析策略 - 每个生态系统可以有不同的实现
#[async_trait]
pub trait VersionStrategy: Send + Sync {
    /// 生态系统名称
    fn ecosystem(&self) -> Ecosystem;

    /// 解析版本请求字符串
    fn parse_request(&self, input: &str) -> Result<VersionRequest>;

    /// 检查版本是否满足约束
    fn satisfies(&self, version: &Version, constraint: &VersionConstraint) -> bool;

    /// 从可用版本列表中选择最佳匹配
    fn select_best_match(
        &self,
        constraint: &VersionConstraint,
        available: &[VersionInfo],
    ) -> Option<VersionInfo>;

    /// 比较两个版本
    fn compare(&self, a: &Version, b: &Version) -> std::cmp::Ordering;

    /// 规范化版本字符串
    fn normalize(&self, version: &str) -> String;
}

/// 默认 semver 策略 (适用于大多数工具)
pub struct SemverStrategy;

/// Python PEP 440 策略
pub struct Pep440Strategy;

/// Node.js semver 策略 (与标准 semver 略有不同)
pub struct NodeSemverStrategy;

/// Go 版本策略 (go1.22 格式)
pub struct GoVersionStrategy;

/// 日期版本策略 (如 python-build-standalone: 20251217)
pub struct DateVersionStrategy;
```

### 4. 版本求解器

```rust
// crates/vx-resolver/src/version/solver.rs

/// 版本求解器
pub struct VersionSolver {
    /// 版本策略注册表
    strategies: HashMap<Ecosystem, Box<dyn VersionStrategy>>,
    /// 锁文件
    lockfile: Option<LockFile>,
    /// 配置
    config: SolverConfig,
}

impl VersionSolver {
    /// 创建新的求解器
    pub fn new() -> Self;

    /// 注册版本策略
    pub fn register_strategy(&mut self, strategy: Box<dyn VersionStrategy>);

    /// 加载锁文件
    pub fn with_lockfile(self, lockfile: LockFile) -> Self;

    /// 解析单个工具的版本
    pub async fn resolve(
        &self,
        tool: &str,
        request: &VersionRequest,
        available: &[VersionInfo],
        ecosystem: Ecosystem,
    ) -> Result<ResolvedVersion>;

    /// 批量解析多个工具
    pub async fn resolve_all(
        &self,
        requests: &[(String, VersionRequest, Ecosystem)],
        version_fetcher: &dyn VersionFetcher,
    ) -> Result<SolverResult>;

    /// 生成锁文件
    pub fn generate_lockfile(&self, resolved: &SolverResult) -> LockFile;
}

/// 求解结果
#[derive(Debug)]
pub struct SolverResult {
    pub status: SolverStatus,
    pub resolved: HashMap<String, ResolvedVersion>,
    pub errors: Vec<SolverError>,
}

/// 求解错误
#[derive(Debug)]
pub enum SolverError {
    /// 无可用版本
    NoVersionFound { tool: String, constraint: String },
    /// 约束冲突
    ConflictingConstraints { tool: String, constraints: Vec<String> },
    /// 网络错误
    FetchError { tool: String, error: String },
}
```

### 5. 锁文件格式 (vx.lock)

```toml
# vx.lock - 自动生成，请勿手动编辑
# Generated by vx 0.7.0

[metadata]
generated_at = "2025-12-30T10:00:00Z"
vx_version = "0.7.0"
platform = "x86_64-pc-windows-msvc"

# 锁定的工具版本
[tools.python]
version = "3.11.11"
source = "python-build-standalone"
release_date = "20251217"
checksum = "sha256:abc123..."
resolved_from = "3.11"  # 原始请求

[tools.node]
version = "20.18.0"
source = "nodejs.org"
checksum = "sha256:def456..."
resolved_from = "20"

[tools.uv]
version = "0.5.14"
source = "github:astral-sh/uv"
checksum = "sha256:ghi789..."
resolved_from = "latest"

# 依赖关系图
[dependencies]
npm = ["node"]
npx = ["node"]
uvx = ["uv"]
```

### 6. 配置增强 (vx.toml)

```toml
# vx.toml v2 - 版本约束增强

[tools]
# 部分版本 - 匹配最新 patch
python = "3.11"        # → 3.11.11
node = "20"            # → 20.18.0

# 精确版本
go = "1.22.5"

# 最新版
uv = "latest"

# 范围约束 (新增)
rust = ">=1.75,<1.80"
java = ">=17,<22"

# 通配符 (新增)
deno = "2.*"

# 兼容版本 (新增)
bun = "^1.0"           # >=1.0.0, <2.0.0
pnpm = "~9.0"          # >=9.0.0, <9.1.0

[settings]
# 版本解析策略
version_strategy = "lockfile-first"  # lockfile-first | latest | exact

# 是否自动更新锁文件
auto_update_lock = true

# 允许预发布版本
allow_prerelease = false
```

### 7. CLI 命令增强

```bash
# 解析并锁定版本
vx lock                    # 生成/更新 vx.lock
vx lock --update           # 更新所有工具到最新兼容版本
vx lock --update python    # 只更新 python

# 同步环境 (使用锁文件)
vx sync                    # 按 vx.lock 安装
vx sync --ignore-lock      # 忽略锁文件，重新解析

# 版本信息
vx version python          # 显示解析后的版本
vx version --all           # 显示所有工具版本

# 检查版本约束
vx check                   # 验证 vx.toml 和 vx.lock 一致性
```

### 8. Provider 集成

```rust
// Provider 不再需要实现 resolve_version
// 只需提供 fetch_versions 返回可用版本列表

#[async_trait]
impl Runtime for PythonRuntime {
    // 移除: async fn resolve_version(...)

    // 保留: 返回可用版本列表
    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        // ...
    }

    // 新增: 指定版本策略
    fn version_strategy(&self) -> Ecosystem {
        Ecosystem::Python  // 使用 Pep440Strategy
    }
}
```

## 向后兼容性

### 兼容策略

1. **无锁文件时的行为**
   - 如果没有 `vx.lock`，行为与当前一致
   - 首次运行 `vx sync` 时自动生成 `vx.lock`

2. **版本字符串兼容**
   - `"3.11"` - 继续支持，解析为部分版本
   - `"latest"` - 继续支持
   - `"3.11.11"` - 继续支持，精确版本

3. **渐进增强**
   - 新的约束语法 (`>=`, `<`, `^`, `~`) 是可选的
   - 不使用新语法的项目不受影响

### 迁移路径

```bash
# 1. 升级 vx
vx self-update

# 2. 生成锁文件 (可选但推荐)
vx lock

# 3. 提交锁文件到版本控制
git add vx.lock
git commit -m "chore: add vx.lock for reproducible builds"
```

## 实现计划

### Phase 1: 核心版本解析 (v0.7.0)

- [ ] `VersionRequest` 和 `VersionConstraint` 类型
- [ ] `SemverStrategy` 默认实现
- [ ] `VersionSolver` 基础实现
- [ ] 部分版本匹配 (`3.11` → `3.11.11`)
- [ ] 更新 `PythonRuntime` 使用新解析器
- [ ] 单元测试

### Phase 2: 锁文件机制 (v0.7.1)

- [ ] `LockFile` 类型和解析
- [ ] `vx lock` 命令
- [ ] `vx sync` 集成锁文件
- [ ] `vx check` 一致性检查
- [ ] 锁文件自动更新

### Phase 3: 多生态系统策略 (v0.8.0)

- [ ] `Pep440Strategy` (Python)
- [ ] `NodeSemverStrategy` (Node.js)
- [ ] `GoVersionStrategy` (Go)
- [ ] Provider 集成
- [ ] 文档更新

### Phase 4: 高级约束 (v0.9.0)

- [ ] 范围约束 (`>=`, `<`, `!=`)
- [ ] 兼容版本 (`^`, `~`)
- [ ] 通配符 (`*`)
- [ ] 约束冲突检测

## 参考资料

- [rez solver API](https://rez.readthedocs.io/en/3.2.0/api/rez.solver.html)
- [PEP 440 - Version Identification](https://peps.python.org/pep-0440/)
- [Semantic Versioning 2.0.0](https://semver.org/)
- [npm semver](https://docs.npmjs.com/cli/v6/using-npm/semver)
- [Cargo resolver](https://doc.rust-lang.org/cargo/reference/resolver.html)
- [uv resolver](https://docs.astral.sh/uv/concepts/resolution/)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-12-30 | Draft | 初始草案 |
