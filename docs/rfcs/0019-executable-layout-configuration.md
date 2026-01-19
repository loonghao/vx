# RFC 0019: Executable Layout Configuration

## 摘要

在 `provider.toml` 中添加可执行文件布局配置，支持声明式处理各种压缩包格式、单文件下载、嵌套目录等场景。

## 动机

当前问题：
1. **YASM**：下载单文件 `yasm-1.3.0-win64.exe`，需要重命名为 `yasm.exe`
2. **FFmpeg**：压缩包中有 `bin/ffmpeg.exe`，需要正确识别
3. **其他工具**：各种布局（根目录、bin/、带版本号等）

需要一个**统一的声明式方案**来处理这些差异。

## 设计方案

### 1. 在 provider.toml 中添加 `[runtimes.layout]` 配置

```toml
[[runtimes]]
name = "yasm"
executable = "yasm"

# 可执行文件布局配置
[runtimes.layout]
# 下载类型：archive（压缩包）或 binary（单文件）
download_type = "binary"

# 单文件下载的重命名规则
[runtimes.layout.binary]
# 下载后的原始文件名模板（支持变量：{version}, {platform}, {arch}）
source_name = "yasm-{version}-{platform}.exe"
# 目标文件名（安装后）
target_name = "yasm.exe"
# 目标目录（相对于安装根目录）
target_dir = "bin"

# 或者对于压缩包
[runtimes.layout.archive]
# 可执行文件在压缩包内的相对路径
# 支持多个路径（用于多个可执行文件的情况）
executable_paths = [
    "bin/{name}.exe",           # Windows
    "bin/{name}",               # Unix
    "{name}-{version}/bin/{name}.exe"  # 嵌套目录
]
# strip_prefix: 解压时去掉的前缀目录
strip_prefix = "yasm-{version}"
```

### 2. 跨平台配置

```toml
[runtimes.layout]
download_type = "archive"

# Windows 平台布局
[runtimes.layout.windows]
executable_paths = ["bin/yasm.exe"]
strip_prefix = ""

# Unix 平台布局
[runtimes.layout.unix]
executable_paths = ["bin/yasm", "yasm"]
strip_prefix = ""
```

### 3. 完整示例：YASM

```toml
[[runtimes]]
name = "yasm"
description = "YASM - Yet Another Assembler"
executable = "yasm"

[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary.windows-x86_64]
source_name = "yasm-{version}-win64.exe"
target_name = "yasm.exe"
target_dir = "bin"

[runtimes.layout.binary.windows-x86]
source_name = "yasm-{version}-win32.exe"
target_name = "yasm.exe"
target_dir = "bin"

[runtimes.layout.binary.macos-x86_64]
source_name = "yasm-{version}-macos"
target_name = "yasm"
target_dir = "bin"
target_permissions = "755"
```

### 4. 完整示例：FFmpeg

```toml
[[runtimes]]
name = "ffmpeg"
executable = "ffmpeg"

[runtimes.layout]
download_type = "archive"

[runtimes.layout.windows]
executable_paths = ["bin/ffmpeg.exe"]
strip_prefix = "ffmpeg-{version}-windows-x64"

[runtimes.layout.macos]
executable_paths = ["bin/ffmpeg"]
strip_prefix = "ffmpeg-{version}"

[runtimes.layout.linux]
executable_paths = ["bin/ffmpeg", "ffmpeg"]
strip_prefix = ""
```

### 5. 支持的变量

在 `source_name`, `target_name`, `executable_paths`, `strip_prefix` 中可用：

- `{version}`: 版本号（如 "1.3.0"）
- `{name}`: Runtime 名称（如 "yasm"）
- `{platform}`: 平台名称（"windows", "macos", "linux"）
- `{arch}`: 架构（"x86_64", "x86", "aarch64"）
- `{os}`: 操作系统（"windows", "darwin", "linux"）
- `{ext}`: 可执行文件扩展名（Windows: ".exe", Unix: ""）

## 实现步骤

### Step 1: 定义 Rust 结构体

