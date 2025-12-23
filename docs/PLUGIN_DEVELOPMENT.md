# VX 插件开发指南

VX 采用模块化插件架构，支持通过插件扩展新的工具和包管理器。本文档详细介绍如何开发和发布 VX 插件。

## 🏗️ 插件架构概览

### 核心概念

- **Plugin（插件）**: 工具的容器和注册表，如 `uv-plugin`
- **Tool（工具）**: 具体的工具实现，如 `uv`, `uvx`
- **Package Manager（包管理器）**: 跨语言包管理器，如 `npm`, `pip`

### 插件类型

1. **工具插件**: 管理特定工具的版本和安装
2. **包管理器插件**: 提供包管理功能
3. **语言插件**: 支持特定编程语言生态

## 🚀 快速开始

### 创建新插件

```bash
# 使用官方模板创建插件
cargo generate --git https://github.com/loonghao/vx-plugin-template

# 或手动创建
mkdir vx-python-plugin
cd vx-python-plugin
cargo init --lib
```

### 基本插件结构

```
vx-python-plugin/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── tools/
│   │   ├── mod.rs
│   │   ├── python.rs
│   │   └── pip.rs
│   └── package_managers/
│       ├── mod.rs
│       └── pip.rs
├── tests/
│   └── integration_tests.rs
└── README.md
```

## 📦 插件清单 (Cargo.toml)

```toml
[package]
name = "vx-python-plugin"
version = "0.1.0"
edition = "2021"
description = "Python ecosystem plugin for VX"
license = "MIT"
repository = "https://github.com/username/vx-python-plugin"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
vx-core = "0.1"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"

# 插件元数据
[package.metadata.vx-plugin]
name = "python"
version = "0.1.0"
description = "Python ecosystem support"
author = "Your Name <your.email@example.com>"
homepage = "https://github.com/username/vx-python-plugin"
categories = ["language", "python"]
keywords = ["python", "pip", "venv"]
tools = ["python", "pip", "pipx"]
package_managers = ["pip"]
```

## 🛠️ 实现工具插件

### 定义工具特征

```rust
// src/tools/python.rs
use async_trait::async_trait;
use vx_core::{Result, Tool, ToolMetadata, Version, VxEnvironment};

pub struct PythonTool;

#[async_trait]
impl Tool for PythonTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: "python".to_string(),
            description: "Python programming language".to_string(),
            homepage: "https://python.org".to_string(),
            aliases: vec!["py".to_string(), "python3".to_string()],
            categories: vec!["language".to_string(), "interpreter".to_string()],
        }
    }

    async fn list_versions(&self) -> Result<Vec<Version>> {
        // 实现版本列表获取
        let versions = fetch_python_versions().await?;
        Ok(versions)
    }

    async fn install(&self, version: &Version, env: &VxEnvironment) -> Result<()> {
        // 实现安装逻辑
        let install_dir = env.get_version_install_dir("python", &version.to_string());
        download_and_install_python(version, &install_dir).await?;
        Ok(())
    }

    async fn uninstall(&self, version: &Version, env: &VxEnvironment) -> Result<()> {
        // 实现卸载逻辑
        let install_dir = env.get_version_install_dir("python", &version.to_string());
        std::fs::remove_dir_all(install_dir)?;
        Ok(())
    }

    async fn get_executable_path(
        &self,
        version: &Version,
        env: &VxEnvironment,
    ) -> Result<std::path::PathBuf> {
        // 返回可执行文件路径
        let install_dir = env.get_version_install_dir("python", &version.to_string());
        Ok(install_dir.join("bin").join("python"))
    }

    async fn verify_installation(
        &self,
        version: &Version,
        env: &VxEnvironment,
    ) -> Result<bool> {
        // 验证安装是否成功
        let exe_path = self.get_executable_path(version, env).await?;
        Ok(exe_path.exists())
    }
}

// 辅助函数
async fn fetch_python_versions() -> Result<Vec<Version>> {
    // 从 python.org API 获取版本列表
    let response = reqwest::get("https://api.python.org/v3/releases/")
        .await?
        .json::<Vec<PythonRelease>>()
        .await?;

    let versions = response
        .into_iter()
        .map(|release| Version::parse(&release.version))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(versions)
}

async fn download_and_install_python(
    version: &Version,
    install_dir: &std::path::Path,
) -> Result<()> {
    // 实现下载和安装逻辑
    let download_url = format!(
        "https://www.python.org/ftp/python/{}/Python-{}.tgz",
        version, version
    );

    // 下载、解压、编译、安装
    // 具体实现省略...

    Ok(())
}

#[derive(serde::Deserialize)]
struct PythonRelease {
    version: String,
    // 其他字段...
}
```

