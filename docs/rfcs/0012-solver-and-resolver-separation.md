# RFC 0012: Solver 与 Resolver 职责分离

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2026-01-15
> **目标版本**: v0.9.0

## 摘要

本 RFC 提议明确区分 `vx-resolver` 和未来 `vx-solver` crate 的职责，避免功能重叠，并为未来引入 SAT-based 依赖解析器（如 pubgrub）做好架构准备。

## 当前状况分析

### vx-resolver 现有职责

`vx-resolver` 目前承担了多项职责：

```
vx-resolver/
├── config.rs           # 解析器配置
├── executor.rs         # 命令执行（核心：转发命令到 runtime）
├── resolver.rs         # 运行时状态检测（installed/system/missing）
├── runtime_map.rs      # 运行时映射（名称 → spec）
├── runtime_spec.rs     # 运行时规格定义
├── runtime_index.rs    # 运行时索引缓存
├── resolution_cache.rs # 解析结果缓存
└── version/
    ├── constraint.rs   # 版本约束表达式
    ├── lockfile.rs     # vx.lock 读写
    ├── request.rs      # 版本请求解析
    ├── resolved.rs     # 解析后的版本
    ├── solver.rs       # 单工具版本解析
    └── strategy.rs     # 不同生态的版本语义
```

### 问题识别

1. **职责模糊**
   - `Resolver` (resolver.rs) 检测运行时状态
   - `VersionSolver` (solver.rs) 解析版本约束
   - 两者名称相似，职责边界不清晰

2. **依赖解析不完整**
   - 当前 `VersionSolver` 只处理单工具版本解析
   - 依赖链解析在 `lock.rs` 中临时实现（刚修复的硬编码问题）
   - 没有真正的依赖图求解

3. **缺乏冲突解释**
   - 当版本约束冲突时，无法给出清晰的解释
   - 主流工具（uv, cargo）都有 human-readable 的冲突报告

## 设计方案

### 职责分离原则

```
┌──────────────────────────────────────────────────────────────────┐
│                          vx-resolver                              │
│  职责：运行时执行与环境解析                                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────┐     ┌─────────────────┐                     │
│  │    Executor     │     │    Resolver     │                     │
│  │  命令执行与转发   │     │  运行时状态检测   │                     │
│  └────────┬────────┘     └────────┬────────┘                     │
│           │                       │                               │
│           ▼                       ▼                               │
│  • 命令路由                  • 检测 vx-managed vs system           │
│  • PATH 管理                 • 依赖可用性检查                       │
│  • 进程转发                  • 可执行文件定位                       │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘

                              │ 调用
                              ▼

┌──────────────────────────────────────────────────────────────────┐
│                          vx-solver (新)                           │
│  职责：版本与依赖求解                                              │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌─────────────────┐     ┌─────────────────┐                     │
│  │  VersionSolver  │     │  DependencySolver│                    │
│  │  版本约束求解    │     │  依赖图求解       │                    │
│  └────────┬────────┘     └────────┬────────┘                     │
│           │                       │                               │
│           ▼                       ▼                               │
│  • 版本约束表达式解析         • 依赖传递解析                        │
│  • 部分版本匹配              • 拓扑排序                            │
│  • 生态系统版本语义          • 冲突检测与解释                       │
│                                                                   │
│  ┌─────────────────┐     ┌─────────────────┐                     │
│  │   LockFile      │     │ ConflictExplainer│                    │
│  │  锁文件管理      │     │  冲突解释器       │                    │
│  └─────────────────┘     └─────────────────┘                     │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### 模块迁移计划

| 当前位置 | 迁移到 | 说明 |
|---------|--------|------|
| `vx-resolver/version/solver.rs` | `vx-solver/version_solver.rs` | 版本约束求解 |
| `vx-resolver/version/lockfile.rs` | `vx-solver/lockfile.rs` | 锁文件管理 |
| `vx-resolver/version/constraint.rs` | `vx-solver/constraint.rs` | 约束表达式 |
| `vx-resolver/version/strategy.rs` | `vx-solver/strategy.rs` | 版本语义策略 |
| `vx-resolver/version/request.rs` | `vx-solver/request.rs` | 版本请求 |
| `vx-resolver/version/resolved.rs` | `vx-solver/resolved.rs` | 解析结果 |
| `vx-resolver/executor.rs` | **保留** | 命令执行 |
| `vx-resolver/resolver.rs` | **保留** | 运行时状态检测 |
| `vx-resolver/runtime_*.rs` | **保留** | 运行时映射 |

### vx-solver 详细设计

#### 目录结构

```
crates/vx-solver/
├── Cargo.toml
├── src/
│   ├── lib.rs              # 公开 API
│   ├── version/
│   │   ├── mod.rs
│   │   ├── constraint.rs   # 版本约束表达式
│   │   ├── request.rs      # 版本请求解析
│   │   ├── resolved.rs     # 解析结果
│   │   ├── solver.rs       # 版本求解器
│   │   └── strategy.rs     # 生态系统策略
│   ├── dependency/
│   │   ├── mod.rs
│   │   ├── graph.rs        # 依赖图构建
│   │   ├── solver.rs       # 依赖求解器
│   │   └── topo_sort.rs    # 拓扑排序
│   ├── lockfile/
│   │   ├── mod.rs
│   │   ├── format.rs       # 锁文件格式
│   │   ├── reader.rs       # 读取
│   │   ├── writer.rs       # 写入
│   │   └── diff.rs         # 差异比较
│   ├── conflict/
│   │   ├── mod.rs
│   │   ├── detector.rs     # 冲突检测
│   │   └── explainer.rs    # 人性化解释
│   └── pubgrub/            # 可选：SAT 求解器适配
│       ├── mod.rs
│       └── adapter.rs      # pubgrub 适配器
└── tests/
    ├── version_tests.rs
    ├── dependency_tests.rs
    └── lockfile_tests.rs
