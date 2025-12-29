# 常见问题

## 一般问题

### vx 是什么？

vx 是一个通用开发工具管理器，让你可以使用一个命令管理多种开发工具（Node.js、Python、Go、Rust 等）。

### vx 与其他版本管理器有什么不同？

- **统一接口**：一个工具管理所有运行时
- **零配置**：开箱即用
- **自动安装**：首次使用时自动安装

### vx 支持哪些平台？

- Windows (x64)
- macOS (x64, ARM64)
- Linux (x64, ARM64)

## 安装问题

### 如何安装 vx？

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

:::

### 如何更新 vx？

```bash
vx self-update
```

### 如何卸载 vx？

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash -s -- --uninstall
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex -Uninstall
```

:::

## 使用问题

### 如何使用特定版本的工具？

```bash
vx node@18 --version
vx python@3.11 script.py
```

### 如何在项目中固定工具版本？

创建 `vx.toml` 文件：

```toml
[tools]
node = "20"
python = "3.11"
```

### 工具安装在哪里？

默认位置：

- Linux/macOS: `~/.local/share/vx/store/`
- Windows: `%LOCALAPPDATA%\vx\store\`

### 如何清理缓存？

```bash
vx clean --cache
```

## 故障排除

### 命令找不到

确保 vx 在你的 PATH 中：

```bash
vx --version
```

### 工具安装失败

尝试：

```bash
vx install <tool> --verbose
```

查看详细错误信息。

### 更多帮助

- [故障排除指南](/zh/appendix/troubleshooting)
- [GitHub Issues](https://github.com/loonghao/vx/issues)
