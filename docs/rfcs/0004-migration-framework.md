# RFC 0004: Migration Framework

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2024-12-28
> **目标版本**: v0.6.0

## 摘要

设计一个独立的、可插件化的迁移框架 `vx-migration`，用于处理 vx 版本升级过程中的配置文件迁移、数据格式转换和兼容性问题。该框架支持钩子机制、依赖解析、回滚能力和历史记录，为未来各种复杂的迁移场景提供灵活的扩展能力。

## 动机

### 当前状态分析

1. **分散的迁移逻辑**: 当前迁移代码散落在 `vx-config` 等多个 crate 中
2. **缺乏统一框架**: 没有标准化的迁移接口和流程
3. **扩展性不足**: 添加新迁移需要修改核心代码
4. **无历史记录**: 无法追踪已执行的迁移
5. **缺少回滚机制**: 迁移失败后难以恢复

### 行业趋势对比

| 工具 | 迁移方案 | 可借鉴 |
|------|----------|--------|
| Rust Edition | 自动迁移工具 cargo fix | 自动化迁移 |
| Django | 数据库迁移框架 | 版本化迁移、依赖解析 |
| Flyway | SQL 迁移 | 历史记录、回滚 |
| mise | 配置迁移 | 渐进式迁移 |

### 需求分析

1. **插件化设计** - 通过 trait 定义迁移接口，支持动态注册
2. **生命周期钩子** - 支持 pre/post 钩子，便于扩展
3. **依赖管理** - 迁移间可定义依赖关系和执行顺序
4. **Dry-run 模式** - 预览变更而不实际执行
5. **历史记录** - 跟踪所有迁移操作
6. **回滚支持** - 可逆迁移支持回滚

## 设计方案

### 架构概览

```
vx-migration/
├── src/
│   ├── lib.rs              # 模块入口和 prelude
│   ├── error.rs            # 错误类型定义
│   ├── version.rs          # 版本解析和比较
│   ├── types.rs            # 通用类型定义
│   ├── traits.rs           # 核心 trait 定义
│   ├── context.rs          # 迁移上下文（共享状态）
│   ├── registry.rs         # 迁移注册表
│   ├── engine.rs           # 迁移引擎（主入口）
│   ├── history.rs          # 迁移历史记录
│   └── migrations/         # 内置迁移实现
│       ├── mod.rs
│       ├── version_detector.rs
│       ├── config_v1_to_v2.rs
│       └── file_rename.rs
└── tests/
    └── migration_tests.rs
```

### 核心 Traits

#### Migration Trait

```rust
/// 迁移插件接口
#[async_trait]
pub trait Migration: Send + Sync {
    /// 返回迁移元数据
    fn metadata(&self) -> MigrationMetadata;

    /// 检查是否需要运行此迁移
    async fn check(&self, ctx: &MigrationContext) -> MigrationResult<bool>;

    /// 执行迁移
    async fn migrate(&self, ctx: &mut MigrationContext) -> MigrationResult<MigrationStepResult>;

    /// 回滚迁移（可选）
    async fn rollback(&self, ctx: &mut MigrationContext) -> MigrationResult<()> {
        Ok(()) // 默认不支持回滚
    }

    /// 验证迁移结果（可选）
    async fn validate(&self, ctx: &MigrationContext) -> MigrationResult<bool> {
        Ok(true)
    }

    /// 获取依赖的迁移 ID
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// 用于向下转型
    fn as_any(&self) -> &dyn Any;
}
```

#### MigrationHook Trait

```rust
/// 迁移生命周期钩子
#[async_trait]
pub trait MigrationHook: Send + Sync {
    /// 钩子名称
    fn name(&self) -> &str;

    /// 迁移开始前
    async fn pre_migrate(&self, ctx: &MigrationContext) -> MigrationResult<()> {
        Ok(())
    }

    /// 迁移完成后
    async fn post_migrate(&self, ctx: &MigrationContext, report: &MigrationReport) -> MigrationResult<()> {
        Ok(())
    }

    /// 单步迁移前
    async fn pre_step(&self, ctx: &MigrationContext, migration: &dyn Migration) -> MigrationResult<()> {
        Ok(())
    }

    /// 单步迁移后
    async fn post_step(&self, ctx: &MigrationContext, migration: &dyn Migration, result: &MigrationStepResult) -> MigrationResult<()> {
        Ok(())
    }

    /// 发生错误时
    async fn on_error(&self, ctx: &MigrationContext, error: &MigrationError) -> MigrationResult<()> {
        Ok(())
    }

    /// 回滚时
    async fn on_rollback(&self, ctx: &MigrationContext, migration: &dyn Migration) -> MigrationResult<()> {
        Ok(())
    }
}
```

