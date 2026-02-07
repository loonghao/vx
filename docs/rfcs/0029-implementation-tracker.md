# RFC 0029: Implementation Tracker

## 总体进度

| Phase | 描述 | 状态 | 完成度 | 目标版本 | 预计工期 |
|-------|------|------|--------|----------|----------|
| Phase 1 | 核心重构（Pipeline 架构） | 进行中 | 95% | v0.7.0 | 2 周 |
| Phase 2 | ManifestRegistry 拆分 | 待开始 | 0% | v0.7.0 | 1 周 |
| Phase 3 | 错误处理改进 | 进行中 | 60% | v0.7.1 | 1 周 |
| Phase 4 | Fallback Chain 与传统配置支持 🆕 | 待开始 | 0% | v0.7.1 | 1.5 周 |
| Phase 5 | Shell 集成与自动切换 🆕 | 待开始 | 0% | v0.8.0 | 1.5 周 |
| Phase 6 | 版本管理增强 🆕 | 待开始 | 0% | v0.8.0 | 1 周 |
| Phase 7 | 任务系统增强 🆕 | 进行中 | 60% | v0.8.0 | 1 周 |
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
- [ ] 实现 `VersionStrategy` 配置
- [ ] 实现 `LatestBehavior` 处理
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
- [ ] 创建 `loader.rs`
- [ ] 迁移清单加载逻辑
- [ ] 单元测试

#### 2.2 ManifestIndex
- [ ] 创建 `index.rs`
- [ ] 实现元数据索引
- [ ] 实现别名解析
- [ ] 实现平台约束合并（取交集）
- [ ] 单元测试

#### 2.3 ProviderBuilder
- [ ] 创建 `builder.rs`
- [ ] 实现 `BuildResult`
- [ ] 返回 warnings 和 errors
- [ ] 单元测试

#### 2.4 CLI 集成
- [ ] 处理 `BuildResult.errors`
- [ ] 添加 `vx info --warnings`
- [ ] 文档更新

### Phase 3: 错误处理改进

#### 3.1 错误类型定义 (借鉴 Volta ErrorKind)
> **Note**: 核心错误类型已在 Phase 1.7 中提前完成（`pipeline/error.rs`）

- [x] 定义 `ResolveError` (7 variants: RuntimeNotFound, VersionNotFound, DependencyCycle, PlatformNotSupported 等)
- [x] 定义 `EnsureError` (6 variants: InstallFailed, DependencyFailed, DownloadFailed 等)
- [x] 定义 `PrepareError` (5 variants: NoExecutable, ProxyNotAvailable 等)
- [x] 定义 `ExecuteError` (4 variants)
- [x] 定义 `PipelineError` (5 variants, wraps all stages)

#### 3.2 错误迁移
- [ ] 迁移 `Executor` 错误（仍使用 anyhow，待迁移到 Pipeline 错误类型）
- [ ] 迁移 `InstallationManager` 错误
- [ ] 迁移 `Resolver` 错误

#### 3.3 CLI 错误输出
- [ ] 改进错误格式化
- [ ] 添加依赖链上下文
- [ ] 添加建议修复步骤
- [ ] 测试验证

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
- [ ] 添加 `vx task <name>` 别名
- [ ] 添加 `vx tasks` 列出所有任务
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
