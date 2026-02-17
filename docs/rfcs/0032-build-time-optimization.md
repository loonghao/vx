# RFC 0032: 构建时间优化

> **状态**: Phase 2 In Progress
> **作者**: vx team
> **创建日期**: 2026-02-15
> **更新日期**: 2026-02-17
> **目标版本**: v0.8.0

## 摘要

当前 vx 项目 dev 全量构建耗时约 **172 秒（2 分 51 秒）**，关键路径上 `vx-runtime`（88s）→ providers → `vx-cli`（76s）几乎无并行收益。本 RFC 提出一系列分层优化措施，目标将 dev 全量构建时间降至 **60-90 秒**，增量构建降至 **10-20 秒**。

## 主流方案调研

在设计本方案之前，我们调研了以下主流 Rust 项目的构建优化实践：

### 1. matklad (rust-analyzer 作者) - Fast Rust Builds

**核心观点**：

- **关注依赖图形状**：链式依赖 `A → B → C → D` 只能串行编译，树状/菱形依赖可以极大提高并行度
- **减少最终产物**：静态链接下多 binary 的链接开销是 `m × n`，考虑 BusyBox 风格合并
- **隔离过程宏**：`syn` 等重型宏库不能被流水线化，应推迟到依赖图末端
- **边界处使用非泛型接口**：在 crate 边界提供非泛型实现函数，只暴露薄泛型包装层
- **精简依赖**：审视 `Cargo.lock`，减少不必要的重量级库

**参考**: https://matklad.github.io/2021/09/04/fast-rust-builds.html

### 2. nnethercote - The Rust Performance Book

**编译时间优化建议**：

- 使用 `cargo build --timings` 可视化瓶颈
- 使用 `-Zmacro-stats` 统计宏生成的代码量
- 使用 `cargo llvm-lines` 找出导致生成最多 LLVM IR 的泛型函数
- 将泛型函数中不依赖泛型的逻辑提取到非泛型函数中

**参考**: https://nnethercote.github.io/perf-book/compile-times.html

### 3. Bevy 引擎

**优化策略**：

- `opt-level = 0` + `debug = false` 的 dev profile 加速日常开发
- 动态链接 feature（`bevy/dynamic_linking`）用于开发阶段
- Workspace 级统一依赖管理

### 方案对比

| 策略 | matklad | nnethercote | Bevy | 适用于 vx |
|------|---------|------------|------|----------|
| 拆分重型 crate 提高并行度 | ✓ 核心建议 | - | - | ✓ 最关键 |
| 使用快速 linker | ✓ | - | - | ✓ |
| 减少泛型/单态化 | ✓ | ✓ | - | △ 中等 |
| 精简 feature flags | ✓ | - | ✓ | ✓ |
| dev profile 优化 | ✓ | - | ✓ | ✓ 已有 |
| 减少 crate 数量 | ✓ | - | - | ✓ |

### 设计启示

基于以上调研，本 RFC 应采用：

1. **拆分 `vx-runtime`** — 采用 matklad 的「提高依赖图并行度」理念，将重型 crate 拆分为轻量 trait crate + 重实现 crate
2. **使用 `rust-lld`** — matklad 推荐的快速 linker 方案
3. **合并同构 provider** — 减少 crate 数量降低固定开销，参考 matklad 的「减少最终产物」思路
4. **精简 feature flags** — 参考 Bevy 的按需启用策略

## 动机

### 当前状态分析

**构建环境**: Windows MSVC, Rust 1.93.0, 65+ workspace crate

**`cargo build --timings` 数据**:

| Crate | Duration | rmeta 完成 | 开始时间 | 说明 |
|-------|----------|-----------|---------|------|
| `vx-runtime` | **88.48s** | 11.42s | 6.35s | 最大单点瓶颈 |
| `vx-resolver` | **77.14s** | 19.92s | 17.77s | 依赖 vx-runtime rmeta |
| `vx-cli` | **76.40s** | 27.31s | 79.22s | 汇聚全部 provider + 核心 crate |
| `vx-provider-msvc` | 38.57s | 17.14s | 53.86s | msvc-kit 依赖重 |
| `vx-extension` | 34.26s | 10.13s | 51.12s | |
| 57 个 provider | 各 15-25s | - | ~17s | 全部等待 vx-runtime |

**关键路径**:

```
时间轴 (秒)
0s          17s              94s           156s         172s
|-----------|----------------|-------------|------------|
  vx-runtime (88s)
              → 57 providers 并行 (~15-25s each)
                               → vx-cli (76s)
                                              → link (16s)
```

**关键路径总长 ≈ 88s + 76s + 16s ≈ 172s**

