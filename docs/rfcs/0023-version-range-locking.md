# RFC 0023: 版本范围锁定系统

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-01-26
> **目标版本**: v0.8.0

## 摘要

本 RFC 提出在 vx 中实现版本范围锁定系统（Version Range Locking），用于解决以下问题：

1. **工具版本兼容性约束** - 如 pnpm 9.x 只支持 Node.js 18+，vite 5.x 需要 Node.js 18+
2. **版本范围推荐** - Provider 可以定义工具的推荐版本范围
3. **版本冲突检测** - 当多个工具的依赖约束冲突时，提供清晰的错误信息和解决建议
4. **自动锁定策略** - 当用户使用 `latest` 时，自动应用安全的版本锁定策略

## 动机

### 当前问题分析

#### 问题 1: latest 版本的不确定性

```toml
# vx.toml
[tools]
vite = "latest"
pnpm = "latest"
```

```bash
# 今天运行
$ vx install
# 解析: vite = 5.4.0, pnpm = 9.0.0
# 依赖: Node.js >= 18+

# 明天运行（vite 6.0 发布）
$ vx install
# 解析: vite = 6.0.0, pnpm = 9.1.0
# 依赖: Node.js >= 20+ (假设 vite 6.0 提升了要求)
# 问题: 依赖要求突然变化，可能导致兼容性问题
```

#### 问题 2: 版本约束信息分散

当前 Provider 定义了 `constraints`，但这些信息仅在安装时使用，用户无法：
- 提前知道工具版本对依赖的要求
- 在 `vx.toml` 中明确表达版本范围意图
- 让 vx 自动选择兼容的版本组合

#### 问题 3: 缺乏版本范围推荐

用户使用 `latest` 时，可能意外升级到不兼容的主版本。例如：
- pnpm 8.x → 9.x 可能有破坏性变化
- vite 4.x → 5.x 可能需要配置迁移

### 行业对比

| 工具 | 版本范围语法 | 锁定策略 | 冲突检测 | 推荐版本 |
|------|-------------|---------|---------|---------|
| **npm/yarn** | ✅ semver | ✅ package-lock.json | ✅ | ⚠️ peer deps |
| **cargo** | ✅ semver | ✅ Cargo.lock | ✅ | ❌ |
| **uv/pip** | ✅ PEP 440 | ✅ uv.lock | ✅ | ❌ |
| **mise** | ⚠️ 基础 | ❌ | ❌ | ❌ |
| **vx (当前)** | ✅ semver | ✅ vx.lock | ⚠️ 部分 | ❌ |

## 设计方案

### 1. vx.toml 版本范围语法增强

#### 1.1 版本范围语法

支持完整的 semver 范围语法：

```toml
# vx.toml
[tools]
# 精确版本
rust = "1.83.0"

# 主版本锁定（推荐）
vite = "^5.0"           # >=5.0.0, <6.0.0
pnpm = "^8.0"           # >=8.0.0, <9.0.0

# 次版本锁定
node = "~20.0"          # >=20.0.0, <20.1.0

# 范围约束
python = ">=3.9,<3.13"

# 主版本通配
go = "1.x"              # >=1.0.0, <2.0.0

# 最新版（带默认锁定策略）
uv = "latest"           # 自动锁定主版本
```

| 语法 | 含义 | 示例展开 |
|------|------|----------|
| `1.2.3` | 精确版本 | `=1.2.3` |
| `^1.2.3` | 主版本锁定 | `>=1.2.3, <2.0.0` |
| `~1.2.3` | 次版本锁定 | `>=1.2.3, <1.3.0` |
| `>=1.2.3` | 最小版本 | `>=1.2.3` |
| `>1.2.3` | 大于 | `>1.2.3` |
| `<2.0.0` | 小于 | `<2.0.0` |
| `<=2.0.0` | 小于等于 | `<=2.0.0` |
| `1.x` | 主版本通配 | `>=1.0.0, <2.0.0` |
| `1.2.x` | 次版本通配 | `>=1.2.0, <1.3.0` |
| `*` 或 `latest` | 最新版本 | 无约束（但有锁定策略） |

#### 1.2 对象语法（高级配置）