```rust
// crates/vx-runtime/src/layout.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutableLayout {
    /// 下载类型
    pub download_type: DownloadType,
    
    /// 单文件配置
    #[serde(default)]
    pub binary: Option<BinaryLayout>,
    
    /// 压缩包配置
    #[serde(default)]
    pub archive: Option<ArchiveLayout>,
    
    /// 平台特定配置
    #[serde(default)]
    pub windows: Option<PlatformLayout>,
    #[serde(default)]
    pub macos: Option<PlatformLayout>,
    #[serde(default)]
    pub linux: Option<PlatformLayout>,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadType {
    Binary,   // 单文件
    Archive,  // 压缩包
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BinaryLayout {
    /// 原始文件名模板
    pub source_name: String,
    /// 目标文件名
    pub target_name: String,
    /// 目标目录（相对路径）
    #[serde(default = "default_bin_dir")]
    pub target_dir: String,
    /// Unix 文件权限（如 "755"）
    #[serde(default)]
    pub target_permissions: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchiveLayout {
    /// 可执行文件在压缩包内的路径（支持多个）
    pub executable_paths: Vec<String>,
    /// 解压时去掉的前缀
    #[serde(default)]
    pub strip_prefix: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlatformLayout {
    /// 可执行文件路径
    pub executable_paths: Vec<String>,
    /// 去掉的前缀
    #[serde(default)]
    pub strip_prefix: Option<String>,
    /// Unix 权限
    #[serde(default)]
    pub permissions: Option<String>,
}

fn default_bin_dir() -> String {
    "bin".to_string()
}
```

### Step 2: 变量插值器

```rust
// crates/vx-runtime/src/layout.rs (续)

pub struct LayoutContext {
    pub version: String,
    pub name: String,
    pub platform: Platform,
}

impl ExecutableLayout {
    /// 解析变量并返回实际路径
    pub fn resolve(
        &self,
        ctx: &LayoutContext,
    ) -> Result<ResolvedLayout> {
        let vars = self.build_variables(ctx);
        
        match self.download_type {
            DownloadType::Binary => {
                let binary = self.binary.as_ref()
                    .ok_or_else(|| anyhow!("Missing binary configuration"))?;
                
                Ok(ResolvedLayout::Binary {
                    source_name: interpolate(&binary.source_name, &vars),
                    target_name: interpolate(&binary.target_name, &vars),
                    target_dir: interpolate(&binary.target_dir, &vars),
                    permissions: binary.target_permissions.clone(),
                })
            }
            DownloadType::Archive => {
                let layout = self.get_platform_layout(ctx.platform.os)?;
                
                Ok(ResolvedLayout::Archive {
                    executable_paths: layout.executable_paths
                        .iter()
                        .map(|p| interpolate(p, &vars))
                        .collect(),
                    strip_prefix: layout.strip_prefix
                        .as_ref()
                        .map(|p| interpolate(p, &vars)),
                })
            }
        }
    }
    
    fn build_variables(&self, ctx: &LayoutContext) -> HashMap<String, String> {
        let mut vars = HashMap::new();
        vars.insert("version".to_string(), ctx.version.clone());
        vars.insert("name".to_string(), ctx.name.clone());
        vars.insert("platform".to_string(), ctx.platform.os.to_string());
        vars.insert("arch".to_string(), ctx.platform.arch.to_string());
        vars.insert("ext".to_string(), ctx.platform.executable_extension());
        vars
    }
    
    fn get_platform_layout(&self, os: Os) -> Result<&PlatformLayout> {
        match os {
            Os::Windows => self.windows.as_ref()
                .or(self.archive.as_ref().map(|a| &a as &PlatformLayout)),
            Os::MacOS => self.macos.as_ref()
                .or(self.archive.as_ref().map(|a| &a as &PlatformLayout)),
            Os::Linux => self.linux.as_ref()
                .or(self.archive.as_ref().map(|a| &a as &PlatformLayout)),
            _ => None,
        }.ok_or_else(|| anyhow!("No layout for platform: {:?}", os))
    }
}

pub enum ResolvedLayout {
    Binary {
        source_name: String,
        target_name: String,
        target_dir: String,
        permissions: Option<String>,
    },
    Archive {
        executable_paths: Vec<String>,
        strip_prefix: Option<String>,
    },
}

fn interpolate(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{}}}", key), value);
    }
    result
}
```

### Step 3: 在 Installer 中应用

