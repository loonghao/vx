# RFC 0006: vx-setup Crate

> **状态**: Implemented
> **作者**: vx team
> **创建日期**: 2025-12-29
> **目标版本**: v0.6.0

## 摘要

将 `vx-config` 中的 setup pipeline 和 CI 环境支持相关功能独立为新的 `vx-setup` crate，实现单一职责原则，提高代码的可维护性和可测试性。

## 动机

### 当前状态分析

目前 `vx-config` crate 承载了过多职责：

1. **配置解析** - 解析 `vx.toml` 配置文件
2. **配置类型** - 定义配置数据结构
3. **配置验证** - 验证配置有效性
4. **配置迁移** - 处理配置版本迁移
5. **Hook 执行** - 执行生命周期钩子
6. **Setup Pipeline** - 执行 setup 流程
7. **CI 环境检测** - 检测和适配 CI 环境
8. **容器生成** - 生成 Dockerfile
9. **远程开发** - 生成 devcontainer.json
10. **安全扫描** - 安全相关功能
11. **测试运行** - 测试框架集成
12. **遥测收集** - 构建指标收集

这违反了单一职责原则，导致：

- 代码难以理解和维护
- 测试复杂度高
- 编译时间长
- 依赖关系混乱

### 需求分析

1. **Setup Pipeline 独立性** - Setup 流程是独立的功能模块，应该有自己的 crate
2. **CI 环境支持** - CI 检测和路径导出是 setup 的核心功能
3. **可扩展性** - 未来可能添加更多 CI 提供商支持
4. **可测试性** - 独立 crate 更容易进行单元测试和集成测试

## 设计方案

### Crate 结构

```
crates/vx-setup/
├── Cargo.toml
├── src/
│   ├── lib.rs           # 模块导出
│   ├── ci/              # CI 环境支持
│   │   ├── mod.rs
│   │   ├── provider.rs  # CiProvider 枚举和检测
│   │   ├── github.rs    # GitHub Actions 特定支持
│   │   ├── gitlab.rs    # GitLab CI 特定支持
│   │   └── exporter.rs  # 路径导出器
│   ├── pipeline/        # Setup Pipeline
│   │   ├── mod.rs
│   │   ├── executor.rs  # SetupPipeline 执行器
│   │   └── hooks.rs     # 内置 hook 实现
│   └── types.rs         # 公共类型定义
└── tests/
    ├── ci_tests.rs
    └── pipeline_tests.rs
```

### 模块职责

#### `vx-setup::ci` - CI 环境支持

```rust
/// CI 提供商枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CiProvider {
    GitHub,
    GitLab,
    Azure,
    CircleCI,
    Jenkins,
    Generic,
    None,
}

impl CiProvider {
    /// 从环境变量检测 CI 提供商
    pub fn detect() -> Self;

    /// 是否在 CI 环境中
    pub fn is_ci(&self) -> bool;

    /// 获取 PATH 导出文件路径
    pub fn path_export_file(&self) -> Option<String>;

    /// 获取环境变量导出文件路径
    pub fn env_export_file(&self) -> Option<String>;
}

/// 路径导出器
pub struct PathExporter {
    provider: CiProvider,
    custom_path_file: Option<String>,
}

impl PathExporter {
    pub fn new(provider: CiProvider) -> Self;
    pub fn with_custom_path_file(self, file: String) -> Self;
    pub fn export(&self, paths: &[PathBuf]) -> Result<ExportResult>;
}
```

#### `vx-setup::pipeline` - Setup Pipeline

```rust
/// Setup Pipeline 执行器
pub struct SetupPipeline {
    working_dir: PathBuf,
    store_dir: PathBuf,
    bin_dir: PathBuf,
    config: SetupPipelineConfig,
    ci_provider: CiProvider,
}

impl SetupPipeline {
    pub fn new(
        working_dir: impl AsRef<Path>,
        store_dir: impl AsRef<Path>,
        bin_dir: impl AsRef<Path>,
    ) -> Self;

    pub fn with_config(self, config: SetupPipelineConfig) -> Self;
    pub fn verbose(self, verbose: bool) -> Self;
    pub fn force_ci(self, force: bool) -> Self;

    /// 执行 setup pipeline
    pub async fn execute<F, Fut>(&self, install_fn: F) -> Result<SetupPipelineResult>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<()>>;
}

/// Pipeline 配置（从 vx-config 的 SetupConfig 转换）
pub struct SetupPipelineConfig {
    pub pipeline: Vec<String>,
    pub hooks: SetupHooksConfig,
    pub ci: Option<CiConfig>,
    pub tools: HashMap<String, String>,
    pub pre_setup: Option<HookCommand>,
    pub post_setup: Option<HookCommand>,
}
```

### 依赖关系

```
vx-setup
├── anyhow          # 错误处理
├── shellexpand     # 路径展开
├── vx-config       # 配置类型（只依赖类型，不依赖执行逻辑）
└── vx-paths        # 路径管理

vx-cli
├── vx-setup        # 新增依赖
├── vx-config       # 保持
└── ...

vx-config
├── (移除 setup_pipeline.rs)
├── (保留类型定义 types/setup.rs)
└── ...
```

### 迁移计划

#### Phase 1: 创建 vx-setup crate

1. 创建 `crates/vx-setup/` 目录结构
2. 迁移 `CiProvider` 到 `vx-setup::ci::provider`
3. 迁移 `SetupPipeline` 到 `vx-setup::pipeline::executor`
4. 迁移路径导出逻辑到 `vx-setup::ci::exporter`

#### Phase 2: 更新依赖

1. `vx-config` 保留类型定义，移除执行逻辑
2. `vx-cli` 添加 `vx-setup` 依赖
3. 更新 `vx-cli/src/commands/setup.rs` 使用新 crate

#### Phase 3: 清理和测试

1. 移除 `vx-config/src/setup_pipeline.rs`
2. 添加单元测试到 `vx-setup/tests/`
3. 验证所有功能正常

## 向后兼容性

### 兼容策略

1. **API 兼容** - 公共 API 保持不变，只是移动到新 crate
2. **配置兼容** - `vx.toml` 格式不变
3. **行为兼容** - `vx setup --ci` 行为不变

### 迁移路径

对于使用 `vx-config` 中 setup 相关功能的代码：

```rust
// Before
use vx_config::{SetupPipeline, SetupPipelineResult, CiProvider};

// After
use vx_setup::{SetupPipeline, SetupPipelineResult};
use vx_setup::ci::CiProvider;
```

## 实现计划

### Phase 1: 创建基础结构 (已完成)

- [x] 创建 RFC 文档
- [x] 创建 `vx-setup` crate 目录结构
- [x] 迁移 `CiProvider` 枚举和实现
- [x] 迁移 `SetupPipeline` 执行器
- [x] 迁移路径导出逻辑

### Phase 2: 集成和测试 (已完成)

- [x] 更新 `vx-cli` 依赖
- [x] 更新 `vx-config` 导出
- [x] 添加单元测试
- [x] 添加集成测试

### Phase 3: 清理 (进行中)

- [ ] 移除 `vx-config` 中的重复代码（可选，保持向后兼容）
- [x] 更新文档
- [ ] 发布新版本

## 参考资料

- [vx-config 当前实现](../../crates/vx-config/src/setup_pipeline.rs)
- [RFC 0001: vx.toml v2 增强](./0001-vx-toml-v2-enhancement.md)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-12-29 | Accepted | 初始提案，开始实施 |
| 2025-12-29 | Implemented | 完成 vx-setup crate 创建和集成 |
