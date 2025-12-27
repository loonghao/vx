# vx 扩展示例

本目录包含演示 vx 扩展系统的示例扩展。

## 示例

### hello-world (Python)

一个简单的基于 Python 的扩展，演示基本的扩展功能。

```bash
# 链接扩展用于开发
vx ext dev examples/extensions/hello-world

# 运行扩展
vx x hello-world
vx x hello-world greet Alice
vx x hello-world info
```

### project-info (Node.js)

一个基于 Node.js 的扩展，用于显示项目信息。

```bash
# 链接扩展用于开发
vx ext dev examples/extensions/project-info

# 运行扩展
vx x project-info
vx x project-info deps
vx x project-info size
```

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

1. 添加脚本：

```
my-extension/
├── vx-extension.toml
├── main.py           # 主入口
└── hello.py          # 子命令脚本
```

1. 链接用于开发：

```bash
vx ext dev /path/to/my-extension
```

1. 测试：

```bash
vx x my-extension
vx x my-extension hello
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
