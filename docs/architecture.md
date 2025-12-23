# VX 架构设计

## 整体架构

VX 采用分层插件化架构，确保高度的可扩展性和模块化。

```
┌─────────────────────────────────────────────────────────────┐
│                     VX CLI Layer                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │   Commands  │ │     UI      │ │   Config    │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
├─────────────────────────────────────────────────────────────┤
│                     VX Core Layer                          │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │   Registry  │ │   Config    │ │   Context   │          │
│  │             │ │   Manager   │ │   Manager   │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
├─────────────────────────────────────────────────────────────┤
│                   Plugin System                            │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │   Plugin    │ │    Tool     │ │   Package   │          │
│  │   Traits    │ │   Traits    │ │   Manager   │          │
│  │             │ │             │ │   Traits    │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
├─────────────────────────────────────────────────────────────┤
│                 Utility Layer                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │   HTTP      │ │   Platform  │ │   Version   │          │
│  │   Utils     │ │   Detection │ │   Parser    │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

## 核心组件

### 1. VX Core (vx-core)

**职责**: 提供核心抽象和基础设施

**主要模块**:

- `plugin.rs`: 插件系统的核心 traits
- `config_figment.rs`: 基于 Figment 的配置管理
- `registry.rs`: 插件注册和管理
- `tool.rs`: 工具执行上下文
- `version.rs`: 版本信息管理
- `platform.rs`: 平台检测和适配
- `http.rs`: HTTP 工具和下载
- `url_builder.rs`: URL 构建工具
- `version_parser.rs`: 版本解析工具

### 2. VX CLI (vx-cli)

**职责**: 命令行界面和用户交互

**主要模块**:

- `commands/`: 各种 CLI 命令实现
- `ui.rs`: 用户界面和进度显示
- `args.rs`: 命令行参数解析

### 3. Tool Plugins (vx-tools/*)

**职责**: 具体工具的实现

**当前实现**:

- `vx-tool-node`: Node.js 工具支持
- `vx-tool-go`: Go 工具支持
- `vx-tool-rust`: Rust 工具支持（计划中）
- `vx-tool-python`: Python 工具支持（计划中）

### 4. Package Manager Plugins (vx-package-managers/*)

**职责**: 包管理器的实现

**当前实现**:

- `vx-pm-npm`: npm 包管理器支持
- `vx-pm-pnpm`: pnpm 包管理器支持（计划中）
- `vx-pm-yarn`: yarn 包管理器支持（计划中）

## 设计模式

### 1. Plugin Pattern

```rust
// 插件接口
pub trait VxPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    fn tools(&self) -> Vec<Box<dyn VxTool>>;
    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>>;
}

// 标准插件实现
pub struct StandardPlugin {
    name: String,
    description: String,
    version: String,
    tool_factory: Box<dyn Fn() -> Box<dyn VxTool> + Send + Sync>,
}
```

### 2. Strategy Pattern

```rust
// URL 构建策略
pub trait UrlBuilder: Send + Sync {
    fn download_url(&self, version: &str) -> Option<String>;
    fn versions_url(&self) -> &str;
}

// 版本解析策略
pub trait VersionParser: Send + Sync {
    fn parse_versions(&self, json: &Value, include_prerelease: bool) -> Result<Vec<VersionInfo>>;
}
```

### 3. Factory Pattern

```rust
// 可配置工具工厂
pub struct ConfigurableTool {
    metadata: ToolMetadata,
    config_manager: FigmentConfigManager,
    url_builder: Box<dyn UrlBuilder>,
    version_parser: Box<dyn VersionParser>,
}

impl ConfigurableTool {
    pub fn new(
        metadata: ToolMetadata,
        url_builder: Box<dyn UrlBuilder>,
        version_parser: Box<dyn VersionParser>,
    ) -> Result<Self> {
        // 工厂方法实现
    }
}
```

## 配置系统架构

### Figment 分层配置

```
Environment Variables (VX_*)     ← 最高优先级
         ↓
Project Config (.vx.toml)
         ↓
Project Detection (pyproject.toml, Cargo.toml, etc.)
         ↓
User Config (~/.config/vx/config.toml)
         ↓
Built-in Defaults                ← 最低优先级
```

### 配置流程

1. **初始化**: 创建 `FigmentConfigManager`
2. **检测**: 自动检测项目类型和配置文件
3. **合并**: 按优先级合并所有配置层
4. **解析**: 提取最终配置
5. **应用**: 将配置应用到工具和包管理器

## 扩展点

### 1. 添加新工具

```rust
// 1. 定义工具元数据
let metadata = ToolMetadata {
    name: "python".to_string(),
    description: "Python programming language".to_string(),
    aliases: vec!["py".to_string(), "python3".to_string()],
};

// 2. 实现 URL 构建器
pub struct PythonUrlBuilder;
impl UrlBuilder for PythonUrlBuilder { /* ... */ }

// 3. 实现版本解析器
pub struct PythonVersionParser;
impl VersionParser for PythonVersionParser { /* ... */ }

// 4. 创建可配置工具
let tool = ConfigurableTool::new(
    metadata,
    Box::new(PythonUrlBuilder),
    Box::new(PythonVersionParser),
)?;

// 5. 创建插件
let plugin = StandardPlugin::new(
    "python".to_string(),
    "Python support for vx".to_string(),
    "0.1.0".to_string(),
    || Box::new(tool),
);
```

### 2. 添加新包管理器

```rust
// 1. 实现包管理器 trait
pub struct PipPackageManager;

impl VxPackageManager for PipPackageManager {
    fn name(&self) -> &str { "pip" }
    fn ecosystem(&self) -> Ecosystem { Ecosystem::Python }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        // pip install 实现
    }

    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        // 检查 requirements.txt 或 pyproject.toml
    }
}

// 2. 集成到插件
impl VxPlugin for PythonPlugin {
    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(PipPackageManager)]
    }
}
```

## 性能考虑

### 1. 懒加载

- 插件按需加载
- 工具实例按需创建
- 配置按需解析

### 2. 缓存策略

- 版本信息缓存
- 配置解析结果缓存
- HTTP 请求缓存

### 3. 并发处理

- 异步 I/O 操作
- 并行下载和安装
- 非阻塞用户界面

## 错误处理

### 1. 分层错误处理

```rust
pub enum VxError {
    ConfigError { message: String },
    ToolNotFound { tool_name: String },
    VersionNotFound { tool_name: String, version: String },
    InstallationFailed { tool_name: String, version: String, reason: String },
    NetworkError { url: String, error: String },
    Other { message: String },
}
```

### 2. 错误恢复

- 配置错误时回退到默认配置
- 网络错误时重试机制
- 安装失败时清理机制

## 测试策略

### 1. 单元测试

- 每个模块独立测试
- Mock 外部依赖
- 覆盖边界条件

### 2. 集成测试

- 插件系统集成测试
- 配置系统集成测试
- 端到端工作流测试

### 3. 性能测试

- 启动时间测试
- 内存使用测试
- 并发性能测试

这种架构设计确保了 VX 的可维护性、可扩展性和性能，同时提供了清晰的开发指导和最佳实践。