```toml
[tools]
# 使用对象语法指定锁定策略
python = { version = "latest", pinning = "major" }
node = { version = "^20", pinning = "minor" }

# 锁定策略选项
# - "major": 锁定主版本（默认）
# - "minor": 锁定次版本
# - "patch": 锁定补丁版本
# - "exact": 精确锁定
# - "none": 无锁定（危险，仅用于开发）
```

### 2. Provider 版本范围配置

#### 2.1 provider.toml 扩展

```toml
# crates/vx-providers/vite/provider.toml

[provider]
name = "vite"
# ...

[[runtimes]]
name = "vite"
# ...

# 新增：版本范围配置
[runtimes.version_ranges]
# "latest" 的默认行为
default = "^5.0"

# 允许的最大版本（突破需要显式指定）
maximum = "<6.0"

# 不推荐使用的版本
deprecated = ["<4.0"]

# 有已知问题的版本
warning = ["5.0.0", "5.1.0"]

# 推荐的稳定版本系列
recommended = "^5.4"
```

#### 2.2 版本范围定义字段

| 字段 | 类型 | 说明 |
|------|------|------|
| `default` | `String` | 当用户指定 `latest` 时，自动应用的版本范围 |
| `maximum` | `String` | 允许的最大版本约束 |
| `minimum` | `String` | 允许的最小版本约束 |
| `deprecated` | `[String]` | 不推荐使用的版本范围列表 |
| `warning` | `[String]` | 有已知问题的版本列表 |
| `recommended` | `String` | 推荐的稳定版本范围 |

### 3. 版本约束验证和冲突检测

#### 3.1 约束验证流程

```
┌─────────────────────────────────────────────────────────────┐
│                     版本解析流程                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  用户请求                Provider 约束                        │
│  vx.toml                 provider.toml                       │
│     │                        │                               │
│     ▼                        ▼                               │
│  ┌─────────┐          ┌─────────────┐                       │
│  │ 解析    │          │ 版本范围    │                       │
│  │ 版本请求│          │ 约束        │                       │
│  └────┬────┘          └──────┬──────┘                       │
│       │                      │                               │
│       └──────────┬───────────┘                               │
│                  ▼                                           │
│          ┌─────────────┐                                    │
│          │ 约束合并    │                                    │
│          └──────┬──────┘                                    │
│                 │                                            │
│                 ▼                                            │
│          ┌─────────────┐                                    │
│          │ 冲突检测    │                                    │
│          └──────┬──────┘                                    │
│                 │                                            │
│         ┌──────┴──────┐                                     │
│         ▼             ▼                                     │
│    ┌─────────┐   ┌─────────┐                               │
│    │ 无冲突  │   │ 有冲突  │                               │
│    └────┬────┘   └────┬────┘                               │
│         │             │                                     │
│         ▼             ▼                                     │
│    ┌─────────┐   ┌─────────┐                               │
│    │ 版本选择│   │ 报告错误│                               │
│    └─────────┘   │ +建议   │                               │
│                  └─────────┘                               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

#### 3.2 冲突检测示例

```bash
$ vx install
⚠ Version conflict detected:

  Tools:
    - vite ^5.0 requires Node.js >= 18
    - legacy-tool ^1.0 requires Node.js <= 16

  Conflict:
    Node.js version must satisfy both >=18 AND <=16
    This is impossible to satisfy.

  Suggestions:
    1. Update legacy-tool: vx install legacy-tool ^2.0 (supports Node.js 18+)
    2. Downgrade vite: vx install vite ^4.0 (supports Node.js 14+)
    3. Remove one of the conflicting tools

  Run with --force to skip version validation (not recommended)
```

### 4. vx.lock 增强

#### 4.1 增强的锁文件格式

```toml
# vx.lock
version = 2

[metadata]
generated_at = "2026-01-26T03:09:05Z"
vx_version = "0.8.0"
platform = "x86_64-pc-windows-msvc"

[tools.vite]
version = "5.4.0"
# 新增字段
original_range = "^5.0"         # 原始版本范围
resolved_from = "vx.toml"       # 来源
pinning = "major"               # 锁定策略
is_latest_in_range = true       # 是否为范围内最新
constraint_source = "provider"  # 约束来源: "user" | "provider" | "merged"

