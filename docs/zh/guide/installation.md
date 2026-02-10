# 安装

vx 可以通过多种方式安装在 Windows、macOS 和 Linux 上。

## 快速安装

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

:::

### 故障排除：GitHub API 限流

如果安装时遇到限流错误，有以下几种解决方案：

**方案 1：使用 GitHub token**
```bash
# Linux/macOS
GITHUB_TOKEN='your_token' curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:GITHUB_TOKEN='your_token'; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

**方案 2：指定版本号**
```bash
# Linux/macOS
VX_VERSION='0.6.7' curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash

# Windows
$env:VX_VERSION='0.6.7'; irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

**方案 3：使用包管理器**（见下文）

## 包管理器

### Homebrew (macOS/Linux)

```bash
brew tap loonghao/vx
brew install vx
```

或直接安装：

```bash
brew install loonghao/vx/vx
```

### Scoop (Windows)

```powershell
scoop bucket add loonghao https://github.com/loonghao/scoop-bucket
scoop install vx
```

### WinGet (Windows)

```powershell
winget install loonghao.vx
```

### Cargo（从源码）

如果你已安装 Rust：

```bash
cargo install vx
```

## 手动安装

### 下载二进制文件

1. 前往 [Releases 页面](https://github.com/loonghao/vx/releases)
2. 下载适合你平台的二进制文件：

   - `vx-x86_64-unknown-linux-gnu.tar.gz` - Linux x64
   - `vx-aarch64-unknown-linux-gnu.tar.gz` - Linux ARM64
   - `vx-x86_64-unknown-linux-musl.tar.gz` - Linux x64 (静态链接)
   - `vx-aarch64-unknown-linux-musl.tar.gz` - Linux ARM64 (静态链接)
   - `vx-x86_64-apple-darwin.tar.gz` - macOS x64
   - `vx-aarch64-apple-darwin.tar.gz` - macOS ARM64 (Apple Silicon)
   - `vx-x86_64-pc-windows-msvc.zip` - Windows x64

   > **注意：** 从 v0.7.0 开始，vx 使用 [cargo-dist](https://opensource.axo.dev/cargo-dist/) 发布。主要二进制文件名不再包含版本号。对于旧版本（v0.5.x、v0.6.x），使用带版本号的命名格式 `vx-{version}-{platform}`。self-update 命令会自动处理两种命名格式，确保跨版本无缝升级。

3. 解压并添加到 PATH：

::: code-group

```bash [Linux/macOS]
tar -xzf vx-*.tar.gz
sudo mv vx /usr/local/bin/
```

```powershell [Windows]
# 解压到 PATH 中的目录
Expand-Archive vx-*.zip -DestinationPath $env:USERPROFILE\.vx\bin
# 添加到 PATH（在提升权限的 PowerShell 中运行）
[Environment]::SetEnvironmentVariable("Path", $env:Path + ";$env:USERPROFILE\.vx\bin", "User")
```

:::

## Shell 集成

为获得最佳体验，将 Shell 集成添加到你的配置文件：

::: code-group

```bash [Bash]
# 添加到 ~/.bashrc
eval "$(vx shell init bash)"
```

```zsh [Zsh]
# 添加到 ~/.zshrc
eval "$(vx shell init zsh)"
```

```fish [Fish]
# 添加到 ~/.config/fish/config.fish
vx shell init fish | source
```

```powershell [PowerShell]
# 添加到 $PROFILE
Invoke-Expression (& vx shell init powershell | Out-String)
```

:::

## 验证安装

```bash
vx --version
```

你应该看到类似这样的输出：

```
vx 0.6.27
```


## 更新 vx

要将 vx 更新到最新版本：

```bash
vx self-update
```

或检查更新但不安装：

```bash
vx self-update --check
```

安装指定版本：

```bash
vx self-update 0.7.7
```

强制重新安装（适用于安装损坏的情况）：

```bash
vx self-update --force
```

self-update 命令特性：
- **自动检测更新方式**：优先使用 cargo-dist 安装回执实现零配置更新，旧版安装回退到多渠道 CDN 下载
- 多渠道下载，自动回退（GitHub Releases → jsDelivr CDN → Fastly CDN）
- 下载进度条，显示速度和预计剩余时间
- SHA256 校验和验证（如果可用）
- Windows 上安全的二进制文件替换（处理 exe 锁定）
- 跨版本兼容：可从任何旧版本（v0.5.x、v0.6.x）更新到最新版

## 卸载

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash -s -- --uninstall
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex -Uninstall
```

:::

### 手动卸载

1. 从 PATH 中删除 vx 二进制文件
2. 删除 vx 数据目录：
   - Linux/macOS: `~/.local/share/vx`
   - Windows: `%LOCALAPPDATA%\vx`
3. 从配置文件中删除 Shell 集成
