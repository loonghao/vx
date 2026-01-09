# RFC 0017: Declarative RuntimeMap

> **状态**: Completed ✅ (v0.11.0 cleanup done)
> **作者**: vx team
> **创建日期**: 2026-01-09
> **目标版本**: v0.10.0
> **依赖**: RFC 0012 (Provider Manifest), RFC 0013 (Manifest-Driven Registration)
> **实现日期**: 2026-01-09
> **完成日期**: 2026-01-09

## 摘要

将 `RuntimeMap` 从硬编码的 `register_builtin_runtimes()` 方法迁移到完全由 `provider.toml` 驱动的声明式设计。这消除了双重数据源问题，使每个 Provider 成为其运行时规格的唯一数据源。

## 动机

### 当前问题

目前 `RuntimeMap` 存在严重的设计问题：

**1. 双重数据源**

```rust
// vx-resolver/src/runtime_map.rs - 硬编码 ~40 个 runtime
fn register_builtin_runtimes(&mut self) {
    self.register(
        RuntimeSpec::new("npm", "Node.js package manager")
            .with_ecosystem(Ecosystem::Node)
            .with_dependency(
                RuntimeDependency::required("node", "npm is bundled with Node.js")
                    .provided_by("node"),
            ),
    );
    // ... 40+ more registrations
}
```

```toml
# node/provider.toml - 也定义了相同信息
[[runtimes]]
name = "npm"
description = "Node Package Manager"
bundled_with = "node"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=12", reason = "npm requires Node.js" }
]
```

**问题**：同样的信息（依赖关系、别名、生态系统）在两处维护，容易不一致。

**2. 违反 DRY 原则**

- 添加新 provider 需要修改中心文件 `runtime_map.rs`
- Provider 信息未能内聚在 provider 目录中
- 违反单一职责原则

**3. 维护成本高**

- 依赖关系硬编码在 Rust 代码中
- 修改需要重新编译
- 难以让用户自定义覆盖

### 当前尝试性解决方案的不足

```rust
// 现有的 apply_manifest_overrides() 方法
pub fn apply_manifest_overrides(&mut self, manifests: &[ProviderManifest]) {
    // 只覆盖部分属性，本质上还是"先硬编码，再覆盖"
}
```

这种方式是补丁式的，没有从根本上解决双重数据源问题。

## 设计目标

1. **单一数据源**：`provider.toml` 是 RuntimeSpec 的唯一定义来源
2. **零硬编码**：删除 `register_builtin_runtimes()` 中的所有硬编码
3. **高内聚**：Provider 的所有信息都在 provider 目录中
4. **向后兼容**：现有的 `RuntimeSpec` API 保持不变

## 架构对比

### Before (当前设计)

```
┌─────────────────────────────────────────────────────────┐
│                   runtime_map.rs                         │
│  register_builtin_runtimes() - 硬编码 ~40 个 runtime     │
│         ↓                                                │
│  apply_manifest_overrides() - 覆盖部分属性               │
└─────────────────────────────────────────────────────────┘
```

### After (新设计)

```
┌───────────────────┐   ┌───────────────────┐   ┌───────────────────┐
│ node/provider.toml│   │ bun/provider.toml │   │python/provider.toml│
│  - node           │   │  - bun            │   │  - python         │
│  - npm (依赖node) │   │  - bunx (依赖bun) │   │  - uv             │
│  - npx (依赖node) │   │                   │   │  - uvx (依赖uv)   │
└────────┬──────────┘   └────────┬──────────┘   └────────┬──────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 ▼
                    ┌─────────────────────────────┐
                    │  RuntimeMap::from_manifests │
                    │  (动态构建，无硬编码)        │
                    └─────────────────────────────┘
```

## 详细设计

### Phase 1: 扩展 provider.toml Schema

在 `RuntimeDef` 中添加可选字段：

```toml
# provider.toml
[[runtimes]]
name = "npm"
description = "Node Package Manager"
executable = "npm"
bundled_with = "node"
aliases = ["npmjs"]

# 新增字段
priority = 80                     # 安装优先级（数值越高越优先）
auto_installable = true           # 是否支持自动安装

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=12", recommended = "20", reason = "npm requires Node.js" }
]
```

### Phase 2: 添加转换层