[tools.pnpm]
version = "9.0.0"
original_range = "latest"
resolved_from = "vx.toml"
pinning = "major"
is_latest_in_range = true
# 记录 provider 应用的默认范围
applied_default = "^9.0"

[tools.node]
version = "20.18.0"
original_range = "^20"
resolved_from = "vx.toml"
pinning = "minor"
is_latest_in_range = true

# 依赖关系
[dependencies]
npm = ["node"]
npx = ["node"]
pnpm = ["node"]
vite = ["node"]
```

### 5. CLI 命令增强

#### 5.1 新增命令

```bash
# 检查版本约束
$ vx check
✓ All version constraints satisfied
  - vite ^5.0 → 5.4.0 (requires node >=18)
  - pnpm ^9.0 → 9.0.0 (requires node >=18)
  - node ^20 → 20.18.0

# 检查可用更新（在范围内）
$ vx outdated
Tool    Current  Latest   Latest in Range  Range
----    -------  ------   ---------------  -----
vite    5.4.0    6.0.0    5.5.0            ^5.0
pnpm    9.0.0    9.1.0    9.1.0            ^9.0
node    20.18.0  22.0.0   20.20.0          ^20

# 更新到范围内最新
$ vx update
✓ Updated vite 5.4.0 → 5.5.0 (within ^5.0)
✓ Updated pnpm 9.0.0 → 9.1.0 (within ^9.0)
✓ Updated node 20.18.0 → 20.20.0 (within ^20)

# 更新到最新（可能突破范围）
$ vx update --latest
⚠ vite 6.0.0 is outside your range ^5.0
  This is a major version upgrade and may have breaking changes.
  Update vx.toml to allow: vite = "^6.0"
  Continue? [y/N]

# 检查依赖兼容性
$ vx compat vite@6.0
Checking compatibility for vite 6.0.0...

Dependencies:
  - node >=20 (your range: ^20 ✓)

Breaking changes from 5.x:
  - Config format changed
  - Some plugins may need updates

Compatible: Yes (with minor adjustments)
```

#### 5.2 现有命令增强

```bash
# vx install 增加版本范围验证
$ vx install
Resolving versions...
  vite: ^5.0 → 5.4.0
  pnpm: latest → ^9.0 → 9.0.0 (provider default applied)
  node: ^20 → 20.18.0

Installing tools...
✓ Installed vite 5.4.0
✓ Installed pnpm 9.0.0
✓ Installed node 20.18.0

# vx sync 增加范围检查
$ vx sync
Syncing from vx.lock...
⚠ vite 5.4.0 is no longer the latest in range ^5.0
  Latest in range: 5.5.0
  Run 'vx update' to update within range
✓ Synced vite 5.4.0
✓ Synced pnpm 9.0.0
✓ Synced node 20.18.0
```

### 6. 核心类型定义

```rust
// crates/vx-resolver/src/version/range.rs

use semver::VersionReq;

/// 版本范围配置（来自 provider.toml）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRangeConfig {
    /// "latest" 的默认版本范围
    pub default: Option<String>,
    /// 允许的最大版本
    pub maximum: Option<String>,
    /// 允许的最小版本
    pub minimum: Option<String>,
    /// 不推荐的版本列表
    #[serde(default)]
    pub deprecated: Vec<String>,
    /// 有警告的版本列表
    #[serde(default)]
    pub warning: Vec<String>,
    /// 推荐版本范围
    pub recommended: Option<String>,
}

/// 锁定策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PinningStrategy {
    /// 锁定主版本（默认）
    Major,
    /// 锁定次版本
    Minor,
    /// 锁定补丁版本
    Patch,
    /// 精确锁定
    Exact,
    /// 无锁定
    None,
}

impl Default for PinningStrategy {
    fn default() -> Self {
        Self::Major
    }
}

/// 版本范围解析器
pub struct VersionRangeResolver {
    strategies: HashMap<Ecosystem, Box<dyn VersionStrategy>>,
}

