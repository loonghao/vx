# Runtime 生命周期设计

本文档定义了 vx Runtime 的完整生命周期，为 Provider 开发者提供清晰的指导。

## 概述

Runtime 的生命周期分为 5 个核心阶段，每个阶段都有明确的职责和扩展点：

```
┌──────────────────────────────────────────────────────────────┐
│                     Runtime 生命周期                           │
└──────────────────────────────────────────────────────────────┘

1️⃣  解析 (Parse)
    ├─ fetch_versions()         → 获取可用版本列表
    ├─ download_url()           → 生成下载 URL
    └─ pre_install()           → 安装前准备 (hook)

2️⃣  下载 (Download)
    └─ [vx-installer 处理]     → 下载文件到临时目录

3️⃣  提取 (Extract)
    ├─ [vx-installer 处理]     → 解压归档文件
    └─ post_extract()          → 提取后处理 (hook)

4️⃣  安装 (Install)
    └─ post_install()          → 平台特定安装步骤 (hook)

5️⃣  验证 (Verify)
    └─ verify_installation()   → 验证安装完整性
```

## 各阶段详解

### 1. 解析阶段 (Parse)

**职责**：获取 Runtime 的元数据和版本信息

#### 必需方法

##### `fetch_versions(ctx: &RuntimeContext) -> Result<Vec<VersionInfo>>`

从官方源获取可用版本列表。

**实现策略**：
- **GitHub Releases**：使用 `vx-version` 的 `GitHubVersionFetcher`
- **npm Registry**：使用 `NpmVersionFetcher`
- **自定义 API**：手动 HTTP 请求解析 JSON
- **硬编码列表**：当官方没有 API 时（如 AWS CLI）

**示例**：

```rust
// 使用 GitHub Releases
async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    let fetcher = GitHubVersionFetcher::new("owner/repo");
    fetcher.fetch_versions(&ctx.http).await
}

// 硬编码列表（AWS CLI 示例）
async fn fetch_versions(&self, _ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
    let versions = vec![
        "2.32.25", "2.32.0", "2.31.0", "latest",
    ];
    Ok(versions.into_iter().map(|v| VersionInfo::new(v)).collect())
}
```

##### `download_url(version: &str, platform: &Platform) -> Result<Option<String>>`

生成特定版本和平台的下载 URL。

**返回值**：
- `Ok(Some(url))` - 有可用的下载链接
- `Ok(None)` - 该平台不支持
- `Err(_)` - 生成 URL 时出错

**示例**：

```rust
async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
    use vx_runtime::{Os, Arch};

    let url = match (&platform.os, &platform.arch) {
        (Os::Linux, Arch::X86_64) => format!(
            "https://example.com/releases/v{}/tool-linux-x64.tar.gz",
            version
        ),
        (Os::Windows, Arch::X86_64) => format!(
            "https://example.com/releases/v{}/tool-windows-x64.zip",
            version
        ),
        _ => return Ok(None),
    };

    Ok(Some(url))
}
```

#### 可选钩子

##### `pre_install(version: &str, ctx: &RuntimeContext) -> Result<()>`

在下载之前执行检查和准备工作。

**用途**：
- 检查系统依赖
- 验证环境变量
- 清理之前失败的安装
- 创建必要的目录

**示例**：

```rust
async fn pre_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
    // 检查是否已经安装
    let install_dir = ctx.paths.store_dir().join(self.name()).join(version);
    if install_dir.exists() {
        eprintln!("⚠️  版本 {} 已安装，正在清理...", version);
        std::fs::remove_dir_all(&install_dir)?;
    }

    // 检查系统依赖
    if !has_required_libs() {
        return Err(anyhow::anyhow!("缺少必需的系统库"));
    }

    Ok(())
}
```

---

### 2. 下载阶段 (Download)

**职责**：将文件从远程源下载到本地

**处理**：由 `vx-installer` 自动处理

**Provider 无需实现**：只需在 `download_url()` 返回正确的 URL 即可

---

### 3. 提取阶段 (Extract)

**职责**：解压归档文件，准备安装

**自动处理**：`vx-installer` 自动识别并解压以下格式：
- `.zip`
- `.tar.gz`、`.tgz`
- `.tar.xz`
- `.tar.bz2`

#### 可选钩子