### 注册工具

```rust
// src/tools/mod.rs
pub mod python;
pub mod pip;

use vx_core::{Plugin, ToolRegistry};

pub fn register_tools(registry: &mut ToolRegistry) {
    registry.register("python", Box::new(python::PythonTool));
    registry.register("pip", Box::new(pip::PipTool));
}
```

## 📋 实现包管理器插件

### 定义包管理器特征

```rust
// src/package_managers/pip.rs
use async_trait::async_trait;
use vx_core::{PackageManager, PackageManagerMetadata, Result};

pub struct PipPackageManager;

#[async_trait]
impl PackageManager for PipPackageManager {
    fn metadata(&self) -> PackageManagerMetadata {
        PackageManagerMetadata {
            name: "pip".to_string(),
            description: "Python package installer".to_string(),
            supported_languages: vec!["python".to_string()],
            config_files: vec!["requirements.txt".to_string(), "pyproject.toml".to_string()],
        }
    }

    async fn install_package(&self, package: &str, version: Option<&str>) -> Result<()> {
        let version_spec = version.map(|v| format!("=={}", v)).unwrap_or_default();
        let package_spec = format!("{}{}", package, version_spec);

        // 执行 pip install
        let output = tokio::process::Command::new("pip")
            .args(&["install", &package_spec])
            .output()
            .await?;

        if !output.status.success() {
            return Err(vx_core::VxError::Other {
                message: format!("Failed to install package: {}", package),
            });
        }

        Ok(())
    }

    async fn uninstall_package(&self, package: &str) -> Result<()> {
        // 执行 pip uninstall
        let output = tokio::process::Command::new("pip")
            .args(&["uninstall", "-y", package])
            .output()
            .await?;

        if !output.status.success() {
            return Err(vx_core::VxError::Other {
                message: format!("Failed to uninstall package: {}", package),
            });
        }

        Ok(())
    }

    async fn list_packages(&self) -> Result<Vec<(String, String)>> {
        // 执行 pip list 并解析输出
        let output = tokio::process::Command::new("pip")
            .args(&["list", "--format=json"])
            .output()
            .await?;

        if !output.status.success() {
            return Err(vx_core::VxError::Other {
                message: "Failed to list packages".to_string(),
            });
        }

        let packages: Vec<PipPackage> = serde_json::from_slice(&output.stdout)?;
        let result = packages
            .into_iter()
            .map(|pkg| (pkg.name, pkg.version))
            .collect();

        Ok(result)
    }

    async fn update_package(&self, package: &str) -> Result<()> {
        // 执行 pip install --upgrade
        let output = tokio::process::Command::new("pip")
            .args(&["install", "--upgrade", package])
            .output()
            .await?;

        if !output.status.success() {
            return Err(vx_core::VxError::Other {
                message: format!("Failed to update package: {}", package),
            });
        }

        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct PipPackage {
    name: String,
    version: String,
}
```

## 🔌 插件入口点

### 主插件文件

```rust
// src/lib.rs
use vx_core::{Plugin, PluginMetadata, ToolRegistry, PackageManagerRegistry};

mod tools;
mod package_managers;

pub struct PythonPlugin;

impl Plugin for PythonPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "python".to_string(),
            version: "0.1.0".to_string(),
            description: "Python ecosystem support for VX".to_string(),
            author: "Your Name".to_string(),
            homepage: "https://github.com/username/vx-python-plugin".to_string(),
            categories: vec!["language".to_string(), "python".to_string()],
        }
    }

    fn register_tools(&self, registry: &mut ToolRegistry) {
        tools::register_tools(registry);
    }

    fn register_package_managers(&self, registry: &mut PackageManagerRegistry) {
        package_managers::register_package_managers(registry);
    }
}

// 插件导出函数
#[no_mangle]
pub extern "C" fn vx_plugin_create() -> *mut dyn Plugin {
    Box::into_raw(Box::new(PythonPlugin))
}

#[no_mangle]
pub extern "C" fn vx_plugin_destroy(plugin: *mut dyn Plugin) {
    if !plugin.is_null() {
        unsafe {
            Box::from_raw(plugin);
        }
    }
}
```

