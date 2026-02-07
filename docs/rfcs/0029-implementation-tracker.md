# RFC 0029: Implementation Tracker

## 总体进度

| Phase | 描述 | 状态 | 完成度 | 目标版本 | 预计工期 |
|-------|------|------|--------|----------|----------|
| Phase 1 | 核心重构（Pipeline 架构） | 进行中 | 95% | v0.7.0 | 2 周 |
| Phase 2 | ManifestRegistry 拆分 | ✅ 完成 | 100% | v0.7.0 | 1 周 |
| Phase 3 | 错误处理改进 | 进行中 | 95% | v0.7.1 | 1 周 |
| Phase 4 | Fallback Chain 与传统配置支持 🆕 | 待开始 | 0% | v0.7.1 | 1.5 周 |
| Phase 5 | Shell 集成与自动切换 🆕 | 待开始 | 0% | v0.8.0 | 1.5 周 |
| Phase 6 | 版本管理增强 🆕 | 待开始 | 0% | v0.8.0 | 1 周 |
| Phase 7 | 任务系统增强 🆕 | ✅ 完成 | 100% | v0.8.0 | 1 周 |
| Phase 8 | 高级特性 | 待开始 | 0% | v0.9.0 | 2 周 |

## 借鉴来源

| 特性 | 借鉴项目 | 优先级 |
|------|----------|--------|
| Pipeline 架构 | Cargo, uv | P0 |
| 错误分类体系 | Volta | P0 |
| Fallback Chain | Volta | P1 |
| 传统配置文件支持 | mise, fnm | P1 |
| 自动版本切换 | fnm | P1 |
| Shell 集成 | fnm | P1 |
| 版本过期检测 | proto | P2 |
| 安装钩子 | proto | P2 |
| 任务系统增强 | mise | P2 |

## 详细进度

### Phase 1: 核心重构 (Pipeline 架构)

#### 1.1 ExecutionPlan 定义
- [x] 定义 `ExecutionPlan` 结构体
- [x] 定义 `PlannedRuntime` 结构体 (避免与 ResolvedVersion 冲突)
- [x] 定义 `VersionResolution` 枚举 (避免与 ResolvedVersion 冲突)
- [x] 定义 `InstallStatus` 枚举
- [x] 定义 `VersionSource` 枚举
- [x] 定义 `ExecutionConfig` 结构体
- [x] 定义 `ProxyConfig` 结构体 (RFC 0028)
- [x] 单元测试 (8 tests)

#### 1.2 ResolveStage 实现
- [x] 定义 `Stage` trait (generic, async)
- [x] 定义 `ResolveRequest` 输入类型
- [x] 定义 `WithDepRequest` 类型
- [x] 实现 `ResolveStage` (delegates to existing Resolver)
- [x] 实现 `resolve_version` 逻辑 (explicit → project config → latest)
- [x] 实现 `determine_source` 逻辑
- [x] 实现 `build_plan` 映射 (ResolutionResult → ExecutionPlan)
- [x] 实现 `VersionStrategy` 配置 — 已在 `version/strategy.rs` 中实现（SemverStrategy, Pep440Strategy, GoVersionStrategy）
- [ ] 实现 `LatestBehavior` 处理 — 推迟至 Phase 8.1 (Locked 模式)
- [x] 单元测试 (20 tests)

#### 1.3 EnsureStage 实现
- [x] 实现 `EnsureStage` (wraps InstallationManager)
- [x] 处理 auto-install 禁用
- [x] 安装依赖 (topological order)
- [x] 安装 primary runtime
- [x] 安装 --with 注入 runtimes
- [x] Re-resolve 获取 executable 路径
- [x] 处理安装失败 (EnsureError)
- [x] 单元测试 (4 tests)

#### 1.4 PrepareStage 实现
- [x] 实现 `PrepareStage` (wraps EnvironmentManager)
- [x] 定义 `PreparedExecution` 输出类型
- [x] 环境变量注入
- [x] Proxy runtime 处理 (RFC 0028) — `try_proxy_execution()` 集成到 PrepareStage
- [ ] --with dependency PATH injection — deferred to Phase 2
- [x] 单元测试 (2 tests)

#### 1.5 ExecuteStage 实现
- [x] 实现 `ExecuteStage` (wraps build_command/run_command)
- [x] 命令执行 (compatibility bridge via ResolutionResult)
- [x] 退出码处理
- [x] 超时支持
- [x] 单元测试 (3 tests)

