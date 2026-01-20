# RFC 0022: Install Normalize (安装后标准化)

## 摘要

在 `provider.toml` 中添加 `[runtimes.normalize]` 配置，支持安装后对文件和目录进行标准化处理，包括重命名、移动、创建符号链接等。

## 动机

当前问题：
1. **不同工具的目录结构不一致**：有的工具直接在根目录放可执行文件，有的在 `bin/`，有的在嵌套目录
2. **可执行文件命名不一致**：如 `ImageMagick-7.1.1-Q16-HDRI/magick.exe` vs `bin/magick.exe`
3. **难以维护**：每个 Provider 需要在 Rust 代码中处理特殊情况

需要一个**声明式的标准化方案**，让所有 runtime 安装后都有统一的目录结构。

## 标准化目录结构

### 目标结构

```
~/.vx/store/{runtime}/{version}/
├── bin/              # 可执行文件（标准位置）
│   ├── {runtime}     # 主可执行文件（Unix）
│   ├── {runtime}.exe # 主可执行文件（Windows）
│   └── ...           # 其他可执行文件
├── lib/              # 库文件（可选）
├── share/            # 共享数据（可选）
└── ...               # 原始文件（保留）
```

### 优势

1. **统一的 PATH 管理**：只需添加 `bin/` 到 PATH
2. **可预测的可执行文件位置**：`~/.vx/store/{runtime}/{version}/bin/{runtime}`
3. **简化 shim 生成**：不需要查找可执行文件位置
4. **跨平台一致性**：Windows 和 Unix 使用相同的目录结构

## 设计方案

### 1. 新增 `[runtimes.normalize]` 配置

```toml
[[runtimes]]
name = "magick"
executable = "magick"

# 安装后标准化配置
[runtimes.normalize]
# 启用标准化（默认 true）
enabled = true

# 可执行文件标准化
[[runtimes.normalize.executables]]
# 源路径模板（支持 glob 和变量）
source = "ImageMagick-*-Q16-HDRI/magick.exe"
# 目标路径（相对于 bin/）
target = "magick.exe"
# 处理方式：move（移动）、copy（复制）、link（符号链接）
action = "link"

[[runtimes.normalize.executables]]
source = "ImageMagick-*-Q16-HDRI/convert.exe"
target = "convert.exe"
action = "link"

# 别名/符号链接
[[runtimes.normalize.aliases]]
# 别名名称
name = "im"
# 指向的可执行文件
target = "magick"
```

### 2. 完整示例

#### ImageMagick (Windows .7z 解压后)

```toml
[[runtimes]]
name = "magick"
executable = "magick"

[runtimes.normalize]
enabled = true

# 主可执行文件
[[runtimes.normalize.executables]]
source = "ImageMagick-{version}-Q16-HDRI/magick.exe"
target = "magick.exe"
action = "link"

# 附带的工具
[[runtimes.normalize.executables]]
source = "ImageMagick-{version}-Q16-HDRI/convert.exe"
target = "convert.exe"
action = "link"

[[runtimes.normalize.executables]]
source = "ImageMagick-{version}-Q16-HDRI/identify.exe"
target = "identify.exe"
action = "link"

# lib 目录
[[runtimes.normalize.directories]]
source = "ImageMagick-{version}-Q16-HDRI"
target = "lib/imagemagick"
action = "link"
```

#### Node.js (解压后已经有良好结构)

```toml
[[runtimes]]
name = "node"
executable = "node"

# Node.js 解压后已经是标准结构，只需简单映射
[runtimes.normalize]
enabled = true

# 标准化路径（strip_prefix 已经处理了大部分）
# 这里只是确保 bin/ 目录存在
[[runtimes.normalize.executables]]
source = "bin/node"
target = "node"
action = "link"

[[runtimes.normalize.executables]]
source = "bin/npm"
target = "npm"
action = "link"
```

#### AppImage 类型 (Linux)

