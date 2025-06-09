# vx 模块化架构设计

## 🏗️ 架构概览

vx 采用了高度模块化的插件架构，支持动态扩展和管理各种开发工具。这种设计确保了系统的可扩展性、可维护性和灵活性。

## 📋 核心设计原则

### 1. 插件化架构 (Plugin Architecture)
- 每个工具作为独立插件实现
- 统一的插件接口和生命周期管理
- 支持动态加载和卸载

### 2. 标准化接口 (Standardized Interface)
- 所有插件实现统一的 `Plugin` trait
- 标准化的安装、卸载、执行流程
- 一致的元数据和配置格式

### 3. 分层设计 (Layered Design)
```
┌─────────────────────────────────────┐
│           CLI Layer                 │  ← 用户交互层
├─────────────────────────────────────┤
│        Plugin Manager               │  ← 插件管理层
├─────────────────────────────────────┤
│         Plugin Registry             │  ← 插件注册层
├─────────────────────────────────────┤
│      Individual Plugins             │  ← 具体插件层
├─────────────────────────────────────┤
│    Core Infrastructure              │  ← 基础设施层
└─────────────────────────────────────┘
```

## 🔧 核心组件

### 1. Plugin Trait
```rust
#[async_trait]
pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    async fn is_installed(&self) -> Result<bool>;
    async fn get_installed_version(&self) -> Result<Option<String>>;
    async fn get_latest_version(&self) -> Result<String>;
    async fn install(&self, version: &str, install_dir: &PathBuf) -> Result<InstallResult>;
    async fn uninstall(&self, version: &str, install_dir: &PathBuf) -> Result<()>;
    fn get_executable_path(&self, version: &str, install_dir: &PathBuf) -> PathBuf;
    async fn validate_installation(&self, install_dir: &PathBuf) -> Result<bool>;
    fn get_commands(&self) -> Vec<PluginCommand>;
    async fn execute_command(&self, command: &str, args: &[String]) -> Result<i32>;
}
```

### 2. Plugin Registry
- 管理所有已注册的插件
- 提供插件发现和查找功能
- 支持插件启用/禁用状态管理

### 3. Plugin Manager
- 高级插件管理接口
- 配置文件管理
- 外部插件发现和加载

## 📦 内置插件

### 当前支持的插件

| 插件名 | 描述 | 类别 | 平台支持 |
|--------|------|------|----------|
| **Go** | Go 编程语言工具链 | Language, BuildTool | Windows, macOS, Linux |
| **Node.js** | JavaScript 运行时和 npm | Runtime, PackageManager, Language | Windows, macOS, Linux |
| **Rust** | Rust 编程语言和 Cargo | Language, BuildTool, PackageManager | Windows, macOS, Linux |
| **UV** | Python 包安装器和解析器 | PackageManager, Language | Windows, macOS, Linux |

### 插件功能示例

#### Go 插件
```bash
# 查看插件信息
vx plugin info go

# 使用 Go 命令
vx go build
vx go run main.go
vx go test ./...
vx go mod tidy
```

#### Node.js 插件
```bash
# 查看插件信息
vx plugin info node

# 使用 Node.js 命令
vx node app.js
vx npm install express
vx npx create-react-app my-app
```

## 🔌 插件开发指南

### 1. 创建新插件

```rust
use crate::plugin::{Plugin, PluginMetadata, PluginCategory, Platform};

pub struct MyToolPlugin {
    metadata: PluginMetadata,
}

impl MyToolPlugin {
    pub fn new() -> Self {
        let metadata = PluginMetadata {
            name: "mytool".to_string(),
            version: "1.0.0".to_string(),
            description: "My awesome development tool".to_string(),
            author: "Your Name".to_string(),
            license: "MIT".to_string(),
            categories: vec![PluginCategory::Utility],
            supported_platforms: vec![Platform::Windows, Platform::Linux],
            // ... 其他字段
        };
        
        Self { metadata }
    }
}

#[async_trait]
impl Plugin for MyToolPlugin {
    // 实现所有必需的方法
}
```

### 2. 注册插件

```rust
// 在 src/plugins/mod.rs 中
pub mod mytool_plugin;

pub fn register_builtin_plugins(registry: &mut PluginRegistry) -> Result<()> {
    // 注册现有插件...
    
    // 注册新插件
    let mytool_plugin = Box::new(mytool_plugin::MyToolPlugin::new());
    registry.register_plugin(mytool_plugin)?;
    
    Ok(())
}
```

