# RFC 0032: 构建时间优化

> **状态**: Phase 2 + 2.5 + 4(partial) + 5 Complete — Phase 3 ❌ 不采用（已回滚）— 方案 D (hakari) ✅ 已实施
> **作者**: vx team
> **创建日期**: 2026-02-15
> **更新日期**: 2026-02-18
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
- Nicholas Nethercote 优化 `#[derive(Reflect)]` 宏，生成代码减少 39%，`cargo check` 时间减少 16%

### 4. 2025-2026 年新兴技术

**Wild 链接器**（实验性）：
- 完全用 Rust 编写的新一代链接器，三层并行架构 + 无锁符号表
- 链接 `rustc-driver.so` 比 mold 快 1.72×（476ms vs 819ms）
- 增量链接功能即将上线（v0.6.0），这是 mold 不具备的
- 目前仅支持 Linux，Windows 暂不可用

**Rust 并行前端编译（-Z threads）**：
- 编译器前端（解析、类型检查、借用检查）并行化
- 大型项目可减少编译时间 30-50%
- 截至 2026 年初仍是 nightly 功能
- 内存使用增加约 35%，小项目可能变慢

**Rust 编译器自身改进（2025.12 nnethercote）**：
- VecCache 优化：指令计数减少 4%+
- Trivial Consts 快速路径：libc crate 编译提速 5-15%
- LLVM 21 集成：平均指令计数减少 1.7%
- `-Zhint-mostly-unused`：加速大型 API crate 编译

**cargo-nextest**（并行测试运行器）：
- 每个测试独立进程并行执行，测试阶段提速 2-3×
- 可与构建优化互补，缩短 CI 总时间

### 方案对比

| 策略 | matklad | nnethercote | Bevy | 新兴 | 适用于 vx |
|------|---------|------------|------|------|----------|
| 拆分重型 crate 提高并行度 | ✓ 核心建议 | - | - | - | ✓ 最关键 |
| 使用快速 linker | ✓ | - | - | Wild | ✓ |
| 减少泛型/单态化 | ✓ | ✓ | - | - | △ 中等 |
| 精简 feature flags | ✓ | - | ✓ | - | ✓ |
| dev profile 优化 | ✓ | - | ✓ | - | ✓ 已有 |
| 减少 crate 数量 | ✓ | - | - | - | ✓ |
| 并行前端 (-Z threads) | - | - | - | ✓ | △ nightly |
| cargo-nextest | - | - | - | ✓ | ✓ 测试阶段 |

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
# Windows MSVC - Use lld linker for faster builds
[target.'cfg(all(target_env = "msvc", target_os = "windows"))']
rustflags = [
    "-C", "target-feature=+crt-static",
    "-C", "link-arg=-fuse-ld=lld",
]

# Linux gnu - Use lld linker for faster builds
[target.'cfg(all(target_os = "linux", target_env = "gnu"))']
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Note: macOS uses default linker (ld64) which is already fast
# lld on macOS has compatibility issues with some system libraries
```

**预估收益**: 
- Windows: 链接阶段从 ~16s 降至 ~3-5s，**节省 10-13s**
- Linux: 类似收益
- macOS: 保持默认链接器（ld64 已经足够快）

**注意**: 
- macOS 不使用 lld，因为 clang 的 `-fuse-ld=lld` 参数有兼容性问题
- CI release 构建可能需要验证平台兼容性

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

### Phase 2: 按功能域拆分 `vx-runtime`（收益最大）

#### 2.0 命名方案：按功能域命名（方案 B）

经过对比分析，我们采用**按功能域命名**的方案。这种方式语义最清晰，且有利于未来独立维护各功能模块。

**命名方案对比**：

| 方案 | 接口层 | HTTP/下载 | 归档 | Provider 改动 | 语义清晰度 |
|------|--------|----------|------|-------------|----------|
| A: `-impl` 后缀 | `vx-runtime` | `vx-runtime-impl` | `vx-runtime-archive` | 🟢 不改 | ⭐⭐⭐ |
| **B: 按功能域 ✓** | **`vx-runtime`** | **`vx-runtime-http`** | **`vx-runtime-archive`** | **🟢 不改** | **⭐⭐⭐⭐** |
| C: `-core` 后缀 | `vx-runtime-core` | `vx-runtime` | `vx-runtime-archive` | 🔴 57 provider 改 | ⭐⭐ |
| D: `-api` 后缀 | `vx-runtime-api` | `vx-runtime` | `vx-runtime-archive` | 🔴 57 provider 改 | ⭐⭐⭐ |
| E: `-full` 后缀 | `vx-runtime` | `vx-runtime-full` | `vx-runtime-archive` | 🟢 不改 | ⭐⭐⭐⭐ |

**选择方案 B 的理由**：

1. **按功能域独立维护**：`vx-runtime-http`（HTTP 下载）、`vx-runtime-archive`（归档解压）各自独立，未来可以单独演进
2. **不需要门面 crate**：没有人为的聚合层，每个 crate 职责清晰
3. **Provider 零改动**：最常用的名字 `vx-runtime` 给最常依赖的接口层
4. **`vx-cli` 按需组合**：只有需要完整功能的 crate 才同时依赖多个子 crate

#### 2.1 拆分策略

将 `vx-runtime`（88s）拆分为 3 个独立功能域 crate：

```
vx-runtime           ← 轻量：trait 定义 + Registry + RuntimeContext + 基础类型
                        依赖：vx-core, vx-manifest, async-trait, anyhow, serde, chrono, bincode
                        预估编译：~8-12s
                        消费者：57 providers, vx-resolver, vx-extension, vx-cli

vx-runtime-http      ← 中量：HTTP 客户端 + 下载逻辑 + 进度条 + CDN 加速
                        依赖：reqwest, indicatif, backon, turbo-cdn(optional)
                        预估编译：~25-35s
                        消费者：vx-cli（唯一需要实际下载功能的）

vx-runtime-archive   ← 重型：归档解压实现（已有）
                        依赖：tar, flate2, xz2, zstd, zip, sevenz-rust
                        预估编译：~30-40s
                        消费者：vx-cli
```

**关键设计**：`vx-runtime-http` 和 `vx-runtime-archive` **互不依赖**，可完全并行编译。

#### 2.2 依赖关系变化

**Before**:

```
vx-runtime (88s) ──→ 57 providers (17s 才能开始)
                 ──→ vx-resolver (77s，等 runtime rmeta)
                 ──→ vx-extension (34s)
                 ──→ vx-cli
