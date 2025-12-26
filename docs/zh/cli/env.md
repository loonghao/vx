# env 命令

管理 vx 环境。

## 语法

```bash
vx env <subcommand> [options]
```

## 子命令

| 子命令 | 说明 |
|--------|------|
| `create` | 创建新环境 |
| `use` | 激活环境 |
| `list` | 列出所有环境 |
| `delete` | 删除环境 |
| `show` | 显示环境详情 |
| `add` | 向环境添加工具 |
| `remove` | 从环境删除工具 |
| `export` | 导出环境变量用于 shell 激活 |
| `import` | 导入环境配置 |
| `activate` | 生成 shell 激活脚本（export 的别名） |

## create

创建新环境。

```bash
vx env create <name> [options]
```

选项：

- `--from <env>` - 从现有环境克隆
- `--set-default` - 设为默认环境

示例：

```bash
vx env create my-env
vx env create new-env --from existing-env
vx env create my-env --set-default
```

## use

切换到环境。

```bash
vx env use <name> [--global]
```

选项：

- `--global` - 设为全局默认

示例：

```bash
vx env use my-env
vx env use my-env --global
```

## list

列出所有环境。

```bash
vx env list [--detailed]
```

选项：

- `--detailed` - 显示详细信息

示例：

```bash
vx env list
vx env list --detailed
```

## delete

删除环境。

```bash
vx env delete <name> [--force]
```

选项：

- `--force` - 强制删除，不需确认

示例：

```bash
vx env delete my-env
vx env delete my-env --force
```

## show

显示环境详情。

```bash
vx env show [name]
```

示例：

```bash
vx env show           # 显示当前环境
vx env show my-env    # 显示指定环境
```

## add

向环境添加工具。

```bash
vx env add <tool>@<version> [--env <name>]
```

选项：

- `--env <name>` - 目标环境（默认为当前环境）

示例：

```bash
vx env add node@20
vx env add go@1.21 --env my-env
```

## remove

从环境删除工具。

```bash
vx env remove <tool> [--env <name>]
```

选项：

- `--env <name>` - 目标环境（默认为当前环境）

示例：

```bash
vx env remove node
vx env remove node --env my-env
```

## export

导出环境变量用于 shell 激活。此命令读取当前目录的 `.vx.toml` 配置，生成设置 PATH 的 shell 脚本，使所有配置的工具可用。

```bash
vx env export [OPTIONS]
```

选项：

- `-f`, `--format <FORMAT>` - 输出格式（未指定时自动检测）：
  - `shell` - Bash/Zsh 兼容格式（Unix 默认）
  - `powershell` - PowerShell 兼容格式（Windows 默认）
  - `batch` - Windows CMD 批处理格式
  - `github` - GitHub Actions 格式（追加到 `$GITHUB_PATH`）

### Shell 激活

export 命令的设计类似于 Python 虚拟环境的激活方式。使用 `eval` 在当前 shell 中激活环境：

**Bash/Zsh:**

```bash
eval "$(vx env export)"
```

**Fish:**

```fish
vx env export | source
```

**PowerShell:**

```powershell
Invoke-Expression (vx env export --format powershell)
```

**Windows CMD:**

```batch
vx env export --format batch > activate.bat && activate.bat
```

### GitHub Actions 集成

在 CI/CD 流水线中，使用 `github` 格式自动将工具路径添加到 `$GITHUB_PATH`：

```yaml
- name: 设置 vx 环境
  run: |
    if [ -f ".vx.toml" ]; then
      vx env export --format github >> $GITHUB_PATH
    fi
```

### 工作原理

1. 从当前目录读取 `.vx.toml`
2. 将所有配置的工具解析为 `~/.vx/store/` 中的安装路径
3. 生成 shell 命令将这些路径添加到 `PATH` 前面

### 输出示例

对于配置了 `uv` 和 `node` 的项目：

**Shell 格式:**

```bash
# vx environment activation
# Generated from: /path/to/project/.vx.toml
export PATH="/home/user/.vx/store/uv/0.5.14:/home/user/.vx/store/node/22.12.0/bin:$PATH"
```

**PowerShell 格式:**

```powershell
# vx environment activation
# Generated from: C:\path\to\project\.vx.toml
$env:PATH = "C:\Users\user\.vx\store\uv\0.5.14;C:\Users\user\.vx\store\node\22.12.0;$env:PATH"
```

**GitHub Actions 格式:**

```
/home/runner/.vx/store/uv/0.5.14
/home/runner/.vx/store/node/22.12.0/bin
```

### 使用场景

1. **Shell 会话**：为交互式开发激活工具
2. **CI/CD**：确保工具在后续工作流步骤中可用
3. **脚本**：在运行项目脚本前 source 激活脚本
4. **IDE 集成**：配置终端配置文件以自动激活

示例：

```bash
# 在当前 shell 激活
eval "$(vx env export)"

# 查看将使用的格式
vx env export --format shell

# 在 CI 中使用
vx env export --format github >> $GITHUB_PATH
```

## import

导入环境配置。

```bash
vx env import <file> [--name <name>] [--force]
```

选项：

- `-n`, `--name <name>` - 环境名称（默认使用文件中的名称）
- `-f`, `--force` - 如果存在则强制覆盖

示例：

```bash
vx env import my-env.toml
vx env import my-env.toml --name new-env
vx env import my-env.toml --force
```

## 完整示例

```bash
# 创建环境
vx env create my-project

# 添加工具
vx env add node@20 --env my-project
vx env add go@1.21 --env my-project

# 切换环境
vx env use my-project

# 导出环境用于 shell 激活
eval "$(vx env export)"

# 导出环境配置到文件
vx env export my-project -o env.toml

# 导入环境
vx env import env.toml
```

## 参见

- [环境管理](/zh/guide/environment-management) - 环境管理指南
- [Shell 集成](/zh/guide/shell-integration) - Shell 设置指南
- [GitHub Actions](/zh/guides/github-action) - CI/CD 集成指南
