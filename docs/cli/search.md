# vx search - 搜索工具

搜索可用的工具和版本。

## 语法

```bash
vx search [OPTIONS] [QUERY]
```

## 描述

`vx search` 命令用于搜索可用的工具。可以按名称、类别或描述搜索工具。

## 选项

- `--category <CATEGORY>` - 按类别过滤搜索结果
- `--installed-only` - 仅显示已安装的工具
- `--available-only` - 仅显示可安装但未安装的工具
- `--format <FORMAT>` - 输出格式 (table, json, yaml)
- `-v, --verbose` - 显示详细信息

## 参数

- `QUERY` - 搜索查询字符串（可选）

## 示例

### 基本搜索
```bash
# 搜索所有工具
vx search

# 搜索包含 "python" 的工具
vx search python

# 搜索 Node.js 相关工具
vx search node
```

### 按类别搜索
```bash
# 搜索 Python 类别的工具
vx search --category python

# 搜索 JavaScript 类别的工具
vx search --category javascript

# 列出所有可用类别
vx search --category
```

### 过滤搜索
```bash
# 仅显示已安装的工具
vx search --installed-only

# 仅显示可安装但未安装的工具
vx search --available-only

# 搜索已安装的 Python 工具
vx search python --installed-only
```

### 输出格式
```bash
# 表格格式（默认）
vx search python

# JSON 格式
vx search python --format json

# YAML 格式
vx search python --format yaml
```

## 工具类别

vx 支持以下工具类别：

- `javascript` - JavaScript/Node.js 工具
- `python` - Python 工具和解释器
- `go` - Go 语言工具
- `rust` - Rust 工具链
- `java` - Java 开发工具
- `dotnet` - .NET 工具
- `php` - PHP 工具
- `ruby` - Ruby 工具
- `package-manager` - 包管理器
- `build-tool` - 构建工具
- `utility` - 实用工具

## 输出示例

```bash
$ vx search python
搜索工具: python

┌─────────┬─────────┬──────────┬─────────────────────────────┐
│ 名称    │ 类别    │ 状态     │ 描述                        │
├─────────┼─────────┼──────────┼─────────────────────────────┤
│ python  │ python  │ 可安装   │ Python 解释器               │
│ uv      │ python  │ 已安装   │ 极快的 Python 包管理器     │
│ poetry  │ python  │ 可安装   │ Python 依赖管理和打包工具   │
│ pipenv  │ python  │ 可安装   │ Python 虚拟环境管理工具     │
└─────────┴─────────┴──────────┴─────────────────────────────┘

找到 4 个工具
```

### 详细信息输出
```bash
$ vx search python --verbose
搜索工具: python

python (python)
  状态: 可安装
  描述: Python 解释器
  最新版本: 3.12.1
  可用版本: 3.8.18, 3.9.18, 3.10.13, 3.11.7, 3.12.1
  官网: https://python.org
  
uv (python)
  状态: 已安装 (0.1.5)
  描述: 极快的 Python 包管理器
  最新版本: 0.1.5
  已安装版本: 0.1.5
  官网: https://github.com/astral-sh/uv
```

## 搜索技巧

### 模糊搜索
```bash
# 搜索包含 "js" 的工具
vx search js

# 搜索包含 "build" 的工具
vx search build
```

### 组合搜索
```bash
# 搜索已安装的构建工具
vx search build --installed-only --category build-tool

# 搜索可用的 Python 包管理器
vx search manager --category python --available-only
```

## 注意事项

1. **网络连接**: 搜索可能需要网络连接来获取最新信息
2. **缓存机制**: 搜索结果会被缓存以提高性能
3. **大小写**: 搜索不区分大小写
4. **正则表达式**: 支持基本的正则表达式模式

## 相关命令

- [`vx list`](./list.md) - 列出已安装的工具
- [`vx install`](./install.md) - 安装工具
- [`vx plugin list`](./plugin.md) - 列出插件
