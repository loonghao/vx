# vx check Command

## 概述

`vx check` 命令用于检测 runtime 是否可用，不会尝试安装。这对于测试和 CI/CD 流程非常有用。

## 用法

```bash
# 基本用法：检查 runtime 是否可用
vx check <runtime>

# 只检查是否已安装在 vx store
vx check <runtime> --installed

# 只检查是否在系统 PATH 中可用
vx check <runtime> --system

# 显示详细的检测信息
vx check <runtime> --detailed

# 静默模式（只返回退出码，不输出）
vx check <runtime> --quiet
```

## 退出码

- `0`: Runtime 可用
- `1`: Runtime 不可用

## 示例

### 检查 Node.js 是否可用

```bash
$ vx check node
✓ Runtime 'node' is installed in vx store
  Versions: 20.0.0, 18.16.0
✓ Runtime 'node' is available on system PATH
  Version: 20.0.0
  Path: C:\Users\user\.vx\store\node\20.0.0\bin\node.exe

✓ Runtime 'node' is available
```

### 检查不支持平台的工具

```bash
$ vx check systemctl
❌ Runtime 'systemctl' does not support the current platform (windows-x64)
   Supported platforms:
   - linux-x64
   - linux-arm64
```

退出码: 1

### 检查未安装且不能自动安装的工具

```bash
$ vx check systemctl --detailed
✗ Runtime 'systemctl' is not installed in vx store
✗ Runtime 'systemctl' is not available on system PATH
✗ Runtime 'systemctl' cannot be auto-installed
  Please install it manually

✗ Runtime 'systemctl' is not available
```

退出码: 1

### 静默模式（用于脚本）

```bash
# 在脚本中使用
if vx check node --quiet; then
    echo "Node.js is available"
else
    echo "Node.js is not available"
    vx install node
fi
```

## 在测试脚本中使用

测试脚本可以使用 `vx check` 来跳过不可用的工具：

```powershell
# PowerShell 示例
function Test-RuntimeAvailable {
    param([string]$Runtime)
    
    try {
        $output = & vx check $Runtime --quiet 2>&1
        $exitCode = $LASTEXITCODE
        return $exitCode -eq 0
    } catch {
        return $false
    }
}

# 使用示例
if (Test-RuntimeAvailable -Runtime "systemctl") {
    # 测试 systemctl
} else {
    Write-Warning "systemctl not available, skipping tests"
}
```

```bash
# Bash 示例
if vx check systemctl --quiet; then
    # 测试 systemctl
    vx systemctl --version
else
    echo "⚠ systemctl not available on this platform, skipping"
fi
```

## 检测逻辑

`vx check` 使用以下逻辑检测 runtime 可用性：

1. **平台支持检查**: 检查 runtime 是否支持当前平台（OS + 架构）
2. **安装状态检查**: 检查是否已安装在 vx store 中
3. **系统可用性检查**: 使用 `detection.command` 检测系统 PATH 中的版本
4. **自动安装能力**: 检查 runtime 是否可以通过 vx 自动安装

## 与其他命令的区别

| 命令 | 用途 | 是否安装 |
|------|------|---------|
| `vx check` | 检测可用性 | ❌ 不安装 |
| `vx list` | 列出已安装版本 | ❌ 不安装 |
| `vx <runtime>` | 执行 runtime | ✓ 自动安装（如果需要） |
| `vx install` | 安装 runtime | ✓ 强制安装 |

## 相关命令

- `vx list <runtime>` - 列出已安装的版本
- `vx which <runtime>` - 显示当前使用的版本路径
- `vx install <runtime>` - 安装 runtime