```toml
[[runtimes]]
name = "magick"
executable = "magick"

[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."linux-x86_64"]
source_name = "ImageMagick.AppImage"
target_name = "magick.AppImage"
target_dir = "bin"
target_permissions = "755"

# normalize 会将 AppImage 链接为标准名称
[runtimes.normalize]
enabled = true

[[runtimes.normalize.executables]]
source = "bin/magick.AppImage"
target = "magick"
action = "link"
```

### 3. 平台特定配置

```toml
[runtimes.normalize]
enabled = true

# Windows 特定
[runtimes.normalize.platforms.windows]
[[runtimes.normalize.platforms.windows.executables]]
source = "ImageMagick-{version}-Q16-HDRI/magick.exe"
target = "magick.exe"
action = "link"

# Unix 特定
[runtimes.normalize.platforms.unix]
[[runtimes.normalize.platforms.unix.executables]]
source = "bin/magick"
target = "magick"
action = "link"
```

### 4. 支持的变量

- `{version}`: 版本号
- `{name}`: Runtime 名称
- `{platform}`: 平台 (windows, macos, linux)
- `{arch}`: 架构 (x86_64, aarch64)
- `{ext}`: 可执行文件扩展名 (.exe 或 空)
- `*`: glob 通配符

### 5. Action 类型

| Action | 说明 | 优点 | 缺点 |
|--------|------|------|------|
| `link` | 创建符号链接 | 不占额外空间 | Windows 需要权限 |
| `copy` | 复制文件 | 跨平台兼容 | 占用空间 |
| `move` | 移动文件 | 不占额外空间 | 原始位置消失 |
| `hardlink` | 创建硬链接 | 跨平台，不占空间 | 不能跨文件系统 |

默认策略：
- Unix: 优先 `link`，失败则 `hardlink`，再失败则 `copy`
- Windows: 优先 `hardlink`，失败则 `copy`

## 实现

### Rust 结构体

```rust
// crates/vx-manifest/src/provider/normalize.rs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Normalize action type
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NormalizeAction {
    /// Create symbolic link (default on Unix)
    #[default]
    Link,
    /// Create hard link
    HardLink,
    /// Copy file
    Copy,
    /// Move file
    Move,
}

/// Executable normalization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutableNormalize {
    /// Source path pattern (supports glob and variables)
    pub source: String,
    /// Target path (relative to bin/)
    pub target: String,
    /// Action to perform
    #[serde(default)]
    pub action: NormalizeAction,
    /// File permissions (Unix only)
    #[serde(default)]
    pub permissions: Option<String>,
}

/// Directory normalization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNormalize {
    /// Source directory pattern
    pub source: String,
    /// Target directory (relative to install root)
    pub target: String,
    /// Action to perform
    #[serde(default)]
    pub action: NormalizeAction,
}

/// Alias/symlink definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasNormalize {
    /// Alias name
    pub name: String,
    /// Target executable (in bin/)
    pub target: String,
}

/// Platform-specific normalize config
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformNormalizeConfig {
    #[serde(default)]
    pub executables: Vec<ExecutableNormalize>,
    #[serde(default)]
    pub directories: Vec<DirectoryNormalize>,
    #[serde(default)]
    pub aliases: Vec<AliasNormalize>,
}

/// Normalize configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizeConfig {
    /// Enable normalization (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Executable normalization rules (cross-platform)
    #[serde(default)]
    pub executables: Vec<ExecutableNormalize>,
    
    /// Directory normalization rules (cross-platform)
    #[serde(default)]
    pub directories: Vec<DirectoryNormalize>,
    
    /// Aliases to create
    #[serde(default)]
    pub aliases: Vec<AliasNormalize>,
    
    /// Platform-specific configurations
    #[serde(default)]
    pub platforms: HashMap<String, PlatformNormalizeConfig>,
}

fn default_enabled() -> bool {
    true
}

impl NormalizeConfig {
    /// Get effective configuration for current platform
    pub fn get_effective_config(&self) -> EffectiveNormalizeConfig {
        let platform_key = if cfg!(windows) {
            "windows"
        } else if cfg!(target_os = "macos") {
            "macos"
        } else {
            "linux"
        };
        
        let mut executables = self.executables.clone();
        let mut directories = self.directories.clone();
        let mut aliases = self.aliases.clone();
        
        // Merge platform-specific config
        if let Some(platform_config) = self.platforms.get(platform_key) {
            executables.extend(platform_config.executables.clone());
            directories.extend(platform_config.directories.clone());
            aliases.extend(platform_config.aliases.clone());
        }
        
        // Also check "unix" for linux/macos
        if !cfg!(windows) {
            if let Some(unix_config) = self.platforms.get("unix") {
                executables.extend(unix_config.executables.clone());
                directories.extend(unix_config.directories.clone());
                aliases.extend(unix_config.aliases.clone());
            }
        }
        
        EffectiveNormalizeConfig {
            enabled: self.enabled,
            executables,
            directories,
            aliases,
        }
    }
}

/// Effective configuration after platform resolution
pub struct EffectiveNormalizeConfig {
    pub enabled: bool,
    pub executables: Vec<ExecutableNormalize>,
    pub directories: Vec<DirectoryNormalize>,
    pub aliases: Vec<AliasNormalize>,
}
```