### 问题根因

1. **`vx-runtime` 过重**（88s）— 集中了 HTTP、6 种归档格式、进度条、动态加载等所有重依赖，57 个 provider 都等它
2. **57 个独立 provider crate**（固定开销 ~2-5s/个）— 大部分是同构的 manifest-driven thin wrapper
3. **Windows MSVC linker 慢**（16s）— 默认 link.exe 远慢于 lld
4. **dev profile 过重** — `opt-level = 1` + `debug = 1` 增加了不必要的编译时间

### `vx-runtime` 依赖清单

```toml
# HTTP 客户端
reqwest = { features = ["json", "stream", "form", "rustls"] }

# 6 种归档格式
tar, flat2, xz2, zstd, zip (7 features), sevenz-rust

# 重型工具库
chrono (+ serde), regex, indicatif, libloading, bincode

# 网络重试
backon

# 内部依赖
vx-core, vx-cache, vx-paths, vx-manifest, vx-system-pm
```

这些依赖导致了 88s 的编译时间，而 57 个 provider 只需要其中的 trait 定义和少量辅助类型。

## 设计方案

### Phase 1: 即时生效优化（零代码改动）

#### 1.1 使用 `rust-lld` 链接器

在 `.cargo/config.toml` 中添加 lld 配置。Rust 1.93+ 已内置 `rust-lld`：

```toml
# 现有配置保持不变
[target.'cfg(all(target_env = "msvc", target_os = "windows"))']
rustflags = ["-C", "target-feature=+crt-static", "-C", "link-arg=-fuse-ld=lld"]

# Linux - 使用 mold（如已安装）或 lld
# [target.x86_64-unknown-linux-gnu]
# rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

**预估收益**: 链接阶段从 ~16s 降至 ~3-5s，**节省 10-13s**

**注意**: 此配置同时影响 release 构建。若 release 遇到兼容性问题，可通过环境变量条件控制，或仅在 dev profile 下使用。

#### 1.2 使用 `dev-fast` profile 进行日常开发

项目已定义但未使用。在 `justfile` 中添加快速构建命令：

```just
# 快速开发构建
build-fast:
    cargo build --profile dev-fast -p vx

# 常规开发构建
build:
    cargo build -p vx
```

当前 dev profile 配置对比：

| 配置 | `dev` | `dev-fast` | 影响 |
|------|-------|-----------|------|
| `opt-level` | 1 | 0 | 优化级别越高编译越慢 |
| `debug` | 1（行号表） | false | 不生成任何调试信息 |
| `incremental` | 默认 | true（显式） | 加速增量构建 |

**预估收益**: 全量构建节省 **10-20s**，增量构建显著加速

### Phase 2: 拆分 `vx-runtime`（收益最大）

#### 2.1 拆分策略

将 `vx-runtime` 拆分为 3 个 crate：

```
vx-runtime-core     ← 轻量：trait 定义 + Registry + 基础类型
                       依赖：vx-core, async-trait, anyhow, serde
                       预估编译：~5-8s

vx-runtime-archive  ← 重型：归档解压实现
                       依赖：tar, flat2, xz2, zstd, zip, sevenz-rust
                       预估编译：~30-40s

vx-runtime          ← 门面：re-export 以上两个 + HTTP + 下载逻辑
                       依赖：reqwest, indicatif, backon, ...
                       预估编译：~40-50s（但不在关键路径上）
```

#### 2.2 依赖关系变化

**Before**:

```
vx-runtime (88s) ──→ 57 providers (17s 才能开始)
                 ──→ vx-resolver
                 ──→ vx-cli
```

**After**:

```
vx-runtime-core (5-8s) ──→ 57 providers (5-8s 即可开始!)
                        ──→ vx-resolver (只需 core)

vx-runtime-archive (30-40s) ──→ vx-runtime (门面)
                                  ──→ vx-cli (仅 cli 需要完整实现)
```

#### 2.3 `vx-runtime-core` 包含内容

```rust
// crates/vx-runtime-core/src/lib.rs

// Trait 定义
pub trait Runtime: Send + Sync { ... }
pub trait Provider: Send + Sync { ... }
pub trait PackageManager: Send + Sync { ... }

// 核心类型
pub struct VersionInfo { ... }
pub struct InstallResult { ... }
pub struct ExecutionResult { ... }
pub struct RuntimeContext { ... }
pub struct Platform { ... }

// Registry
pub struct ProviderRegistry { ... }