##### `post_extract(version: &str, install_path: &PathBuf) -> Result<()>`

在提取后、验证前立即执行（**同步方法**）。

**用途**：
- 重命名文件到标准名称
- 设置可执行权限
- 移动文件到预期位置
- 删除不需要的文件

**示例**：

```rust
fn post_extract(&self, _version: &str, install_path: &PathBuf) -> Result<()> {
    // pnpm 在 macOS 上下载为 pnpm-macos-arm64，需要重命名
    let downloaded = install_path.join("pnpm-macos-arm64");
    let target = install_path.join("pnpm");

    if downloaded.exists() {
        std::fs::rename(downloaded, &target)?;
    }

    // 设置可执行权限 (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&target)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&target, perms)?;
    }

    Ok(())
}
```

**注意**：
- 这是**同步**方法，不能使用 `async`
- 在 `verify_installation()` 之前执行
- 适合快速的文件操作

---

### 4. 安装阶段 (Install)

**职责**：执行平台特定的安装过程

#### 可选钩子

##### `post_install(version: &str, ctx: &RuntimeContext) -> Result<()>`

在提取后执行额外的安装步骤（**异步方法**）。

**用途**：
- 运行安装脚本
- 执行 MSI/PKG 安装
- 设置环境变量
- 安装捆绑工具
- 初始化配置

**何时需要 `post_install()`**：

| 下载格式 | 是否需要 | 原因 |
|---------|---------|------|
| `.zip` / `.tar.gz` 预编译二进制 | ❌ 通常不需要 | 解压即可用 |
| `.msi` (Windows) | ✅ 需要 | 必须用 `msiexec` 安装 |
| `.pkg` (macOS) | ✅ 需要 | 必须用 `installer` 或脚本 |
| 包含 `install` 脚本 | ✅ 需要 | 需要运行脚本 |
| 需要初始化配置 | ✅ 需要 | 首次运行设置 |

**示例**：

```rust
// AWS CLI - Windows MSI 安装
async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        let install_dir = ctx.paths.store_dir().join("aws").join(version);
        let msi_file = install_dir.join("AWSCLIV2.msi");

        let output = Command::new("msiexec.exe")
            .arg("/i")
            .arg(&msi_file)
            .arg("/qn")                     // 静默安装
            .arg("/norestart")               // 不重启
            .arg(format!("TARGETDIR={}", install_dir.display()))
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("MSI 安装失败"));
        }
    }

    Ok(())
}

// AWS CLI - Linux/macOS 运行安装脚本
async fn post_install(&self, version: &str, ctx: &RuntimeContext) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let install_dir = ctx.paths.store_dir().join("aws").join(version);
        let install_script = install_dir.join("aws").join("install");

        // 设置可执行权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&install_script)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&install_script, perms)?;
        }

        // 运行安装脚本
        let output = Command::new(&install_script)
            .arg("--install-dir").arg(install_dir.join("aws-cli"))
            .arg("--bin-dir").arg(install_dir.join("bin"))
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("安装脚本执行失败"));
        }
    }

    Ok(())
}
```

---

### 5. 验证阶段 (Verify)

**职责**：确保安装完整且可执行

##### `verify_installation(version: &str, install_path: &Path, platform: &Platform) -> VerificationResult`

检查安装是否成功。

**默认行为**：
1. 检查 `executable_relative_path()` 指向的文件是否存在
2. 检查文件是否有可执行权限（Unix）

**何时需要自定义**：
- 可执行文件可能在多个位置
- 需要额外的健康检查
- 需要验证依赖文件

**示例**：

```rust
fn verify_installation(
    &self,
    _version: &str,
    install_path: &Path,
    _platform: &Platform,
) -> VerificationResult {
    // 尝试多个可能的位置
    let possible_paths = vec![
        install_path.join("bin").join("aws"),
        install_path.join("aws-cli").join("aws"),
        install_path.join("aws").join("dist").join("aws"),
    ];

    for exe_path in &possible_paths {
        if exe_path.exists() {
            return VerificationResult::success(exe_path.clone());
        }
    }

    VerificationResult::failure(
        vec![format!(
            "可执行文件未在 {} 中找到。已检查：{}",
            install_path.display(),
            possible_paths.iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )],
        vec!["尝试手动运行安装脚本。".to_string()],
    )
}
```

