# ext - 扩展管理

管理 vx 扩展。

## 语法

```bash
vx ext <子命令>
vx extension <子命令>  # 别名
```

## 子命令

### list

列出已安装的扩展。

```bash
vx ext list
vx ext ls              # 别名
vx ext list --verbose  # 显示详细信息
```

**选项：**

| 选项 | 描述 |
|------|------|
| `-v, --verbose` | 显示详细的扩展信息 |

**输出示例：**

```
Extensions:
  docker-compose (v1.0.0) - Docker Compose 包装器 [dev]
  scaffold (v2.1.0) - 项目脚手架工具 [user]
  lint-all (v1.0.0) - 运行所有 linter [project]
```

### info

显示扩展的详细信息。

```bash
vx ext info <名称>
vx ext info docker-compose
```

**输出示例：**

```
Extension: docker-compose
  Version: 1.0.0
  Description: Docker Compose wrapper with vx integration
  Author: Your Name
  Source: dev (~/.vx/extensions-dev/docker-compose)
  Runtime: python >= 3.10
  Commands:
    - up: Start services
    - down: Stop services
    - logs: View logs
```

### dev

链接本地扩展用于开发。

```bash
vx ext dev <路径>
vx ext dev /path/to/my-extension
vx ext dev . # 链接当前目录
```

**选项：**

| 选项 | 描述 |
|------|------|
| `--unlink` | 取消链接而不是链接 |

**示例：**

```bash
# 链接扩展用于开发
vx ext dev ~/projects/my-extension

# 取消链接扩展
vx ext dev --unlink my-extension
```

### install

从远程源安装扩展。

```bash
vx ext install <源>
```

**支持的源格式：**

| 格式 | 示例 |
|------|------|
| GitHub 简写 | `github:user/repo` |
| GitHub 简写带版本 | `github:user/repo@v1.0.0` |
| GitHub HTTPS URL | `https://github.com/user/repo` |
| GitHub SSH URL | `git@github.com:user/repo.git` |

**示例：**

```bash
# 从 GitHub 安装
vx ext install github:user/vx-ext-docker

# 安装特定版本
vx ext install github:user/vx-ext-docker@v1.0.0

# 从 HTTPS URL 安装
vx ext install https://github.com/user/vx-ext-docker
```

### uninstall

卸载扩展。

```bash
vx ext uninstall <名称>
vx ext uninstall my-extension
```

### update

更新已安装的扩展。

```bash
vx ext update <名称>
vx ext update --all
```

**选项：**

| 选项 | 描述 |
|------|------|
| `--all` | 更新所有已安装的扩展 |

**示例：**

```bash
# 更新特定扩展
vx ext update docker-compose

# 更新所有扩展
vx ext update --all
```

### check

检查扩展更新。

```bash
vx ext check <名称>
vx ext check --all
```

**选项：**

| 选项 | 描述 |
|------|------|
| `--all` | 检查所有已安装的扩展 |

**示例：**

```bash
# 检查特定扩展
vx ext check docker-compose

# 检查所有扩展
vx ext check --all
```

**输出示例：**

```
Updates Available:
  docker-compose: 1.0.0 -> 1.1.0
  scaffold: 2.0.0 -> 2.1.0

Run 'vx ext update --all' to update all extensions
```

## 扩展执行

使用 `vx x` 执行扩展命令：

```bash
vx x <扩展> [命令] [参数...]
```

**示例：**

```bash
# 运行扩展的主入口
vx x docker-compose

# 运行特定命令
vx x docker-compose up -d
vx x scaffold create react-app my-app
vx x lint-all --fix
```

## 扩展配置

扩展通过 `vx-extension.toml` 配置：

