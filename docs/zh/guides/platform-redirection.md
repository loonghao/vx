# 平台重定向指南

## 概述

vx 使用**平台无关的 API**结合自动的平台特定存储。这带来了：

1. **统一 API**：使用 `<provider>/<version>/` 路径访问工具
2. **平台特定存储**：文件存储在 `<provider>/<version>/<platform>/` 目录中
3. **自动重定向**：PathManager 透明地重定向到当前平台
4. **离线包支持**：单个包可以包含所有平台

## 目录结构

```
~/.vx/store/
├── node/
│   └── 20.0.0/           # 统一版本目录（API）
│       ├── windows-x64/      # 平台特定（存储）
│       ├── darwin-x64/
│       └── linux-x64/
├── python/
│   └── 3.9.21/
│       ├── windows-x64/
│       └── linux-x64/
└── uv/
    └── 0.5.0/
        ├── windows-x64/
        └── linux-x64/
```

## API 变更

### PathManager

`vx_paths::PathManager` 中新增的方法：

```rust
/// 获取当前平台的目录名
/// 返回："windows-x64"、"darwin-arm64"、"linux-x64" 等
pub fn platform_dir_name(&self) -> String

/// 获取平台特定的实际存储目录
/// 返回：~/.vx/store/<runtime>/<version>/<platform>
pub fn platform_store_dir(&self, runtime_name: &str, version: &str) -> PathBuf

/// 获取平台特定目录中的实际可执行文件路径
/// 返回：~/.vx/store/<runtime>/<version>/<platform>/bin/<runtime>.exe
pub fn platform_executable_path(&self, runtime_name: &str, version: &str) -> PathBuf

/// 检查某个运行时版本是否已安装（检查平台特定目录）
pub fn is_version_in_store(&self, runtime_name: &str, version: &str) -> bool

/// 列出所有已安装版本（检查平台特定目录）
pub fn list_store_versions(&self, runtime_name: &str) -> Result<Vec<String>>
```

### PathResolver

所有存储查找方法现在自动使用平台特定目录：

```rust
// 这些方法内部都使用 platform_store_dir：
pub fn find_tool(&self, tool_name: &str) -> Result<Option<ToolLocation>>
pub fn find_in_store(&self, tool_name: &str) -> Result<Option<ToolLocation>>
pub fn find_in_store_with_exe(
    &self,
    tool_name: &str,
    exe_name: &str,
) -> Result<Option<ToolLocation>>
pub fn find_all_in_store(&self, tool_name: &str) -> Result<Vec<ToolLocation>>
pub fn find_all_in_store_with_exe(
    &self,
    tool_name: &str,
    exe_name: &str,
) -> Result<Vec<ToolLocation>>
pub fn find_tool_version(&self, tool_name: &str, version: &str) -> Option<ToolLocation>
pub fn find_tool_version_with_executable(
    &self,
    tool_name: &str,
    version: &str,
    exe_name: &str,
) -> Option<ToolLocation>>
```

## 运行时安装

安装运行时时，文件存储在平台特定目录中：

```rust
// 在 Runtime::install() 中：
let base_install_path = ctx.paths.version_store_dir(store_name, version);
let install_path = base_install_path.join(platform.as_str());
// 文件提取到：~/.vx/store/<name>/<version>/<platform>/
```

## 离线包创建

创建跨所有平台的离线包：

```bash
# 包结构
vx-bundle/
└── store/
    └── node/
        └── 20.0.0/
            ├── windows-x64/    # 所有平台在一个包中
            ├── darwin-x64/
            └── linux-x64/
```

提取包时，vx 会自动为当前系统选择正确的平台目录。

## 迁移指南

### 对于 Runtime 实现者

如果你实现自定义运行时，请确保安装使用平台目录：

```rust
async fn install(&self, version: &str, ctx: &RuntimeContext) -> Result<InstallResult> {
    let store_name = self.store_name();
    let platform = Platform::current();

    // 使用平台特定目录
    let base_install_path = ctx.paths.version_store_dir(store_name, version);
    let install_path = base_install_path.join(platform.as_str());

    // 下载并提取到 install_path
    ctx.installer.download_and_extract(&url, &install_path).await?;

    Ok(InstallResult::success(install_path, exe_path, version.to_string()))
}
```

### 对于路径使用

查找已安装工具时，使用平台特定目录：

```rust
let resolver = PathResolver::new()?;

// 这些方法自动使用平台特定目录
let tool = resolver.find_tool("node")?;
let latest = resolver.find_latest_tool("node")?;
let versions = resolver.list_store_versions("node")?;

// 查找特定版本（使用平台目录）
let node_20 = resolver.find_tool_version("node", "20.0.0")?;
```

## 优势

1. **跨平台兼容**：同一个包在 Windows、macOS、Linux 上都能工作
2. **磁盘效率**：多个平台可以共享同一个包
3. **透明 API**：用户不需要了解平台细节
4. **便于分发**：单个包支持所有平台
5. **面向未来**：易于添加新平台支持

## 实现细节

### 平台检测

`vx_paths` 包含轻量级的平台检测器：

```rust
pub struct CurrentPlatform {
    pub os: &'static str,   // "windows"、"darwin"、"linux" 等
    pub arch: &'static str, // "x64"、"arm64" 等
}

impl CurrentPlatform {
    pub fn current() -> Self { /* 编译时检测 */ }
    pub fn as_str(&self) -> String { /* "windows-x64" 等 */ }
}
```

### 路径解析

1. **查找**：请求 `<provider>/<version>/`
2. **重定向**：检查 `<provider>/<version>/<platform>/`
3. **回退**：如果未找到则返回 None

这在 `PathResolver` 方法中自动完成。

## 测试

测试平台特定路径：

```rust
#[test]
fn test_platform_redirection() {
    let manager = PathManager::new()?;
    let platform_dir = manager.platform_store_dir("node", "20.0.0");

    assert!(platform_dir.ends_with("node/20.0.0/windows-x64"));

    let versions = manager.list_store_versions("node")?;
    // 只列出包含当前平台子目录的版本
}
```
