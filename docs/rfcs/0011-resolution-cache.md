# RFC 0011: 解析结果缓存（Resolution Cache）与执行计划（Execution Plan）

> **状态**: In Progress（Phase 1/2 已落地，Phase 3 规划中）
> **作者**: vx team
> **创建日期**: 2026-01-05
> **目标版本**: v0.7.0

## 摘要

本 RFC 提议在 `vx-resolver` 的“解析 → 依赖闭包 → 下载/安装 → 执行”流水线中引入**解析结果缓存（Resolution Cache）**与显式的**执行计划（Execution Plan）**。目标是：在短时间内重复执行同一条命令（例如 `vx npx ...`、`vx uvx ...`）时，避免重复解析带来的性能消耗，同时保证行为正确、可控（支持 `Refresh`/`Offline`/`NoCache` 语义），并严格落实“先解析依赖，再下载，再运行工具”的执行顺序。

## 主流方案调研

### 1. vx（当前实现）

- **版本列表缓存（持久化）**：`vx-runtime` 已实现 `VersionCache`，用于缓存各 Runtime 的“可用版本列表”（落地到 `~/.vx/cache/versions`），支持 TTL、离线与刷新模式。
- **运行时查找缓存（内存）**：`ProviderRegistry` 对 runtime 名/别名到 provider 的映射有内存缓存。
- **缺口**：目前缺少对“解析结果/依赖闭包/执行计划”的持久化缓存；同一命令短时间重复执行仍会重复走 resolver 逻辑。

### 2. uv/uvx（经验启示）

uv 的整体体验强调“warm cache”带来的稳定性能：
- **全局缓存**：通过全局缓存提升重复安装与工具运行的性能（去重、复用）。
- **命令面向缓存运维**：提供缓存清理等命令（例如 `uv cache ...`）。

参考：
- 项目主页：[astral-sh/uv](https://github.com/astral-sh/uv)

> 说明：本 RFC 当前调研侧重“产品体验与设计方向”，后续实现阶段将补充对 uv 具体缓存 key/锁/目录结构的源码级对照。

### 3. Cargo（经验启示）

- **锁文件保证可复现**：通过 `Cargo.lock` 固化解析结果。
- **缓存与离线**：本地 registry/cache + `--locked`/`--offline` 等语义形成稳定的可复现与可运维体验。

对 vx 的启示：
- “解析结果缓存”与“可复现锁文件（`vx.lock`）”应**互补**：前者主要服务性能，后者主要服务可复现与团队一致性。

## 动机

### 当前问题分析

1. **重复解析开销**
   - 用户在同一目录、同一配置下频繁执行相同命令时（CI 重试、编辑-运行循环、脚本多次调用），resolver 仍要重复构建依赖图、解析别名、生成 runtime 请求等。

2. **流水线边界不够显式**
   - 我们希望执行模型明确：
     1) 解析目标 runtime 与依赖闭包
     2) 生成可执行的安装/运行计划
     3) 按计划下载/安装
     4) 执行目标工具
   - 显式的 `ExecutionPlan` 能让这条边界更清晰，也更适合缓存与调试。

3. **缓存语义需要统一**
   - `vx-runtime` 已有 `CacheMode` 语义（Normal/Refresh/Offline/NoCache）。解析缓存应该复用同一套语义，做到“全链路一致”。

### 需求分析

1. **正确性优先**：缓存不得改变功能行为；缓存命中失败必须自动回退到完整解析。
2. **短期重复执行加速**：同一目录/同一输入的重复执行显著减少解析开销。
3. **可控性**：支持刷新、离线、禁用缓存；可被 CLI 或环境变量控制。
4. **可观测与可运维**：支持 stats、prune/clear，避免缓存无限增长。
5. **跨平台**：Windows/macOS/Linux 一致行为。

## 设计方案

### 1. 执行流水线：Execution Pipeline

将一次 `vx <runtime> [args...]` 抽象为以下阶段：

