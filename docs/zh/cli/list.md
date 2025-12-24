# list 命令

列出可用工具和已安装版本。

## 语法

```bash
vx list [tool] [options]
```

## 参数

| 参数 | 描述 |
|------|------|
| `tool` | 可选，特定工具名称 |

## 选项

| 选项 | 描述 |
|------|------|
| `--installed, -i` | 仅显示已安装的工具 |
| `--available, -a` | 显示可用版本 |
| `--json` | JSON 格式输出 |

## 示例

```bash
# 列出所有支持的工具
vx list

# 列出已安装的工具
vx list --installed

# 列出特定工具的可用版本
vx list node --available

# JSON 格式输出
vx list --json
```

## 输出示例

```
Supported Tools:

Node.js Ecosystem:
  node     Node.js runtime
  npm      Node.js package manager
  npx      Node.js package runner
  pnpm     Fast package manager
  yarn     Yarn package manager
  bun      Bun JavaScript runtime

Python Ecosystem:
  python   Python interpreter
  uv       Fast Python package manager
  uvx      UV tool runner
  pip      Python package installer

Go Ecosystem:
  go       Go programming language

Rust Ecosystem:
  cargo    Rust package manager
  rustc    Rust compiler
```

## 参见

- [install](./install) - 安装工具
- [versions](./versions) - 显示可用版本