```toml
[extension]
name = "my-extension"
version = "1.0.0"
description = "我的自定义扩展"
authors = ["Your Name"]
type = "command"  # command, hook, 或 provider

[runtime]
requires = "python >= 3.10"  # 或 "node >= 18", "bash" 等
dependencies = ["requests", "pyyaml"]  # 运行时依赖

[entrypoint]
main = "main.py"  # 主入口点
args = ["--config", "config.yaml"]  # 默认参数

[commands.hello]
description = "打招呼"
script = "commands/hello.py"

[commands.build]
description = "构建项目"
script = "commands/build.sh"
args = ["--production"]

# Hook 扩展类型
[hooks]
pre-install = "hooks/pre-install.py"
post-install = "hooks/post-install.py"
pre-run = "hooks/pre-run.sh"
post-run = "hooks/post-run.sh"
```

## 扩展类型

### Command 扩展

通过 `vx x <扩展>` 提供新的 CLI 命令：

```toml
[extension]
name = "docker-compose"
type = "command"

[commands.up]
description = "启动服务"
script = "up.py"
```

### Hook 扩展

在特定生命周期事件时执行：

```toml
[extension]
name = "pre-commit-check"
type = "hook"

[hooks]
pre-install = "check.py"
post-install = "setup.py"
pre-run = "validate.sh"
```

**可用的 Hook 事件：**

| 事件 | 描述 |
|------|------|
| `pre-install` | 安装运行时之前 |
| `post-install` | 安装运行时之后 |
| `pre-uninstall` | 卸载运行时之前 |
| `post-uninstall` | 卸载运行时之后 |
| `pre-run` | 运行命令之前 |
| `post-run` | 运行命令之后 |
| `enter-project` | 进入项目目录时 |
| `leave-project` | 离开项目目录时 |

## 扩展位置

扩展按优先级从多个位置发现：

1. **开发扩展** (`~/.vx/extensions-dev/`) - 最高优先级
2. **项目扩展** (`.vx/extensions/`) - 项目特定
3. **用户扩展** (`~/.vx/extensions/`) - 用户安装
4. **内置扩展** - 随 vx 一起提供

## 环境变量

扩展接收以下环境变量：

| 变量 | 描述 |
|------|------|
| `VX_VERSION` | 当前 vx 版本 |
| `VX_EXTENSION_DIR` | 扩展目录 |
| `VX_EXTENSION_NAME` | 扩展名称 |
| `VX_PROJECT_DIR` | 当前工作目录 |
| `VX_RUNTIMES_DIR` | vx 运行时目录 |
| `VX_HOME` | vx 主目录 |

**Hook 特定变量：**

| 变量 | 描述 |
|------|------|
| `VX_HOOK_EVENT` | 触发的 hook 事件 |
| `VX_HOOK_RUNTIME` | 运行时名称（用于安装/卸载 hook） |
| `VX_HOOK_VERSION` | 运行时版本（用于安装/卸载 hook） |
| `VX_HOOK_COMMAND` | 正在运行的命令（用于 pre/post-run hook） |
| `VX_HOOK_ARGS` | 命令参数 |
| `VX_HOOK_PROJECT_DIR` | 项目目录 |

## 创建扩展

1. 创建包含 `vx-extension.toml` 的目录
2. 添加脚本
3. 链接用于开发：`vx ext dev /path/to/extension`
4. 测试：`vx x my-extension`

**示例结构：**

```
my-extension/
├── vx-extension.toml
├── main.py           # 主入口点
├── commands/
│   ├── hello.py
│   └── build.sh
└── hooks/
    ├── pre-install.py
    └── post-install.py
```

## 发布扩展

发布你的扩展：

1. 创建 GitHub 仓库
2. 在根目录添加 `vx-extension.toml`
3. 使用语义化版本标记发布（如 `v1.0.0`）
4. 用户可以通过以下方式安装：`vx ext install github:user/repo`

## 另请参阅

- [扩展开发](/zh/advanced/extension-development) - 详细的扩展开发指南
- [Provider 开发](/zh/advanced/plugin-development) - 创建 Provider
- [配置](/zh/config/vx-toml) - 项目配置