```

#### 核心类型定义

```rust
// crates/vx-solver/src/lib.rs

//! VX Solver - Version and Dependency Resolution
//!
//! This crate provides:
//! - Version constraint parsing and matching
//! - Dependency graph resolution
//! - Lock file management
//! - Conflict detection and explanation

pub mod conflict;
pub mod dependency;
pub mod lockfile;
pub mod version;

// 可选的 pubgrub 集成
#[cfg(feature = "pubgrub")]
pub mod pubgrub;

// 统一的求解请求
pub struct SolveRequest {
    /// 需要解析的工具列表
    pub tools: Vec<ToolRequest>,
    /// 现有的锁文件（用于增量解析）
    pub existing_lock: Option<LockFile>,
    /// 是否允许更新已锁定的版本
    pub allow_updates: bool,
}

pub struct ToolRequest {
    /// 工具名称
    pub name: String,
    /// 版本约束
    pub constraint: VersionConstraint,
    /// 来源（vx.toml 行号，用于错误报告）
    pub source: Option<SourceLocation>,
}

// 统一的求解结果
pub struct SolveResult {
    /// 求解状态
    pub status: SolveStatus,
    /// 解析出的版本
    pub resolved: HashMap<String, ResolvedTool>,
    /// 依赖关系图
    pub dependencies: DependencyGraph,
    /// 警告信息
    pub warnings: Vec<SolveWarning>,
}

pub enum SolveStatus {
    /// 求解成功
    Success,
    /// 求解失败（含冲突解释）
    Failed(ConflictExplanation),
    /// 部分成功（某些工具无法解析）
    Partial { failed: Vec<String> },
}
```

#### 依赖图求解

```rust
// crates/vx-solver/src/dependency/graph.rs

use std::collections::{HashMap, HashSet};

/// 依赖图
pub struct DependencyGraph {
    /// 节点：工具名 -> 工具信息
    nodes: HashMap<String, DependencyNode>,
    /// 边：工具名 -> 依赖列表
    edges: HashMap<String, Vec<DependencyEdge>>,
}

pub struct DependencyNode {
    pub name: String,
    pub resolved_version: Option<String>,
    pub constraint: VersionConstraint,
    pub is_direct: bool, // 是否在 vx.toml 中直接声明
}

pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub constraint: Option<VersionConstraint>,
}

impl DependencyGraph {
    /// 从 registry 构建依赖图
    pub fn build(
        tools: &[ToolRequest],
        registry: &ProviderRegistry,
    ) -> Result<Self, GraphBuildError> {
        let mut graph = Self::new();
        let mut visited = HashSet::new();
        
        for tool in tools {
            graph.add_tool_recursive(&tool.name, true, registry, &mut visited)?;
        }
        
        Ok(graph)
    }
    