```rust
// vx-manifest/src/provider.rs
impl RuntimeDef {
    /// Convert manifest RuntimeDef to resolver RuntimeSpec
    pub fn to_runtime_spec(&self, provider_ecosystem: Ecosystem) -> RuntimeSpec {
        let mut spec = RuntimeSpec::new(&self.name, self.description.as_deref().unwrap_or(""));
        
        // Basic fields
        spec = spec
            .with_ecosystem(provider_ecosystem)
            .with_aliases(self.aliases.clone());
        
        if let Some(exe) = &self.executable {
            spec = spec.with_executable(exe);
        }
        
        if !self.command_prefix.is_empty() {
            spec = spec.with_command_prefix(self.command_prefix.iter().map(|s| s.as_str()).collect());
        }
        
        // Convert bundled_with to dependency
        if let Some(bundled) = &self.bundled_with {
            spec = spec.with_dependency(
                RuntimeDependency::required(bundled, format!("{} is bundled with {}", self.name, bundled))
                    .provided_by(bundled)
            );
        }
        
        // Convert constraints to dependencies
        for constraint in &self.constraints {
            for dep in &constraint.requires {
                let mut runtime_dep = RuntimeDependency::required(&dep.runtime, &dep.reason);
                if let Some(ref rec) = dep.recommended {
                    runtime_dep = runtime_dep.with_recommended_version(rec);
                }
                // Parse version constraint for min/max
                // ...
                spec = spec.with_dependency(runtime_dep);
            }
        }
        
        // Optional fields
        if let Some(priority) = self.priority {
            spec = spec.with_priority(priority);
        }

        spec
    }
}
```

### Phase 3: 新增 RuntimeMap::from_manifests

```rust
// vx-resolver/src/runtime_map.rs
impl RuntimeMap {
    /// Build RuntimeMap entirely from provider manifests (no hardcoding)
    pub fn from_manifests(manifests: &[ProviderManifest]) -> Self {
        let mut map = Self::default();

        for manifest in manifests {
            let ecosystem = manifest.provider.ecosystem.unwrap_or_default();

            for runtime in &manifest.runtimes {
                let spec = runtime.to_runtime_spec(ecosystem);
                map.register(spec);
            }
        }

        map
    }

    /// Create with builtin manifests (for production use)
    pub fn with_builtin_manifests() -> Self {
        let manifests = load_builtin_manifests();
        Self::from_manifests(&manifests)
    }
}
```

### Phase 4: 废弃 register_builtin_runtimes

```rust
impl RuntimeMap {
    /// Create a new runtime map with built-in runtime definitions
    #[deprecated(since = "0.10.0", note = "Use RuntimeMap::with_builtin_manifests() instead")]
    pub fn new() -> Self {
        // For backward compatibility, still works
        let mut map = Self::default();
        map.register_builtin_runtimes();
        map
    }

    #[deprecated(since = "0.10.0", note = "Runtime specs are now loaded from provider.toml")]
    fn register_builtin_runtimes(&mut self) {
        // Keep existing code for backward compatibility
        // Will be removed in v0.11.0
    }
}
```

## 需要扩展的字段映射

| RuntimeSpec 字段    | provider.toml 字段        | 状态                      |
|--------------------|---------------------------|---------------------------|
| name               | runtimes.name             | ✅ 已存在                 |
| description        | runtimes.description      | ✅ 已存在                 |
| aliases            | runtimes.aliases          | ✅ 已存在                 |
| executable         | runtimes.executable       | ✅ 已存在                 |
| command_prefix     | runtimes.command_prefix   | ✅ 已存在                 |
| ecosystem          | provider.ecosystem        | ✅ 已存在，继承自 provider |
| dependencies       | runtimes.constraints      | ✅ 已存在，需转换          |
| bundled_with→deps  | runtimes.bundled_with     | ✅ 已存在，需转换为依赖   |
| priority           | runtimes.priority         | ❌ 需添加（可选，默认 0）  |
| auto_installable   | runtimes.auto_installable | ❌ 需添加（可选，默认 true）|

## 迁移策略

### 阶段式迁移

| Phase | 内容 | 风险 | 版本 |
|-------|------|------|------|
| 1 | 扩展 `RuntimeDef` 添加 `priority`、`auto_installable` | 低 | v0.9.x |
| 2 | 实现 `RuntimeDef::to_runtime_spec()` 转换方法 | 低 | v0.9.x |
| 3 | 实现 `RuntimeMap::from_manifests()` | 中 | v0.10.0 |
| 4 | 补全所有 provider.toml 中的依赖信息 | 中 | v0.10.0 |
| 5 | 废弃 `register_builtin_runtimes()` | 高 | v0.10.0 |
| 6 | 移除 `register_builtin_runtimes()` | 高 | v0.11.0 |

### 需要更新的 provider.toml 文件

检查并确保以下 provider 包含完整的依赖信息：

**已完整**：
- ✅ node/provider.toml (node, npm, npx)
- ✅ bun/provider.toml (bun, bunx)

**需检查/更新**：
- ⏳ python/provider.toml
- ⏳ rust/provider.toml
- ⏳ go/provider.toml
- ⏳ java/provider.toml
- ⏳ uv/provider.toml
- ⏳ yarn/provider.toml
- ⏳ pnpm/provider.toml
- ⏳ 其他 providers...

## 优势分析

| 方面 | 改进前 | 改进后 |
|------|--------|--------|
| **数据源** | 两个（代码 + toml） | 一个（toml） |
| **新增 provider** | 修改中心文件 + toml | 仅添加 toml |
| **维护成本** | 高（需同步两处） | 低（单一数据源） |
| **可扩展性** | 需修改代码 | 仅需修改配置 |
| **用户覆盖** | 不支持 | 可通过 override.toml 覆盖 |
| **测试** | 需 mock 代码 | 可用测试 toml 文件 |

