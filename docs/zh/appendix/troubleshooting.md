# 故障排除

常见问题的解决方案。

## 安装问题

### 安装脚本失败

**症状**：安装脚本报错或无响应。

**解决方案**：

1. 检查网络连接
2. 尝试手动下载：
   - 访问 [Releases](https://github.com/loonghao/vx/releases)
   - 下载适合你平台的二进制文件
   - 手动添加到 PATH

### PATH 未更新

**症状**：安装后 `vx` 命令找不到。

**解决方案**：

1. 重启终端
2. 手动添加到 PATH：

::: code-group

```bash [Linux/macOS]
export PATH="$HOME/.local/bin:$PATH"
```

```powershell [Windows]
$env:Path = "$env:USERPROFILE\.vx\bin;$env:Path"
```

:::

## 工具安装问题

### 工具下载失败

**症状**：工具安装时下载失败。

**解决方案**：

1. 检查网络连接
2. 尝试使用代理：

```bash
export HTTP_PROXY=http://proxy:port
export HTTPS_PROXY=http://proxy:port
vx install node
```

3. 清理缓存后重试：

```bash
vx clean --cache
vx install node
```

### 版本不可用

**症状**：请求的版本不存在。

**解决方案**：

1. 列出可用版本：

```bash
vx list node --available
```

2. 使用有效的版本号

## 运行时问题

### 工具执行失败

**症状**：工具安装成功但执行失败。

**解决方案**：

1. 使用详细模式：

```bash
vx --verbose node --version
```

2. 检查权限：

```bash
ls -la ~/.local/share/vx/store/node/
```

### 环境变量问题

**症状**：环境变量未正确设置。

**解决方案**：

1. 检查 shell 集成：

```bash
echo $VX_ENV
echo $PATH | grep vx
```

2. 重新初始化 shell 集成：

```bash
eval "$(vx shell init bash)"
```

## Shell 集成问题

### 自动切换不工作

**症状**：进入项目目录时环境不自动切换。

**解决方案**：

1. 确保 shell 集成已启用
2. 检查 `.vx.toml` 文件存在
3. 重启终端

### 补全不工作

**症状**：Tab 补全不工作。

**解决方案**：

1. 重新生成补全脚本：

```bash
vx shell completions bash > ~/.local/share/bash-completion/completions/vx
```

2. 重启终端

## 获取更多帮助

- 启用调试模式：`VX_DEBUG=true vx <command>`
- 查看日志：`~/.local/share/vx/logs/`
- 提交 Issue：[GitHub Issues](https://github.com/loonghao/vx/issues)
