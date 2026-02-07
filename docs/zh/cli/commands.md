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
| `--status` | 显示安装状态和版本详情 |
| `--all, -a` | 显示所有工具，包括当前平台不支持的 |
| `--installed` | 仅显示已安装 |
| `--available` | 显示可用版本 |

### uninstall

从全局存储卸载工具版本。

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

### add

添加工具到项目配置 (vx.toml)。

```bash
vx add <tool>
vx add node
vx add node --version 20
```

### remove

从项目配置 (vx.toml) 移除工具。

```bash
vx remove <tool>
vx remove node
vx rm node           # rm 是 remove 的别名
```

### run

运行 `vx.toml` 中定义的脚本。

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

## 诊断命令

### info

显示系统信息、能力概览和构建诊断。

```bash
vx info
vx info --json
vx info --warnings
```

选项：

| 选项 | 描述 |
|------|------|
| `--json` | 以 JSON 格式输出（推荐 AI 和脚本使用） |
| `--warnings` | 显示构建警告和诊断信息 |

详细文档请参阅 [info 命令](./info.md)。

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

更新 vx 本身到最新版本，具有增强功能：

- **多渠道下载**：自动在 GitHub Releases、jsDelivr CDN 和 Fastly CDN 之间切换
- **进度条显示**：实时显示下载进度、速度和预计剩余时间
- **校验和验证**：对下载的二进制文件进行 SHA256 验证（如果可用）
- **指定版本**：可以安装特定版本而不是最新版本
- **安全替换**：使用 `self_replace` 在 Windows 上可靠地替换二进制文件
- **向后兼容**：支持旧版（v0.5.x）和新版（v0.6.0+）的 artifact 命名格式
- **智能版本比较**：正确处理 `vx-v0.6.27`、`v0.6.27` 和 `0.6.27` 等版本格式

```bash
# 更新到最新版本
vx self-update

# 仅检查更新，不安装
vx self-update --check

# 强制更新，即使已是最新版本
vx self-update --force

# 安装指定版本
vx self-update 0.5.28

# 包含预发布版本
vx self-update --prerelease

# 使用 GitHub token 避免 API 速率限制
vx self-update --token <GITHUB_TOKEN>
```

选项：

| 选项 | 描述 |
|------|------|
| `--check` | 仅检查更新，不安装 |
| `--force`, `-f` | 强制更新，即使已是最新版本 |
| `--prerelease` | 包含预发布版本 |
| `--token <TOKEN>` | 用于认证 API 请求的 GitHub token |
| `<VERSION>` | 要安装的特定版本（例如 `0.5.28`） |

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

## Extension 管理

### ext list

列出已安装的扩展。

```bash
vx ext list
vx ext ls
vx ext list --verbose
```

### ext info

显示扩展信息。

```bash
vx ext info <名称>
vx ext info docker-compose
```

### ext dev

链接本地扩展用于开发。

```bash
vx ext dev <路径>
vx ext dev /path/to/my-extension
vx ext dev --unlink my-extension
```

### ext install

从远程源安装扩展。

```bash
vx ext install <源>
vx ext install github:user/repo
```

### ext uninstall

卸载扩展。

```bash
vx ext uninstall <名称>
vx ext uninstall my-extension
```

### x

执行扩展命令。

```bash
vx x <扩展> [命令] [参数...]
vx x docker-compose up -d
vx x scaffold create react-app my-app
```