## 向后兼容性

1. **API 兼容**：`RuntimeMap::new()` 继续工作，但标记为 deprecated
2. **行为兼容**：所有现有 runtime 规格保持不变
3. **渐进迁移**：可逐步更新每个 provider.toml

## 替代方案

### 方案 A: 保持现状

继续使用硬编码 + manifest override 的双重机制。

**优点**: 无需改动
**缺点**: 维护成本高，信息冗余，容易不一致

### 方案 B: 只保留硬编码

删除 provider.toml 中的依赖信息，只在代码中维护。

**优点**: 单一数据源
**缺点**: 违反"配置即代码"原则，不利于用户自定义

### 方案 C: 代码生成

使用 build.rs 从 provider.toml 生成 Rust 代码。

**优点**: 编译时完成，类型安全
**缺点**: 增加构建复杂度，调试困难

**选择方案**: 采用本 RFC 提出的声明式设计，因为它提供了最佳的可维护性和可扩展性。

## 实现计划

### v0.9.x (准备阶段) - ✅ 完成
- [x] 在 `RuntimeDef` 中添加 `priority` 和 `auto_installable` 字段
- [x] 实现 `RuntimeMap::from_manifests()` 转换方法（在 vx-resolver 中实现）
- [x] 添加单元测试验证转换正确性

### v0.10.0 (主要迁移) - ✅ 完成
- [x] 实现 `RuntimeMap::from_manifests()`
- [x] 标记 `RuntimeMap::new()` 为 deprecated
- [x] 标记 `RuntimeMap::new_with_manifests()` 为 deprecated
- [x] 标记 `register_builtin_runtimes()` 为 deprecated
- [x] 补全主要 provider.toml 的扩展信息（node, yarn, pnpm, python）
- [x] 切换 vx-cli 到 `RuntimeMap::from_manifests()`
- [x] 添加集成测试

### v0.11.0 (清理) - ✅ 完成
- [x] 移除 `register_builtin_runtimes()` 代码
- [x] 移除 `RuntimeMap::new()` 的 deprecated 实现
- [x] 更新文档

**注意**: RuntimeMap 现在是完全声明式的实现，没有任何硬编码。使用 `from_manifests()` 或 `Default::default()` 构造。

## 测试策略

### 单元测试

```rust
#[test]
fn test_runtime_def_to_spec_basic() {
    let toml = r#"
        name = "npm"
        description = "Node Package Manager"
        executable = "npm"
        bundled_with = "node"
    "#;
    let runtime_def: RuntimeDef = toml::from_str(toml).unwrap();
    let spec = runtime_def.to_runtime_spec(Ecosystem::Node);

    assert_eq!(spec.name, "npm");
    assert_eq!(spec.ecosystem, Ecosystem::Node);
    assert!(spec.dependencies.iter().any(|d| d.runtime_name == "node"));
}

#[test]
fn test_runtime_map_from_manifests() {
    let manifests = load_test_manifests();
    let map = RuntimeMap::from_manifests(&manifests);

    // Verify all expected runtimes are registered
    assert!(map.contains("node"));
    assert!(map.contains("npm"));
    assert!(map.contains("bun"));
    assert!(map.contains("bunx"));

    // Verify dependencies are correct
    let npm = map.get("npm").unwrap();
    assert!(npm.dependencies.iter().any(|d| d.runtime_name == "node"));
}
```

### 集成测试

```rust
#[test]
fn test_backward_compatibility() {
    // Old way
    let old_map = RuntimeMap::new();

    // New way
    let manifests = load_builtin_manifests();
    let new_map = RuntimeMap::from_manifests(&manifests);

    // Should have same runtimes
    for name in old_map.runtime_names() {
        assert!(new_map.contains(name), "Missing runtime: {}", name);
    }
}
```

## 参考资料

- [RFC 0012: Provider Manifest](./0012-provider-manifest.md)
- [RFC 0013: Manifest-Driven Provider Registration](./0013-manifest-driven-registration.md)
- [RFC 0018: Extended Provider Schema](./0018-extended-provider-schema.md) - 扩展 provider.toml 支持更多高级特性
- [Spack Package Specification](https://spack.readthedocs.io/en/latest/packaging_guide.html)
- [Rez Package Definition](https://rez.readthedocs.io/en/stable/package_definition.html)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-09 | Draft | 初始草案 |
| 2026-01-09 | Partially Implemented | 实现 Phase 1-3: RuntimeDef 扩展字段、RuntimeMap::from_manifests()、更新 node/yarn/pnpm/python provider.toml |
| 2026-01-09 | v0.10.0 Implementation Complete | 标记 RuntimeMap::new()/new_with_manifests()/register_builtin_runtimes() 为 deprecated；vx-cli 切换到 from_manifests() |
| 2026-01-09 | v0.11.0 Cleanup Complete | 移除所有 deprecated 代码，RuntimeMap 现在是纯声明式实现，无硬编码 |

