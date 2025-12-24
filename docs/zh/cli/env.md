# env 命令

管理 vx 环境。

## 语法

```bash
vx env <subcommand> [options]
```

## 子命令

### list

列出所有环境。

```bash
vx env list [--detailed]
```

### create

创建新环境。

```bash
vx env create <name> [options]
```

选项：

- `--from <env>` - 从现有环境克隆
- `--set-default` - 设为默认环境

### use

切换到环境。

```bash
vx env use <name> [--global]
```

### show

显示环境详情。

```bash
vx env show [name]
```

### add

向环境添加工具。

```bash
vx env add <tool>[@version] [--env <name>]
```

### remove

从环境删除工具。

```bash
vx env remove <tool> [--env <name>]
```

### delete

删除环境。

```bash
vx env delete <name> [--force]
```

### export

导出环境配置。

```bash
vx env export <name> -o <file> [--format <format>]
```

### import

导入环境配置。

```bash
vx env import <file> [--name <name>] [--force]
```

### activate

生成环境激活脚本。

```bash
vx env activate <name> [--shell <shell>]
```

## 示例

```bash
# 创建环境
vx env create my-project

# 添加工具
vx env add node@20 --env my-project
vx env add go@1.21 --env my-project

# 切换环境
vx env use my-project

# 导出环境
vx env export my-project -o env.toml

# 导入环境
vx env import env.toml
```

## 参见

- [环境管理](/zh/guide/environment-management) - 环境管理指南
