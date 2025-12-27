# Extension 开发指南

本指南说明如何为 vx 创建扩展。扩展允许你使用 Python、Shell 或 Node.js 等脚本语言添加自定义命令和功能。

## 概述

vx 扩展利用 vx 已经管理的运行时。你的扩展脚本可以使用 Python、Node.js 或 vx 支持的任何其他运行时，无需用户手动安装任何东西。

```
┌─────────────────────────────────────────────────────────────┐
│                    vx 扩展系统                               │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   scaffold   │  │docker-compose│  │  my-tool     │  ...  │
│  │  (Python)    │  │   (Shell)    │  │  (Node.js)   │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│         │                 │                 │                │
│         ▼                 ▼                 ▼                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              vx 管理的运行时                         │    │
│  │   python 3.12  │  bash  │  node 20  │  ...          │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## 扩展类型

### 1. 命令扩展

添加可通过 `vx x <extension> [subcommand]` 访问的新 CLI 命令：

```bash
vx x docker-compose up
vx x scaffold create react-app my-app
vx x my-tool run --verbose
```

### 2. Hook 扩展（未来）

在特定事件上执行脚本：

```toml
[extension]
type = "hook"

[hooks]
pre-install = "check.py"
post-install = "setup.py"
```

## 快速开始

### 1. 创建扩展目录

```bash
mkdir -p ~/.vx/extensions/my-extension
cd ~/.vx/extensions/my-extension
```

### 2. 创建配置文件

创建 `vx-extension.toml`：

```toml
[extension]
name = "my-extension"
version = "1.0.0"
description = "我的自定义扩展"
type = "command"

[runtime]
requires = "python >= 3.8"

[entrypoint]
main = "main.py"

[commands.hello]
description = "打招呼"
script = "main.py"
args = ["hello"]

[commands.greet]
description = "问候某人"
script = "main.py"
args = ["greet"]
```

### 3. 创建脚本

创建 `main.py`：

```python
#!/usr/bin/env python3
import sys
import os

