# env 命令

管理 vx 环境。

## 概述

vx 支持两种类型的环境：

- **项目环境**：创建在项目目录下的 `.vx/env/`。当存在 `.vx.toml` 时，这是默认选项。
- **全局环境**：创建在 `~/.vx/envs/`，用于跨项目使用。

所有工具都全局存储在 `~/.vx/store/`（内容寻址存储）。环境包含指向全局 store 的软链接，节省磁盘空间的同时允许每个项目有独立的工具配置。

## 语法

```bash
vx env <subcommand> [options]
```

## 子命令

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新环境（项目或全局） |
| `use` | 激活环境 |
| `list` | 列出所有环境 |
| `delete` | 删除环境 |
| `show` | 显示环境详情 |
| `add` | 向环境添加工具 |
| `remove` | 从环境删除工具 |
| `sync` | 从 .vx.toml 同步项目环境 |

> **注意**：如需 shell 激活（导出 PATH），请使用 `vx dev --export`。详见 [dev](dev)。

## create

创建新环境。

```bash
vx env create [NAME] [OPTIONS]
```

选项：

- `-g`, `--global` - 创建全局环境（需要 NAME）
- `--from <ENV>` - 从现有环境克隆
- `--set-default` - 创建后设为默认

**项目环境（默认）：**

当存在 `.vx.toml` 时，在 `.vx/env/` 创建项目本地环境：

```bash
# 在有 .vx.toml 的项目中
vx env create              # 创建 .vx/env/
vx env create --from dev   # 从全局 'dev' 环境克隆
```

**全局环境：**

```bash
vx env create --global my-env
vx env create -g dev --from default
vx env create -g production --set-default
```

## sync

从 `.vx.toml` 同步项目环境。为配置中定义的所有工具在 `.vx/env/` 创建软链接。

```bash
vx env sync
```

此命令：

1. 从 `.vx.toml` 读取工具版本
2. 在 `.vx/env/` 创建/更新指向 `~/.vx/store/` 的软链接
3. 报告需要安装的缺失工具

示例：

```bash
# 运行 'vx setup' 安装工具后
vx env sync

# 输出：
# Synced 3 tool(s) to project environment
```

## use

激活环境。

```bash
vx env use [NAME] [OPTIONS]
```

选项：

- `--global` - 使用全局环境

示例：

```bash
vx env use                  # 使用项目环境
vx env use --global dev     # 使用全局 'dev' 环境
vx env use my-env           # 按名称使用全局环境
```

## list

列出所有环境。

```bash
vx env list [OPTIONS]
```

选项：

- `--detailed` - 显示详细信息
- `--global` - 仅显示全局环境

示例：

```bash
vx env list
vx env list --detailed
vx env list --global
```

输出：

```
Project Environment:

* project (active)

Global Environments:

* default (default)
  dev
  production
```

## delete

删除环境。

```bash
vx env delete [NAME] [OPTIONS]
```

选项：

- `-g`, `--global` - 删除全局环境
- `--force` - 强制删除，不需确认

示例：

```bash
vx env delete                    # 删除项目环境
vx env delete --global dev       # 删除全局 'dev' 环境
vx env delete -g old-env --force
```

## show

显示环境详情。

```bash
vx env show [NAME]
```

示例：

```bash
vx env show           # 显示项目或默认环境
vx env show dev       # 显示全局 'dev' 环境
```

输出：

```
Environment: project
Type: project
Path: /path/to/project/.vx/env

Tools:
  node -> /home/user/.vx/store/node/20.0.0
  uv -> /home/user/.vx/store/uv/0.5.14
```

## add

向环境添加工具。

```bash
vx env add <TOOL>@<VERSION> [OPTIONS]
```

选项：

- `-g`, `--global` - 添加到全局环境（需要 `--env`）
- `--env <NAME>` - 目标全局环境名称

示例：

```bash
# 添加到项目环境（默认）
vx env add node@20.0.0
vx env add uv@0.5.14

# 添加到全局环境
vx env add node@20 --global --env dev
vx env add go@1.21 --env production
```

## remove

从环境删除工具。

```bash
vx env remove <TOOL> [OPTIONS]
```

选项：

- `-g`, `--global` - 从全局环境删除
- `--env <NAME>` - 目标全局环境名称

示例：

```bash
vx env remove node              # 从项目环境删除
vx env remove node --global --env dev
```

## 目录结构

```
~/.vx/
├── store/                    # 全局工具存储（内容寻址）
│   ├── node/20.0.0/
│   ├── uv/0.5.14/
│   └── go/1.21.0/
├── envs/                     # 全局环境
│   ├── default/
│   │   └── node -> ../../store/node/20.0.0
│   └── dev/
│       ├── node -> ../../store/node/20.0.0
│       └── go -> ../../store/go/1.21.0
└── ...

/path/to/project/
├── .vx.toml                  # 项目配置
├── .vx/
│   └── env/                  # 项目环境（软链接）
│       ├── node -> ~/.vx/store/node/20.0.0
│       └── uv -> ~/.vx/store/uv/0.5.14
└── src/
```

## 参见

- [dev](dev) - 进入开发环境（包含 `--export` 用于 shell 激活）
- [setup](setup) - 安装项目工具
