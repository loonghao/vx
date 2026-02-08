# CLI 概览

vx 提供了一个全面的命令行界面来管理开发工具。

## 基本语法

```bash
vx [选项] <命令> [参数]
vx [选项] <工具> [工具参数]
```

## 全局选项

| 选项 | 描述 |
|------|------|
| `--verbose`, `-v` | 启用详细输出 |
| `--debug` | 启用调试输出 |
| `--use-system-path` | 使用系统 PATH 而不是 vx 管理的工具 |
| `--help`, `-h` | 显示帮助 |
| `--version`, `-V` | 显示版本 |

## 命令

### 工具管理

| 命令 | 描述 |
|------|------|
| [`install`](./install) | 安装工具版本 |
| [`list`](./list) | 列出可用工具 |
| `uninstall` | 删除工具版本 |
| `which` | 显示工具位置 |
| `versions` | 显示可用版本 |
| `switch` | 切换到不同版本 |

### 项目管理

| 命令 | 描述 |
|------|------|
| `init` | 初始化项目配置 |
| [`setup`](./setup) | 安装所有项目工具 |
| `sync` | 同步工具与配置 |
| `add` | 向项目配置添加工具 |
| `remove` | 从项目配置删除工具 |

### 脚本执行

| 命令 | 描述 |
|------|------|
| [`run`](./run) | 运行 `vx.toml` 中的脚本 |
| [`dev`](./dev) | 进入开发环境 |

### 环境管理

| 命令 | 描述 |
|------|------|
| [`env`](./env) | 管理环境 |
| [`global`](./global) | 管理全局包（隔离安装） |
| `venv` | Python 虚拟环境管理 |

### 配置

| 命令 | 描述 |
|------|------|
| [`config`](./config) | 管理配置 |
| [`shell`](./shell) | Shell 集成 |

### Extension 管理

| 命令 | 描述 |
|------|------|
| [`ext`](./ext) | 管理扩展 |
| `x` | 执行扩展命令 |

### 维护

| 命令 | 描述 |
|------|------|
| `clean` | 清理缓存和孤立文件 |
| `stats` | 显示磁盘使用统计 |
| [`metrics`](./metrics) | 查看性能指标和诊断信息 |
| `self-update` | 更新 vx 本身 |

## 直接工具执行

通过将工具作为命令来运行任何工具：

```bash
vx node --version
vx python script.py
vx go build
vx cargo test
```

## 包执行（RFC 0027）

使用统一语法执行全局安装包或按需运行包，无需显式安装：

### 语法

```
vx <生态系统>[@运行时版本]:<包名>[@版本][::可执行文件] [参数...]
```

### 示例

```bash
# 直接运行已安装包的可执行文件
vx tsc --version                    # 执行已安装的 typescript 中的 tsc

# 显式包语法
vx npm:typescript::tsc --version    # 包名与可执行文件名不同
vx pip:httpie::http GET example.com # httpie 包提供 'http' 命令
vx npm:eslint .                     # 包名 = 可执行文件名

# 带作用域的 npm 包
vx npm:@openai/codex::codex         # @scope/包名::可执行文件
vx npm:@biomejs/biome::biome check .

# 指定包版本
vx npm:typescript@5.3::tsc --version
vx pip:ruff@0.3 check .

# 指定运行时版本
vx npm@20:typescript::tsc --version  # 使用 Node.js 20
vx pip@3.11:black .                  # 使用 Python 3.11
```

### 支持的生态系统

| 生态系统 | 运行时 | 示例 |
|----------|--------|------|
| `npm` | Node.js | `vx npm:typescript::tsc` |
| `pip` | Python | `vx pip:httpie::http` |
| `uv` | Python | `vx uv:ruff` |
| `cargo` | Rust | `vx cargo:ripgrep::rg` |
| `go` | Go | `vx go:golangci-lint` |
| `bun` | Bun | `vx bun:typescript::tsc` |
| `yarn` | Node.js | `vx yarn:typescript::tsc` |
| `pnpm` | Node.js | `vx pnpm:typescript::tsc` |

### `::` 分隔符说明

当包名与可执行文件名不同时，使用 `::` 分隔符：

| 包名 | 可执行文件 | 命令 |
|------|------------|------|
| `typescript` | `tsc` | `vx npm:typescript::tsc` |
| `httpie` | `http` | `vx pip:httpie::http` |
| `@openai/codex` | `codex` | `vx npm:@openai/codex::codex` |
| `ripgrep` | `rg` | `vx cargo:ripgrep::rg` |

### 自动安装

如果包尚未安装，vx 会自动下载并安装（类似于 npx/uvx 的行为）：

```bash
# 首次运行会自动安装
vx npm:typescript --version     # 自动安装 typescript
vx pip:ruff check .             # 自动安装 ruff
```

### 与 npx/uvx 对比

| 场景 | npx | uvx | vx |
|------|-----|-----|-----|
| 包名=可执行文件 | `npx eslint` | `uvx ruff` | `vx npm:eslint` 或 `vx pip:ruff` |
| 包名≠可执行文件 | `npx -p typescript tsc` | `uvx --from httpie http` | `vx npm:typescript::tsc` |
| 指定版本 | `npx eslint@8` | `uvx ruff@0.3` | `vx npm:eslint@8` 或 `vx pip:ruff@0.3` |
| 跨语言支持 | ❌ 仅 Node | ❌ 仅 Python | ✅ 所有生态系统 |
| 运行时版本 | ❌ 不支持 | `uvx --python 3.11 ruff` | `vx pip@3.11:ruff` |

## 退出码

| 代码 | 含义 |
|------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 命令未找到 |
| 3 | 工具未安装 |
| 4 | 配置错误 |

## 环境变量

| 变量 | 描述 |
|------|------|
| `VX_HOME` | 覆盖 vx 数据目录 |
| `VX_ENV` | 当前环境名称 |
| `VX_AUTO_INSTALL` | 启用/禁用自动安装 |
| `VX_VERBOSE` | 启用详细输出 |
| `VX_DEBUG` | 启用调试输出 |

## 获取帮助

```bash
# 一般帮助
vx --help

# 命令特定帮助
vx install --help
vx env --help
```