1. **Parse**：解析 CLI 输入，得到 `RuntimeRequest`（目标 runtime、版本约束、子命令/别名、原始参数）。
2. **Resolve**：解析依赖闭包、选择 provider/runtime，实现“先解析依赖”。
3. **Plan**：生成 `ExecutionPlan`（安装顺序、需要的下载/安装任务、最终可执行路径、环境变量变更）。
4. **Ensure Installed**：按拓扑序执行安装（必要时下载），实现“再下载”。
5. **Execute**：执行目标工具，实现“再运行工具”。

缓存命中的位置在 **Resolve/Plan** 阶段之间：命中则直接得到 `ExecutionPlan`（或等价的 `ResolvedGraph`），跳过重复解析。

### 2. 缓存对象：ExecutionPlan / ResolvedGraph

建议先缓存更“稳定”的中间产物 `ResolvedGraph`，再由它生成 `ExecutionPlan`：

- **`ResolvedGraph`（建议缓存）**：
  - 目标 runtime（canonical name）
  - 依赖闭包（拓扑序或依赖 DAG + 可重建拓扑）
  - 每个节点的 runtime 选择结果（provider、版本、分发来源、安装布局关键信息）
  - 与 `vx.lock` 的关联信息（若存在：lockfile hash / generation id）

- **`ExecutionPlan`（可选缓存）**：
  - 在 `ResolvedGraph` 基础上加上“本机路径/环境注入”后的结果
  - 风险：本机路径更容易受外部变化影响（如 store 迁移、PATH 改变、MSVC 环境注入策略变化）

结论：
- **P0**：缓存 `ResolvedGraph`。
- **P1**：在验证稳定性后，再考虑缓存 `ExecutionPlan`。

### 3. 缓存 Key：ResolutionCacheKey

缓存 key 需要能表达“同一条命令在同一上下文下的解析等价性”，建议由以下信息构成：

- **命令输入**：runtime 名/别名、显式版本、子命令、原始 args 的摘要（避免把敏感参数原文写入磁盘）。
- **项目配置指纹**：`vx.toml` 内容 hash、`vx.lock` 内容 hash（如存在）。
- **平台指纹**：OS/arch、`rustc -Vv` 不需要，但 vx 版本号需要。
- **解析器版本**：resolver 相关结构体的 schema version（用于未来升级/不兼容变更）。
- **关键环境摘要**：
  - 影响解析结果的环境变量白名单（如镜像源、proxy、MSVC 选择相关 env）
  - `PATH` 的摘要（可选，若解析会受 PATH 影响）

### 4. 失效策略

#### 4.1 强失效（必定 miss）

- `vx.toml`/`vx.lock` hash 改变
- vx 版本改变（或 resolver schema version 改变）
- provider 注册集发生变化（可用“providers 列表 + 版本号”做摘要）

#### 4.2 弱失效（TTL）

- 默认短 TTL（建议 10～30 分钟），面向“开发态短时间重复执行”。
- TTL 过期后：
  - Normal：当作 miss，重新解析并回写
  - Offline：允许返回过期结果（并在 UI/日志中提示）

#### 4.3 运行前校验（防止缓存导致错误）

即使缓存命中，也需要做快速校验：
- 若 `ResolvedGraph` 中声明的某些 runtime 已安装，则继续。
- 若发现缺失，仍可按计划进入 Ensure Installed 阶段补齐。
- 若发现 plan 与当前 store 布局不兼容（例如 schema 升级），直接回退到完整解析。

### 5. 缓存模式：复用 CacheMode

解析缓存应与版本缓存保持一致语义：

- **Normal**：命中即用；过期则重新解析。
- **Refresh**：忽略缓存并强制重新解析，成功后覆盖写回。
- **Offline**：仅使用缓存；若无缓存则报错；若有过期缓存可返回（可配置）。
- **NoCache**：完全禁用解析缓存。

### 6. 落地位置与文件格式

- **目录**：`~/.vx/cache/resolutions/`
- **文件名**：`<key_hash>.json`
- **格式**：JSON（serde），包含：
  - `schema_version`
  - `created_at`、`ttl_secs`
  - `key`（`ResolutionCacheKey`，不含敏感原文）
  - `value`（`ResolvedGraph`）

