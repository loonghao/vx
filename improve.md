### 架构诊断书（vx / 分支：`feature/version-syntax-and-constraints`）

> 用途：可用于评审与重构决策的“架构诊断书”，强调 **证据链、风险分级、渐进式改造路线**，避免泛泛而谈。

---

### 1) 结论（供评审快速决策）
- **需要重构，但不建议“大翻新”**：核心链路（CLI → Resolver/Executor → Runtime/Provider → Paths/Installer）已经具备清晰分层与可扩展骨架；主要问题集中在 **“事实来源分裂（Source of Truth drift）”** 与 **“可插拔机制落地不一致（manifest/extension/config 三套并行演进）”**。
- 建议采用 **渐进式改造路线**：先消除高风险安全与一致性问题（S0/S1），再做体验/性能/可维护性优化（S2/S3）。

---

### 2) 系统边界图（System Boundary）

#### 2.1 边界与外部依赖
- **用户输入边界**：命令行参数、环境变量（如 `VX_HOME`）、工作目录（影响配置/lock 探测）。
- **OS 边界**：系统 `PATH` 查找工具、进程创建与退出码传递（`std::process::exit`）。
- **网络边界**：版本列表/下载（Provider 的 `fetch_versions()`、Installer 下载）。
- **文件系统边界**：`~/.vx` 目录结构、cache/lock/config 文件读写、extension 扫描目录。

#### 2.2 逻辑组件图（简化）
- **`vx`(bin)** → `vx-cli`(命令解析/路由/展示)
  → `vx-resolver`(解析依赖图/选择可执行/缓存)
  → `vx-runtime`(ProviderRegistry + Runtime trait + Context 注入)
  → `vx-providers/*`(各 runtime 的版本获取/安装/执行)
  → `vx-installer`(下载/解压/校验) + `vx-paths`(目录/寻址/查找)
  → `vx-config`(vx.toml + 继承/迁移/安全/telemetry/container 等)
  → `vx-extension`(扩展发现/加载/执行，另一条可扩展路径)

---

### 3) 关键执行链路（Evidence-backed “Happy Path”）

#### 3.1 入口与委托
- **证据**：`src/main.rs` 直接委托给 `vx_cli::main()`（单一入口、可测性较好）。

#### 3.2 CLI → Executor（动态转发）
- **证据**：`crates/vx-cli/src/commands/execute.rs` 构造 `ResolverConfig`，并通过 `Executor::with_registry_and_context` 执行 `execute_with_version()`；失败时透传退出码（`std::process::exit`）。

#### 3.3 Resolver/Executor 的职责边界（解析 → 安装保证 → 再解析 → 执行）
- **证据**：`crates/vx-resolver/src/executor.rs` 将执行分为：平台检查、依赖/约束检查、（带磁盘 cache 的）解析、确保版本安装、自动安装缺失项、再解析后执行。
- **额外证据**：`crates/vx-resolver/src/resolution_cache.rs` 的 cache key 绑定 \(os, arch, cwd, args\) 并对 `vx.toml` / `vx.lock` 做 digest 指纹，用于加速重复解析（但也引出一致性风险，见风险条目）。

---

### 4) 证据清单（可追溯到文件/符号）

#### 4.1 模块边界与依赖拓扑
- **workspace 拓扑**：根 `Cargo.toml` 明确列出核心 crate 与大量 `vx-providers/*`（可扩展面广，但注册/元数据一致性风险更高）。
  - 证据：`Cargo.toml` 的 `[workspace].members` 与 `[workspace.dependencies]`（包含 `vx-resolver`、`vx-runtime`、`vx-paths`、`vx-config`、`vx-extension` 等）。
- **ProviderRegistry**：`crates/vx-runtime/src/registry.rs` 用 `RwLock` 管 providers 与 runtime-name→provider-index cache（并发读多写少场景合理）。
- **路径/存储结构**：`crates/vx-paths/src/lib.rs` 展示 `~/.vx/{store,npm-tools,pip-tools,envs,bin,cache,config,tmp}`；并支持 `VX_HOME` 覆盖（与存储架构规则一致）。
  - 证据：`VxPaths::new()` 读取 `VX_HOME`（`crates/vx-paths/src/lib.rs`），`PathConfig::from_env()` 支持更多 `VX_*_DIR`（`crates/vx-paths/src/config.rs`）。

#### 4.2 Provider 注册与扩展机制现状
- **当前 Provider 注册方式：编译期静态注册**
  - 证据：`crates/vx-cli/src/registry.rs` 中 `create_registry()` 逐一 `registry.register(vx_provider_xxx::create_provider())`。
- **manifest 体系存在，但未进入主链路**
  - 证据：存在 `crates/vx-manifest`、多个 RFC（`docs/rfcs/0012-provider-manifest.md`、`0013-manifest-driven-registration.md`），且 `provider.toml` 文件存在；但在代码中几乎找不到 `vx_manifest::` / `ManifestLoader` 被 CLI/Resolver/Runtime 实际使用（本次搜索只命中 manifest crate 与其测试/RFC）。
