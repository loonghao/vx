# 命令参考

本页列出所有 vx 命令及其选项。

## 工具执行

### 直接执行

```bash
vx <tool> [args...]
vx <tool>@<version> [args...]
```

示例：

```bash
vx node --version
vx node@18 script.js
vx python -m pytest
vx go build
```

## 工具管理命令

### install

安装工具版本。

```bash
vx install <tool>[@version]
```

选项：

| 选项 | 描述 |
|------|------|
| `--force` | 强制重新安装 |
| `--global` | 设为全局默认版本 |

### list

列出可用工具和已安装版本。

```bash
vx list [tool]
```

选项：

| 选项 | 描述 |
|------|------|
| `--installed` | 仅显示已安装 |
| `--available` | 显示可用版本 |

### uninstall

卸载工具版本。

```bash
vx uninstall <tool>[@version]
```

### which

显示工具位置。

```bash
vx which <tool>
```

## 项目命令

### init

初始化项目配置。

```bash
vx init [options]
```

选项：

| 选项 | 描述 |
|------|------|
| `-i, --interactive` | 交互模式 |
| `--template <name>` | 使用模板 |
| `--tools <list>` | 指定工具 |

### setup

安装项目所有工具。

```bash
vx setup [options]
```

选项：

| 选项 | 描述 |
|------|------|
| `--dry-run` | 仅显示将执行的操作 |
| `--force` | 强制重新安装 |
| `--no-parallel` | 顺序安装 |

### run

运行 `.vx.toml` 中定义的脚本。

```bash
vx run <script> [-- args...]
```

### dev

进入开发环境。

```bash
vx dev [options]
```

选项：

| 选项 | 描述 |
|------|------|
| `-c, --command` | 运行命令后退出 |
| `--shell <shell>` | 指定 shell |

## 环境命令

### env list

列出所有环境。

```bash
vx env list [--detailed]
```

### env create

创建新环境。

```bash
vx env create <name> [options]
```

### env use

切换到环境。

```bash
vx env use <name> [--global]
```

### env delete

删除环境。

```bash
vx env delete <name> [--force]
```

## 配置命令

### config show

显示当前配置。

```bash
vx config show
```

### config set

设置配置值。

```bash
vx config set <key> <value>
```

### config get

获取配置值。

```bash
vx config get <key>
```

## 维护命令

### clean

清理缓存和临时文件。

```bash
vx clean [options]
```

选项：

| 选项 | 描述 |
|------|------|
| `--cache` | 仅清理缓存 |
| `--all` | 清理所有 |

### self-update

更新 vx 本身。

```bash
vx self-update [--check]
```

## Shell 命令

### shell init

生成 shell 集成脚本。

```bash
vx shell init <shell>
```

支持的 shell：`bash`、`zsh`、`fish`、`powershell`

### shell completions

生成命令补全脚本。

```bash
vx shell completions <shell>
```
