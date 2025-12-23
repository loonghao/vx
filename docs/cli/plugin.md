# vx plugin - 插件管理

管理 vx 工具插件。

## 语法

```bash
vx plugin <SUBCOMMAND> [OPTIONS] [ARGS]
```

## 描述

`vx plugin` 命令用于管理 vx 的工具插件。插件定义了如何安装、管理和使用特定的工具。

## 子命令

- `list` - 列出插件
- `info` - 显示插件信息
- `enable` - 启用插件
- `disable` - 禁用插件
- `search` - 搜索插件
- `stats` - 显示插件统计

## 选项

### 全局选项

- `-v, --verbose` - 显示详细输出
- `--format <FORMAT>` - 输出格式 (table, json, yaml)

### list 子命令选项

- `--enabled` - 仅显示已启用的插件
- `--disabled` - 仅显示已禁用的插件
- `--category <CATEGORY>` - 按类别过滤

### search 子命令选项

- `--category <CATEGORY>` - 按类别搜索
- `--official-only` - 仅搜索官方插件

## 示例

### 列出插件

```bash
# 列出所有插件
vx plugin list

# 仅列出已启用的插件
vx plugin list --enabled

# 按类别过滤
vx plugin list --category python
```

### 插件信息

```bash
# 显示插件详细信息
vx plugin info uv

# 显示多个插件信息
vx plugin info node python go
```

### 启用/禁用插件

```bash
# 启用插件
vx plugin enable uv

# 禁用插件
vx plugin disable uv

# 批量启用
vx plugin enable node python go
```

### 搜索插件

```bash
# 搜索插件
vx plugin search python

# 按类别搜索
vx plugin search --category javascript

# 仅搜索官方插件
vx plugin search --official-only
```

### 插件统计

```bash
# 显示插件统计信息
vx plugin stats

# 详细统计
vx plugin stats --verbose
```

## 输出示例

### 列出插件

```bash
$ vx plugin list
VX 插件列表

┌─────────┬─────────────┬────────┬─────────────────────────────┐
│ 名称    │ 类别        │ 状态   │ 描述                        │
├─────────┼─────────────┼────────┼─────────────────────────────┤
│ node    │ javascript  │ 启用   │ Node.js 运行时环境          │
│ python  │ python      │ 启用   │ Python 解释器               │
│ uv      │ python      │ 启用   │ 极快的 Python 包管理器     │
│ go      │ go          │ 启用   │ Go 编程语言                 │
│ rust    │ rust        │ 禁用   │ Rust 编程语言               │
│ poetry  │ python      │ 禁用   │ Python 依赖管理工具         │
└─────────┴─────────────┴────────┴─────────────────────────────┘

总计: 6 个插件 (4 个启用, 2 个禁用)
```

### 插件信息

```bash
$ vx plugin info uv
插件信息: uv

基本信息:
  名称: uv
  版本: 1.0.0
  类别: python
  状态: 启用
  官方插件: 是

描述:
  极快的 Python 包管理器，用 Rust 编写，比 pip 快 10-100 倍

支持的功能:
  ✓ 版本管理
  ✓ 自动安装
  ✓ 虚拟环境集成
  ✓ 跨平台支持

支持的平台:
  ✓ Windows (x64, arm64)
  ✓ macOS (x64, arm64)
  ✓ Linux (x64, arm64)

配置选项:
  auto_install: true
  default_version: "latest"
  install_timeout: 300

相关链接:
  官网: https://github.com/astral-sh/uv
  文档: https://docs.astral.sh/uv/
  问题反馈: https://github.com/astral-sh/uv/issues
```

### 搜索插件

```bash
$ vx plugin search python
搜索插件: python

┌─────────┬─────────┬────────┬─────────────────────────────┐
│ 名称    │ 类别    │ 状态   │ 描述                        │
├─────────┼─────────┼────────┼─────────────────────────────┤
│ python  │ python  │ 启用   │ Python 解释器               │
│ uv      │ python  │ 启用   │ 极快的 Python 包管理器     │
│ poetry  │ python  │ 禁用   │ Python 依赖管理工具         │
│ pipenv  │ python  │ 可用   │ Python 虚拟环境管理工具     │
│ conda   │ python  │ 可用   │ Python 科学计算环境管理     │
└─────────┴─────────┴────────┴─────────────────────────────┘

找到 5 个相关插件
```

## 插件类别

vx 支持以下插件类别：

### 编程语言

- `javascript` - JavaScript/Node.js 生态
- `python` - Python 生态
- `go` - Go 语言生态
- `rust` - Rust 语言生态
- `java` - Java 生态
- `dotnet` - .NET 生态

### 工具类型

- `package-manager` - 包管理器
- `build-tool` - 构建工具
- `runtime` - 运行时环境
- `compiler` - 编译器
- `utility` - 实用工具

## 插件状态

### 启用状态

- `启用` - 插件已启用，可以使用
- `禁用` - 插件已禁用，不可使用
- `可用` - 插件可安装但未启用
- `错误` - 插件配置错误

### 插件来源

- `官方` - vx 官方维护的插件
- `社区` - 社区贡献的插件
- `本地` - 本地自定义插件

## 插件配置

### 全局插件配置

```toml
# ~/.vx/config/plugins.toml
[plugins.uv]
enabled = true
auto_install = true
default_version = "latest"

[plugins.node]
enabled = true
auto_install = true
default_version = "lts"
```

### 项目插件配置

```toml
# .vx.toml
[plugins]
required = ["node", "uv"]
optional = ["go", "rust"]

[plugins.node]
version_constraint = ">=18.0.0"
```

## 插件开发

### 插件结构

```
my-plugin/
├── plugin.toml          # 插件配置
├── install.sh           # 安装脚本
├── versions.json        # 版本信息
└── README.md           # 插件文档
```

### 插件配置示例

```toml
# plugin.toml
[plugin]
name = "my-tool"
version = "1.0.0"
category = "utility"
description = "My custom tool"

[install]
platforms = ["windows", "macos", "linux"]
architectures = ["x64", "arm64"]

[commands]
install = "./install.sh"
list_versions = "./list-versions.sh"
```

## 注意事项

1. **插件依赖**: 某些插件可能依赖其他插件
2. **平台支持**: 不是所有插件都支持所有平台
3. **版本兼容**: 插件版本需要与 vx 版本兼容
4. **安全性**: 仅从可信来源安装插件

## 相关命令

- [`vx list`](./list.md) - 列出已安装的工具
- [`vx search`](./search.md) - 搜索可用工具
- [`vx install`](./install.md) - 安装工具