写入采用 atomic rename；并发读写采用文件锁或 lockfile（与 `VersionCache` 的实现策略对齐）。

### 7. 是否需要新 crate？

#### 推荐方案：新增 `vx-cache` crate

新增 `crates/vx-cache/`，提供通用磁盘缓存基础设施，`vx-resolver` 在其上实现 `ResolutionCache`：

- **理由**：
  - 解析缓存与版本缓存属于同一类基础设施能力，未来还会有下载索引缓存等需求。
  - 使 `vx-resolver` 保持专注解析逻辑，缓存能力作为基础库复用。

- **初步结构**：

```text
crates/vx-cache/
├── src/
│   ├── lib.rs
│   ├── mode.rs            # CacheMode（未来可从 vx-runtime 迁移/复用）
│   ├── file_cache.rs      # 原子写入/锁/ttl/统计
│   └── types.rs           # CacheEntry/CacheStats
└── tests/
    └── file_cache_tests.rs
```

`vx-resolver` 新增：
- `resolution_cache.rs`：基于 `vx-cache` 的 `ResolutionCache` 实现。

#### 替代方案：先在 vx-resolver 内部实现

- 优点：最小改动、最快落地。
- 缺点：缓存能力将来更难复用，且与 `vx-runtime::VersionCache` 容易出现语义分叉。

本 RFC 建议：**直接创建 `vx-cache` crate**，但实现可以分阶段推进（先 minimal，再逐步抽象）。

## 向后兼容性

- **默认行为兼容**：不开启任何新开关时，用户可感知变化应主要是“更快”，不应出现行为差异。
- **故障回退**：缓存读取失败、格式不兼容、校验失败，一律回退到完整解析。
- **可禁用**：提供 `NoCache` 语义（未来可通过 CLI flag 或环境变量暴露）。

## 实现计划

### Phase 1：接口与最小可用缓存（v0.7.0）

- [x] 定义 `ResolvedGraph`（P0：先不缓存 `ExecutionPlan`）
- [x] 定义 `ResolutionCacheKey`
- [x] 在 `vx-resolver` 内实现 `ResolutionCache`（最小可用：读/写/TTL/atomic write）
- [x] 将 resolver 执行路径重构为显式“Resolve → Plan → Ensure Installed → Execute”
- [x] 添加单元测试（放入 `crates/vx-resolver/tests/` 或 `crates/vx-cache/tests/`）

### Phase 2：抽象为 `vx-cache` crate（v0.7.1）

- [x] 提取通用缓存基础设施到 `vx-cache`
- [x] `vx-runtime::VersionCache` 与解析缓存统一 `CacheMode` 语义（必要时迁移 `CacheMode` 定义位置）
- [x] CLI 增强：`vx cache stats`/`vx cache clear` 覆盖 resolutions
- [x] CLI 增强：新增全局 `--cache-mode <normal|refresh|offline|no-cache>` 统一控制 versions/resolutions

### Phase 3：观测与性能（v0.8.0）

- [ ] 增加 stats/prune（按大小/按时间）
- [ ] 记录 cache hit/miss 指标（可选，结合 tracing）
- [ ] 基准测试：对比命中/未命中的解析耗时

## 替代方案

1. **完全依赖 `vx.lock`**
   - `vx.lock` 解决可复现，但对“短期重复执行性能”帮助有限（仍需要解析与校验），且并非所有场景都存在 lock。

2. **只做内存缓存**
   - 对单进程多次调用有效，但对 CLI 多次启动（最常见）收益很小。

## 参考资料

- [RFC 0008: 通用版本解析器设计](./0008-version-solver.md)
- [RFC 0006: vx-setup Crate](./0006-vx-setup-crate.md)
- uv 项目主页：[astral-sh/uv](https://github.com/astral-sh/uv)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-05 | Draft | 初始草案 |
| 2026-01-06 | Phase 1/2 | 落地最小可用解析缓存与 `vx-cache` 基础设施（TTL/atomic write/CacheMode/回退 + tests），补齐 `ResolvedGraph` 与显式执行流水线，并在 CLI 增加全局 `--cache-mode` 以及 `vx cache` 的 resolutions 运维能力 |