```

**After**:

```
vx-runtime (8-12s) ──→ 57 providers (8-12s 即可开始!)
                    ──→ vx-resolver (只需 trait + types)
                    ──→ vx-extension (只需 trait + types)

vx-runtime-http (25-35s)    ──→ vx-cli (按需组合)
vx-runtime-archive (30-40s) ──→ vx-cli (按需组合)
                ↑ 这两个与 providers 完全并行编译!
```

#### 2.3 `vx-runtime`（轻量接口层）包含内容

```rust
// crates/vx-runtime/src/lib.rs

// Trait 定义
pub trait Runtime: Send + Sync { ... }
pub trait Provider: Send + Sync { ... }
pub trait PackageManager: Send + Sync { ... }
pub trait HttpClient: Send + Sync { ... }

// 核心类型
pub struct VersionInfo { ... }
pub struct InstallResult { ... }
pub struct ExecutionResult { ... }
pub struct RuntimeContext { ... }      // 统一为唯一定义
pub struct Platform { ... }
pub struct GitHubReleaseOptions { ... } // 纯数据结构

// Registry
pub struct ProviderRegistry { ... }
pub struct ManifestRegistry { ... }

// 版本缓存（bincode 轻量依赖）
pub struct VersionCache { ... }

// 错误类型
pub enum RuntimeError { ... }
```

#### 2.4 `vx-runtime-http`（HTTP 功能域）包含内容

```rust
// crates/vx-runtime-http/src/lib.rs

// 真实 HTTP 客户端实现
pub struct ReqwestHttpClient { ... }
impl HttpClient for ReqwestHttpClient { ... }

// 下载管理器（带进度条、重试、CDN 加速）
pub struct DownloadManager { ... }

// 真实安装器实现
pub struct RealInstaller { ... }

// RuntimeContext 工厂函数
pub fn create_runtime_context(...) -> RuntimeContext { ... }
```

#### 2.5 预估关键路径变化

```
Before:
  vx-runtime(88s) → providers(~20s) → vx-cli(76s) → link(16s) = 172s

After:
  vx-runtime(10s) → providers(~20s) ──┐
  vx-runtime-http(30s)    ────────────┼──→ vx-cli(~45s) → link(5s) = 80s
  vx-runtime-archive(35s) ────────────┘
                                                                    ≈ 53% 提升
```

providers 提前 ~78s 开始编译。`vx-runtime-http` 和 `vx-runtime-archive` 与 providers **完全并行**编译，不在关键路径上。

### Phase 2.5: 拆分其他瓶颈 crate

除了 `vx-runtime`（88s），依赖图分析还发现以下拆分机会：

#### 2.5.1 `vx-cli` 自身拆分（76s → ~45s）

`vx-cli`（76s）是关键路径上的第二大瓶颈。分析发现它直接依赖了大量重型三方库，其中部分只被特定功能使用：

**`vx-cli` 直接依赖的重型库分析**：

| 依赖 | 使用位置 | 是否核心功能 | 拆分可能 |
|------|---------|-------------|----------|
| `reqwest` | `self_update.rs` | 仅 self-update | ✓ 可拆分 |
| `zip`, `tar`, `flate2` | `self_update.rs` | 仅 self-update | ✓ 可拆分 |
| `clap` | CLI 解析 | 核心 | ✗ |
| `indicatif` | 进度条 | 核心 | ✗ |
| `regex` | 多处 | 核心 | ✗ |
| `axoupdater` (optional) | self-update | 仅 self-update | ✓ 已 optional |

**方案**：将 self-update 逻辑拆分到 `vx-self-update` crate：

```
vx-self-update   ← self-update 专用：reqwest, zip, tar, flate2, axoupdater
                    预估编译：~20-25s（与核心逻辑并行）
                    消费者：仅 vx-cli

vx-cli           ← 精简后：不再直接依赖 reqwest/zip/tar/flate2
                    预估编译：~45s（从 76s 降低）
```

**预估收益**：减少 `vx-cli` 自身编译时间 **~20-30s**（重型依赖移到并行路径），但由于 `vx-cli` 仍通过 `vx-runtime-http` 间接依赖这些库，实际收益取决于 cargo 的增量编译是否能跳过。保守估计 **~10-15s**。

#### 2.5.2 `vx-resolver` 依赖优化（77s → ~40s）

`vx-resolver`（77s）目前依赖 `vx-runtime`（88s），是因为它使用了：

```rust
// 实际使用的类型（仅接口层）
use vx_runtime::{CacheMode, ProviderRegistry, RuntimeContext};
use vx_runtime::{VersionInfo, InstallResult};
```

这些类型在 Phase 2 拆分后都在轻量的 `vx-runtime`（10s）中。
此外 `vx-resolver` 还依赖了 `vx-console`（含 indicatif、anstream），可以评估是否通过 trait 抽象解耦。

**Phase 2 后 `vx-resolver` 的变化**：
- 依赖从 `vx-runtime(88s)` 改为 `vx-runtime(10s)`
- rmeta 可用时间从 17.77s 降至 ~10s
- 预估编译时间从 77s 降至 **~40s**

#### 2.5.3 `vx-extension` 依赖优化（34s → ~15s）

`vx-extension`（34s）依赖 `vx-runtime`，但分析发现它**实际不使用任何 vx-runtime 的 API**（grep 结果为空）。它只需要 `vx-core`、`vx-manifest`、`vx-paths`、`vx-args`。

**方案**：直接移除 `vx-extension` 对 `vx-runtime` 的依赖。

**预估收益**：`vx-extension` 不再等待 `vx-runtime`，从 34s 降至 **~15s**。

#### 2.5.4 `vx-env` 间接依赖优化

`vx-env` 依赖 `vx-resolver`，而 `vx-resolver` 依赖 `vx-runtime`。Phase 2 拆分后，这条链路自动受益：

```
Before: vx-runtime(88s) → vx-resolver(77s) → vx-env → vx-shim → vx-cli
After:  vx-runtime(10s) → vx-resolver(40s) → vx-env → vx-shim → vx-cli
```

#### 2.5.5 拆分总收益分析

```
完整依赖图（After Phase 2 + 2.5）：

时间轴 (秒)
0s     10s      30s          50s          70s       80s
|------|---------|-----------|-----------|---------|

vx-runtime (10s) ──→ providers 并行 (15-20s) ──┐
vx-runtime-http (30s，并行) ──────────────────┐ │
vx-runtime-archive (35s，并行) ──────────────┐│ │
vx-extension (15s, 不等 runtime) ───────────┐││ │
vx-resolver (40s, 等 runtime 10s) ─────────┐│││ │
                                            ↓↓↓↓ ↓
                                          vx-cli (~40s) → link (5s)
                                                                    = ~75s
