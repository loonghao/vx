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

从远程源安装扩展（未来功能）。

```bash
vx ext install <源>
vx ext install github:user/repo
vx ext install https://github.com/user/repo
```

### uninstall

卸载扩展。

```bash
vx ext uninstall <名称>
vx ext uninstall my-extension
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
author = "Your Name"
type = "command"  # command, hook, 或 provider

[runtime]
requires = "python >= 3.10"  # 或 "node >= 18", "bash" 等

[entrypoint]
main = "main.py"  # 主入口点

[commands.hello]
description = "打招呼"
script = "commands/hello.py"

[commands.build]
description = "构建项目"
script = "commands/build.sh"
```

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
└── commands/
    ├── hello.py
    └── build.sh
```

## 另请参阅

- [Provider 开发](/zh/advanced/plugin-development) - 创建 Provider
- [配置](/zh/config/vx-toml) - 项目配置
