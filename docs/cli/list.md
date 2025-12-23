# vx list - 列出工具

列出支持的工具和已安装的版本。

## 语法

```bash
vx list [tool] [options]
```

## 参数

- `[tool]` - 可选的工具名称，指定时显示该工具的详细信息

## 选项

- `--status` - 显示安装状态
- `--verbose` - 显示详细信息
- `--installed-only` - 仅显示已安装的工具
- `--available-only` - 仅显示可用但未安装的工具
- `--format <format>` - 输出格式：`table`, `json`, `yaml`

## 示例

### 基本使用

```bash
# 列出所有支持的工具
vx list

# 列出特定工具的版本
vx list node
vx list python
vx list go
```

### 显示安装状态

```bash
# 显示所有工具的安装状态
vx list --status

# 仅显示已安装的工具
vx list --installed-only

# 仅显示可用但未安装的工具
vx list --available-only
```

### 详细信息

```bash
# 显示详细信息
vx list --verbose

# 以JSON格式输出
vx list --format json

# 显示特定工具的详细信息
vx list node --verbose
```

## 输出格式

### 默认输出

```
Available Tools:
  node     Node.js JavaScript runtime
  python   Python programming language
  go       Go programming language
  rust     Rust programming language
  uv       Python package installer

Installed Tools:
  node@18.17.0    (active)
  node@20.10.0
  python@3.11.5   (active)
  uv@0.1.0        (active)
```

### 状态输出

```bash
$ vx list --status
Tool     Status      Active Version    Available Versions
node     installed   18.17.0          16.20.0, 18.17.0, 20.10.0
python   installed   3.11.5           3.9.18, 3.10.13, 3.11.5
go       available   -                1.20.12, 1.21.6
uv       installed   0.1.0            0.1.0, 0.1.1
```

### 详细输出

```bash
$ vx list node --verbose
Tool: node
Description: Node.js JavaScript runtime
Category: runtime
Homepage: https://nodejs.org/
Documentation: https://nodejs.org/docs/

Installed Versions:
  18.17.0 (active)
    - Path: ~/.vx/tools/node/18.17.0/
    - Size: 45.2 MB
    - Installed: 2024-01-15 10:30:00
  20.10.0
    - Path: ~/.vx/tools/node/20.10.0/
    - Size: 47.8 MB
    - Installed: 2024-01-10 14:20:00

Available Versions:
  16.20.0, 18.17.0, 20.10.0, 21.5.0

Configuration:
  Registry: https://nodejs.org/dist/
  Auto-install: enabled
  Update check: enabled
```

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

## 过滤和搜索

### 按类别过滤

```bash
# 显示特定类别的工具
vx list --category runtime
vx list --category package-manager
vx list --category build-tool
```

### 按状态过滤

```bash
# 仅显示已安装的工具
vx list --installed-only

# 仅显示可更新的工具
vx list --updatable-only

# 仅显示有问题的工具
vx list --issues-only
```

## JSON 输出格式

```bash
$ vx list --format json
{
  "tools": [
    {
      "name": "node",
      "description": "Node.js JavaScript runtime",
      "category": "runtime",
      "homepage": "https://nodejs.org/",
      "installed_versions": [
        {
          "version": "18.17.0",
          "active": true,
          "path": "~/.vx/tools/node/18.17.0/",
          "size": 47382528,
          "installed_at": "2024-01-15T10:30:00Z"
        }
      ],
      "available_versions": [
        "16.20.0",
        "18.17.0",
        "20.10.0",
        "21.5.0"
      ],
      "configuration": {
        "registry": "https://nodejs.org/dist/",
        "auto_install": true,
        "update_check": true
      }
    }
  ]
}
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
- [update](./update.md) - 更新工具
- [search](./search.md) - 搜索工具
- [plugin](./plugin.md) - 插件管理