```

### Phase 3: 合并同构 Provider（❌ 不采用）

> **决定**: 2026-02-18 评估后决定**不采用**此方案。
>
> **原因**:
> 1. **维护性差**: 合并 40+ 个 provider 到一个 crate 后，修改任意一个 provider 触发整个 `vx-providers-builtin` 重编译，代码导航和模块引用都变得更复杂
> 2. **收益极小**: 实测数据显示 providers 已在 384-481s 区间并行编译完毕（仅占 ~100s 窗口），合并后预估仅节省 10-20s，性价比极低
> 3. **不一致性**: 部分 provider 独立保留、部分合并到 builtin，新增 provider 时需要判断放哪里，增加心智负担
> 4. **更好的替代方案存在**: workspace-hack crate (cargo hakari) 可以零维护成本地统一依赖编译，或等待 Wild linker / -Z threads 获得更大收益
>
> **曾做的尝试**: 创建了 `vx-providers-builtin` crate 并迁移了 32 个 provider，但在编译验证阶段发现维护成本过高，已完全回滚。

#### 3.1 原始分析

57 个 provider 中，绝大多数是纯 manifest-driven 的 thin wrapper，代码结构完全一致（3-16KB）。每个独立 crate 有 ~2-5s 的固定开销（rustc 启动、元数据生成、codegen 初始化）。

#### 3.2 分类

| 类型 | Provider | 说明 |
|------|----------|------|
| **可合并** (~40+) | awscli, bat, brew, cmake, docker, fd, ffmpeg, fzf, gcloud, gh, hadolint, helm, imagemagick, jq, kubectl, make, meson, nasm, ninja, ollama, pre-commit, protoc, pwsh, release-please, rcedit, ripgrep, spack, starship, task, terraform, vite, winget, yq, dagu, prek, actrun, ... | 纯 manifest-driven，无额外依赖 |
| **独立保留** (~15) | node, go, uv, python, rust, bun, pnpm, yarn, deno, zig, java, msvc, dotnet, msbuild, nuget | 有自定义逻辑或额外依赖 |

#### 3.3 方案（已取消）

~~创建 `vx-providers-builtin` crate，合并所有纯 manifest-driven provider~~

**预估收益**: ~~40 个 crate × ~3s 固定开销 → 1 个 crate，节省 15-30s~~ → 实际收益远低于预期，不值得维护成本

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

### Phase 5: 重复依赖统一升级（依赖治理）

Bench 3 数据揭示了大量重复版本的第三方依赖，这些重复编译浪费了可观的 CPU 时间。

#### 5.0 重复依赖全景图

当前 workspace 存在 **10 组重复版本依赖**：

| 依赖 | 版本数 | 版本详情 | 总编译耗时 | 来源分析 |
|------|-------|---------|-----------|---------|
| **toml** | 3 | v0.8.23, v0.9.12, v1.0.1 | ~134s | 0.9=workspace, 0.8=figment, 1.0=msvc-kit |
| **toml_edit** | 3 | v0.22.27, v0.23.10, v0.24.1 | ~191s | 0.24=workspace, 0.22=toml 0.8, 0.23=rstest(dev) |
| **indicatif** | 2 | v0.17.11, v0.18.3 | ~68s | 0.17=workspace, 0.18=msvc-kit/turbo-cdn |
| **console** | 2 | v0.15.11, v0.16.2 | ~48s | 0.15=indicatif 0.17, 0.16=workspace/dialoguer |
| **reqwest** | 2 | v0.12.28, v0.13.2 | ~76s | 0.12=axoupdater, 0.13=workspace |
| **zip** | 2 | v7.4.0, v8.0.0 | ~28s+ | 7.0=workspace, 8.0=msvc-kit |
| **windows-sys** | 5 | v0.48/52/59/60/61 | ~133s | 多个第三方库跨代依赖 |
| **getrandom** | 3 | v0.2/0.3/0.4 | ~47s+ | ring(0.2), 中间层(0.3), 最新(0.4) |
| **socket2** | 2 | v0.5/v0.6 | ~19s+ | 旧 tokio 依赖链 |
| **hashbrown** | 2 | v0.14/v0.16 | 小 | indexmap 间接 |

**重复编译总浪费**: 保守估计 **~200-300s** 的 CPU 时间（虽然部分可并行）

#### 5.1 重复依赖来源链路分析

```
┌──────────────────────────────────────────────────────────────────┐
│                     vx workspace 依赖链路                        │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  [我们控制]           [第三方]              [间接依赖]            │
│                                                                  │
│  workspace            turbo-cdn ─→ figment ─→ toml 0.8.23       │
│  toml = "0.9.10" ───────────────────────────→ toml_edit 0.22    │
│                       msvc-kit ────────────→ toml 1.0.1          │
│                                                                  │
│  indicatif = "0.17" ─────────────────────→ console 0.15         │
│                       turbo-cdn ──→ indicatif 0.18 → console 0.16│
│                       msvc-kit ───→ indicatif 0.18 → console 0.16│
│                                                                  │
│  reqwest = "0.13" ───────────────────────→ (workspace 核心)      │
│                       axoupdater ─→ axoasset ─→ reqwest 0.12    │
│                                                                  │
│  zip = "7.0" ─────────────────────────→ (workspace 核心)         │
│                       msvc-kit ────────────→ zip 8.0             │
│                                                                  │
│  console = "0.16" ──────────────────────→ dialoguer              │
│                                            (但 indicatif 0.17   │
│                                             依赖 console 0.15!) │
└──────────────────────────────────────────────────────────────────┘
```

#### 5.2 可操作的统一升级方案

##### 优先级 1: 升级 indicatif 0.17 → 0.18（消除 indicatif + console 重复）

**当前状态**:
- workspace 声明 `indicatif = "0.17"`，依赖 `console 0.15`
- turbo-cdn 和 msvc-kit 使用 `indicatif 0.18`，依赖 `console 0.16`
- workspace 声明 `console = "0.16"`，但 indicatif 0.17 依然拉入 console 0.15

**方案**:
```toml
# Cargo.toml [workspace.dependencies]
indicatif = "0.18"   # 从 0.17 升级
# console = "0.16"   # 已是正确版本，无需改动
```

**API 变化**: indicatif 0.18 主要因 console 升级到 0.16 而 bump 大版本，API 基本不变

**前置条件**: `tracing-indicatif` 需要升级到 0.3.10+（支持 indicatif 0.18）。当前 pin 到 0.3.9 的注释说"0.3.10+ requires Rust 2024 Edition"，但 workspace 已经是 `edition = "2024"` + `rust-version = "1.93.0"`，所以可以安全升级。

**预估节省**: ~48s（消除 console 0.15 的 28.5s + indicatif 0.17 的 35s 中部分重叠 ≈ 节省一个版本的编译时间）

##### 优先级 2: 升级 toml 0.9 → 1.0（消除 toml 三版本共存）

**当前状态**:
- workspace 声明 `toml = "0.9.10"` (→ 0.9.12)，用 `toml_edit 0.24`
- figment (←turbo-cdn) 依赖 `toml 0.8.23`，用 `toml_edit 0.22`
- msvc-kit 依赖 `toml 1.0.1`，用 `toml_edit 0.24`

**方案**:
```toml
# Cargo.toml [workspace.dependencies]
toml = "1.0"         # 从 0.9.10 升级到 1.0
# toml_edit = "0.24" # 已是正确版本，无需改动
```

**API 变化**: toml 0.9 → 1.0 是同一 toml_edit 0.24 系列的自然升级，API 高度兼容

**效果**:
- workspace 的 toml 统一到 1.0，与 msvc-kit 共享 → 消除 toml 0.9.12 (59.5s)
- figment 的 toml 0.8.23 仍然存在（第三方无法控制）
- **净效果**: 3 版本 → 2 版本

**预估节省**: ~60s（消除 toml 0.9 的 59.5s）

##### 优先级 3: 升级 zip 7 → 8（消除 zip 重复）

**当前状态**:
- workspace 声明 `zip = "7.0"` (→ 7.4.0)
- msvc-kit 依赖 `zip 8.0.0`

**方案**:
```toml
# Cargo.toml [workspace.dependencies]  
zip = { version = "8.0", default-features = false, features = ["deflate", "zstd"] }
```

**注意**: zip 7→8 有 breaking changes，需要验证 `vx-runtime-archive` 和相关代码的兼容性。如果改动量大，可推迟。

**预估节省**: ~28s（消除 zip 7.4 的重复编译）

##### 优先级 4: 将 schemars 移到 feature flag 后面

**当前状态**: schemars + schemars_derive 编译耗时 ~54s，仅用于 `vx config schema` 命令

**方案**:
```toml
# crates/vx-config/Cargo.toml
[features]
default = []
schema = ["dep:schemars"]

