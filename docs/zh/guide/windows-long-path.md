# Windows 长路径支持

Windows 系统传统上有 260 个字符的路径长度限制（`MAX_PATH`）。这在处理深层嵌套的目录结构时可能会出现问题，尤其是在 Node.js 项目中的 `node_modules` 依赖很常见。

vx 内置了长路径支持，确保即使在复杂的项目结构下也能顺畅运行。

## 问题描述

当您安装具有深层依赖树的 npm 包时，生成的路径很容易超过 260 个字符：

```
C:\Users\用户名\.vx\store\node\20.0.0\node_modules\@scope\package\node_modules\another\node_modules\deeply\nested\file.js
```

这可能导致：
- 安装失败
- 文件访问错误
- 压缩包解压失败

## 解决方案

vx 通过多层方式解决此问题：

### 1. 自动检测和警告

在安装过程中，vx 会自动检查 Windows 长路径支持是否已启用。如果未启用，会显示有用的提示信息：

```powershell
# 运行 install.ps1 时
⚠️  Windows 长路径支持未启用

vx 可能在处理深层目录路径（>260 字符）时遇到问题，
特别是在安装具有嵌套依赖的 npm 包时。

启用长路径支持的方法（推荐）：

方法一：运行此 PowerShell 命令（需要管理员权限）：
  New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
      -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force

方法二：通过组策略（Windows 10 专业版/企业版）：
  1. 打开 gpedit.msc
  2. 导航到：计算机配置 > 管理模板 > 系统 > 文件系统
  3. 启用"启用 Win32 长路径"

方法三：使用较短的 VX_HOME 路径：
  $env:VX_HOME = "C:\vx"
```

### 2. 内置扩展路径支持

即使没有系统级的长路径支持，vx 在解压压缩包时也会在内部使用 Windows 扩展长度路径前缀（`\\?\`）。这允许路径长度达到 32,767 个字符。

当路径接近或超过 260 字符限制时：
- vx 记录警告日志
- 自动将路径转换为扩展格式（`\\?\C:\...`）
- 继续成功执行操作

### 3. 短基础路径选项

您可以配置 vx 使用较短的基础路径，以最大程度减少路径长度问题：

```powershell
# 设置较短的 VX_HOME 目录
$env:VX_HOME = "C:\vx"

# 添加到 PowerShell 配置文件以持久保存
Add-Content $PROFILE '$env:VX_HOME = "C:\vx"'
```

## 启用 Windows 长路径支持

### 方法一：PowerShell（推荐）

以管理员身份运行 PowerShell 并执行：

```powershell
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" `
    -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### 方法二：组策略（Windows 10/11 专业版/企业版）

1. 按 `Win + R`，输入 `gpedit.msc`，按回车
2. 导航到：**计算机配置** → **管理模板** → **系统** → **文件系统**
3. 双击 **启用 Win32 长路径**
4. 选择 **已启用** 并点击 **确定**

### 方法三：注册表编辑器

1. 按 `Win + R`，输入 `regedit`，按回车
2. 导航到：`HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\FileSystem`
3. 找到或创建 `LongPathsEnabled`（DWORD 类型）
4. 将值设置为 `1`

> **注意：** 启用长路径支持后，请重启终端或重启 Windows 以使更改生效。

## 检查当前状态

您可以检查长路径支持是否已启用：

```powershell
# 检查注册表值
Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled"

# 启用时的预期输出：
# LongPathsEnabled : 1
```

## API 参考（开发者）

vx 提供了 `vx_paths::windows` 模块用于程序化处理长路径：

```rust
use vx_paths::windows::{
    to_long_path,           // 转换为 \\?\ 格式
    from_long_path,         // 移除 \\?\ 前缀
    check_path_length,      // 检查路径是否超过限制
    is_long_path_enabled,   // 检查系统设置
    PathLengthStatus,       // Safe/Warning/TooLong
};

// 转换路径以支持扩展长度
let long_path = to_long_path(&my_path);

// 检查路径长度状态
match check_path_length(&path) {
    PathLengthStatus::Safe => { /* 正常 */ }
    PathLengthStatus::Warning { length, .. } => { 
        println!("路径接近限制：{} 字符", length);
    }
    PathLengthStatus::TooLong { length, .. } => {
        println!("路径超过限制：{} 字符", length);
    }
}

// 检查系统是否启用了长路径支持
if !is_long_path_enabled() {
    println!("建议启用 Windows 长路径支持");
}
```

## 最佳实践

1. **启用系统级支持**：这是最全面的解决方案
2. **使用短基础路径**：将 `VX_HOME` 设置为短路径，如 `C:\vx`
3. **避免深层嵌套结构**：尽可能扁平化项目结构
4. **使用 pnpm**：pnpm 的扁平 `node_modules` 结构显著减少路径长度

## 故障排除

### 错误："文件名或扩展名太长"

当 Windows 无法处理长路径时会出现此错误。解决方案：
1. 启用长路径支持（见上文）
2. 将 `VX_HOME` 设置为较短的路径
3. vx 会自动尝试使用 `\\?\` 前缀

### 警告："路径长度接近 Windows 限制"

vx 检测到路径接近 260 个字符。虽然仍可工作，建议考虑：
1. 启用系统级长路径支持
2. 使用较短的 `VX_HOME` 路径

### 压缩包解压静默失败

某些压缩操作可能在没有明确错误消息的情况下失败。vx 现在会：
1. 对接近限制的路径记录警告
2. 在需要时自动使用扩展长度路径
3. 报告成功/失败及路径信息

## 相关文档

- [安装指南](./installation.md)
- [配置说明](./configuration.md)
- [环境变量](../config/env-vars.md)