def main():
    args = sys.argv[1:]

    if not args:
        print("用法: vx x my-extension <hello|greet> [args...]")
        sys.exit(1)

    cmd = args[0]

    if cmd == "hello":
        print("来自我的扩展的问候！")
    elif cmd == "greet":
        name = args[1] if len(args) > 1 else "世界"
        print(f"你好，{name}！")
    else:
        print(f"未知命令: {cmd}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

### 4. 测试扩展

```bash
# 列出扩展
vx ext list

# 运行命令
vx x my-extension hello
vx x my-extension greet Alice
```

## 配置参考

### vx-extension.toml

```toml
[extension]
name = "extension-name"           # 必需：唯一标识符
version = "1.0.0"                 # 必需：语义化版本
description = "描述"              # 必需：简短描述
type = "command"                  # 必需：command | hook | provider

[runtime]
requires = "python >= 3.8"        # 必需：运行时依赖
# 支持的格式：
# - "python >= 3.8"
# - "node >= 18"
# - "bash"

[entrypoint]
main = "main.py"                  # 默认运行的脚本
args = ["--config", "config.yaml"] # 默认参数

[commands.subcommand]
description = "子命令描述"
script = "subcommand.py"          # 此子命令的脚本
args = ["--flag"]                 # 额外参数
```

## 扩展位置

扩展按以下位置加载（按优先级顺序）：

| 优先级 | 位置 | 描述 |
|--------|------|------|
| 1（最高） | `~/.vx/extensions-dev/` | 开发扩展（符号链接） |
| 2 | `.vx/extensions/` | 项目级扩展 |
| 3 | `~/.vx/extensions/` | 用户级扩展 |

### 开发模式

对于活跃开发，使用 `vx ext dev` 链接你的扩展：

```bash
# 从任意目录链接扩展
vx ext dev /path/to/my-extension

# 完成后取消链接
vx ext dev --unlink my-extension
```

这会在 `~/.vx/extensions-dev/` 中创建符号链接，使其具有最高优先级。

## 环境变量

vx 在运行扩展脚本时注入这些环境变量：

| 变量 | 描述 |
|------|------|
| `VX_VERSION` | 当前 vx 版本 |
| `VX_EXTENSION_DIR` | 扩展目录路径 |
| `VX_EXTENSION_NAME` | 扩展名称 |
| `VX_PROJECT_DIR` | 当前项目目录（如果在项目中） |
| `VX_RUNTIMES_DIR` | vx 运行时目录路径 |
| `VX_HOME` | vx 主目录（`~/.vx`） |

### 使用环境变量

```python
#!/usr/bin/env python3
import os
from pathlib import Path

# 获取扩展目录以加载资源
ext_dir = Path(os.environ.get("VX_EXTENSION_DIR", "."))
templates_dir = ext_dir / "templates"

# 获取项目目录
project_dir = os.environ.get("VX_PROJECT_DIR")
if project_dir:
    print(f"在项目中运行: {project_dir}")
```

## 示例：项目脚手架扩展

一个完整的脚手架扩展示例：

### 目录结构

```
~/.vx/extensions/scaffold/
├── vx-extension.toml
├── main.py
└── templates/
    ├── react-app/
    │   ├── package.json
    │   └── src/
    │       └── index.js
    └── python-cli/
        ├── pyproject.toml
        └── src/
            └── main.py
```

### vx-extension.toml

```toml
[extension]
name = "scaffold"
version = "1.0.0"
description = "项目脚手架工具"
type = "command"

[runtime]
requires = "python >= 3.8"

[entrypoint]
main = "main.py"

[commands.create]
description = "从模板创建新项目"
script = "main.py"
args = ["create"]

[commands.list]
description = "列出可用模板"
script = "main.py"
args = ["list"]
```

### main.py

```python
#!/usr/bin/env python3
"""vx 项目脚手架扩展。"""

import sys
import os
import shutil
from pathlib import Path

def get_templates_dir() -> Path:
    """获取模板目录。"""
    ext_dir = Path(os.environ.get("VX_EXTENSION_DIR", "."))
    return ext_dir / "templates"

def list_templates():
    """列出所有可用模板。"""
    templates_dir = get_templates_dir()

    if not templates_dir.exists():
        print("未找到模板目录")
        return

    print("可用模板:")
    for template in templates_dir.iterdir():
        if template.is_dir():
            print(f"  - {template.name}")

def create_project(template_name: str, project_name: str):
    """从模板创建新项目。"""
    templates_dir = get_templates_dir()
    src = templates_dir / template_name

    if not src.exists():
        print(f"错误: 未找到模板 '{template_name}'")
        print("可用模板:")
        list_templates()
        sys.exit(1)

    dst = Path.cwd() / project_name

    if dst.exists():
        print(f"错误: 目录 '{project_name}' 已存在")
        sys.exit(1)

    shutil.copytree(src, dst)
    print(f"✓ 从模板 '{template_name}' 创建了 '{project_name}'")
    print(f"  cd {project_name}")

def main():
    args = sys.argv[1:]

    if not args:
        print("用法: vx x scaffold <create|list> [args...]")
        print("\n命令:")
        print("  list              列出可用模板")
        print("  create <t> <n>    从模板 <t> 创建项目 <n>")
        sys.exit(1)

    cmd = args[0]

    if cmd == "list":
        list_templates()
    elif cmd == "create":
        if len(args) < 3:
            print("用法: vx x scaffold create <template> <project-name>")
            sys.exit(1)
        create_project(args[1], args[2])
    else:
        print(f"未知命令: {cmd}")
        sys.exit(1)

if __name__ == "__main__":
    main()
```

### 使用方法

```bash
# 列出模板
vx x scaffold list

# 创建新项目
vx x scaffold create react-app my-app
vx x scaffold create python-cli my-cli
```

## 最佳实践

### 1. 优雅地处理错误

```python
import sys

def main():
    try:
        # 你的代码
        pass
    except FileNotFoundError as e:
        print(f"错误: 文件未找到 - {e}")
        sys.exit(1)
    except Exception as e:
        print(f"错误: {e}")
        sys.exit(1)
```

### 2. 提供帮助信息

```python
def show_help():
    print("""
用法: vx x my-extension <command> [options]

命令:
  create    创建新项目
  list      列出所有项目
  delete    删除项目

选项:
  -h, --help    显示帮助信息
  -v, --verbose 启用详细输出
""")
```

### 3. 使用结构化输出

```python
import json

def output_json(data):
    """输出 JSON 格式数据，便于机器解析。"""
    print(json.dumps(data, indent=2, ensure_ascii=False))
```

## CLI 命令

### 管理扩展

```bash
# 列出所有已安装扩展
vx ext list

# 显示扩展详情
vx ext info <extension-name>

# 链接本地扩展用于开发
vx ext dev /path/to/extension

# 取消链接开发扩展
vx ext dev --unlink <extension-name>

# 从远程安装（未来）
vx ext install github:user/vx-ext-name

# 卸载扩展
vx ext uninstall <extension-name>
```

### 运行扩展命令

```bash
# 运行扩展命令
vx x <extension> [subcommand] [args...]

# 示例
vx x scaffold list
vx x scaffold create react-app my-app
vx x docker-compose up
vx x docker-compose logs api
```

## 故障排除

### 扩展未找到

```bash
# 检查扩展是否已安装
vx ext list

# 验证扩展目录是否存在
ls ~/.vx/extensions/my-extension/

# 检查 vx-extension.toml 语法
cat ~/.vx/extensions/my-extension/vx-extension.toml
```

### 运行时不可用

```bash
# 检查所需运行时是否已安装
vx list python

# 安装运行时
vx install python 3.12
```

### 权限被拒绝

在 Unix 系统上，确保脚本可执行：

```bash
chmod +x ~/.vx/extensions/my-extension/main.py
```

## 参见

- [CLI 参考: ext 命令](/zh/cli/ext)
- [Provider 开发指南](./plugin-development)