// 错误类型
pub enum RuntimeError { ... }
```

#### 2.4 预估关键路径变化

```
Before: vx-runtime(88s) → providers(~20s) → vx-cli(76s) → link(16s) = 172s
After:  vx-runtime-core(8s) → providers(~20s) → vx-cli(60s) → link(5s) = 93s
                                                                         ≈ 46% 提升
```

providers 提前 ~80s 开始编译，`vx-runtime-archive` 和 `vx-runtime` 与 providers 并行编译，不再阻塞关键路径。

### Phase 3: 合并同构 Provider

#### 3.1 分析

57 个 provider 中，绝大多数是纯 manifest-driven 的 thin wrapper，代码结构完全一致（3-16KB）。每个独立 crate 有 ~2-5s 的固定开销（rustc 启动、元数据生成、codegen 初始化）。

#### 3.2 分类

| 类型 | Provider | 说明 |
|------|----------|------|
| **可合并** (~40+) | awscli, bat, brew, cmake, docker, fd, ffmpeg, fzf, gcloud, gh, hadolint, helm, imagemagick, jq, kubectl, make, meson, nasm, ninja, ollama, pre-commit, protoc, pwsh, release-please, rcedit, ripgrep, spack, starship, task, terraform, vite, winget, yq, dagu, prek, actrun, ... | 纯 manifest-driven，无额外依赖 |
| **独立保留** (~15) | node, go, uv, python, rust, bun, pnpm, yarn, deno, zig, java, msvc, dotnet, msbuild, nuget | 有自定义逻辑或额外依赖 |

#### 3.3 方案

创建 `vx-providers-builtin` crate，合并所有纯 manifest-driven provider：

```rust
// crates/vx-providers-builtin/src/lib.rs
mod awscli;
mod bat;
mod brew;
mod cmake;
mod docker;
// ... 40+ 个模块

pub fn builtin_providers() -> Vec<Box<dyn Provider>> {
    vec![
        Box::new(awscli::AwscliProvider),
        Box::new(bat::BatProvider),
        // ...
    ]
}
```

**预估收益**: 40 个 crate × ~3s 固定开销 → 1 个 crate，**节省 15-30s**（主要在并行阶段释放 CPU 资源和减少 I/O）

### Phase 4: 精简 Feature Flags

#### 4.1 `zip` crate feature 精简

当前启用了 7 个 feature，评估实际使用情况：

```toml
# Before
zip = { version = "7.0", features = ["aes-crypto", "bzip2", "deflate64", "deflate", "ppmd", "time", "zstd"] }

