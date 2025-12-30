# vx 扩展示例

本目录包含演示 vx 扩展系统的示例扩展。

## 快速开始

```bash
# 直接从 GitHub 安装 hello-world 示例
vx ext install https://github.com/loonghao/vx/tree/main/examples/extensions/hello-world

# 运行扩展
vx x hello-world
vx x hello-world greet Alice
```

### 其他安装格式

```bash
# GitHub 简写带路径
vx ext install github:loonghao/vx/examples/extensions/hello-world

# 安装独立的扩展仓库
vx ext install github:user/vx-ext-name
vx ext install github:user/vx-ext-name@v1.0.0
```

## 示例

### hello-world (Python)

一个简单的基于 Python 的扩展，演示基本的扩展功能。

```bash
# 从 GitHub 安装
vx ext install https://github.com/loonghao/vx/tree/main/examples/extensions/hello-world

# 或本地链接用于开发
vx ext dev ./examples/extensions/hello-world

# 运行扩展
vx x hello-world
vx x hello-world greet Alice
vx x hello-world info
```

### project-info (Node.js)

一个基于 Node.js 的扩展，用于显示项目信息。

```bash
# 从 GitHub 安装
vx ext install https://github.com/loonghao/vx/tree/main/examples/extensions/project-info

# 或本地链接用于开发
vx ext dev ./examples/extensions/project-info

# 运行扩展
vx x project-info
vx x project-info deps
vx x project-info size
```

## 命令参考

| 命令 | 描述 |
|------|------|
| `vx ext list` | 列出所有已安装的扩展 |
| `vx ext install <source>` | 从远程源安装扩展 |
| `vx ext dev <path>` | 链接本地扩展用于开发 |
| `vx ext dev --unlink <name>` | 取消链接开发扩展 |
| `vx ext uninstall <name>` | 卸载扩展 |
| `vx ext info <name>` | 显示扩展详情 |
| `vx ext update <name>` | 更新扩展 |
| `vx ext check --all` | 检查更新 |
| `vx x <extension> [args]` | 运行扩展 |

### 支持的安装源格式

| 格式 | 示例 |
|------|------|
| GitHub tree URL | `https://github.com/user/repo/tree/branch/path/to/ext` |
| GitHub 简写 | `github:user/repo` |
| GitHub 带路径 | `github:user/repo/path/to/ext` |
| GitHub 带版本 | `github:user/repo@v1.0.0` |
| GitHub HTTPS URL | `https://github.com/user/repo` |
| GitHub SSH URL | `git@github.com:user/repo.git` |

## 创建自己的扩展

1. 创建一个包含 `vx-extension.toml` 文件的目录：

```toml
[extension]
name = "my-extension"
version = "1.0.0"
description = "我的自定义扩展"
type = "command"

[runtime]
requires = "python >= 3.8"  # 或 "node >= 16", "bash" 等

[entrypoint]
main = "main.py"

[commands.hello]
description = "打招呼"
script = "hello.py"
```

2. 添加脚本：

```
my-extension/
├── vx-extension.toml
├── main.py           # 主入口
└── hello.py          # 子命令脚本
```

3. 链接用于开发：

```bash
vx ext dev /path/to/my-extension
```

4. 测试：

```bash
vx x my-extension
vx x my-extension hello
```

5. 发布（可选）：

```bash
# 创建一个 GitHub 仓库，在根目录放置 vx-extension.toml
# 用户可以通过以下方式安装：
vx ext install github:your-username/my-extension
```

## 环境变量

扩展会接收以下环境变量：

| 变量 | 描述 |
|------|------|
| `VX_VERSION` | 当前 vx 版本 |
| `VX_EXTENSION_DIR` | 扩展所在目录 |
| `VX_EXTENSION_NAME` | 扩展名称 |
| `VX_PROJECT_DIR` | 当前工作目录 |
| `VX_RUNTIMES_DIR` | vx 运行时目录 |
| `VX_HOME` | vx 主目录 |

## 扩展类型

- **command**: 通过 `vx x <extension>` 提供 CLI 命令
- **hook**: 在生命周期事件时执行（未来功能）
- **provider**: 提供新的运行时支持（未来功能）

## 扩展位置

扩展按以下优先级顺序被发现：

1. `~/.vx/extensions-dev/` - 开发扩展（最高优先级）
2. `.vx/extensions/` - 项目级扩展
3. `~/.vx/extensions/` - 用户安装的扩展
4. 内置扩展（最低优先级）