## 🎯 插件类别系统

### 预定义类别

```rust
pub enum PluginCategory {
    Language,        // 编程语言 (Go, Rust, Python)
    Runtime,         // 运行时环境 (Node.js, JVM)
    PackageManager,  // 包管理器 (npm, pip, cargo)
    BuildTool,       // 构建工具 (make, cmake, gradle)
    VersionControl,  // 版本控制 (git, svn)
    Database,        // 数据库 (postgres, mysql, redis)
    Cloud,           // 云工具 (aws-cli, gcloud, kubectl)
    DevOps,          // DevOps 工具 (docker, terraform, ansible)
    Editor,          // 编辑器和 IDE (vim, vscode)
    Utility,         // 通用工具
}
```

### 按类别查找插件

```bash
# 查看所有语言类插件
vx plugin list --category language

# 查看所有包管理器插件
vx plugin list --category packagemanager
```

## 🔧 插件管理命令

### 基本命令

```bash
# 列出所有插件
vx plugin list

# 列出已启用的插件
vx plugin list --enabled

# 按类别过滤
vx plugin list --category language

# 查看插件详细信息
vx plugin info <plugin-name>

# 搜索插件
vx plugin search <keyword>

# 启用/禁用插件
vx plugin enable <plugin-name>
vx plugin disable <plugin-name>

# 查看插件统计
vx plugin stats
```

## 📁 目录结构

```
src/
├── plugin.rs              # 插件 trait 和核心类型
├── plugin_manager.rs      # 插件管理器
├── plugins/
│   ├── mod.rs             # 插件模块入口
│   ├── go_plugin.rs       # Go 插件实现
│   ├── node_plugin.rs     # Node.js 插件实现
│   ├── rust_plugin.rs     # Rust 插件实现
│   └── uv_plugin.rs       # UV 插件实现
├── package_manager.rs     # 包管理功能
├── installer.rs           # 安装器
└── ...

~/.vx/
├── plugins/               # 外部插件目录
├── tools/                 # 工具安装目录
├── plugin_config.json    # 插件配置
└── registry.json         # 包注册表
```

## 🚀 扩展计划

### Phase 1: 更多内置插件
- [ ] Python 插件 (python, pip, poetry)
- [ ] Java 插件 (java, maven, gradle)
- [ ] .NET 插件 (dotnet, nuget)
- [ ] Docker 插件 (docker, docker-compose)

### Phase 2: 外部插件支持
- [ ] 动态库加载 (.dll, .so, .dylib)
- [ ] 脚本插件支持 (JavaScript, Python)
- [ ] 插件市场和分发

### Phase 3: 高级功能
- [ ] 插件依赖管理
- [ ] 插件版本控制
- [ ] 插件安全验证
- [ ] 插件性能监控

## 🎯 最佳实践

### 1. 插件设计
- 保持插件轻量和专注
- 实现完整的错误处理
- 提供清晰的用户反馈
- 支持多平台兼容性

### 2. 配置管理
- 使用标准化的配置格式
- 提供合理的默认值
- 支持用户自定义配置

### 3. 测试策略
- 为每个插件编写单元测试
- 测试跨平台兼容性
- 模拟网络和文件系统操作

## 📊 性能指标

### 当前状态
- ✅ 4 个内置插件
- ✅ 统一的插件接口
- ✅ 动态插件管理
- ✅ 配置驱动的行为
- ✅ 多平台支持

### 目标指标
- 📦 支持 20+ 内置插件
- 🔌 支持外部插件加载
- ⚡ 插件启动时间 < 100ms
- 💾 内存占用 < 50MB
- 🔄 热重载插件支持

## 🎉 总结

vx 的模块化架构为开发工具管理提供了强大而灵活的基础。通过插件系统，用户可以：

1. **统一管理** - 通过单一接口管理所有开发工具
2. **按需扩展** - 只启用需要的插件，保持系统轻量
3. **自定义配置** - 根据项目需求定制工具行为
4. **社区贡献** - 开发和分享自定义插件

这种设计确保了 vx 能够适应不断变化的开发生态系统，为开发者提供一致且高效的工具管理体验。