# After - 只保留常用格式
zip = { version = "7.0", default-features = false, features = ["deflate", "zstd"] }
```

大部分工具分发使用 deflate 或 zstd 压缩，`aes-crypto`、`bzip2`、`deflate64`、`ppmd` 极少遇到。

#### 4.2 `chrono` serde feature

评估哪些 crate 真正需要 `chrono/serde`，对不需要的 crate 使用不带 serde 的 chrono。

#### 4.3 `reqwest` stream feature

评估是否真正使用了流式下载。如果只使用 `response.bytes()`，可以去掉 `stream` feature。

**预估收益**: **5-10s**

## 综合预期效果

### 全量构建时间（dev profile）

| 阶段 | 措施 | 预估节省 | 累计时间 |
|------|------|---------|---------|
| 当前 | — | — | **172s** |
| Phase 1.1 | rust-lld linker | 10-13s | ~160s |
| Phase 1.2 | dev-fast profile | 10-20s | ~142s |
| Phase 2 | 拆分 vx-runtime | 30-50s | ~100s |
| Phase 3 | 合并同构 provider | 15-30s | ~80s |
| Phase 4 | 精简 features | 5-10s | ~70s |

**目标**: 全量构建 **70-90s**（当前 172s → 提升 50-60%）

### 增量构建时间

Phase 1 + Phase 2 完成后，修改单个 provider 的增量构建预计 **5-15s**。

## 向后兼容性

### Phase 1: 完全兼容

- linker 和 profile 变更不影响任何 API
- CI/release 构建使用 release profile，不受 dev profile 影响

### Phase 2: 内部重构

- `vx-runtime-core` 对外暴露的 API 不变
- Provider 的 `Cargo.toml` 需要将 `vx-runtime` 依赖改为 `vx-runtime-core`
- `vx-cli` 等需要完整实现的 crate 改为依赖 `vx-runtime`（包含 re-export）

### Phase 3: 内部重构

- 合并后的 provider 对外行为完全不变
- `vx-cli/src/registry.rs` 中的注册方式需要调整

### Phase 4: 功能可能受限

- 精简 zip features 后，如遇到使用 bzip2/ppmd 压缩的归档文件会无法解压
- 需要先审计现有 provider 的实际下载格式，确认无影响后再精简

## 实现计划

### Phase 1: 即时优化（v0.7.x）

- [x] 添加 `rust-lld` linker 配置到 `.cargo/config.toml`
- [x] 在 `justfile` 中添加 `build-fast` 命令（已存在）
- [x] 验证 lld 在 Windows/Linux/macOS 上的兼容性
- [x] 基准测试对比

### Phase 2: 拆分 vx-runtime（v0.8.0）

- [x] 创建 `vx-runtime-core` crate，迁移 trait + 类型定义
- [x] 创建 `vx-runtime-archive` crate，迁移归档处理逻辑
- [x] vx-runtime 添加对 vx-runtime-core 和 vx-runtime-archive 的依赖
- [x] vx-runtime re-export 核心类型，作为门面 crate
- [ ] 将所有 provider 的 `vx-runtime` 依赖改为 `vx-runtime-core`（后续 PR）
- [ ] `vx-runtime` 变为完整门面 crate，re-export core + archive（后续 PR）
- [ ] 更新 `vx-resolver` 依赖为 `vx-runtime-core`（后续 PR）
- [ ] 运行全量测试，确保无回归
- [ ] 基准测试对比

### Phase 3: 合并同构 Provider（v0.8.0）

- [ ] 审计所有 provider，确认哪些可以合并
- [ ] 创建 `vx-providers-builtin` crate
- [ ] 逐步迁移 manifest-driven provider 到 builtin
- [ ] 更新 `vx-cli/src/registry.rs` 注册逻辑
- [ ] 清理已合并的独立 provider crate
- [ ] 基准测试对比

### Phase 4: Feature 精简（v0.8.x）

- [ ] 审计所有 provider 的下载格式，确认 zip feature 需求
- [ ] 精简 zip、chrono、reqwest 的 feature flags
- [ ] 基准测试对比

## 替代方案

### 方案 A: 动态链接（不采用）

Bevy 使用 `dynamic_linking` feature 加速开发构建。但 vx 作为 CLI 工具需要单一可执行文件分发，动态链接会增加部署复杂度。不适合。

### 方案 B: sccache 分布式编译缓存（补充）

可以在 CI 中使用 `sccache` 缓存编译产物。这不影响本地构建时间，但可以加速 CI。可以作为补充方案，但不替代本 RFC 的结构性优化。

### 方案 C: 使用 cranelift 后端（实验性）

Rust nightly 支持 cranelift 后端，编译速度比 LLVM 快但生成代码质量差。目前仍为实验性，不推荐用于生产项目。可以在 `dev-fast` profile 中作为可选项：

```toml
# 未来可选（Rust nightly）
# RUSTFLAGS="-Zcodegen-backend=cranelift" cargo build --profile dev-fast
```

### 方案 D: workspace-hack crate（不优先）

使用 `cargo hakari` 创建 workspace-hack crate 统一依赖编译。这对有大量共享依赖但版本不一致的 workspace 有效。vx 已使用 workspace dependencies 统一版本，收益有限。

## 风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| rust-lld 在特定平台不兼容 | 低 | 低 | 仅配置为 dev 默认，CI release 不受影响 |
| 拆分 vx-runtime 引入 API 回归 | 中 | 中 | 完善测试覆盖，Phase 2 前确保测试通过 |
| 合并 provider 后构建反而变慢 | 低 | 低 | 先合并 5-10 个验证效果 |
| 精简 zip features 导致解压失败 | 中 | 低 | 先审计所有 provider 下载格式 |

## 参考资料

### 主流项目
- [Fast Rust Builds - matklad](https://matklad.github.io/2021/09/04/fast-rust-builds.html) — 本 RFC 的核心参考
- [Compile Times - The Rust Performance Book](https://nnethercote.github.io/perf-book/compile-times.html)
- [Bevy Getting Started - Compile Optimizations](https://bevyengine.org/learn/book/getting-started/setup/)

### 工具
- `cargo build --timings` — 编译时间可视化
- `cargo llvm-lines` — LLVM IR 生成量分析
- `rust-lld` — Rust 内置快速链接器（1.93+）
- `cargo-hakari` — workspace-hack crate 生成器

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-15 | Draft | 初始草案，基于 `cargo build --timings` 数据分析 |
| 2026-02-17 | Phase 1 Completed | 完成 Phase 1：添加 lld linker 配置到所有平台 |
| 2026-02-17 | Phase 2 Started | 创建 vx-runtime-core 和 vx-runtime-archive crate |
| 2026-02-17 | Phase 2 Progress | vx-runtime 集成 vx-runtime-core 和 vx-runtime-archive，作为门面 crate |