#### 1.6 ExecutionPipeline 编排
- [x] 实现 `ExecutionPipeline` orchestrator
- [x] Stage 组合: Resolve → Ensure → Prepare → Execute
- [x] Pipeline 也实现 `Stage<ResolveRequest, i32>`
- [x] 错误自动包装为 `PipelineError`
- [x] 单元测试 (2 tests)

#### 1.7 错误类型 (提前自 Phase 3)
- [x] 定义 `ResolveError` (7 variants)
- [x] 定义 `EnsureError` (6 variants)
- [x] 定义 `PrepareError` (5 variants)
- [x] 定义 `ExecuteError` (4 variants)
- [x] 定义 `PipelineError` (5 variants, wraps all stages)
- [x] 单元测试 (8 tests)

#### 1.8 迁移现有代码
- [x] 迁移 `Executor::execute_with_with_deps` 到 Pipeline 架构
- [x] 清理死代码（5 个废弃方法）
- [ ] E2E 测试

### Phase 2: ManifestRegistry 拆分

#### 2.1 ManifestLoader
- [x] 创建 `manifest/loader.rs`（`ManifestStore` — 封装 `vx_manifest::ManifestLoader`）
- [x] 迁移清单加载逻辑（`load_from_directory`, `load_from_manifests`, `get`, `names`, `find_runtime`）
- [x] 单元测试（3 tests: load_from_directory, load_from_manifests, find_runtime）

#### 2.2 ManifestIndex
- [x] 创建 `manifest/index.rs`（`ManifestIndex` — HashMap 索引）
- [x] 实现元数据索引（`from_manifests`, `get_runtime`, `get_provider`）
- [x] 实现别名解析（`resolve_alias`, `has_runtime`）
- [x] 实现平台约束合并（`PlatformConstraint::intersect()` 取交集替代 `or_else`）
- [x] 单元测试（7 tests: basic_lookup, alias_resolution, platform_intersection, no_constraint, multiple_providers, supported_runtimes, provider_metadata）

#### 2.3 ProviderBuilder
- [x] 创建 `manifest/builder.rs`（`ProviderBuilder`）
- [x] 实现 `BuildResult`（含 `registry`, `warnings`, `errors`）
- [x] 返回 warnings 和 errors（`BuildWarning`, `BuildError` 结构化类型）
- [x] 单元测试（5 tests: build_with_factory, build_missing_factory, build_mixed, build_from_factories, factory_names）

#### 2.4 CLI 集成
- [x] `ManifestRegistry` 重构为 `ManifestStore` + `ProviderBuilder` 组合（向后兼容）
- [x] 添加 `build_registry_with_result()` 方法返回 `BuildResult`
- [x] 添加 `build_index()` 方法返回 `ManifestIndex`
- [x] `PlatformConstraint::intersect()` 添加到 vx-manifest（取代 `or_else`）
- [x] 在 `create_registry()` 中处理 `BuildResult.errors`（`build_registry_with_result()` + `store_build_diagnostics()`）
- [x] 添加 `vx info --warnings`（显示 `BuildDiagnostics` 错误/警告 + 彩色输出）

### Phase 3: 错误处理改进

#### 3.1 错误类型定义 (借鉴 Volta ErrorKind)
> **Note**: 核心错误类型已在 Phase 1.7 中提前完成（`pipeline/error.rs`）

- [x] 定义 `ResolveError` (7 variants: RuntimeNotFound, VersionNotFound, DependencyCycle, PlatformNotSupported 等)
- [x] 定义 `EnsureError` (6 variants: InstallFailed, DependencyFailed, DownloadFailed 等)
- [x] 定义 `PrepareError` (5 variants: NoExecutable, ProxyNotAvailable 等)
- [x] 定义 `ExecuteError` (4 variants)
- [x] 定义 `PipelineError` (5 variants, wraps all stages)

#### 3.2 错误迁移
- [x] 迁移 `Executor` 错误（executor.rs: 5 处 anyhow::anyhow! → PrepareError/ResolveError/EnsureError）
- [x] 迁移 `InstallationManager` 错误（installation.rs: 7 处 → EnsureError）
- [x] 迁移 `FallbackInstaller` 错误（fallback.rs: 11 处 → EnsureError）
- [x] 迁移 `CommandBuilder` 错误（command.rs: 1 处 → ExecuteError::Timeout）
- [x] 迁移 `BundleExecutor` 错误（bundle.rs: 2 处 → ExecuteError）
- [ ] 迁移 `RuntimeIndex` 错误（runtime_index.rs: 7 处 anyhow::Result，需定义 RuntimeIndexError）

