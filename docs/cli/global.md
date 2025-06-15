# vx global - 全局工具管理

管理全局安装的工具，包括查看、清理和依赖关系管理。

## 语法

```bash
vx global <subcommand> [options]
```

## 子命令

- `list` - 列出全局安装的工具
- `info` - 显示特定工具的详细信息
- `remove` - 移除全局工具
- `dependents` - 显示工具的依赖关系
- `cleanup` - 清理未使用的全局工具

## vx global list

列出全局安装的工具。

### 语法
```bash
vx global list [options]
```

### 选项
- `--verbose` - 显示详细信息
- `--size` - 显示磁盘使用情况
- `--sort <field>` - 排序字段：`name`, `version`, `size`, `date`

### 示例
```bash
# 列出所有全局工具
vx global list

# 显示详细信息
vx global list --verbose

# 显示磁盘使用
vx global list --size

# 按大小排序
vx global list --sort size
```

### 输出示例
```bash
$ vx global list --verbose
Global Tools:
  node@18.17.0
    Path: ~/.vx/tools/node/18.17.0/
    Size: 45.2 MB
    Installed: 2024-01-15 10:30:00
    Used by: 2 virtual environments
    
  node@20.10.0
    Path: ~/.vx/tools/node/20.10.0/
    Size: 47.8 MB
    Installed: 2024-01-10 14:20:00
    Used by: 0 virtual environments
    
  python@3.11.5
    Path: ~/.vx/tools/python/3.11.5/
    Size: 67.3 MB
    Installed: 2024-01-12 09:15:00
    Used by: 1 virtual environment
    
  uv@0.1.0
    Path: ~/.vx/tools/uv/0.1.0/
    Size: 8.7 MB
    Installed: 2024-01-14 16:45:00
    Used by: 3 virtual environments

Total: 4 tools, 168.0 MB
```

## vx global info

显示特定工具的详细信息。

### 语法
```bash
vx global info <tool>
```

### 示例
```bash
# 显示node工具信息
vx global info node

# 显示python工具信息
vx global info python
```

### 输出示例
```bash
$ vx global info node
Tool: node
Description: Node.js JavaScript runtime

Installed Versions:
  18.17.0 (active in 2 environments)
    - Path: ~/.vx/tools/node/18.17.0/
    - Size: 45.2 MB
    - Installed: 2024-01-15 10:30:00
    - Checksum: sha256:abc123...
    
  20.10.0 (unused)
    - Path: ~/.vx/tools/node/20.10.0/
    - Size: 47.8 MB
    - Installed: 2024-01-10 14:20:00
    - Checksum: sha256:def456...

Used by Virtual Environments:
  - myproject (18.17.0)
  - webapp (18.17.0)

Configuration:
  Registry: https://nodejs.org/dist/
  Auto-install: enabled
  Update check: enabled
```

## vx global remove

移除全局工具。

### 语法
```bash
vx global remove <tool>[@version] [options]
```

### 选项
- `--force` - 强制移除，忽略虚拟环境引用
- `--all` - 移除工具的所有版本
- `--dry-run` - 预览移除操作

### 示例
```bash
# 移除特定版本（仅当无虚拟环境引用时）
vx global remove node@20.10.0

# 强制移除（忽略虚拟环境引用）
vx global remove node@20.10.0 --force

# 移除所有版本
vx global remove node --all

# 预览移除操作
vx global remove node@20.10.0 --dry-run
```

## vx global dependents

显示工具的依赖关系。

### 语法
```bash
vx global dependents <tool>[@version]
```

### 示例
```bash
# 显示node的依赖关系
vx global dependents node

# 显示特定版本的依赖关系
vx global dependents node@18.17.0
```

### 输出示例
```bash
$ vx global dependents node@18.17.0
Tool: node@18.17.0

Used by Virtual Environments:
  - myproject
    Created: 2024-01-15 10:30:00
    Last used: 2024-01-20 14:22:00
    
  - webapp
    Created: 2024-01-18 09:15:00
    Last used: 2024-01-20 16:45:00

Referenced by Projects:
  - /home/user/projects/myproject (.vx.toml)
  - /home/user/projects/webapp (.vx.toml)

Dependencies:
  This tool is required by 2 environments.
  Cannot be safely removed without --force.
```

## vx global cleanup

清理未使用的全局工具。

### 语法
```bash
vx global cleanup [options]
```

### 选项
- `--dry-run` - 预览清理操作，不实际删除
- `--aggressive` - 激进清理，包括最近未使用的工具
- `--older-than <duration>` - 清理超过指定时间的工具
- `--size-threshold <size>` - 仅清理超过指定大小的工具

### 示例
```bash
# 清理未使用的工具
vx global cleanup

# 预览清理操作
vx global cleanup --dry-run

# 清理30天前的工具
vx global cleanup --older-than 30d

# 激进清理
vx global cleanup --aggressive
```

### 输出示例
```bash
$ vx global cleanup --dry-run
🧹 Global Cleanup Preview

Will remove:
  📦 node@20.10.0 (47.8 MB)
    - Reason: Not used by any virtual environment
    - Last used: Never
    
  📦 python@3.10.12 (65.1 MB)
    - Reason: Superseded by python@3.11.5
    - Last used: 2024-01-05 (15 days ago)

Will keep:
  ✅ node@18.17.0 (45.2 MB)
    - Reason: Used by 2 virtual environments
    
  ✅ python@3.11.5 (67.3 MB)
    - Reason: Used by 1 virtual environment
    
  ✅ uv@0.1.0 (8.7 MB)
    - Reason: Used by 3 virtual environments

Summary:
  - Will free: 112.9 MB
  - Will keep: 121.2 MB
  - Total savings: 48.3%

Run 'vx global cleanup' to execute this plan.
```

## 清理策略

### 自动清理规则
1. **未引用工具** - 不被任何虚拟环境使用的工具
2. **重复版本** - 同一工具的多个版本，保留最新和被使用的版本
3. **过期工具** - 超过指定时间未使用的工具
4. **损坏安装** - 安装不完整或损坏的工具

### 保护规则
1. **活跃使用** - 被虚拟环境引用的工具
2. **最近安装** - 7天内安装的工具
3. **配置指定** - 在项目配置中明确指定的版本

## 磁盘使用分析

```bash
$ vx global list --size --sort size
Global Tools (sorted by size):
  python@3.11.5    67.3 MB  (used)
  node@20.10.0     47.8 MB  (unused) ⚠️
  node@18.17.0     45.2 MB  (used)
  uv@0.1.0         8.7 MB   (used)

Total: 169.0 MB
Unused: 47.8 MB (28.3%)

💡 Run 'vx global cleanup' to free 47.8 MB
```

## 故障排除

### 清理失败
```bash
# 检查权限
ls -la ~/.vx/tools/

# 强制清理
vx global cleanup --force

# 手动删除
rm -rf ~/.vx/tools/node/20.10.0/
```

### 依赖检查错误
```bash
# 刷新依赖信息
vx global dependents node --refresh

# 检查虚拟环境状态
vx venv list
```

## 相关命令

- [venv](./venv.md) - 虚拟环境管理
- [install](./install.md) - 安装工具
- [remove](./remove.md) - 移除工具
- [cleanup](./cleanup.md) - 系统清理