impl VersionRangeResolver {
    /// 应用 Provider 的版本范围配置
    pub fn apply_provider_config(
        &self,
        request: &VersionRequest,
        config: &VersionRangeConfig,
    ) -> Result<VersionRequest> {
        // 如果是 latest，应用 provider 的默认范围
        if request.is_latest() {
            if let Some(default) = &config.default {
                return Ok(VersionRequest::parse(default));
            }
        }
        Ok(request.clone())
    }

    /// 检查版本是否在允许范围内
    pub fn check_bounds(
        &self,
        version: &Version,
        config: &VersionRangeConfig,
    ) -> Result<BoundsCheckResult> {
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // 检查最大版本
        if let Some(max) = &config.maximum {
            let max_req = VersionReq::parse(max)?;
            if !max_req.matches(version) {
                errors.push(format!(
                    "Version {} exceeds maximum allowed {}",
                    version, max
                ));
            }
        }

        // 检查最小版本
        if let Some(min) = &config.minimum {
            let min_req = VersionReq::parse(min)?;
            if !min_req.matches(version) {
                errors.push(format!(
                    "Version {} is below minimum required {}",
                    version, min
                ));
            }
        }

        // 检查不推荐版本
        for dep in &config.deprecated {
            let dep_req = VersionReq::parse(dep)?;
            if dep_req.matches(version) {
                warnings.push(format!(
                    "Version {} is deprecated (matches {})",
                    version, dep
                ));
            }
        }

        // 检查警告版本
        for warn in &config.warning {
            let warn_req = VersionReq::parse(warn)?;
            if warn_req.matches(version) {
                warnings.push(format!(
                    "Version {} has known issues (matches {})",
                    version, warn
                ));
            }
        }

        Ok(BoundsCheckResult { warnings, errors })
    }
}

/// 边界检查结果
#[derive(Debug)]
pub struct BoundsCheckResult {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl BoundsCheckResult {
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}
```

### 7. 冲突检测实现

```rust
// crates/vx-resolver/src/conflict.rs

/// 冲突检测器
pub struct ConflictDetector {
    runtime_map: RuntimeMap,
}

impl ConflictDetector {
    /// 检测工具集合的依赖冲突
    pub fn detect_conflicts(
        &self,
        tools: &[(String, VersionRequest)],
    ) -> Result<Vec<Conflict>> {
        let mut conflicts = Vec::new();
        let mut requirements: HashMap<String, Vec<DependencyRequirement>> = HashMap::new();

        // 收集所有工具的依赖要求
        for (tool_name, version_req) in tools {
            let constraints = self.get_tool_constraints(tool_name, version_req)?;
            for constraint in constraints {
                requirements
                    .entry(constraint.runtime.clone())
                    .or_default()
                    .push(DependencyRequirement {
                        from_tool: tool_name.clone(),
                        version_range: constraint.version.clone(),
                    });
            }
        }

        // 检测冲突
        for (runtime, reqs) in &requirements {
            if reqs.len() > 1 {
                // 尝试合并所有要求
                match self.try_merge_requirements(reqs) {
                    Ok(_) => {} // 可以合并，无冲突
                    Err(conflict_info) => {
                        conflicts.push(Conflict {
                            runtime: runtime.clone(),
                            requirements: reqs.clone(),
                            message: conflict_info,
                            suggestions: self.generate_suggestions(runtime, reqs),
                        });
                    }
                }
            }
        }

        Ok(conflicts)
    }