[dependencies]
schemars = { version = "1.0", features = ["derive"], optional = true }
```

**预估节省**: 日常 `cargo build` 不编译 schemars，**节省 ~54s**

##### 优先级 5: 替换已废弃的 serde_yaml

**当前状态**: serde_yaml 编译耗时 55.6s，在 vx-config 中仅 2 处使用，且已标记 `deprecated`

**方案**: 替换为 `serde_yml`（社区维护的继任者）或直接用 JSON 格式

**预估节省**: 如果消除 YAML 支持则 **~55s**；如果换为更轻量库则 **~20-30s**

##### 不可操作项（第三方约束）

以下重复依赖由第三方 crate 间接引入，无法通过 workspace 升级解决：

| 依赖 | 原因 | 建议 |
|------|------|------|
| **toml 0.8** (figment) | figment 0.10 pin 到 toml 0.8，无法控制 | 等 figment 升级到 toml 1.0 |
| **reqwest 0.12** (axoupdater) | axoupdater→axoasset pin 到 reqwest 0.12 | 等 axoupdater 更新 |
| **windows-sys** (5 版本) | 各第三方库依赖不同 windows-sys 版本 | 无法控制，等生态统一 |
| **getrandom** (3 版本) | ring 依赖 0.2，其他依赖 0.3/0.4 | 无法控制 |
| **toml_edit 0.22** (figment) | figment→toml 0.8→toml_edit 0.22 | 跟随 figment 升级 |
| **toml_edit 0.23** (rstest) | rstest→proc-macro-crate→toml_edit 0.23 | 仅 dev-dep，不影响 release 构建 |

#### 5.3 统一升级实施计划

**批次 1（低风险，高收益）** — 预估节省 100-120s:
- [ ] 升级 `indicatif` 0.17 → 0.18
- [ ] 升级 `tracing-indicatif` 0.3.9 → 0.3.10+
- [ ] 升级 `toml` 0.9 → 1.0
- [ ] 移除 tracing-indicatif 的 pin 注释（MSRV 已满足 2024 Edition）

**批次 2（中风险，高收益）** — 预估节省 50-80s:
- [ ] 将 `schemars` 移到 optional feature flag
- [ ] 替换或移除 `serde_yaml`

**批次 3（需验证兼容性）** — 预估节省 ~28s:
- [ ] 升级 `zip` 7 → 8（需测试 vx-runtime-archive 兼容性）

#### 5.4 预估总收益

| 操作 | 预估节省 | 风险 | 推荐度 |
|------|---------|------|--------|
| indicatif 0.17→0.18 + tracing-indicatif | ~48s | ⭐ 低 | 🔥🔥🔥 |
| toml 0.9→1.0 | ~60s | ⭐ 低 | 🔥🔥🔥 |
| schemars optional feature | ~54s | ⭐ 中 | 🔥🔥🔥 |
| 替换 serde_yaml | ~55s | ⭐ 中 | 🔥🔥 |
| zip 7→8 | ~28s | ⭐⭐ 中 | 🔥🔥 |
| **合计** | **~180-245s** | | |

执行批次 1+2 后，预估总构建时间从 761s 降至 **~540-610s**。

## 综合预期效果

### 全量构建时间（dev profile）

| 阶段 | 措施 | 预估节省 | 累计时间 |
|------|------|---------|---------|
| Bench 2 基线 | Phase 2 + 2.5 完成后 | — | **793s** |
| Bench 3 基线 | 移除 providers 无用 reqwest | 32s | **761s** |
| Phase 4 | 精简 zip/chrono/reqwest features | 5-10s | ~752s |
| Phase 5 批次 1 | indicatif 0.18 + toml 1.0 | ~100-120s | ~640s |
| Phase 5 批次 2 | schemars optional + 替换 serde_yaml | ~50-80s | ~570s |
| Phase 5 批次 3 | zip 7→8 | ~28s | ~540s |
| ~~Phase 3~~ | ~~合并同构 provider~~ | ~~10-20s~~ | ❌ 不采用（维护性差，收益极小） |

**目标**: 全量构建 **525-600s**（从 Bench 2 基线 793s → 提升 25-34%）

> 注意：以上预估基于 Bench 3 实测的重复依赖编译耗时。由于 cargo 的并行编译调度，实际节省可能因 CPU 利用率变化而有偏差。保守估计节省 150-200s。

### 增量构建时间

Phase 1 + Phase 2 完成后，修改单个 provider 的增量构建预计 **5-15s**。
修改 `vx-cli` 核心逻辑的增量构建预计 **10-20s**（不触发 HTTP/归档重编译）。

## 向后兼容性

### Phase 1: 完全兼容

- linker 和 profile 变更不影响任何 API
- CI/release 构建使用 release profile，不受 dev profile 影响

### Phase 2: 内部重构（Provider 零改动）

- **Provider 的 `Cargo.toml` 不需要改**：`vx-runtime` 仍然是接口层的名字
- Provider 的 `use vx_runtime::` 导入路径完全不变
- `vx-cli` 新增 `vx-runtime-http` 和 `vx-runtime-archive` 依赖
- `vx-runtime-core` 将被合并回 `vx-runtime`（废弃独立 crate）

### Phase 2.5: 内部重构

- `vx-extension` 移除虚假依赖，不影响 API
- `self_update` 功能拆分到 `vx-self-update`，对外行为不变

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

### Phase 2: 按功能域拆分 vx-runtime（v0.8.0）

> **命名方案**: 方案 B — 按功能域命名
>
> | Crate | 职责 | 消费者 |
> |-------|------|--------|
> | `vx-runtime` | 轻量接口层（trait + types + registry） | 57 providers, vx-resolver, vx-extension |
> | `vx-runtime-http` | HTTP 下载 + 进度条 + CDN 加速 | vx-cli |
> | `vx-runtime-archive` | 归档解压（tar/zip/7z/xz/zst） | vx-cli |

#### Step 1: 创建 archive crate（已完成）
- [x] 创建 `vx-runtime-archive` crate，迁移归档处理逻辑
- [x] 添加 vx-runtime-archive 到 workspace.dependencies

#### Step 2: 精简 `vx-runtime` 为轻量接口层（✅ 完成）

将 `vx-runtime` 从重型门面（88s）精简为轻量接口层（~10s）：

- [x] 将 `GitHubReleaseOptions` 保留在 `vx-runtime`（纯数据结构）
- [x] 将 `fetch_github_releases` 逻辑保留在 `vx-runtime`（通过 HttpClient trait）
- [x] 将 `VersionCache`（bincode 轻量依赖）保留在 `vx-runtime`
- [x] 移除 `vx-runtime` 对 reqwest、indicatif、backon 等重型依赖
- [x] 移除 `vx-runtime` 对 archive 库（tar/zip/xz2/zstd/sevenz）的直接依赖
- [x] 将 `libloading` 改为 optional（feature = "plugin"）
- [x] 保留轻量 impls：RealCommandExecutor、RealFileSystem、RealPathProvider
- [ ] 统一 `RuntimeContext` 为唯一定义（合并 core 和 runtime 两套定义）— 推迟

#### Step 3: 创建 `vx-runtime-http` crate（✅ 完成）

- [x] 创建 `vx-runtime-http` crate
- [x] 迁移 `RealHttpClient` 实现（http_client.rs）
- [x] 迁移 `RealInstaller` 实现（installer.rs，含进度条、重试、CDN 加速）
- [x] 迁移 `create_runtime_context()` / `create_runtime_context_with_base()` 工厂函数
- [x] 迁移 `region.rs`（区域检测模块）
- [x] 迁移 cdn_tests.rs 到 vx-runtime-http/tests/
- [x] 添加 cdn-acceleration feature（turbo-cdn optional）
- [x] 添加到 workspace.dependencies

#### Step 4: 更新消费者（✅ 完成）
- [x] `vx-cli` 新增依赖 `vx-runtime-http`
- [x] `vx-cli` cdn-acceleration feature 指向 `vx-runtime-http/cdn-acceleration`
- [x] 更新 `registry.rs`：`create_runtime_context` 从 `vx_runtime_http` 导入
- [x] 更新 `tools.rs`：`create_runtime_context` 从 `vx_runtime_http` 导入
- [x] 更新 `handler.rs`：`create_runtime_context_with_base` 从 `vx_runtime_http` 导入
- [x] `cargo check --workspace` 全部通过
- [ ] `vx-extension` 移除 `vx-runtime` 依赖（实际不使用）— Phase 2.5
- [ ] 运行全量测试，确保无回归
- [ ] 基准测试对比

> **注意**: 此方案中 `vx-runtime-core` 将被废弃/合并回 `vx-runtime`，因为轻量接口层直接用 `vx-runtime` 这个名字。

### Phase 2.5: 拆分其他瓶颈 crate（v0.8.0）

- [x] 移除 `vx-extension` 对 `vx-runtime` 的虚假依赖
- [x] `vx-runtime-http` 复用 `vx-runtime::region` 模块（消除 region.rs 代码重复）
- [x] `vx-resolver` 依赖优化确认（Phase 2 后自动受益，无需额外修改）
- [x] 移除 `vx-runtime` 对 `vx-runtime-archive` 和 `vx-runtime-core` 的未使用 re-export 依赖
- [x] 移除 26 个 provider 中未使用的 `reqwest` 直接依赖（关键发现：providers 不使用 reqwest 但声明了依赖，导致等待 reqwest 编译完 586s 才能开始）
- [ ] 将 `vx-cli/src/commands/self_update.rs` 拆分到 `vx-self-update` crate（推迟：性价比低，reqwest 已通过 vx-runtime-http 共享编译）
- [x] 基准测试对比（见实测数据章节）- [x] 基准测试对比（见「实测数据」章节）
- [x] 移除 vx-runtime 对 vx-runtime-archive 和 vx-runtime-core 的无用 re-export

## 实测数据

### 测试环境

- **OS**: Windows 11, MSVC
- **Rust**: 1.93.0
- **CPU**: 多核（并行编译）
- **构建命令**: `cargo clean && cargo build --timings -p vx-cli`

### 全量构建时间

**总构建时间: 793s (13m 14s)** — 从 `cargo clean` 开始，含所有第三方依赖编译

> 注意：首次全量构建含编译 aws-lc-sys(478s), zstd-sys(298s), lzma-sys(230s) 等 C 依赖的 build.rs 时间。这些在增量构建中不会重复。

### 关键路径分析（Phase 2 + 2.5 优化后）

**内部 crate 编译时间（按 start 排序）**:

| Crate | Start | Duration | rmeta | rmeta Done | End |
|-------|-------|----------|-------|-----------|-----|
| vx-core | 125.8s | 22.1s | 5.4s | 131.1s | 147.9s |
| vx-ecosystem-pm | 125.8s | 42.7s | 7.1s | 132.9s | 168.5s |
| vx-system-pm | 133.0s | 45.0s | 16.3s | 149.3s | 178.0s |
| vx-cache | 134.8s | 29.0s | 9.4s | 144.2s | 163.8s |
| vx-paths | 147.8s | 59.1s | 8.1s | 155.9s | 206.9s |
| vx-manifest | 161.6s | 215.1s | 24.4s | 186.0s | 376.8s |
| **vx-runtime** | **186.2s** | **174.2s** | **34.3s** | **220.5s** | 360.5s |
| vx-version-fetcher | 220.7s | 25.3s | 5.4s | 226.0s | 246.0s |
| vx-config | 251.7s | 254.5s | 43.5s | 295.2s | 506.2s |
| vx-console | 285.5s | 30.0s | 6.7s | 292.1s | 315.4s |
| vx-resolver | 321.6s | 171.0s | 34.6s | 356.2s | 492.6s |
| vx-env | 357.0s | 38.9s | 7.5s | 364.5s | 395.9s |
| vx-bridge | 362.9s | 12.4s | 2.7s | 365.6s | 375.3s |
| vx-args | 362.9s | 29.9s | 6.7s | 369.6s | 392.8s |
| vx-migration | 369.7s | 51.7s | 11.5s | 381.2s | 421.4s |
| vx-shim | 369.8s | 12.4s | 5.5s | 375.3s | 382.2s |
| vx-extension | 375.3s | 62.4s | 14.0s | 389.3s | 437.8s |
| vx-metrics | 376.8s | 69.8s | 7.2s | 384.0s | 446.6s |
| vx-project-analyzer | 379.4s | 102.0s | 20.7s | 400.1s | 481.4s |
| vx-setup | 462.6s | 25.0s | 4.2s | 466.8s | 487.6s |
| **vx-runtime-http** | **614.7s** | **75.9s** | **46.2s** | 661.0s | 690.6s |
| **vx-cli** | **664.5s** | **127.6s** | **51.6s** | 716.0s | **792.0s** |

**53 个 Providers**:
- First start: **382.3s** (vx-provider-vscode)
- Last end: **679.9s** (vx-provider-msvc, 63.4s)
- 全部在 vx-runtime rmeta(220.5s) 完成后才开始（受 vx-config 等间接依赖阻塞）

**Top 5 最慢的第三方 crate**:

| Crate | Duration | 说明 |
|-------|----------|------|
| aws-lc-sys | 478.2s | C 依赖 build.rs，仅首次编译 |
| zstd-sys | 297.7s | C 依赖 build.rs，仅首次编译 |
| lzma-sys | 229.6s | C 依赖 build.rs，仅首次编译 |
| moxcms | 209.9s | |
| windows | 175.4s | |

### 关键路径图

```
时间轴 (秒)
0s       126s   162s   186s  220s        382s              665s    792s
|---------|------|------|-----|-----------|-----------------|-------|--|
  3rd party deps            |  vx-runtime (rmeta@220s)    |
           vx-core           |                              |
                  vx-manifest (rmeta@186s)                  |
                         vx-runtime (rmeta@220s)            |
                              vx-config (rmeta@295s)        |
                                   vx-resolver (rmeta@356s) |
                                        providers (382-680s)|
                                                  vx-runtime-http (615-691s, 并行)
                                                            vx-cli (665-792s)
