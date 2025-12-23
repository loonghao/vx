# VX 核心概念

VX 是一个通用的开发工具管理器，采用插件化架构设计。本文档详细说明了 VX 中的核心概念和组件。

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                        VX Core                              │
├─────────────────────────────────────────────────────────────┤
│  Plugin Registry  │  Configuration Manager  │  Tool Context │
├─────────────────────────────────────────────────────────────┤
│                      Plugin System                          │
├─────────────────────────────────────────────────────────────┤
│  Plugin A    │    Plugin B    │    Plugin C    │  Plugin D  │
│  ┌─────────┐ │   ┌─────────┐  │   ┌─────────┐  │ ┌─────────┐ │
│  │ Tool 1  │ │   │ Tool 2  │  │   │   PM 1  │  │ │ Tool 3  │ │
│  │ Tool 2  │ │   │   PM 1  │  │   │   PM 2  │  │ │   PM 2  │ │
│  └─────────┘ │   │   PM 2  │  │   └─────────┘  │ └─────────┘ │
│              │   └─────────┘  │                │             │
└─────────────────────────────────────────────────────────────┘
```

## 核心概念定义

### 1. Plugin (插件)

**定义**: Plugin 是 VX 系统中的顶级组织单元，负责提供一组相关的工具和包管理器。

**职责**:

- 作为工具和包管理器的容器和注册器
- 提供插件元数据（名称、描述、版本）
- 管理插件的生命周期（初始化、启用/禁用）
- 声明插件支持的工具和包管理器

**特点**:

- 一个插件可以提供多个工具和包管理器
- 插件之间相互独立，可以单独启用或禁用
- 插件可以有依赖关系

**示例**:

```rust
// Node.js 插件提供 node 工具和 npm 包管理器
pub struct NodePlugin {
    // 插件实现
}

impl VxPlugin for NodePlugin {
    fn name(&self) -> &str { "node" }
    fn tools(&self) -> Vec<Box<dyn VxTool>> {
        vec![Box::new(NodeTool::default())]
    }
    fn package_managers(&self) -> Vec<Box<dyn VxPackageManager>> {
        vec![Box::new(NpmPackageManager::default())]
    }
}
```

### 2. Tool (工具)

**定义**: Tool 是具体的开发工具实现，负责特定工具的版本管理、下载、安装和执行。

**职责**:

- 版本管理：获取可用版本列表、检查已安装版本
- 下载和安装：从官方或镜像源下载并安装工具
- 执行管理：在隔离环境中执行工具命令
- 配置管理：处理工具特定的配置和环境变量

**特点**:

- 每个工具对应一个具体的开发工具（如 Node.js、Go、Python）
- 支持多版本并存和版本切换
- 可配置下载源和安装路径
- 支持环境隔离

**示例**:

```rust
// Go 工具实现
pub struct GoTool {
    // 工具实现
}

impl VxTool for GoTool {
    fn name(&self) -> &str { "go" }
    fn description(&self) -> &str { "Go programming language" }
    fn aliases(&self) -> Vec<&str> { vec!["golang"] }

    async fn fetch_versions(&self, include_prerelease: bool) -> Result<Vec<VersionInfo>> {
        // 获取 Go 的可用版本
    }

    async fn install_version(&self, version: &str, install_dir: &Path) -> Result<PathBuf> {
        // 安装指定版本的 Go
    }
}
```

### 3. Package Manager (包管理器)

**定义**: Package Manager 是负责管理项目依赖包的组件，提供包的安装、更新、删除等功能。

**职责**:

- 包管理：安装、更新、删除项目依赖
- 项目检测：识别项目类型和配置文件
- 环境管理：管理项目的依赖环境
- 生态系统集成：与特定语言生态系统集成

**特点**:

- 每个包管理器对应一个特定的生态系统（如 npm for Node.js）
- 支持项目级和全局级包管理
- 可以检测和适配项目配置
- 支持多种安装源和镜像

**示例**:

```rust
// npm 包管理器实现
pub struct NpmPackageManager {
    // 包管理器实现
}

impl VxPackageManager for NpmPackageManager {
    fn name(&self) -> &str { "npm" }
    fn ecosystem(&self) -> Ecosystem { Ecosystem::JavaScript }

    async fn install_packages(&self, packages: &[PackageSpec], project_path: &Path) -> Result<()> {
        // 安装 npm 包
    }

    fn is_preferred_for_project(&self, project_path: &Path) -> bool {
        // 检查是否存在 package.json
        project_path.join("package.json").exists()
    }
}
```

## 组件关系

### Plugin ↔ Tool 关系

- **一对多**: 一个插件可以提供多个工具
- **容器关系**: 插件是工具的容器和管理者
- **生命周期**: 插件控制工具的创建和销毁

### Plugin ↔ Package Manager 关系

- **一对多**: 一个插件可以提供多个包管理器
- **容器关系**: 插件是包管理器的容器和管理者
- **生态系统**: 通常一个插件对应一个语言生态系统

### Tool ↔ Package Manager 关系

- **协作关系**: 工具和包管理器可以协作（如 Node.js 工具 + npm 包管理器）
- **独立性**: 它们可以独立工作，不强制绑定
- **共享配置**: 可以共享配置和环境设置

## 配置系统

VX 使用 Figment 提供分层配置管理：

### 配置层级（优先级从低到高）

1. **内置默认配置**: 系统默认设置
2. **全局用户配置**: `~/.config/vx/config.toml`
3. **项目配置检测**: 从 `pyproject.toml`、`Cargo.toml`、`package.json`、`go.mod` 等检测
4. **VX 项目配置**: `.vx.toml`
5. **环境变量**: `VX_*` 前缀的环境变量

### 配置示例

```toml
# .vx.toml
[defaults]
auto_install = true
check_updates = true

[tools.go]
version = "1.21.0"
install_method = "official"
custom_sources.default = "https://mirrors.example.com/go{version}.{platform}-{arch}.{ext}"

[tools.node]
version = "20.11.0"
registry = "taobao"

[registries.taobao]
name = "Taobao Mirror"
base_url = "https://registry.npmmirror.com"
priority = 100
enabled = true
```

## 扩展性设计

### 添加新工具

1. 实现 `VxTool` trait
2. 实现 `UrlBuilder` 和 `VersionParser`（可选）
3. 创建插件包装器
4. 注册到插件系统

### 添加新包管理器

1. 实现 `VxPackageManager` trait
2. 定义生态系统类型
3. 实现项目检测逻辑
4. 集成到相应插件

### 配置扩展

- 支持自定义下载源
- 支持代理和镜像配置
- 支持工具特定的环境变量
- 支持项目级配置覆盖

这种设计确保了 VX 的高度可扩展性和灵活性，同时保持了清晰的职责分离和良好的用户体验。