- **Extension 体系是另一条“可插拔路径”**
  - 证据：`crates/vx-extension/src/discovery.rs` 扫描 `~/.vx/extensions`、`~/.vx/extensions-dev`、以及项目 `.vx/extensions`，并按优先级排序（dev > project > user）。这对扩展能力很强，但也放大了供应链/执行边界风险（见 S0）。

#### 4.3 配置体系现状：存在“并行演进/双轨制”
- **`vx-config` crate**：宣称是 `vx.toml` 的 typed config，并提供 inheritance/migration/security/telemetry/container 等模块（`crates/vx-config/src/lib.rs`）。
- **`vx-cli` 内部又存在一套 `VxConfig`**：`crates/vx-cli/src/config.rs` 定义 `VxConfig`/`SetupTask` 等，并内含 OS 命令执行逻辑（`SetupTask::is_satisfied()` 直接跑 shell）。
- **CLI 命令层确实在多处引用 `vx_config::...`**：例如 `commands/setup.rs`、`commands/services.rs`、`commands/container.rs` 等（搜索命中）。

=> 这意味着 **同一概念（项目配置 / vx.toml）存在至少两套结构与解析语义**，属于典型一致性隐患与重构信号（见 S1）。

#### 4.4 测试规范落地情况（与规则冲突的“硬证据”）
规则要求：单元测试应放到各 crate 的 `tests/` 目录，**禁止 `#[cfg(test)] mod tests` 内联**。

但目前在多个 `src/*.rs` 中仍存在内联测试，例如：
- **证据 1**：`crates/vx-manifest/src/loader.rs` 存在 `#[cfg(test)] mod tests`。
- **证据 2**：`crates/vx-cli/src/tracing_setup.rs` 存在内联测试（`test_init_tracing()`）。
- **证据 3**：`crates/vx-config/src/parser.rs` 存在大量内联测试（解析/兼容性/别名等）。

=> 这不是功能 bug，但属于“工程治理债”，会降低测试分层清晰度与长期可维护性（见 S2）。

---

### 5) 风险分级（Risk Ranking with Evidence & Impact）
> 分级口径（建议用于评审打分）：**影响面（Impact）× 发生概率（Likelihood）× 可检测性（Detectability，越难检测越高风险）**。
> S0=阻断/安全事故级；S1=高风险一致性/扩展性缺陷；S2=中风险工程债；S3=低风险优化项。

#### S0（最高优先级）
- **S0-1：扩展/远程配置的供应链与执行边界风险**
  - **证据**：`vx-extension` 会从用户/项目目录扫描扩展（`crates/vx-extension/src/discovery.rs`）；`vx-config` 的 `InheritanceManager` 支持 `extends` 从 `github:` 或 `vx:` 远程拉取，并允许只给 version、不强制 hash（`crates/vx-config/src/inheritance.rs` 的 `parse_extends()`/`resolve_preset_url()`）。
  - **风险**：当扩展/远程 preset 与命令执行能力组合时，可能导致 **隐式代码执行面扩大**（尤其是项目级 `.vx/extensions` 优先级较高）。
  - **建议**：短期引入“安全默认值”（见路线图 Phase 0/1）：对远程 preset **强制校验（sha256 或签名）**，对 extension **显式启用/白名单**，并在 CLI 输出中展示来源与校验状态。

#### S1（需要尽快解决，否则扩展会变慢/变危险）
- **S1-1：Provider/Runtime 元数据与注册机制“事实来源分裂”**
  - **证据**：providers 目前在 `vx-cli/src/registry.rs` 静态注册；同时存在 `provider.toml` 与 `vx-manifest` crate、manifest-driven RFC，但未进入执行主链路。
  - **后果**：新增 Provider 需要改代码+重新编译；同时 `provider.toml` 可能变成“文档/摆设”，长期会出现 **实现与声明不一致**。

- **S1-2：配置系统“双轨制”（`vx-config` vs `vx-cli::config::VxConfig`）**
  - **证据**：`vx-config` 提供 typed config 与继承/迁移；`vx-cli` 又有另一套 `VxConfig` 结构与示例，并且命令层同时使用 `vx_config`（多命令文件命中）。
  - **后果**：同一 `vx.toml` 的解析/字段语义可能在不同命令间不一致，出现“某些命令读 A 配置，另一些读 B 配置”的灰色 bug。

- **S1-3：依赖解析/约束来源可能重复**
  - **证据**：`vx-resolver` 内有 `RuntimeMap`/`RuntimeSpec`；`vx-runtime::Runtime` 也有 `dependencies()`；`Executor` 注释表明“supplements RuntimeMap-based resolution with provider-specific constraints”（`crates/vx-resolver/src/executor.rs`）。
  - **后果**：依赖链/约束有机会出现冲突（例如 alias/版本约束在不同层重复定义），引入难以定位的问题。

#### S2（中期治理债，建议在 S0/S1 之后处理）
- **S2-1：测试分层规则未严格落地**
  - **证据**：多个 `src/*.rs` 内联 `#[cfg(test)] mod tests`（`vx-manifest`/`vx-cli`/`vx-config` 等）。
  - **后果**：测试资产分布分散，不利于按 crate/层次做覆盖率与测试策略演进；也与团队规范冲突，影响协作。

