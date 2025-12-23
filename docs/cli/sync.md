# vx sync - 项目同步

同步安装项目所需的所有工具，确保项目环境一致性。

## 语法

```bash
vx sync [options]
```

## 描述

`vx sync` 命令读取项目配置文件（`.vx.toml`），自动安装所有必需的工具和版本，确保项目环境的一致性。这是设置新项目环境的最快方式。

## 选项

- `--check` - 仅检查不安装，显示需要安装的工具
- `--force` - 强制重新安装所有工具，即使已存在
- `--dry-run` - 预览操作，不实际执行
- `--verbose` - 显示详细的同步过程
- `--parallel` - 并行安装多个工具（默认启用）
- `--no-auto-install` - 禁用自动安装，仅检查已安装的工具

## 示例

### 基本同步

```bash
# 同步安装项目所需的所有工具
vx sync

# 仅检查不安装
vx sync --check

# 预览同步操作
vx sync --dry-run
```

### 高级选项

```bash
# 强制重新安装所有工具
vx sync --force

# 显示详细过程
vx sync --verbose

# 禁用自动安装
vx sync --no-auto-install
```

## 同步过程

1. **读取配置** - 解析 `.vx.toml` 文件
2. **版本解析** - 解析每个工具的版本要求
3. **状态检查** - 检查工具是否已安装
4. **依赖分析** - 分析工具间的依赖关系
5. **安装计划** - 生成安装计划和顺序
6. **并行安装** - 并行下载和安装工具
7. **验证** - 验证所有工具安装成功
8. **配置更新** - 更新环境配置

## 项目配置格式

### 基本配置

```toml
# .vx.toml
[tools]
node = "18.17.0"          # 精确版本
uv = "latest"             # 最新版本
go = "^1.21.0"            # 语义化版本范围
python = "3.11"           # 主版本

[settings]
auto_install = true       # 启用自动安装
cache_duration = "7d"     # 版本缓存时间
parallel_install = true   # 并行安装
```

### 高级配置

```toml
[tools]
node = "18.17.0"
python = "3.11.5"
go = "1.21.6"

[settings]
auto_install = true
cache_duration = "7d"
verify_checksums = true
install_timeout = "300s"

[scripts]
post_sync = "npm install"
pre_sync = "echo 'Starting sync...'"

[env]
NODE_ENV = "development"
PYTHONPATH = "./src"
```

## 同步输出

### 成功同步

```bash
$ vx sync
🔍 Reading project configuration (.vx.toml)
📋 Found 3 tools to sync: node, python, uv

📦 Installing tools:
  ⬇️  node@18.17.0 (downloading...)
  ⬇️  python@3.11.5 (downloading...)
  ⬇️  uv@latest (resolving version...)

✅ node@18.17.0 installed successfully
✅ python@3.11.5 installed successfully
✅ uv@0.1.1 installed successfully

🎉 Project sync completed! All tools are ready.

Next steps:
  vx node --version
  vx python --version
  vx uv --version
```

### 检查模式

```bash
$ vx sync --check
🔍 Checking project requirements...

Required tools:
  ✅ node@18.17.0 (installed)
  ❌ python@3.11.5 (not installed)
  ❌ uv@latest (not installed)

Summary:
  - 1 tool already installed
  - 2 tools need installation

Run 'vx sync' to install missing tools.
```

### 预览模式

```bash
$ vx sync --dry-run
🔍 Sync plan preview:

Will install:
  📦 python@3.11.5
    - Download from: https://www.python.org/ftp/python/3.11.5/
    - Install to: ~/.vx/tools/python/3.11.5/
    - Estimated size: 25.4 MB

  📦 uv@0.1.1 (latest)
    - Download from: https://github.com/astral-sh/uv/releases/
    - Install to: ~/.vx/tools/uv/0.1.1/
    - Estimated size: 8.7 MB

Will skip:
  ⏭️  node@18.17.0 (already installed)

Total download size: ~34.1 MB
Estimated time: 2-5 minutes

Run 'vx sync' to execute this plan.
```

## 错误处理

### 配置文件错误

```bash
$ vx sync
❌ Error: Invalid .vx.toml file

   Line 3: Invalid version format 'node = "invalid"'
   Expected: semantic version (e.g., "18.17.0", "latest", "^18.0.0")

   Fix the configuration and try again.
```

### 网络错误

```bash
$ vx sync
❌ Error: Failed to download node@18.17.0

   Network error: Connection timeout

   Suggestions:
   - Check your internet connection
   - Try again with: vx sync --verbose
   - Use a mirror: vx config set mirrors.node "https://npmmirror.com/mirrors/node/"
```

## 配置检测

VX 会自动检测项目类型并建议配置：

### Node.js 项目

```bash
$ vx sync
🔍 Detected Node.js project (package.json found)
💡 Suggested configuration:

[tools]
node = "18.17.0"  # from package.json engines.node
npm = "latest"

Would you like to create .vx.toml with these settings? (y/N)
```

### Python 项目

```bash
$ vx sync
🔍 Detected Python project (pyproject.toml found)
💡 Suggested configuration:

[tools]
python = "3.11"   # from pyproject.toml requires-python
uv = "latest"

Would you like to create .vx.toml with these settings? (y/N)
```

## 故障排除

### 同步失败

```bash
# 显示详细错误信息
vx sync --verbose

# 清理缓存重试
vx cleanup --cache-only
vx sync --force

# 检查配置文件
vx config validate --local
```

### 版本冲突

```bash
# 检查版本要求
vx list node

# 更新到兼容版本
vx config set tools.node "^18.0.0" --local
vx sync
```

### 权限问题

```bash
# 检查安装目录权限
ls -la ~/.vx/tools/

# 修复权限
chmod -R 755 ~/.vx/tools/
vx sync
```

## 相关命令

- [init](./init.md) - 初始化项目配置
- [config](./config.md) - 配置管理
- [install](./install.md) - 手动安装工具
- [list](./list.md) - 列出工具状态
