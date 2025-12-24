# 支持的工具概览

vx 支持广泛的开发工具和运行时。

## 工具生态系统

### Node.js 生态

| 工具 | 描述 |
|------|------|
| `node` | Node.js 运行时 |
| `npm` | Node.js 包管理器 |
| `npx` | Node.js 包运行器 |
| `pnpm` | 快速包管理器 |
| `yarn` | Yarn 包管理器 |
| `bun` | Bun JavaScript 运行时 |

### Python 生态

| 工具 | 描述 |
|------|------|
| `python` | Python 解释器 |
| `uv` | 快速 Python 包管理器 |
| `uvx` | UV 工具运行器 |
| `pip` | Python 包安装器 |

### Go 生态

| 工具 | 描述 |
|------|------|
| `go` | Go 编程语言 |

### Rust 生态

| 工具 | 描述 |
|------|------|
| `cargo` | Rust 包管理器 |
| `rustc` | Rust 编译器 |

### DevOps 工具

| 工具 | 描述 |
|------|------|
| `kubectl` | Kubernetes CLI |
| `helm` | Kubernetes 包管理器 |
| `terraform` | 基础设施即代码 |

### 实用工具

| 工具 | 描述 |
|------|------|
| `just` | 命令运行器 |
| `jq` | JSON 处理器 |
| `ripgrep` | 快速搜索工具 |

## 使用示例

```bash
# Node.js
vx node --version
vx npm install
vx npx create-react-app my-app

# Python
vx python script.py
vx uvx ruff check .

# Go
vx go build
vx go test ./...

# Rust
vx cargo build --release
```

## 版本管理

```bash
# 安装特定版本
vx install node@20
vx install python@3.11

# 列出已安装版本
vx list --installed

# 列出可用版本
vx list node --available
```