## 🧪 测试插件

### 单元测试

```rust
// tests/integration_tests.rs
use vx_core::{Plugin, ToolRegistry, VxEnvironment};
use vx_python_plugin::PythonPlugin;

#[tokio::test]
async fn test_python_tool_registration() {
    let plugin = PythonPlugin;
    let mut registry = ToolRegistry::new();

    plugin.register_tools(&mut registry);

    assert!(registry.has_tool("python"));
    assert!(registry.has_tool("pip"));
}

#[tokio::test]
async fn test_python_version_listing() {
    let plugin = PythonPlugin;
    let mut registry = ToolRegistry::new();
    plugin.register_tools(&mut registry);

    let tool = registry.get_tool("python").unwrap();
    let versions = tool.list_versions().await.unwrap();

    assert!(!versions.is_empty());
}

#[tokio::test]
async fn test_python_installation() {
    let plugin = PythonPlugin;
    let mut registry = ToolRegistry::new();
    plugin.register_tools(&mut registry);

    let tool = registry.get_tool("python").unwrap();
    let env = VxEnvironment::new().unwrap();
    let version = vx_core::Version::parse("3.11.0").unwrap();

    // 注意：这个测试可能需要网络连接和较长时间
    tool.install(&version, &env).await.unwrap();

    assert!(tool.verify_installation(&version, &env).await.unwrap());
}
```

### 集成测试

```rust
// tests/cli_integration.rs
use std::process::Command;

#[test]
fn test_plugin_cli_integration() {
    // 测试插件是否能正确集成到 vx CLI
    let output = Command::new("vx")
        .args(&["list", "python"])
        .output()
        .expect("Failed to execute vx command");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("python"));
}
```

## 📦 发布插件

### 发布到 crates.io

```bash
# 构建插件
cargo build --release

# 运行测试
cargo test

# 发布到 crates.io
cargo publish
```

### 插件注册表

```toml
# 提交到官方插件注册表
# ~/.config/vx/plugins.toml
[plugins.python]
name = "vx-python-plugin"
version = "0.1.0"
source = "crates.io"
enabled = true

[plugins.python.config]
auto_install = true
default_version = "3.11"
```

## 🔧 高级功能

### 配置支持

```rust
// 支持插件特定配置
#[derive(serde::Deserialize)]
pub struct PythonPluginConfig {
    pub default_version: String,
    pub auto_install: bool,
    pub use_pyenv: bool,
}

impl Plugin for PythonPlugin {
    fn load_config(&self, config: &vx_core::Config) -> Result<()> {
        let plugin_config: PythonPluginConfig = config
            .get("plugins.python")
            .unwrap_or_default();

        // 应用配置...
        Ok(())
    }
}
```

### 钩子系统

```rust
// 支持生命周期钩子
impl Plugin for PythonPlugin {
    async fn on_tool_installed(&self, tool: &str, version: &str) -> Result<()> {
        if tool == "python" {
            // 安装完成后的处理，如设置 pip
            self.setup_pip(version).await?;
        }
        Ok(())
    }

    async fn on_tool_uninstalled(&self, tool: &str, version: &str) -> Result<()> {
        // 卸载后的清理工作
        Ok(())
    }
}
```

## 📚 最佳实践

### 1. 错误处理

- 使用 `vx_core::Result` 类型
- 提供详细的错误信息
- 实现适当的重试机制

### 2. 异步编程

- 所有 I/O 操作使用异步
- 合理使用并发控制
- 避免阻塞操作

### 3. 跨平台支持

- 处理不同操作系统的差异
- 使用标准库的跨平台 API
- 测试多个平台

### 4. 性能优化

- 缓存版本信息
- 并行下载和安装
- 最小化网络请求

### 5. 用户体验

- 提供进度指示
- 清晰的错误消息
- 合理的默认配置

## 🔗 相关资源

- [VX Core API 文档](https://docs.rs/vx-core)
- [插件模板](https://github.com/loonghao/vx-plugin-template)
- [官方插件示例](https://github.com/loonghao/vx/tree/main/crates)
- [社区插件列表](https://github.com/loonghao/vx-plugins)