    /// 递归添加工具及其依赖
    fn add_tool_recursive(
        &mut self,
        tool_name: &str,
        is_direct: bool,
        registry: &ProviderRegistry,
        visited: &mut HashSet<String>,
    ) -> Result<(), GraphBuildError> {
        if visited.contains(tool_name) {
            return Ok(()); // 避免循环
        }
        visited.insert(tool_name.to_string());
        
        // 添加节点
        self.nodes.insert(tool_name.to_string(), DependencyNode {
            name: tool_name.to_string(),
            resolved_version: None,
            constraint: VersionConstraint::Any,
            is_direct,
        });
        
        // 获取依赖
        if let Some(provider) = registry.get_provider(tool_name) {
            if let Some(runtime) = provider.get_runtime(tool_name) {
                for dep in runtime.dependencies() {
                    // 添加边
                    self.edges
                        .entry(tool_name.to_string())
                        .or_default()
                        .push(DependencyEdge {
                            from: tool_name.to_string(),
                            to: dep.name.clone(),
                            constraint: dep.min_version.as_ref().map(|v| 
                                VersionConstraint::GreaterOrEqual(v.clone())
                            ),
                        });
                    
                    // 递归添加依赖
                    self.add_tool_recursive(&dep.name, false, registry, visited)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// 拓扑排序，返回安装顺序
    pub fn topological_sort(&self) -> Result<Vec<String>, CycleError> {
        // Kahn's algorithm
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = Vec::new();
        
        // 计算入度
        for name in self.nodes.keys() {
            in_degree.insert(name.clone(), 0);
        }
        for edges in self.edges.values() {
            for edge in edges {
                *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            }
        }
        
        // 入度为 0 的节点入队
        for (name, &degree) in &in_degree {
            if degree == 0 {
                queue.push(name.clone());
            }
        }
        
        // BFS
        while let Some(node) = queue.pop() {
            result.push(node.clone());
            if let Some(edges) = self.edges.get(&node) {
                for edge in edges {
                    if let Some(degree) = in_degree.get_mut(&edge.to) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(edge.to.clone());
                        }
                    }
                }
            }
        }
        
        if result.len() != self.nodes.len() {
            return Err(CycleError::CycleDetected);
        }
        
        Ok(result)
    }
}
```

#### 冲突解释器

```rust
// crates/vx-solver/src/conflict/explainer.rs

/// 冲突解释
pub struct ConflictExplanation {
    /// 冲突类型
    pub kind: ConflictKind,
    /// 人性化消息
    pub message: String,
    /// 受影响的工具
    pub affected_tools: Vec<String>,
    /// 建议的解决方案
    pub suggestions: Vec<String>,
    /// 冲突链（用于调试）
    pub chain: Vec<ConflictStep>,
}

pub enum ConflictKind {
    /// 版本约束无法满足
    UnsatisfiableConstraint,
    /// 循环依赖
    CyclicDependency,
    /// 工具不存在
    UnknownTool,
    /// 平台不支持
    UnsupportedPlatform,
}

impl ConflictExplanation {
    /// 格式化输出（彩色终端）
    pub fn display(&self) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("✗ {}\n\n", self.message));
        
        if !self.chain.is_empty() {
            output.push_str("Conflict chain:\n");
            for (i, step) in self.chain.iter().enumerate() {
                output.push_str(&format!("  {}. {}\n", i + 1, step));
            }
            output.push('\n');
        }
        
        if !self.suggestions.is_empty() {
            output.push_str("Suggestions:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("  • {}\n", suggestion));
            }
        }
        
        output
    }
}

/// 类似 uv 的冲突解释风格
/// 
/// 例如:
/// ```
/// ✗ No solution found when resolving dependencies:
///   Because pre-commit requires uv>=0.5.0 and uv 0.5.0 is not available
///   for windows-x86, we can not satisfy pre-commit.
///
/// Suggestions:
///   • Try using a different platform
///   • Check if uv has a newer version that supports your platform
/// ```
pub fn explain_like_uv(conflict: &ConflictExplanation) -> String {
    // ...实现
}
```

### pubgrub 集成（可选功能）

```toml
# crates/vx-solver/Cargo.toml

[package]
name = "vx-solver"
version.workspace = true
edition.workspace = true

[dependencies]
# 核心依赖
semver = "1.0"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"

# 可选的 SAT 求解器
pubgrub = { version = "0.3", optional = true }

[features]
default = []
pubgrub = ["dep:pubgrub"]
```

```rust
// crates/vx-solver/src/pubgrub/adapter.rs

#[cfg(feature = "pubgrub")]
mod pubgrub_impl {
    use pubgrub::solver::{Dependencies, DependencyProvider};
    use pubgrub::version::SemanticVersion;
    
    /// 将 vx 的依赖模型适配到 pubgrub
    pub struct VxDependencyProvider {
        registry: Arc<ProviderRegistry>,
        versions_cache: HashMap<String, Vec<SemanticVersion>>,
    }
    
