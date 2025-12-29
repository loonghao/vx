# RFC 0002: vx 扩展系统设计

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2024-12-27
> **目标版本**: v0.3.0

## 摘要

设计一个扩展系统，允许用户通过脚本（Python、Shell、Node.js 等）扩展 vx 的功能，充分利用 vx 已管理的运行时环境执行扩展脚本，使 vx 成为完整的开发工具链管理器。

## 动机

### 当前状态分析

vx 目前专注于运行时管理（Node.js、Go、Python 等），但用户在开发过程中还需要很多辅助工具：

- **DevOps**: Docker Compose 管理、CI/CD 脚本
- **项目脚手架**: 项目初始化、模板生成
- **代码质量**: 自定义 lint 规则、格式化配置
- **团队协作**: 共享脚本、项目约定

### 行业趋势对比

| 工具 | 扩展机制 | 可借鉴 |
|------|----------|--------|
| npm scripts | package.json 配置 | 声明式配置 |
| Makefile | 任务定义 | 依赖管理 |
| mise (rtx) | 插件系统 | 运行时集成 |
| asdf | Shell 插件 | 简单易写 |
| cargo-make | 任务运行器 | Rust 生态集成 |

### 核心优势

vx 的独特优势：**已经管理了运行时环境**。扩展可以直接使用 vx 管理的 Python、Node.js、Go 等运行时，无需用户额外配置。

## 设计方案

### 完整配置预览

#### vx-extension.toml

```toml
[extension]
name = "docker-compose"
version = "1.0.0"
description = "Manage Docker Compose services"
type = "command"  # command | hook | provider

[runtime]
requires = "python >= 3.10"

[entrypoint]
main = "main.py"
args = ["--config", "compose.yaml"]

[commands.up]
description = "Start all services"
script = "up.py"

[commands.down]
description = "Stop all services"
script = "down.py"

[commands.logs]
description = "View service logs"
script = "logs.py"
args = ["-f"]
```

### 扩展类型

#### 1. Command 扩展

提供新的 CLI 命令：

```bash
vx x docker-compose up
vx x docker-compose down
vx x scaffold react-app
```

#### 2. Hook 扩展

在特定事件触发时执行：

```toml
[extension]
name = "pre-commit-check"
type = "hook"

[hooks]
pre-install = "check.py"
post-install = "setup.py"
```

#### 3. Provider 扩展（未来）

提供新的运行时支持：

```toml
[extension]
name = "deno-provider"
type = "provider"

[provider]
runtime = "deno"
ecosystem = "javascript"
```

### 扩展来源优先级

```
1. ~/.vx/extensions-dev/    # 本地开发扩展（最高优先级）
2. .vx/extensions/          # 项目级扩展
3. ~/.vx/extensions/        # 用户级扩展
4. 内置扩展                  # 随 vx 发布的扩展
```

### 目录结构

```
~/.vx/
├── extensions/                    # 用户级扩展
│   └── docker-compose/
│       ├── vx-extension.toml
│       ├── main.py
│       └── scripts/
│           ├── up.py
│           └── down.py
│
├── extensions-dev/                # 本地开发扩展
│   └── my-extension -> /path/to/dev/my-extension
│
└── extensions-cache/              # 远程扩展缓存
    └── github.com/
        └── user/
            └── vx-ext-docker/
```

### CLI 命令设计

#### 扩展管理

```bash
# 列出已安装扩展
vx ext list

# 安装远程扩展
vx ext install github:user/vx-ext-docker
vx ext install https://github.com/user/vx-ext-docker

# 卸载扩展
vx ext uninstall docker-compose

# 更新扩展
vx ext update docker-compose
vx ext update --all

# 链接本地开发扩展
vx ext dev /path/to/my-extension
vx ext dev --unlink my-extension
```

#### 执行扩展命令

```bash
# 执行扩展命令
vx x <extension> [subcommand] [args...]

# 示例
vx x docker-compose up
vx x docker-compose down --volumes
vx x scaffold create react-app my-app
```

### 执行流程

```
用户执行: vx x docker-compose up

1. 解析扩展名: docker-compose
2. 查找扩展 (按优先级):
   - ~/.vx/extensions-dev/docker-compose/
   - .vx/extensions/docker-compose/
   - ~/.vx/extensions/docker-compose/
3. 读取 vx-extension.toml
4. 检查运行时依赖: python >= 3.10
5. 确保运行时已安装 (调用 vx 核心能力)
6. 解析子命令: up -> up.py
7. 构建执行命令:
   ~/.vx/runtimes/python/3.12.0/bin/python \
     ~/.vx/extensions/docker-compose/up.py
8. 执行并转发 stdout/stderr
```

### 扩展 API

扩展脚本可以通过环境变量和标准输入/输出与 vx 交互：