### Normalizer 实现

```rust
// crates/vx-installer/src/normalizer.rs

use glob::glob;
use std::path::Path;

pub struct Normalizer;

impl Normalizer {
    /// Apply normalization to an installed runtime
    pub fn normalize(
        install_path: &Path,
        config: &NormalizeConfig,
        context: &NormalizeContext,
    ) -> Result<NormalizeResult> {
        let effective = config.get_effective_config();
        
        if !effective.enabled {
            return Ok(NormalizeResult::default());
        }
        
        let bin_dir = install_path.join("bin");
        std::fs::create_dir_all(&bin_dir)?;
        
        let mut result = NormalizeResult::default();
        
        // Process executables
        for rule in &effective.executables {
            let source_pattern = context.expand(&rule.source);
            let target_name = context.expand(&rule.target);
            let target_path = bin_dir.join(&target_name);
            
            // Find matching source files
            let full_pattern = install_path.join(&source_pattern);
            for entry in glob(full_pattern.to_str().unwrap())? {
                let source_path = entry?;
                
                Self::apply_action(&source_path, &target_path, &rule.action)?;
                
                // Set permissions on Unix
                #[cfg(unix)]
                if let Some(perms) = &rule.permissions {
                    Self::set_permissions(&target_path, perms)?;
                }
                
                result.executables_normalized.push(target_name.clone());
                break; // Only first match
            }
        }
        
        // Process directories
        for rule in &effective.directories {
            let source_pattern = context.expand(&rule.source);
            let target_name = context.expand(&rule.target);
            let target_path = install_path.join(&target_name);
            
            let full_pattern = install_path.join(&source_pattern);
            for entry in glob(full_pattern.to_str().unwrap())? {
                let source_path = entry?;
                
                if source_path.is_dir() {
                    if let Some(parent) = target_path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    Self::apply_action(&source_path, &target_path, &rule.action)?;
                    result.directories_normalized.push(target_name.clone());
                    break;
                }
            }
        }
        
        // Create aliases
        for alias in &effective.aliases {
            let alias_path = bin_dir.join(&alias.name);
            let target_path = bin_dir.join(&alias.target);
            
            if target_path.exists() {
                Self::create_link(&target_path, &alias_path)?;
                result.aliases_created.push(alias.name.clone());
            }
        }
        
        Ok(result)
    }
    
    fn apply_action(source: &Path, target: &Path, action: &NormalizeAction) -> Result<()> {
        match action {
            NormalizeAction::Link => Self::create_link(source, target),
            NormalizeAction::HardLink => Self::create_hardlink(source, target),
            NormalizeAction::Copy => Self::copy_recursive(source, target),
            NormalizeAction::Move => std::fs::rename(source, target).map_err(Into::into),
        }
    }
    
    fn create_link(source: &Path, target: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(source, target)?;
        }
        
        #[cfg(windows)]
        {
            if source.is_dir() {
                std::os::windows::fs::symlink_dir(source, target)?;
            } else {
                std::os::windows::fs::symlink_file(source, target)?;
            }
        }
        
        Ok(())
    }
    
    fn create_hardlink(source: &Path, target: &Path) -> Result<()> {
        if source.is_file() {
            std::fs::hard_link(source, target)?;
        } else {
            // For directories, create symlink instead
            Self::create_link(source, target)?;
        }
        Ok(())
    }
    
    fn copy_recursive(source: &Path, target: &Path) -> Result<()> {
        if source.is_dir() {
            copy_dir_all(source, target)?;
        } else {
            std::fs::copy(source, target)?;
        }
        Ok(())
    }
    
    #[cfg(unix)]
    fn set_permissions(path: &Path, mode: &str) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mode = u32::from_str_radix(mode, 8)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
        Ok(())
    }
}

pub struct NormalizeContext {
    pub version: String,
    pub name: String,
}

impl NormalizeContext {
    pub fn expand(&self, template: &str) -> String {
        template
            .replace("{version}", &self.version)
            .replace("{name}", &self.name)
            .replace("{ext}", if cfg!(windows) { ".exe" } else { "" })
    }
}

#[derive(Default)]
pub struct NormalizeResult {
    pub executables_normalized: Vec<String>,
    pub directories_normalized: Vec<String>,
    pub aliases_created: Vec<String>,
}
```

