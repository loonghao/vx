# vx venv - 虚拟环境管理

管理项目特定的虚拟环境，隔离不同项目的工具版本。

## 语法

```bash
vx venv <subcommand> [options]
```

## 子命令

- `create` - 创建新的虚拟环境
- `list` - 列出所有虚拟环境
- `use` - 激活虚拟环境
- `current` - 显示当前激活的虚拟环境
- `run` - 在虚拟环境中运行命令
- `shell` - 在虚拟环境中启动shell
- `add` - 向虚拟环境添加工具
- `remove-tool` - 从虚拟环境移除工具
- `remove` - 删除虚拟环境

## vx venv create

创建新的虚拟环境。

### 语法
```bash
vx venv create <name> [options]
```

### 选项
- `--tools <tool1@version1,tool2@version2>` - 指定要安装的工具
- `--from-config` - 基于当前项目配置创建
- `--force` - 强制创建，覆盖已存在的环境

### 示例
```bash
# 创建空的虚拟环境
vx venv create myproject

# 创建并指定工具
vx venv create myproject --tools node@18.17.0,uv@latest

# 基于当前项目配置创建
vx venv create myproject --from-config
```

## vx venv list

列出所有虚拟环境。

### 语法
```bash
vx venv list [options]
```

### 选项
- `--verbose` - 显示详细信息
- `--active-only` - 仅显示当前激活的环境

### 示例
```bash
# 列出所有虚拟环境
vx venv list

# 显示详细信息
vx venv list --verbose
```

## vx venv use

激活虚拟环境。

### 语法
```bash
vx venv use <name>
```

### 示例
```bash
# 激活虚拟环境
vx venv use myproject

# 激活后，vx命令会自动使用环境中的工具版本
vx node --version  # 使用虚拟环境中的node版本
```

## vx venv current

显示当前激活的虚拟环境。

### 语法
```bash
vx venv current
```

### 示例
```bash
$ vx venv current
myproject
```

## vx venv run

在指定虚拟环境中运行命令。

### 语法
```bash
vx venv run <name> <command> [args...]
```

### 示例
```bash
# 在虚拟环境中运行命令
vx venv run myproject node --version
vx venv run myproject npm install
vx venv run myproject uv pip install requests
```

## vx venv shell

在虚拟环境中启动交互式shell。

### 语法
```bash
vx venv shell <name>
```

### 示例
```bash
# 在虚拟环境中启动shell
vx venv shell myproject
# 进入子shell，所有工具命令都使用环境中的版本
```

## vx venv add

向虚拟环境添加工具。

### 语法
```bash
vx venv add <name> <tool>[@version]
```

### 示例
```bash
# 向虚拟环境添加工具
vx venv add myproject uv@latest
vx venv add myproject python@3.11
```

## vx venv remove-tool

从虚拟环境移除工具。

### 语法
```bash
vx venv remove-tool <name> <tool>
```

### 示例
```bash
# 从虚拟环境移除工具
vx venv remove-tool myproject uv
```

## vx venv remove

删除虚拟环境。

### 语法
```bash
vx venv remove <name> [options]
```

### 选项
- `--force` - 强制删除，不询问确认

### 示例
```bash
# 删除虚拟环境
vx venv remove myproject

# 强制删除
vx venv remove myproject --force
```

## 虚拟环境工作流

### 创建和使用
```bash
# 1. 创建虚拟环境
vx venv create myproject --tools node@18.17.0,uv@latest

# 2. 激活虚拟环境
vx venv use myproject

# 3. 在环境中工作
node --version  # 使用虚拟环境中的版本
uv --version

# 4. 或者直接在环境中运行命令
vx venv run myproject npm install
```

### 环境管理
```bash
# 查看所有环境
vx venv list

# 查看当前环境
vx venv current

# 向环境添加工具
vx venv add myproject python@3.11

# 移除工具
vx venv remove-tool myproject python
```

## 环境配置

虚拟环境配置存储在：
```
~/.vx/venvs/<name>/config/venv.toml
```

配置格式：
```toml
name = "myproject"
created_at = "2024-01-15T10:30:00Z"
modified_at = "2024-01-15T10:30:00Z"
is_active = false

[tools]
node = "18.17.0"
uv = "latest"
python = "3.11"
```

## 环境隔离

VX 虚拟环境提供以下隔离：

1. **工具版本隔离** - 每个环境使用独立的工具版本
2. **配置隔离** - 环境特定的配置不会影响其他环境
3. **依赖隔离** - 工具依赖在环境间独立管理

## 故障排除

### 环境创建失败
```bash
# 检查权限
ls -la ~/.vx/venvs/

# 清理并重新创建
vx venv remove myproject --force
vx venv create myproject --tools node@18.17.0
```

### 激活失败
```bash
# 检查环境是否存在
vx venv list

# 手动激活
eval "$(vx venv activate myproject)"
```

## 相关命令

- [global](./global.md) - 全局工具管理
- [install](./install.md) - 安装工具
- [config](./config.md) - 配置管理