#### 3.3 CLI 错误输出
- [x] 改进错误格式化（`error_handler.rs`：PipelineError downcast + 分类输出）
- [x] 添加依赖链上下文（`format_generic_error` 显示 anyhow error chain）
- [x] 添加建议修复步骤（每个错误变体提供 `vx install`/`vx list` 等修复命令）
- [x] 测试验证（20 个测试覆盖所有 PipelineError 变体 + 泛型错误）

### Phase 4: Fallback Chain 与传统配置支持 🆕

*借鉴 Volta/mise/fnm 的版本解析策略*

#### 4.1 版本解析 Fallback Chain (借鉴 Volta)
- [ ] 定义 `VersionResolver` trait
- [ ] 实现 `ExplicitVersionResolver` (命令行参数)
- [ ] 实现 `ProjectConfigResolver` (vx.toml)
- [ ] 实现 `UserDefaultResolver` (用户默认)
- [ ] 实现 `InstalledLatestResolver`
- [ ] 实现 `RemoteLatestResolver`
- [ ] 实现 `VersionFallbackChain` 编排器
- [ ] 单元测试

#### 4.2 传统配置文件支持 (借鉴 mise/fnm)
- [ ] 实现 `LegacyConfigResolver`
- [ ] 支持 `.nvmrc` (Node.js)
- [ ] 支持 `.node-version` (Node.js)
- [ ] 支持 `.python-version` (Python)
- [ ] 支持 `.ruby-version` (Ruby)
- [ ] 支持 `.go-version` (Go)
- [ ] 支持 `rust-toolchain.toml` (Rust)
- [ ] 支持 `.tool-versions` (asdf 兼容)
- [ ] 支持 `package.json` volta 字段 (Volta 兼容)
- [ ] 添加配置项 `[resolver.legacy]`
- [ ] 单元测试

#### 4.3 用户默认版本
- [ ] 创建 `~/.vx/defaults.toml`
- [ ] 添加 `vx config set default.node 20` 命令
- [ ] 添加 `vx config get default.node` 命令
- [ ] 文档更新

### Phase 5: Shell 集成与自动切换 🆕

*借鉴 fnm 的极速启动和自动切换*

#### 5.1 Shell Hooks 实现
- [ ] 创建 `vx-shell` crate
- [ ] 实现 `ShellHooks` 模块
- [ ] 实现 Bash 集成
- [ ] 实现 Zsh 集成
- [ ] 实现 Fish 集成
- [ ] 实现 PowerShell 集成

#### 5.2 自动版本切换
- [ ] 实现 `--use-on-cd` 功能
- [ ] 实现目录钩子
- [ ] 添加 `vx env --shell <shell>` 命令
- [ ] 添加配置项 `[shell]`

#### 5.3 Shell 初始化命令
- [ ] 实现 `vx shell init bash`
- [ ] 实现 `vx shell init zsh`
- [ ] 实现 `vx shell init fish`
- [ ] 实现 `vx shell init powershell`
- [ ] 文档更新

#### 5.4 性能优化
- [ ] 建立 shim 启动时间基准
- [ ] 优化冷启动路径
- [ ] 目标: < 5ms
- [ ] 基准测试

### Phase 6: 版本管理增强 🆕

*借鉴 proto 的版本管理功能*

#### 6.1 版本过期检测
- [ ] 实现 `OutdatedChecker`
- [ ] 实现 `OutdatedReport` 结构
- [ ] 支持安全更新检测
- [ ] 支持 LTS 版本检测
- [ ] 单元测试

#### 6.2 CLI 命令
- [ ] 添加 `vx outdated` 命令
- [ ] 添加 `vx upgrade [runtime]` 命令
- [ ] 添加 `vx upgrade --all` 命令
- [ ] 添加 `--security-only` 选项

#### 6.3 安装钩子
- [ ] 实现 `InstallHooks`
- [ ] 支持 `pre_install` 钩子
- [ ] 支持 `post_install` 钩子
- [ ] 支持 `pre_uninstall` 钩子
- [ ] 支持 `post_uninstall` 钩子
- [ ] 添加配置项 `[hooks.<runtime>]`
- [ ] 变量替换 (`$RUNTIME`, `$VERSION`, `$INSTALL_DIR`)
- [ ] 单元测试

### Phase 7: 任务系统增强 🆕

*借鉴 mise 的任务系统*

#### 7.1 任务定义增强
- [x] 支持任务依赖 `depends = ["lint", "test"]`
- [ ] 支持任务条件 `sources = ["src/**"]`
- [x] 支持任务环境变量 `env = { KEY = "value" }`
- [ ] 支持复杂任务语法

