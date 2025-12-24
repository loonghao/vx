# vx list - 列出工具

列出支持的工具和已安装的版本。

## 语法

```bash
vx list [tool] [options]
```

## 参数

- `[tool]` - 可选的工具名称，指定时显示该工具的详细信息

## 选项

- `--status` - 显示安装状态和版本详情
- `--all, -a` - 显示所有工具，包括当前平台不支持的工具
- `--installed` - 仅显示已安装的工具
- `--available` - 仅显示可用但未安装的工具

## 示例

### 基本使用

```bash
# 列出当前平台支持的工具
vx list

# 列出所有工具（包括不支持的）
vx list --all
vx list -a

# 列出特定工具的版本
vx list node
vx list python
vx list go
```

### 显示安装状态

```bash
# 显示所有工具的安装状态
vx list --status

# 显示所有工具（包括不支持的）的状态
vx list --all --status

# 仅显示已安装的工具
vx list --installed

# 仅显示可用但未安装的工具
vx list --available
```

### 详细信息

```bash
# 显示特定工具的详细信息
vx list node --status
```

## 输出格式

### 默认输出（仅当前平台支持的工具）

```
📦 Available Tools (windows-x64):
  ✅ node - JavaScript runtime built on Chrome's V8 engine
  ❌ go - Go programming language
  ✅ uv - Fast Python package installer
  ❌ bun - Fast JavaScript runtime
  ...

   2 tools hidden (not supported on windows-x64). Use --all to show all.
```

### 使用 --all 显示所有工具

```bash
$ vx list --all
📦 Available Tools (showing all, including 2 unsupported):
  ✅ node - JavaScript runtime built on Chrome's V8 engine
  ❌ go - Go programming language
  ✅ uv - Fast Python package installer
  ❌ bun - Fast JavaScript runtime
  ⚠️  choco - Chocolatey package manager (not supported on linux-x64)
  ⚠️  rcedit - Windows resource editor (not supported on linux-x64)
  ...
```

### 状态输出

```bash
$ vx list --status
📦 Available Tools (windows-x64):
  ✅ node - JavaScript runtime built on Chrome's V8 engine
     Versions: 18.17.0, 20.10.0
  ❌ go - Go programming language
  ✅ uv - Fast Python package installer
     Versions: 0.1.0

📊 Summary: 2/18 tools installed
   2 tools hidden (not supported on windows-x64). Use --all to show all.
```

## 状态图标说明

| 图标 | 含义 |
|------|------|
| ✅ | 已安装 |
| ❌ | 未安装（但支持当前平台） |
| ⚠️ | 当前平台不支持（仅在 --all 模式显示） |

## 工具分类

VX 支持的工具按类别组织：

### 运行时环境

- **node** - Node.js JavaScript runtime
- **python** - Python programming language
- **go** - Go programming language
- **rust** - Rust programming language

### 包管理器

- **npm** - Node.js package manager
- **yarn** - Fast, reliable package manager
- **pnpm** - Fast, disk space efficient package manager
- **pip** - Python package installer
- **uv** - Fast Python package installer

### 构建工具

- **cargo** - Rust package manager and build tool
- **go** - Go compiler and tools

### 开发工具

- **rustc** - Rust compiler
- **gofmt** - Go code formatter

### Windows 专属工具

- **choco** - Chocolatey package manager (Windows only)
- **rcedit** - Windows resource editor (Windows only)

## 过滤和搜索

### 按状态过滤

```bash
# 仅显示已安装的工具
vx list --installed

# 仅显示可用但未安装的工具
vx list --available
```

## 故障排除

### 工具列表为空

```bash
# 检查插件状态
vx plugin list

# 重新加载配置
vx config validate

# 检查网络连接
vx --verbose list
```

### 版本信息不准确

```bash
# 刷新版本缓存
vx update --refresh-cache

# 强制更新版本信息
vx list node --refresh
```

## 相关命令

- [install](./install.md) - 安装工具
- [run](./run.md) - 运行工具
- [search](./overview.md) - 搜索工具