#### VersionDetector Trait

```rust
/// 版本检测器接口
#[async_trait]
pub trait VersionDetector: Send + Sync {
    /// 检测器名称
    fn name(&self) -> &str;

    /// 检测版本
    async fn detect(&self, path: &Path) -> MigrationResult<Option<Version>>;

    /// 支持的版本范围
    fn supported_range(&self) -> VersionRange;
}
```

### 类型定义

#### MigrationMetadata

```rust
pub struct MigrationMetadata {
    /// 唯一标识符
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 描述
    pub description: String,
    /// 源版本范围
    pub from_version: VersionRange,
    /// 目标版本
    pub to_version: Version,
    /// 迁移类别
    pub category: MigrationCategory,
    /// 优先级
    pub priority: MigrationPriority,
    /// 是否可回滚
    pub reversible: bool,
    /// 是否破坏性
    pub breaking: bool,
    /// 预计耗时（毫秒）
    pub estimated_duration_ms: Option<u64>,
}
```

#### MigrationCategory

```rust
pub enum MigrationCategory {
    /// 配置文件迁移
    Config,
    /// 文件结构变更
    FileStructure,
    /// 数据格式转换
    Data,
    /// Schema 变更
    Schema,
    /// 环境设置
    Environment,
    /// 自定义
    Custom(String),
}
```

#### MigrationPriority

```rust
pub enum MigrationPriority {
    Critical = 0,   // 必须最先执行
    High = 1,       // 高优先级
    Normal = 2,     // 默认
    Low = 3,        // 低优先级
    Cleanup = 4,    // 清理任务，最后执行
}
```

### MigrationEngine

```rust
impl MigrationEngine {
    /// 创建新引擎
    pub fn new() -> Self;

    /// 注册迁移
    pub fn register<M: Migration + 'static>(self, migration: M) -> Self;

    /// 注册钩子
    pub fn register_hook<H: MigrationHook + 'static>(self, hook: H) -> Self;

    /// 执行迁移
    pub async fn migrate(&self, path: &Path, options: &MigrationOptions) -> MigrationResult<MigrationReport>;

    /// 检查需要的迁移
    pub async fn check(&self, path: &Path) -> MigrationResult<Vec<MigrationMetadata>>;

    /// 回滚到指定版本
    pub async fn rollback(&self, path: &Path, to_version: &Version) -> MigrationResult<MigrationReport>;
}
```

### MigrationOptions

```rust
pub struct MigrationOptions {
    /// 是否 dry-run 模式
    pub dry_run: bool,
    /// 是否创建备份
    pub backup: bool,
    /// 备份目录
    pub backup_dir: Option<PathBuf>,
    /// 目标版本（None 表示最新）
    pub target_version: Option<Version>,
    /// 是否交互模式
    pub interactive: bool,
    /// 是否详细输出
    pub verbose: bool,
    /// 失败时是否回滚
    pub rollback_on_failure: bool,
    /// 要跳过的迁移 ID
    pub skip_migrations: HashSet<String>,
    /// 只运行指定的迁移 ID
    pub only_migrations: Option<HashSet<String>>,
}
```

### 使用示例

#### 基本用法

```rust
use vx_migration::prelude::*;
use vx_migration::migrations::create_default_engine;

// 使用默认引擎（包含内置迁移）
let engine = create_default_engine();
let options = MigrationOptions::default();
let result = engine.migrate(Path::new("./my-project"), &options).await?;

println!("迁移完成: {} 个成功, {} 个跳过",
    result.successful_count, result.skipped_count);
```

#### Dry-run 模式

```rust
let options = MigrationOptions {
    dry_run: true,
    verbose: true,
    ..Default::default()
};

let result = engine.migrate(path, &options).await?;
for step in &result.steps {
    println!("将执行: {} - {}", step.migration_id, step.description);
}
```

#### 自定义迁移

