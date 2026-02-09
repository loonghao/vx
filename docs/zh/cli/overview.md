# CLI 概览

vx 提供了全面的命令行界面，用于管理开发工具、项目环境和开发工作流。

## 基本语法

```bash
# 子命令模式
vx [选项] <命令> [参数]

# 直接执行模式（透明代理）
vx [选项] <工具> [工具参数]
```

## 全局选项

| 选项 | 描述 |
|------|------|
| `--verbose`, `-v` | 启用详细输出 |
| `--debug` | 启用调试输出 |
| `--use-system-path` | 使用系统 PATH 而非 vx 管理的工具 |
| `--no-auto-install` | 禁用缺失工具的自动安装 |
| `--help`, `-h` | 显示帮助 |
| `--version`, `-V` | 显示版本 |

## 命令一览

### 工具管理

| 命令 | 别名 | 描述 |
|------|------|------|
| [`install`](./install) | `i` | 安装工具版本（`vx install node@22 python@3.12`） |
| `uninstall` | — | 卸载已安装的工具版本 |
| [`list`](./list) | `ls` | 列出已安装工具和可用运行时 |
| `versions` | — | 显示工具的可用版本 |
| `which` | `where` | 显示当前活跃工具的路径 |
| `switch` | — | 切换到不同的已安装版本 |
| `search` | — | 搜索可用工具 |
| [`test`](./test) | — | 测试运行时可用性和 Provider 功能 |
| [`global`](./global) | `g` | 管理全局安装的包（隔离） |

### 项目管理

| 命令 | 别名 | 描述 |
|------|------|------|
| `init` | — | 为项目初始化 `vx.toml` |
| `add` | — | 向 `vx.toml` 添加工具需求 |
| `remove` | `rm` | 从 `vx.toml` 移除工具 |
| `sync` | — | 同步项目工具与 `vx.toml` |
| `lock` | — | 生成/更新 `vx.lock` |
| `check` | — | 检查版本约束和工具可用性 |
| `bundle` | — | 离线开发环境打包 |
| `analyze` | — | 分析项目依赖、脚本和工具 |

### 脚本与环境

| 命令 | 描述 |
|------|------|
| [`run`](./run) | 运行 `vx.toml` 中定义的脚本 |
| [`dev`](./dev) | 进入开发环境（交互式 Shell） |
| [`setup`](./setup) | 安装所有项目工具并运行设置钩子 |
| [`env`](./env) | 管理虚拟环境 |

### 配置与 Shell

| 命令 | 别名 | 描述 |
|------|------|------|
| [`config`](./config) | `cfg` | 管理全局和项目配置 |
| [`shell`](./shell) | — | Shell 集成（初始化、补全） |

### 扩展与插件

| 命令 | 别名 | 描述 |
|------|------|------|
| [`ext`](./ext) | `extension` | 管理 vx 扩展 |
| `x` | — | 执行扩展命令 |
| [`plugin`](./plugin) | — | 管理 Provider 插件 |

### 系统与维护

| 命令 | 描述 |
|------|------|
| [`info`](./info) | 显示系统信息、能力和诊断 |
| [`metrics`](./metrics) | 查看执行性能指标 |
| `cache` | 缓存管理（信息、列表、清理、清除） |
| `self-update` | 更新 vx 自身 |
| `version` | 显示 vx 版本 |
| `migrate` | 迁移配置和数据格式 |
| `hook` | 管理生命周期钩子 |
| `services` | 开发服务管理 |
| `container` | 容器/Dockerfile 管理 |
| `auth` | 认证管理 |

## 直接工具执行

使用工具名称作为命令来运行任何管理的工具：

```bash
vx node --version        # 运行 Node.js
vx python script.py      # 运行 Python 脚本
vx go build ./...        # 构建 Go 项目
vx cargo test            # 运行 Rust 测试
vx uv run pytest         # 通过 uv 运行 Python 测试
```

工具在首次使用时自动安装，依赖也会被解析和安装（例如 `vx npm install` 会确保 Node.js 可用）。

## 包执行语法

使用统一语法按需执行包：

```
vx <生态系统>[@运行时版本]:<包名>[@版本][::可执行文件] [参数...]
```

### 示例

```bash
# 运行包的可执行文件
vx npm:typescript::tsc --version     # TypeScript 编译器
vx pip:httpie::http GET example.com  # HTTPie 客户端

# npm 作用域包
vx npm:@biomejs/biome::biome check .

# 版本锁定
vx npm:typescript@5.3::tsc --version

# 指定运行时版本
vx npm@20:typescript::tsc --version  # 使用 Node.js 20
```

### 支持的生态系统

| 生态系统 | 运行时 | 示例 |
|----------|--------|------|
| `npm` | Node.js | `vx npm:typescript::tsc` |
| `pip` / `uv` | Python | `vx pip:httpie::http` |
| `cargo` | Rust | `vx cargo:ripgrep::rg` |
| `go` | Go | `vx go:golangci-lint` |
| `bun` | Bun | `vx bun:typescript::tsc` |
| `yarn` / `pnpm` | Node.js | `vx yarn:typescript::tsc` |

当包名与可执行文件名不同时使用 `::` 分隔。

## 退出码

| 退出码 | 含义 |
|--------|------|
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
| `VX_AUTO_INSTALL` | 启用/禁用自动安装（`true`/`false`） |
| `VX_VERBOSE` | 启用详细输出 |
| `VX_DEBUG` | 启用调试输出 |
| `VX_CDN_ENABLED` | 启用 CDN 加速 |

## 获取帮助

```bash
# 通用帮助
vx --help

# 命令专属帮助
vx install --help
vx env --help
vx global --help
```