#### 7.2 环境变量管理
- [x] 支持 `[env]` 配置块
- [x] 支持环境变量文件 `_.file = [".env"]`
- [x] 支持 `.env` 格式解析
- [x] 环境变量继承与覆盖

#### 7.3 CLI 命令
- [x] 增强 `vx run <task>` 命令
  - [x] 依赖脚本拓扑排序执行
  - [x] 脚本级 cwd 覆盖
  - [x] 脚本级 env 覆盖
  - [x] 脚本描述显示 (`--list`, `--script-help`)
- [x] ~~添加 `vx task <name>` 别名~~ — **取消**：`task` 命名空间已被 go-task provider 占用（`vx task` 转发到 go-task 的 Taskfile.yml），与 vx.toml 脚本语义冲突
- [x] ~~添加 `vx tasks` 列出所有任务~~ — **取消**：同上，使用 `vx run --list` 即可
- [x] 支持任务参数传递

### Phase 8: 高级特性

#### 8.1 Locked 模式
- [ ] 实现 `LatestBehavior::Locked`
- [ ] 集成锁文件读取
- [ ] 单元测试

#### 8.2 CI 环境支持
- [ ] CI 环境自动检测
- [ ] 默认配置覆盖
- [ ] 文档更新

#### 8.3 vx lock 增强
- [ ] 支持多运行时锁定
- [ ] 依赖版本锁定
- [ ] 锁文件更新命令

#### 8.4 性能优化
- [ ] 并行版本解析
- [ ] 解析结果缓存
- [ ] 基准测试

#### 8.5 配置信任机制 (借鉴 mise)
- [ ] 实现 `vx trust` 命令
- [ ] 首次执行提示确认
- [ ] 安全执行配置脚本

## 测试计划

### 单元测试

#### ResolveStage 测试
- [x] 测试具体版本解析
- [ ] 测试 latest -> installed 解析
- [ ] 测试 latest -> remote 解析
- [ ] 测试范围版本解析
- [x] 测试依赖解析
- [ ] 测试循环依赖检测

#### Fallback Chain 测试 🆕
- [ ] 测试显式版本优先
- [ ] 测试项目配置优先级
- [ ] 测试传统配置文件读取
- [ ] 测试 .nvmrc 解析
- [ ] 测试 .tool-versions 解析
- [ ] 测试 package.json volta 字段
- [ ] 测试用户默认版本

#### Shell Hooks 测试 🆕
- [ ] 测试 Bash 脚本生成
- [ ] 测试 Zsh 脚本生成
- [ ] 测试 Fish 脚本生成
- [ ] 测试 PowerShell 脚本生成
- [ ] 测试 --use-on-cd 功能

#### 版本管理测试 🆕
- [ ] 测试版本过期检测
- [ ] 测试安全更新检测
- [ ] 测试 LTS 版本检测
- [ ] 测试安装钩子执行

### 集成测试

- [ ] Pipeline 端到端流程
- [ ] 配置优先级测试
- [ ] 向后兼容性测试
- [ ] 传统配置文件迁移测试 🆕
- [ ] Shell 集成测试 🆕

### E2E 测试

- [ ] `vx node --version` 基本流程
- [ ] `vx npm --version` 自动安装依赖
- [ ] `vx --with go node` 注入运行时
- [ ] 错误场景测试
- [ ] 进入 .nvmrc 目录自动切换 🆕
- [ ] `vx outdated` 版本检测 🆕
- [ ] `vx upgrade` 批量升级 🆕

### 性能测试 🆕

- [ ] Shim 冷启动时间 (目标 < 5ms)
- [ ] 版本解析时间
- [ ] 版本切换时间
- [ ] 对比 fnm/Volta/nvm

## 文档更新

- [ ] 配置参考文档
- [ ] 用户指南更新
- [ ] 迁移指南
  - [ ] 从 nvm 迁移
  - [ ] 从 fnm 迁移
  - [ ] 从 Volta 迁移
  - [ ] 从 asdf/mise 迁移
- [ ] API 文档
- [ ] CHANGELOG
- [ ] Shell 集成指南 🆕

## 风险与缓解

| 风险 | 可能性 | 影响 | 缓解措施 |
|------|--------|------|----------|
| 向后兼容性问题 | 中 | 高 | 全面的 E2E 测试，灰度发布 |
| 性能回退 | 低 | 中 | 基准测试，性能对比 |
| 用户迁移困难 | 低 | 低 | 清晰的迁移文档，警告提示 |
| Shell 集成复杂性 | 中 | 中 | 参考 fnm 成熟实现 |
| 传统配置文件冲突 | 低 | 低 | 明确优先级，配置项控制 |