---

## 可执行文件路径配置

Runtime trait 提供了分层的可执行文件路径配置方法：

### 简单情况（推荐）

**只需覆盖这些方法之一**：

```rust
// 1. 可执行文件名与 runtime 名不同
fn executable_name(&self) -> &str {
    "python3"  // Runtime 名是 "python"
}

// 2. 使用 .cmd 或 .bat (Windows)
fn executable_extensions(&self) -> &[&str] {
    &[".cmd", ".exe"]  // npm、yarn 使用 .cmd
}

// 3. 可执行文件在子目录
fn executable_dir_path(&self, version: &str, platform: &Platform) -> Option<String> {
    let dir = format!("node-v{}-{}", version, platform.as_str());
    if platform.is_windows() {
        Some(dir)  // node-v22.0.0-win-x64/node.exe
    } else {
        Some(format!("{}/bin", dir))  // node-v22.0.0-linux-x64/bin/node
    }
}
```

### 复杂情况

**只在必要时覆盖**：

```rust
fn executable_relative_path(&self, version: &str, platform: &Platform) -> String {
    // 自定义完整路径逻辑
    // 只在上述方法无法满足时使用
}
```

---

## 最佳实践

### ✅ 推荐做法

1. **优先使用官方 API**：使用 `GitHubVersionFetcher` 或 `NpmVersionFetcher`
2. **最小化 hook 使用**：只在必要时实现 hook
3. **清晰的错误消息**：提供可操作的错误提示
4. **跨平台测试**：在所有支持的平台上测试
5. **文档化特殊行为**：在注释中说明为什么需要自定义逻辑

### ❌ 避免做法

1. **不要在 `post_extract()` 中执行异步操作**：这是同步方法
2. **不要在 `verify_installation()` 中修改文件**：这只是验证
3. **不要硬编码平台假设**：使用 `Platform::is_windows()` 等方法
4. **不要在多个 hook 中重复逻辑**：选择最合适的 hook

---

## Provider 开发检查清单

创建新 Provider 时，按以下顺序实现：

- [ ] **基础信息**
  - [ ] `name()` - Runtime 名称
  - [ ] `description()` - 简短描述
  - [ ] `ecosystem()` - 所属生态系统
  - [ ] `aliases()` - 别名（如果有）

- [ ] **版本管理**
  - [ ] `fetch_versions()` - 获取版本列表
  - [ ] `download_url()` - 生成下载 URL

- [ ] **可执行文件路径**
  - [ ] `executable_name()` / `executable_dir_path()` / `executable_extensions()` 之一
  - [ ] 或 `executable_relative_path()`（复杂情况）

- [ ] **平台支持**
  - [ ] `supported_platforms()` - 列出支持的平台

- [ ] **生命周期 Hook**（按需）
  - [ ] `pre_install()` - 安装前检查
  - [ ] `post_extract()` - 提取后处理
  - [ ] `post_install()` - 运行安装脚本/MSI
  - [ ] `verify_installation()` - 自定义验证逻辑

- [ ] **测试**
  - [ ] 单元测试（`tests/` 目录）
  - [ ] 跨平台测试
  - [ ] 版本解析测试

---

## 示例 Provider

### 简单 Provider（预编译二进制）

```rust
#[async_trait]
impl Runtime for SimpleRuntime {
    fn name(&self) -> &str { "simple" }

    async fn fetch_versions(&self, ctx: &RuntimeContext) -> Result<Vec<VersionInfo>> {
        let fetcher = GitHubVersionFetcher::new("owner/repo");
        fetcher.fetch_versions(&ctx.http).await
    }

    async fn download_url(&self, version: &str, platform: &Platform) -> Result<Option<String>> {
        Ok(Some(format!(
            "https://github.com/owner/repo/releases/download/v{}/simple-{}-{}.tar.gz",
            version,
            platform.os.as_str(),
            platform.arch.as_str()
        )))
    }

    // 其他方法使用默认实现
}
```

### 复杂 Provider（需要安装步骤）

参考 `vx-providers/awscli` 的完整实现。

---

## 参考资料

- [Runtime Trait 文档](../../crates/vx-runtime/src/runtime.rs)
- [AWS CLI Provider 示例](../../crates/vx-providers/awscli/)