```rust
pub struct MyCustomMigration;

#[async_trait]
impl Migration for MyCustomMigration {
    fn metadata(&self) -> MigrationMetadata {
        MigrationMetadata {
            id: "my-custom-migration".into(),
            name: "My Custom Migration".into(),
            description: "自定义迁移示例".into(),
            from_version: VersionRange::any(),
            to_version: Version::new(2, 0, 0),
            category: MigrationCategory::Custom("example".into()),
            priority: MigrationPriority::Normal,
            reversible: true,
            breaking: false,
            estimated_duration_ms: Some(100),
        }
    }

    async fn check(&self, ctx: &MigrationContext) -> MigrationResult<bool> {
        // 检查是否需要迁移
        Ok(true)
    }

    async fn migrate(&self, ctx: &mut MigrationContext) -> MigrationResult<MigrationStepResult> {
        // 执行迁移逻辑
        Ok(MigrationStepResult {
            success: true,
            changes: vec![Change::new(ChangeType::Modified, "config.toml")],
            warnings: vec![],
            duration: Duration::from_millis(50),
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

// 注册自定义迁移
let engine = MigrationEngine::new()
    .register(MyCustomMigration);
```

#### 自定义钩子

```rust
pub struct LoggingHook;

#[async_trait]
impl MigrationHook for LoggingHook {
    fn name(&self) -> &str {
        "logging"
    }

    async fn pre_step(&self, ctx: &MigrationContext, migration: &dyn Migration) -> MigrationResult<()> {
        tracing::info!("开始迁移: {}", migration.metadata().name);
        Ok(())
    }

    async fn post_step(&self, ctx: &MigrationContext, migration: &dyn Migration, result: &MigrationStepResult) -> MigrationResult<()> {
        if result.success {
            tracing::info!("迁移成功: {}", migration.metadata().name);
        } else {
            tracing::error!("迁移失败: {}", migration.metadata().name);
        }
        Ok(())
    }
}

let engine = MigrationEngine::new()
    .register_hook(LoggingHook);
```

### 内置迁移

| 迁移 ID | 描述 | 版本范围 |
|---------|------|----------|
| `config-v1-to-v2` | 配置格式 v1 → v2 | 1.x → 2.0 |
| `file-rename` | `.vx.toml` → `vx.toml` | any |
| `legacy-paths` | 旧路径结构迁移 | < 0.5 |

### 迁移历史

迁移历史存储在 `~/.vx/migration-history.json`:

```json
{
  "version": "1.0.0",
  "entries": [
    {
      "id": "uuid",
      "migration_id": "config-v1-to-v2",
      "timestamp": "2024-12-28T10:00:00Z",
      "from_version": "1.0.0",
      "to_version": "2.0.0",
      "status": "completed",
      "duration_ms": 150,
      "changes": [
        {"type": "modified", "path": "vx.toml"}
      ],
      "machine_id": "hostname"
    }
  ]
}
```

## 向后兼容性

### 兼容策略

1. **版本检测** - 自动检测配置文件版本
2. **渐进增强** - 所有新字段都是可选的
3. **默认值** - 为新字段提供合理默认值
4. **警告处理** - 对未知字段发出警告而非错误

### 迁移路径

```bash
# 检查需要的迁移
vx migrate --check

# 预览迁移（dry-run）
vx migrate --dry-run

# 执行迁移
vx migrate

# 执行迁移并创建备份
vx migrate --backup

# 回滚到指定版本
vx migrate --rollback v1.0.0
```

## 实现计划

### Phase 1: 核心框架 (v0.6.0)

- [ ] 核心 traits 定义 (`Migration`, `MigrationHook`)
- [ ] `MigrationEngine` 实现
- [ ] `MigrationRegistry` 实现
- [ ] `MigrationContext` 共享状态
- [ ] 基本错误处理
- [ ] 单元测试

### Phase 2: 内置迁移 (v0.6.0)

- [ ] `VxVersionDetector` 版本检测
- [ ] `ConfigV1ToV2Migration` 配置迁移
- [ ] `FileRenameMigration` 文件重命名
- [ ] 集成测试

### Phase 3: 高级功能 (v0.7.0)

- [ ] 迁移历史记录
- [ ] 回滚支持
- [ ] Dry-run 模式
- [ ] CLI 集成 (`vx migrate` 命令)

### Phase 4: 扩展功能 (v0.8.0)

- [ ] 交互式迁移
- [ ] 迁移依赖解析
- [ ] 并行迁移支持
- [ ] 迁移报告生成

## 参考资料

- [Django Migrations](https://docs.djangoproject.com/en/4.2/topics/migrations/)
- [Flyway](https://flywaydb.org/)
- [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)
- [RFC 0001: Config v2 Enhancement](./0001-vx-toml-v2-enhancement.md)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2024-12-28 | Draft | 初始草案 |