    impl DependencyProvider for VxDependencyProvider {
        type P = String;  // Package type = tool name
        type V = SemanticVersion;
        type VS = pubgrub::range::Range<SemanticVersion>;
        type M = String;
        
        fn choose_version(
            &self,
            package: &Self::P,
            range: &Self::VS,
        ) -> Result<Option<Self::V>, Self::M> {
            // 从 registry 获取可用版本，选择满足 range 的最高版本
            // ...
        }
        
        fn get_dependencies(
            &self,
            package: &Self::P,
            version: &Self::V,
        ) -> Result<Dependencies<Self::P, Self::VS, Self::M>, Self::M> {
            // 从 runtime.dependencies() 获取依赖
            // ...
        }
    }
}
```

### 迁移路径

#### Phase 1: 创建 vx-solver crate（无破坏性）

1. 创建新的 `crates/vx-solver/`
2. 将版本/锁文件相关代码复制（不是移动）到新 crate
3. `vx-resolver` 添加 `vx-solver` 作为依赖
4. 添加 re-export 保持兼容性

```rust
// crates/vx-resolver/src/lib.rs（Phase 1）

// 保持向后兼容的 re-export
pub use vx_solver::{
    LockFile, LockFileError, LockedTool, 
    VersionConstraint, VersionRequest, VersionSolver,
};
```

#### Phase 2: 迁移调用方

1. 更新 `vx-cli` 直接使用 `vx-solver`
2. 更新其他 crate 的导入路径
3. 运行测试确保兼容

#### Phase 3: 清理 vx-resolver

1. 移除 `vx-resolver/version/` 目录
2. 移除不再需要的 re-export
3. 更新文档

#### Phase 4: 增强 vx-solver（可选）

1. 实现依赖图求解
2. 添加冲突解释器
3. 可选：集成 pubgrub

### 命名约定

| 术语 | 定义 | crate |
|------|------|-------|
| **Resolver** | 运行时状态检测，判断是否已安装 | vx-resolver |
| **Executor** | 命令转发和执行 | vx-resolver |
| **Solver** | 版本/依赖约束求解 | vx-solver |
| **LockFile** | 锁文件管理 | vx-solver |

## 与现有 RFC 的关系

| RFC | 关系 |
|-----|------|
| RFC 0008 (Version Solver) | 本 RFC 是对 0008 的架构澄清，0008 中的版本解析功能将放入 vx-solver |

## 替代方案

### 方案 A: 不分离，继续在 vx-resolver 中扩展

**优点**:
- 无需迁移
- 更少的 crate

**缺点**:
- 职责不清晰
- 命名混淆（resolver vs solver）
- 难以独立测试

### 方案 B: 重命名 vx-resolver 为 vx-executor

**优点**:
- 更准确的命名

**缺点**:
- 大量破坏性改动
- 需要更新所有依赖

### 方案 C: 本 RFC 提议的分离方案（推荐）

**优点**:
- 清晰的职责边界
- 渐进式迁移
- 便于独立测试和复用
- 为 pubgrub 集成做好准备

**缺点**:
- 多一个 crate
- 需要迁移工作

## 实施计划

| 阶段 | 工作内容 | 预计耗时 |
|------|---------|---------|
| Phase 1 | 创建 vx-solver，复制代码，添加 re-export | 2 天 |
| Phase 2 | 迁移调用方，更新导入路径 | 1 天 |
| Phase 3 | 清理 vx-resolver，移除重复代码 | 0.5 天 |
| Phase 4 | 实现依赖图求解和冲突解释 | 3-5 天 |
| Phase 5 | （可选）集成 pubgrub | 3-5 天 |

## 未解决的问题

1. **是否使用 pubgrub?**
   - vx 的依赖场景相对简单（runtime 工具依赖，不是深层包依赖）
   - pubgrub 主要优势是复杂约束求解和冲突解释
   - 建议：Phase 4 先用简单实现，如果不够再引入

2. **锁文件格式是否需要变更?**
   - 当前格式是否足够表达依赖图？
   - 是否需要添加 `dependencies` 字段？

3. **vx-solver 是否需要异步?**
   - 版本获取需要网络请求
   - 建议：核心求解逻辑同步，版本获取通过 trait 抽象

## 参考资料

- [pubgrub (Astral-sh fork)](https://github.com/astral-sh/pubgrub) - PubGrub 算法 Rust 实现
- [resolvo](https://github.com/prefix-dev/resolvo) - CDCL SAT 求解器
- [RFC 0008: Version Solver](./0008-version-solver.md) - 版本解析器设计
- [uv resolver 设计](https://github.com/astral-sh/uv) - uv 的依赖解析器