```rust
// crates/vx-installer/src/installer.rs

impl Installer {
    pub async fn install_with_layout(
        &self,
        url: &str,
        install_path: &Path,
        layout: &ExecutableLayout,
        ctx: &LayoutContext,
    ) -> Result<()> {
        let resolved = layout.resolve(ctx)?;
        
        match resolved {
            ResolvedLayout::Binary { source_name, target_name, target_dir, permissions } => {
                // 下载单文件
                let download_path = self.download(url).await?;
                
                // 创建目标目录
                let target_dir_path = install_path.join(&target_dir);
                fs::create_dir_all(&target_dir_path)?;
                
                // 移动并重命名
                let target_path = target_dir_path.join(&target_name);
                fs::rename(&download_path, &target_path)?;
                
                // 设置 Unix 权限
                #[cfg(unix)]
                if let Some(perm) = permissions {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = u32::from_str_radix(&perm, 8)?;
                    fs::set_permissions(&target_path, fs::Permissions::from_mode(mode))?;
                }
                
                Ok(())
            }
            ResolvedLayout::Archive { executable_paths, strip_prefix } => {
                // 下载并解压
                let archive = self.download(url).await?;
                self.extract(&archive, install_path, strip_prefix.as_deref()).await?;
                
                // 验证可执行文件存在
                let found = executable_paths.iter()
                    .any(|p| install_path.join(p).exists());
                
                if !found {
                    return Err(anyhow!(
                        "Executable not found at any expected path: {:?}",
                        executable_paths
                    ));
                }
                
                Ok(())
            }
        }
    }
}
```

### Step 4: 在 Runtime trait 中使用

```rust
// crates/vx-runtime/src/traits.rs

#[async_trait]
pub trait Runtime: Send + Sync {
    // ... 现有方法 ...
    
    /// 返回可执行文件布局配置（从 manifest 解析）
    fn executable_layout(&self) -> Option<ExecutableLayout> {
        None  // 默认无特殊布局
    }
    
    /// 获取可执行文件相对路径（现在可以从 layout 派生）
    fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
        if let Some(layout) = self.executable_layout() {
            let ctx = LayoutContext {
                version: version.to_string(),
                name: self.name().to_string(),
                platform: platform.clone(),
            };
            
            if let Ok(resolved) = layout.resolve(&ctx) {
                return match resolved {
                    ResolvedLayout::Binary { target_name, target_dir, .. } => {
                        format!("{}/{}", target_dir, target_name)
                    }
                    ResolvedLayout::Archive { executable_paths, .. } => {
                        // 返回第一个存在的路径
                        executable_paths.first()
                            .cloned()
                            .unwrap_or_else(|| format!("bin/{}", self.name()))
                    }
                };
            }
        }
        
        // 默认：bin/{name}{ext}
        let ext = if platform.os == Os::Windows { ".exe" } else { "" };
        format!("bin/{}{}", self.name(), ext)
    }
}
```

## 优势

1. **声明式配置**：所有布局逻辑在 TOML 中声明，无需修改 Rust 代码
2. **跨平台支持**：不同平台可以有不同的布局配置
3. **变量插值**：灵活处理版本号、平台名等动态值
4. **统一处理**：安装器自动处理重命名、移动、权限设置
5. **向后兼容**：没有 layout 配置时使用默认行为

## 迁移指南

### 现有 Provider 迁移

#### YASM
```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary.windows-x86_64]
source_name = "yasm-{version}-win64.exe"
target_name = "yasm.exe"
target_dir = "bin"
```

#### FFmpeg
```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.windows]
executable_paths = ["bin/ffmpeg.exe", "bin/ffprobe.exe", "bin/ffplay.exe"]
strip_prefix = "ffmpeg-{version}-windows-x64"
```

## 未来扩展

1. **自定义安装脚本**：对于特殊需求，支持 `install_script` 字段
2. **多文件处理**：支持批量重命名、移动
3. **依赖文件**：支持声明运行时依赖的 DLL/SO 文件位置
4. **验证规则**：支持声明哪些文件必须存在

## 相关 RFC

- RFC 0013: Manifest-Driven Provider Registration
- RFC 0016: Extended Provider Manifest Schema
- RFC 0018: Provider Manifest Extensions
