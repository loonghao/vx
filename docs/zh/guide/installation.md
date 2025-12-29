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
   - `vx-x86_64-apple-darwin.tar.gz` - macOS x64
   - `vx-aarch64-apple-darwin.tar.gz` - macOS ARM64 (Apple Silicon)
   - `vx-x86_64-pc-windows-msvc.zip` - Windows x64

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
vx 0.5.11
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
vx self-update 0.5.28
```

self-update 命令特性：
- 多渠道下载，自动回退（GitHub → jsDelivr → Fastly）
- 下载进度条，显示速度和预计剩余时间
- SHA256 校验和验证（如果可用）
- Windows 上安全的二进制文件替换

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