    /// 生成解决建议
    fn generate_suggestions(
        &self,
        runtime: &str,
        reqs: &[DependencyRequirement],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 建议 1: 更新工具版本
        for req in reqs {
            if let Some(newer) = self.find_compatible_version(&req.from_tool, runtime) {
                suggestions.push(format!(
                    "Update {}: vx install {} {}",
                    req.from_tool, req.from_tool, newer
                ));
            }
        }

        // 建议 2: 降级到兼容版本
        if let Some(common) = self.find_common_compatible_version(runtime, reqs) {
            suggestions.push(format!(
                "Use compatible {} version: vx install {} {}",
                runtime, runtime, common
            ));
        }

        suggestions
    }
}

/// 依赖冲突
#[derive(Debug)]
pub struct Conflict {
    pub runtime: String,
    pub requirements: Vec<DependencyRequirement>,
    pub message: String,
    pub suggestions: Vec<String>,
}

/// 依赖要求
#[derive(Debug, Clone)]
pub struct DependencyRequirement {
    pub from_tool: String,
    pub version_range: String,
}
```

## 向后兼容性

### 兼容策略

1. **版本字符串兼容**
   - `"latest"` 继续工作，但会应用 Provider 的默认范围
   - `"1.2.3"` 继续工作，精确版本
   - 新语法（`^`, `~`, `>=`）是可选的

2. **vx.lock 兼容**
   - 版本 1 的锁文件继续支持
   - 新字段是增量添加，不影响旧文件
   - 自动迁移到新格式

3. **Provider 兼容**
   - `version_ranges` 段是可选的
   - 没有定义版本范围的 Provider 使用默认行为

### 迁移路径

```bash
# 1. 升级 vx
vx self-update

# 2. 更新 vx.toml（可选但推荐）
# 旧配置
[tools]
vite = "latest"
pnpm = "latest"

# 新配置（推荐）
[tools]
vite = "^5.0"
pnpm = "^9.0"

# 3. 重新生成锁文件
vx lock

# 4. 验证约束
vx check
```

## 实现计划

### Phase 1: 版本范围配置 (v0.8.0)

- [x] Provider `version_ranges` 段解析
- [x] `VersionRangeConfig` 类型实现
- [x] `latest` → 默认范围转换
- [x] `vx check` 命令基础实现
- [x] 单元测试

### Phase 2: 冲突检测 (v0.8.1)

- [x] `ConflictDetector` 实现
- [x] 约束合并逻辑
- [x] 冲突报告和建议
- [ ] `vx install` 集成冲突检测
- [ ] 集成测试

### Phase 3: vx.lock 增强 (v0.8.2)

- [x] 锁文件格式 v2
- [x] 新字段：`original_range`, `pinning`, `applied_default`
- [x] 锁文件迁移
- [x] `vx sync` 范围检查

### Phase 4: CLI 增强 (v0.9.0)

- [ ] `vx outdated` 命令
- [ ] `vx update` 命令增强
- [ ] `vx compat` 命令
- [ ] 文档更新

## 替代方案

### 方案 A: 纯 Provider 约束

只依赖 Provider 的 `constraints` 段，不允许用户在 vx.toml 中指定范围。

**优点**: 简单，减少用户负担
**缺点**: 不够灵活，用户无法表达自己的约束意图

**结论**: 不采用，用户需要能够控制自己的版本范围

### 方案 B: 强制精确版本

强制所有版本都必须是精确版本，不支持范围。

**优点**: 最大确定性
**缺点**: 不切实际，用户需要频繁手动更新

**结论**: 不采用，与行业实践相悖

### 方案 C: 完全 semver

完全采用 npm/cargo 的 semver 语法和语义。

**优点**: 用户熟悉
**缺点**: 某些生态系统（如 Python PEP 440）有不同的版本语义

**结论**: 部分采用，支持 semver 语法但保留生态系统特定策略

## 参考资料

### 主流项目源码

- [npm semver](https://github.com/npm/node-semver) - semver 实现参考
- [Cargo resolver](https://github.com/rust-lang/cargo/blob/master/src/cargo/core/resolver/mod.rs) - Cargo 的版本解析
- [uv resolver](https://github.com/astral-sh/uv/tree/main/crates/uv-resolver) - uv 的版本解析

### 依赖库

- [`semver`](https://crates.io/crates/semver) - Rust semver 实现
- [`pep440-rs`](https://crates.io/crates/pep440_rs) - Python PEP 440 实现

### 相关文档

- [RFC 0008: 通用版本解析器设计](./0008-version-solver.md) - vx 版本解析器基础
- [RFC 0017: Declarative Runtime Map](./0017-declarative-runtime-map.md) - Runtime 依赖关系
- [Semantic Versioning 2.0.0](https://semver.org/)
- [PEP 440](https://peps.python.org/pep-0440/)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-26 | Draft | 初始草案 |