## Provider 分析结论 (2026-02-07)

### 已删除
- **Volta**: 竞品工具（只管理 Node.js 生态），与 vx 功能重叠，不应作为 provider 集成
- **Proto (moonrepo)**: 同理，是通用版本管理器竞品，集成会形成"套娃"架构

### 推荐新增 Providers（按优先级）

| Provider | 类别 | 理由 | 优先级 |
|----------|------|------|--------|
| `ripgrep` (rg) | CLI 工具 | 最流行的代码搜索工具，GitHub 50k+ stars | P1 |
| `fd` | CLI 工具 | 现代 find 替代，搭配 rg 使用 | P1 |
| `bat` | CLI 工具 | 现代 cat 替代，语法高亮 | P2 |
| `delta` | CLI 工具 | Git diff 美化工具 | P2 |
| `lazygit` | Git 工具 | 终端 Git UI，开发者高频使用 | P2 |
| `shellcheck` | Lint 工具 | Shell 脚本静态分析，CI 必备 | P1 |
| `yq` | CLI 工具 | YAML/JSON/XML 处理器，搭配 jq | P1 |
| `buf` | API 工具 | 现代 Protobuf 工具链（搭配 protoc） | P2 |
| `trivy` | 安全工具 | 容器/代码漏洞扫描 | P2 |
| `cosign` | 安全工具 | 容器签名工具 | P3 |
| `act` | CI 工具 | 本地运行 GitHub Actions | P2 |
| `mkcert` | 安全工具 | 本地 HTTPS 证书生成 | P2 |
| `grpcurl` | API 工具 | gRPC CLI 客户端 | P3 |
| `k9s` | K8s 工具 | 终端 Kubernetes UI | P2 |
| `minikube` | K8s 工具 | 本地 Kubernetes 集群 | P2 |
| `wasm-tools` | WASM 工具 | WebAssembly 工具链 | P3 |

## 更新日志

| 日期 | 变更 |
|------|------|
| 2026-02-05 | 创建跟踪文档 |
| 2026-02-05 | 添加 Phase 4-8: Volta/mise/fnm/proto 借鉴特性 |
| 2026-02-07 | 删除 Volta provider；添加 Provider 分析结论；Phase 1 开始实施 |
| 2026-02-07 | Phase 1.1-1.7 完成：Pipeline 核心类型、四个 Stage 实现、ExecutionPipeline 编排器、52 个测试全部通过 |
| 2026-02-07 | Phase 1.8 完成：迁移 execute_with_with_deps 到 Pipeline，清理 5 个死代码方法 |
| 2026-02-07 | Phase 7 进行中：增强 vx run — ConfigView.scripts 改为 ScriptConfig，实现依赖拓扑排序执行、cwd/env 覆盖、描述显示 |
| 2026-02-07 | Phase 1.4 补完：PrepareStage 集成 proxy execution（RFC 0028），修复 bundled runtime（如 msbuild）executable 查找失败问题 |
| 2026-02-07 | Phase 3.1 提前完成：5 层结构化错误类型已在 Phase 1.7 全部定义（27 个 error variants），更新 tracker 反映真实进度 |
| 2026-02-07 | Phase 7 完成（100%）：取消 `vx task`/`vx tasks` 别名 — `task` 命名空间已被 go-task provider 占用，`vx run` 已完整覆盖任务系统功能 |
| 2026-02-07 | Phase 3.2 完成（5/6）：迁移 executor 子模块全部 26 处 `anyhow::anyhow!()` 到结构化错误类型，新增 9 个 error variants 和 9 个测试，122 个测试全部通过 |
| 2026-02-07 | Phase 3.3 完成：CLI 错误输出改进 — `error_handler.rs` 模块实现 PipelineError downcast + 分类格式化，`main.rs` 不再使用 anyhow 默认输出，20 个测试全通过 |
| 2026-02-07 | Phase 2 进行中（80%）：ManifestRegistry 拆分为 `ManifestStore` + `ManifestIndex` + `ProviderBuilder` 三个子模块。新增 `PlatformConstraint::intersect()` 替代 `or_else`，`BuildResult` 结构化错误取代 silent warn。20 个新测试 + 全部旧测试通过 |
| 2026-02-07 | Phase 2 完成（100%）：`create_registry()` 使用 `build_registry_with_result()` + `store_build_diagnostics()` 结构化诊断存储；新增 `vx info --warnings` 命令显示 build 错误/警告（彩色输出） |