#### 环境变量

```python
import os

# vx 提供的环境变量
VX_VERSION = os.environ.get("VX_VERSION")
VX_EXTENSION_DIR = os.environ.get("VX_EXTENSION_DIR")
VX_PROJECT_DIR = os.environ.get("VX_PROJECT_DIR")
VX_RUNTIMES_DIR = os.environ.get("VX_RUNTIMES_DIR")
```

#### 输出协议（可选）

```python
import json
import sys

# 结构化输出 (可选)
def vx_output(data):
    print(json.dumps({"vx_data": data}))

# 请求 vx 执行操作 (可选)
def vx_request(action, params):
    print(json.dumps({"vx_request": action, "params": params}))
    sys.stdout.flush()
```

### 示例扩展

#### 项目脚手架扩展

```
~/.vx/extensions/scaffold/
├── vx-extension.toml
├── main.py
└── templates/
    ├── react-app/
    └── vue-app/
```

**vx-extension.toml:**

```toml
[extension]
name = "scaffold"
version = "1.0.0"
description = "Project scaffolding tool"

[runtime]
requires = "python >= 3.8"

[entrypoint]
main = "main.py"

[commands.create]
description = "Create a new project from template"
script = "main.py"
args = ["create"]

[commands.list]
description = "List available templates"
script = "main.py"
args = ["list"]
```

**main.py:**

```python
#!/usr/bin/env python3
import sys
import os
import shutil
from pathlib import Path

def main():
    args = sys.argv[1:]
    if not args:
        print("Usage: vx x scaffold <create|list> [args...]")
        sys.exit(1)

    cmd = args[0]
    ext_dir = Path(os.environ.get("VX_EXTENSION_DIR", "."))
    templates_dir = ext_dir / "templates"

    if cmd == "list":
        for t in templates_dir.iterdir():
            if t.is_dir():
                print(f"  - {t.name}")
    elif cmd == "create":
        if len(args) < 3:
            print("Usage: vx x scaffold create <template> <name>")
            sys.exit(1)
        template, name = args[1], args[2]
        src = templates_dir / template
        dst = Path.cwd() / name
        shutil.copytree(src, dst)
        print(f"Created {name} from {template}")

if __name__ == "__main__":
    main()
```

## 向后兼容性

### 兼容策略

1. **新增功能**: 扩展系统是全新功能，不影响现有命令
2. **命令空间隔离**: 扩展命令通过 `vx x` 前缀隔离，不与核心命令冲突
3. **可选启用**: 用户可以完全不使用扩展功能

### 与现有功能的关系

| 现有功能 | 扩展系统 | 关系 |
|----------|----------|------|
| `vx run` | `vx x` | 互补，run 执行 vx.toml 任务，x 执行扩展命令 |
| `vx install` | `vx ext install` | 独立，install 安装运行时，ext install 安装扩展 |
| Provider | 扩展 | Provider 是编译时集成，扩展是运行时脚本 |

## 实现计划

### Phase 1: 核心框架 (v0.3.0) ✅

- [x] `vx-extension` crate 基础结构
- [x] `vx-extension.toml` 解析
- [x] 本地扩展加载 (`~/.vx/extensions/`)
- [x] `vx ext list` 命令
- [x] `vx x <ext> [cmd]` 执行框架
- [x] 环境变量注入

### Phase 2: 开发体验 (v0.3.1) ✅

- [x] `vx ext dev` 本地开发链接
- [x] 项目级扩展支持 (`.vx/extensions/`)
- [x] 扩展优先级处理
- [x] 错误处理和诊断信息

### Phase 3: 远程扩展 (v0.4.0) ✅

- [x] `vx ext install` 远程安装
- [x] GitHub 仓库支持
- [x] 扩展版本管理
- [x] 扩展更新检查 (`vx ext update`, `vx ext check`)

### Phase 4: 高级功能 (v0.5.0) ✅

- [x] Hook 扩展支持
- [x] 扩展依赖管理
- [ ] 扩展市场/索引
- [ ] Provider 扩展（动态运行时支持）

## 参考资料

- [mise plugins](https://mise.jdx.dev/plugins.html)
- [asdf plugins](https://asdf-vm.com/manage/plugins.html)
- [cargo-make](https://sagiegurari.github.io/cargo-make/)
- [npm scripts](https://docs.npmjs.com/cli/v10/using-npm/scripts)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2024-12-27 | Draft | 初始草案 |
| 2024-12-27 | v0.1 | Phase 1 & 2 实现完成 |
| 2024-12-28 | v0.2 | Phase 2 错误处理和诊断信息完成 |
| 2024-12-29 | v0.3 | Phase 3 & 4 实现完成（远程安装、Hook、依赖管理） |