```

**关键路径**: 3rd-party → vx-core → vx-manifest.rmeta → vx-runtime.rmeta → ... → vx-config.rmeta → vx-resolver.rmeta → providers → vx-cli → end

### 优化效果验证

#### ✅ Phase 2 拆分验证

1. **`vx-runtime-http` 与 providers 完全并行**：
   - vx-runtime-http start=614.7s, providers span=382-680s
   - 两者在 614-680s 区间重叠编译，验证并行度提升成功

2. **`vx-runtime-archive` 和 `vx-runtime-core` 已不编译**：
   - 移除无用 re-export 后，这两个 crate 不在依赖图中
   - 节省了约 51s 的编译依赖等待时间（archive 36s + core 15s）

3. **`vx-runtime` rmeta 在 220.5s 完成**，仅 34s rmeta 时间

#### 🔍 发现的新瓶颈

1. **Providers 在 382s 才开始**（而非预期的 220.5s 之后立即开始）：
   - 原因：providers 还依赖 `vx-config`(rmeta@295s)、`vx-resolver`(rmeta@356s) 等间接依赖
   - 建议：Phase 3 合并 providers 可减少固定开销

2. **`vx-config` 和 `vx-manifest` 编译时间异常长**：
   - vx-config: 254.5s duration（start=251.7s → end=506.2s）
   - vx-manifest: 215.1s duration
   - 这些时间包含了等待依赖 + 实际编译，需要更深入分析

3. **C 依赖 build.rs 极其耗时**（仅首次）：
   - aws-lc-sys(478s) + zstd-sys(298s) + lzma-sys(230s) = 1006s
   - 增量构建不受影响，但 CI clean build 需要关注

### Phase 3: 合并同构 Provider（❌ 不采用，已取消）

> 2026-02-18 决定不采用此方案。详见上方 Phase 3 的决定说明。
>
> 曾做的尝试已完全回滚（vx-providers-builtin crate 已删除）。

- [x] 审计所有 provider，确认哪些可以合并
- [x] ~~创建 `vx-providers-builtin` crate~~ → 已回滚删除
- [x] ~~逐步迁移 manifest-driven provider 到 builtin~~ → 已回滚
- [x] ~~更新 `vx-cli/src/registry.rs` 注册逻辑~~ → 已回滚
- [x] ~~清理已合并的独立 provider crate~~ → 已回滚
- ~~基准测试对比~~ → 不适用

### Phase 4: Feature 精简（v0.8.x）

- [x] 审计所有 provider 的下载格式，确认 zip feature 需求
- [x] 精简 zip features（移除 aes-crypto, bzip2, deflate64, ppmd, time，保留 deflate + zstd）
- [ ] 精简 chrono、reqwest 的 feature flags
- [ ] 基准测试对比

### Phase 5: 重复依赖统一升级（v0.8.x）

#### 批次 1: 低风险升级 ✅
- [x] 升级 `indicatif` 0.17 → 0.18（workspace Cargo.toml）
- [x] 升级 `tracing-indicatif` 0.3.9 → 0.3.10+，移除 MSRV pin 注释
- [x] 升级 `toml` 0.9.10 → 1.0（workspace Cargo.toml）
- [x] 验证 API 兼容性，修复编译错误
- [x] `cargo build --workspace` 通过
- [ ] 基准测试对比

#### 批次 2: 中风险优化 ✅
- [x] 将 `schemars` 改为 optional（feature = "schema"）
- [x] 评估替换 `serde_yaml` 的可行性 → 使用轻量 json_value_to_yaml() 替代
- [x] 移除 `serde_yaml` 依赖，实现内置 YAML 转换
- [ ] 基准测试对比

#### 批次 3: 需验证兼容性 ✅
- [x] 评估 `zip` 7 → 8 的 API breaking changes
- [x] 更新 `vx-runtime-archive` 中的 zip 用法
- [x] `cargo check --workspace` 通过
- [ ] 基准测试对比

## 实测数据

### Bench 2: Phase 2 + 2.5 完成后（移除无用依赖前）

**总构建时间**: 793s (13m14s) — `cargo build --timings -p vx-cli` (clean build)

**关键路径 crate 时序**:

| Crate | Start | Duration | End | rmeta | rmeta_done | 说明 |
|-------|-------|----------|-----|-------|------------|------|
| `vx-core` | 125.8s | 22.1s | 147.9s | 5.4s | 131.1s | 基础 trait |
| `vx-runtime` | 186.2s | 174.2s | 360.5s | 34.2s | 220.5s | 轻量接口层（rmeta 快） |
| `vx-config` | 251.7s | 254.5s | 506.2s | 43.5s | 295.2s | 配置管理（意外的重） |
| `vx-resolver` | 321.6s | 171.0s | 492.6s | 34.6s | 356.2s | 解析器 |
| `reqwest` (dev) | 572.3s | 99.6s | 671.8s | — | 586.8s | HTTP 客户端 |
| `vx-runtime-http` | 614.7s | 75.9s | 690.6s | 46.2s | 661.0s | HTTP 功能域 |
| `vx-cli` | 664.5s | 127.6s | 792.0s | 51.6s | 716.0s | 最终二进制 |

**Provider 时序分析**:

| 批次 | Start 范围 | 数量 | 等待原因 |
|------|-----------|------|----------|
| 第一批 | 382-466s | ~30 | 等待 `vx-runtime` rmeta (220.5s) + `vx-resolver` rmeta (356.2s) |
| 第二批 | 586.9s | ~26 | 等待 `reqwest` rmeta (586.8s) — **关键瓶颈** |

**关键发现**:

1. **26 个 provider 声明了未使用的 `reqwest` 依赖**：这些 provider 的源码中完全不使用 reqwest，但 Cargo.toml 中声明了依赖，导致 cargo 认为它们需要等待 reqwest 编译完成（586.8s）才能开始编译。这比 vx-runtime rmeta (220.5s) 多等了 **366s**！

2. **`vx-config` 异常耗时 254.5s**：需要后续调查原因（可能是重型宏或不必要的依赖）

3. **`vx-runtime` rmeta 已降至 34.2s**：证明 Phase 2 拆分有效，轻量接口层快速产出 rmeta

### Bench 3: 移除 provider 无用 reqwest 依赖后

**总构建时间**: **761s (12m41s)** — 从 793s 降低 32s (-4%)

**关键改善**:

| 指标 | Bench 2 (有 reqwest) | Bench 3 (无 reqwest) | 变化 |
|------|---------------------|---------------------|------|
| 总构建时间 | 793s | **761s** | -32s (-4%) |
| Providers 首个 start | 382s | **384s** | 持平 |
| Providers 最后 end | ~660s | **481s** | **-179s (-27%)** |
| vx-cli start | 664.5s | **613.9s** | **-50.6s** |

**Provider 时序分析**:
- 所有 53 个 providers 统一在 384-481s 区间完成编译
- 不再有"第二批等到 586s"的问题
- CPU 利用率显著提高

**重复依赖编译耗时（Top 15）**:

| 依赖 | Start | Duration | 说明 |
|------|-------|----------|------|
| toml_edit (v0.24.1) | 269.5s | **115s** | workspace 版本，vx-config 使用 |
| msvc-kit | 575.3s | 70.4s | 拉入 toml v1.0, indicatif 0.18, zip 8.0 |
| windows-sys (v0.48) | 13.7s | 68.5s | 被 ipconfig (reqwest 间接) 依赖 |
| turbo-cdn | 575.3s | 67.7s | 拉入 figment→toml 0.8, indicatif 0.18 |
| ring (v0.17) | 138.6s | 64.3s | reqwest 0.12 (axoupdater) 依赖 |
| toml (v0.9.12) | 66.5s | 59.5s | workspace 声明版本 |
| toml_edit (v0.22) | 254.8s | 57.3s | figment→toml 0.8 的间接依赖 |
| figment | 329.8s | 56s | turbo-cdn 依赖，拉入 toml 0.8 |
| serde_yaml | 242.7s | 55.6s | vx-config 依赖（deprecated） |
| schemars | 265.5s | 53.9s | vx-config 依赖 (JSON Schema) |
| toml (v0.8.23) | 327.6s | 52.6s | figment→turbo-cdn 间接依赖 |
| reqwest (v0.13) | 559.8s | 50.9s | workspace 版本 |
| indicatif (v0.17) | 242.7s | 35s | workspace 声明版本 |
| indicatif (v0.18) | 267.9s | 33.3s | msvc-kit/turbo-cdn 带入 |
| console (v0.15) | 226.7s | 28.5s | indicatif 0.17 依赖 |

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

### 方案 D: workspace-hack crate（✅ 已实施）

使用 `cargo hakari` 创建 workspace-hack crate 统一依赖编译。所有 workspace 成员共享统一的依赖 feature 组合，避免 cargo 在不同 crate 间重复编译同一依赖的不同 feature 变体。

**实施状态**（2026-02-18）：
- ✅ 安装 cargo-hakari v0.9.37
- ✅ 创建 `crates/workspace-hack/` crate
- ✅ 配置 `.config/hakari.toml`（platforms: windows-msvc, linux-gnu, apple-darwin x2）
- ✅ `cargo hakari generate` 生成统一依赖（~150 行依赖声明）
- ✅ `cargo hakari manage-deps -y` 自动给所有 80+ workspace crate 添加 workspace-hack 依赖
- ✅ `cargo hakari verify` 验证通过

**维护方式**：
- 每次修改 Cargo.toml 依赖后运行 `cargo hakari generate`
- CI 中添加 `cargo hakari generate --diff` 和 `cargo hakari manage-deps --dry-run` 校验

**预估收益**: 减少重复依赖编译，预估节省 **30-60s**（需 benchmark 验证）

### 方案 E: Wild 链接器（⏳ 等待 Windows 支持）

Wild 是完全用 Rust 编写的新一代链接器，比 mold 更快且支持增量链接。但目前仅支持 Linux，Windows 不可用。当 Wild 支持 Windows 后可评估替换 lld。

**跟踪状态**（截至 2026-02-18）：
- Linux: ✅ 可用（v0.8 已发布，含 LoongArch64 支持和性能提升）
- Windows: ❌ 暂不支持，无活跃开发计划
- macOS: ❌ 暂不支持
- 增量链接: ✅ v0.7+ 已支持
- 仓库: https://github.com/davidlattimore/wild
- **结论**: Wild 设计为 Linux ELF 专用链接器，短期内不太可能支持 Windows PE/COFF。我们当前使用 rust-lld 已足够，继续观望。

### 方案 F: 并行前端 -Z threads（⏳ 等待稳定化）

`RUSTFLAGS="-Z threads=8"` 可将编译器前端并行化，大型项目提速 30-50%。目前仍是 nightly 功能。当稳定化后可直接启用，零代码改动。

**跟踪状态**（截至 2026-02-18）：
- 状态: nightly-only（`-Z` flag），尚未进入 stabilize 流程
- 预期收益: 中大型项目编译时间减少 30-50%（小项目可能变慢）
- 注意: 内存使用增加约 35%，偶有 deadlock/ICE 报告
- nightly 中默认仍为单线程模式，需显式 `-Z threads=N` 启用
- 原计划 2024 年稳定化，但因线程安全和单线程回退性能问题推迟
- 跟踪 issue: https://github.com/rust-lang/rust/issues/113349
- **结论**: 预计 2026 年内可能稳定化，持续关注。可在开发者本地用 nightly 试用。

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
- `cargo-nextest` — 并行测试运行器（测试阶段提速 2-3×）
- `cargo machete` — 检测未使用的依赖
- Wild — 全 Rust 编写的实验性链接器（仅 Linux）

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-02-15 | Draft | 初始草案，基于 `cargo build --timings` 数据分析 |
| 2026-02-17 | Phase 1 Completed | 完成 Phase 1：添加 lld linker 配置到 Windows 和 Linux |
| 2026-02-17 | Phase 1 Fix | 移除 macOS lld 配置（兼容性问题，使用默认 ld64） |
| 2026-02-17 | Phase 2 Started | 创建 vx-runtime-core 和 vx-runtime-archive crate |
| 2026-02-17 | Phase 2 Progress | vx-runtime 集成 vx-runtime-core 和 vx-runtime-archive，作为门面 crate |
| 2026-02-17 | Phase 2 Progress | 添加 workspace dependencies，导出 RuntimeContext/ExecutionContext |
| 2026-02-17 | Phase 2 Note | Provider 迁移推迟到后续 PR（依赖协调复杂度高） |
| 2026-02-17 | Phase 2 Strategy | 确定 provider 迁移策略：先补充 core 的 provider 支持能力，再批量迁移 |
| 2026-02-17 | Research Update | 添加互联网调研：Wild linker、-Z threads、cargo-nextest、nnethercote 最新优化 |
| 2026-02-17 | Naming Redesign | 采用方案 B（按功能域命名），vx-runtime 为轻量接口层，新增 vx-runtime-http |
| 2026-02-17 | Phase 2.5 Added | 新增 Phase 2.5：拆分 vx-cli self-update、移除 vx-extension 虚假依赖、优化 vx-resolver |
| 2026-02-17 | Phase 2 Complete | Phase 2 核心完成：vx-runtime-http 创建并迁移 HTTP/Installer/Context，vx-runtime 精简为轻量接口层，workspace 全量编译通过 |
| 2026-02-17 | Phase 2.5 Partial | Phase 2.5 部分完成：移除 vx-extension 虚假依赖、region.rs 去重、vx-resolver 自动受益；self-update 拆分推迟 |
| 2026-02-17 | Bench 2 | 首次全量基准测试：793s (clean build)，发现 26 个 providers 无用 reqwest 依赖导致 366s 等待 |
| 2026-02-17 | Dep Cleanup | 移除 vx-runtime 对 vx-runtime-archive/core 的未使用 re-export；移除 26 个 providers 的无用 reqwest 依赖 |
| 2026-02-17 | Phase 2.5 Cleanup | 移除 vx-runtime 对 vx-runtime-archive 和 vx-runtime-core 的无用 re-export 依赖 |
| 2026-02-17 | Benchmark Complete | 全量构建基准测试完成（793s），记录详细 timings 数据和关键路径分析 |
| 2026-02-17 | Bench 3 Complete | Bench 3 完成：761s（-32s/-4%），所有 providers 统一在 384-481s 完成 |
| 2026-02-17 | Phase 5 Analysis | 新增 Phase 5：重复依赖统一升级分析，识别 10 组重复版本，预估节省 180-245s |
| 2026-02-18 | Phase 5 Batch 1 ✅ | 完成批次 1：升级 indicatif 0.18 + toml 1.0 |
| 2026-02-18 | Phase 5 Batch 2 ✅ | 完成批次 2：schemars 改为 optional，移除 serde_yaml |
| 2026-02-18 | Phase 5 Batch 3 ✅ | 完成批次 3：升级 zip 7→8 |
| 2026-02-18 | Phase 4 Partial ✅ | 完成 Phase 4 部分：精简 zip features（移除 5 个不必要 feature） |
| 2026-02-18 | Phase 3 ❌ Cancelled | Phase 3 合并 provider 方案不采用——维护性差、收益极小(~10-20s)、已回滚全部改动 |
| 2026-02-18 | Future Tracking | 记录等待项：Wild linker Windows 支持、-Z threads 稳定化 |
| 2026-02-18 | 方案 D ✅ Hakari | 实施 cargo hakari workspace-hack 方案：安装 v0.9.37，生成统一依赖，所有 80+ crate 已自动添加依赖 |
| 2026-02-18 | 方案 E/F 更新 | 更新 Wild linker 状态（v0.8 发布，仍不支持 Windows）；-Z threads 仍 nightly-only，原计划 2024 稳定化已推迟 |