## 集成点

### 1. 在安装流程中调用

```rust
// crates/vx-runtime/src/impls/installer.rs

async fn install_runtime(
    &self,
    runtime: &dyn Runtime,
    version: &str,
    ctx: &RuntimeContext,
) -> Result<InstallResult> {
    // ... 现有安装逻辑 ...
    
    // 安装后标准化
    if let Some(normalize_config) = runtime.manifest().normalize.as_ref() {
        let normalize_ctx = NormalizeContext {
            version: version.to_string(),
            name: runtime.name().to_string(),
        };
        
        let result = Normalizer::normalize(&install_path, normalize_config, &normalize_ctx)?;
        
        tracing::debug!(
            "Normalized: {} executables, {} directories, {} aliases",
            result.executables_normalized.len(),
            result.directories_normalized.len(),
            result.aliases_created.len()
        );
    }
    
    // ... 返回结果 ...
}
```

### 2. 更新 RuntimeDef 结构

```rust
// crates/vx-manifest/src/provider/runtime.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDef {
    // ... 现有字段 ...
    
    /// Normalize configuration (RFC 0022)
    #[serde(default)]
    pub normalize: Option<NormalizeConfig>,
}
```

## 迁移示例

### 迁移 ImageMagick

**之前** (需要在 Rust 代码中处理):
```rust
// runtime.rs
fn get_executable_path(&self, version: &str, install_path: &Path) -> PathBuf {
    // 复杂的路径查找逻辑
    for entry in install_path.read_dir()? {
        if entry.file_name().to_string_lossy().starts_with("ImageMagick-") {
            return entry.path().join("magick.exe");
        }
    }
}
```

**之后** (声明式配置):
```toml
[runtimes.normalize]
enabled = true

[[runtimes.normalize.executables]]
source = "ImageMagick-*-Q16-HDRI/magick.exe"
target = "magick.exe"
action = "link"

[[runtimes.normalize.executables]]
source = "ImageMagick-*-Q16-HDRI/convert.exe"
target = "convert.exe"
action = "link"
```

## 优势

1. **声明式**：所有路径处理在 TOML 中配置，无需修改 Rust 代码
2. **一致性**：所有 runtime 安装后都有统一的 `bin/` 目录结构
3. **可维护**：添加新工具只需编写配置，不需要理解代码
4. **可测试**：normalize 规则可以独立测试
5. **灵活性**：支持 glob 模式、平台特定配置、多种 action 类型

## 相关 RFC

- RFC 0019: Executable Layout Configuration
- RFC 0013: Manifest-Driven Provider Registration