- **S2-2：库代码错误类型一致性**
  - **证据**：workspace 依赖同时包含 `anyhow` 与 `thiserror`；且 `vx-resolver`/`vx-cli` 中大量 `anyhow::anyhow!` 路径（需要进一步统一“库 vs 应用”的边界）。
  - **后果**：错误可编程处理能力弱、上下文链条不稳定，长期会拖慢诊断与可观测性建设。

#### S3（低风险优化项）
- **S3-1：可观测性粒度与一致性**
  - **证据**：`vx-cli/src/tracing_setup.rs` 已建立基础 tracing；但核心链路缺少统一 span 语义（例如“resolve/install/execute”阶段指标与事件）。
  - **建议**：在不改变行为的前提下补齐关键 span/fields，配合 cache hit/miss 指标。

---

### 6) 渐进式改造路线（不破坏主链路，逐步收敛事实来源）

#### Phase 0（1–3 天）：止血 + 建立“可控演进护栏”
- **目标**：降低 S0/S1 的演进阻力，防止问题继续扩散。
- **动作**：
  - **安全默认值**：远程 preset（extends）在未提供 `sha256` 时输出强 warning/可配置为 hard-fail；extension 扫描结果输出来源与优先级（dev/project/user）。
  - **测试规范护栏**：新增 CI/脚本检查，禁止 `src/` 下出现 `#[cfg(test)] mod tests`（先允许白名单，逐步清零）。

#### Phase 1（1–2 周）：收敛 Provider 注册机制（manifest 驱动但保留回退）
- **目标**：让 Provider “声明即事实来源”，同时不影响现有用户。
- **动作**：
  - 引入 **manifest-driven registration（可 feature gate）**：优先从 providers 目录/内置资源加载 `provider.toml`，生成注册清单；与现有 `vx-cli/src/registry.rs` 静态注册并行，形成“新旧双栈可切换”。
  - 明确 `provider.toml` 的权威字段：name/alias/platform/support/依赖/下载策略等，避免与代码重复表达。

#### Phase 2（2–4 周）：统一配置系统“单一事实来源”
- **目标**：`vx.toml` 的解析与语义只由一个 crate 定义（建议以 `vx-config` 为主）。
- **动作**：
  - 迁移 `vx-cli/src/config.rs` 的 `VxConfig`/`SetupTask`：要么变成对 `vx-config` 的 re-export/adapter，要么完全删除，CLI 只消费 `vx-config::types::VxConfig`。
  - 对外给出明确的配置分层与优先级（项目/用户/环境变量/远程 preset），并让解析入口唯一。

#### Phase 3（4–8 周）：依赖解析与约束“单模型化”
- **目标**：Runtime 依赖/别名/约束不要在 `RuntimeMap`、Provider 代码、manifest 三处各写一遍。
- **动作**：
  - 建立“Runtime Catalog”概念：由 Provider(代码) + manifest(声明) 合成（或明确谁是 source of truth），Resolver 只消费 catalog。
  - 让版本约束/依赖拓扑排序在一个地方完成（贴合你们的“动态执行器”规则：解析 → 拓扑排序 → 安装 → 执行）。

#### Phase 4（持续演进）：可观测性/缓存正确性/安全闭环
- **目标**：让线上/用户侧问题可定位、可量化。
- **动作**：
  - 标准化 span：`resolve_graph`、`ensure_installed`、`install_runtimes`、`execute_process`；输出 cache 命中率、安装耗时、失败原因分布。
  - 复核 `ResolutionCacheKey` 的“正确性边界”：是否需要纳入 registry/provider 版本、extension 状态、关键 env 等（避免“缓存命中但语义已变”的幽灵问题）。

---

### 7) 评审时建议关注的“决策点”（必须拍板）
- **配置权威**：`vx.toml` 的 schema 与解析逻辑，最终以 `vx-config` 为唯一入口，还是允许 CLI 自定义子集？
- **Provider 权威**：Provider 元数据到底写在代码（trait impl）还是 manifest（`provider.toml`）？如果两者并存，谁覆盖谁、如何校验一致性？
- **扩展安全模型**：extension 是否默认启用？项目级 `.vx/extensions` 是否需要显式允许（避免“clone 仓库即执行扩展”的隐患）？

---

### 8) 总结（4 句以内）
当前架构主链路分层清晰且可扩展，但存在 **Provider/manifest、配置系统、依赖模型** 三处“事实来源分裂”，会在功能膨胀期显著放大一致性与安全风险。证据显示 `provider.toml`/`vx-manifest` 尚未进入实际注册与执行链路，而 CLI 侧静态注册与双套 `VxConfig` 正在制造长期漂移。建议先以安全与一致性为先（S0/S1），用 feature gate 推进 manifest 驱动注册与配置单一入口，再逐步单模型化依赖解析。这样可以在不破坏现有用户体验的前提下，把架构债务收敛到可控范围。
