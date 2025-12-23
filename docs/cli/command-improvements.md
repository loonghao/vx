# VX 命令改进和迁移指南

## 概述

基于CLI设计最佳实践，VX已经改进了命令结构，提供更一致、直观的用户体验。本文档详细说明了这些改进和迁移路径。

## 🔄 命令改进

### 标准化命令

VX提供标准化的CLI命令，遵循最佳实践：

| 命令 | 说明 | 状态 |
|------|------|------|
| `uninstall` | 标准的卸载命令 | ✅ 可用 |
| `which` | 标准的位置查询命令 | ✅ 可用 |
| `versions` | 直观的版本查询命令 | ✅ 可用 |
| `clean` | 简洁的清理命令 | ✅ 可用 |
| `shell` | 统一的Shell集成命令 | ✅ 可用 |

### 命令别名

为提高效率，VX提供了常用命令的简短别名：

| 别名 | 完整命令 | 说明 |
|------|----------|------|
| `i` | `install` | 安装工具 |
| `rm` | `remove` / `uninstall` | 移除工具 |
| `ls` | `list` | 列出工具 |
| `up` | `update` | 更新工具 |
| `cfg` | `config` | 配置管理 |
| `clean` | `cleanup` | 清理系统 |

## 📋 完整命令列表

### 工具管理命令

```bash
# 安装工具
vx install node@18.17.0    # 完整命令
vx i node@18.17.0          # 别名

# 卸载工具
vx uninstall node          # 标准命令
vx rm node                 # 别名

# 列出工具
vx list                    # 完整命令
vx ls                      # 别名
vx list --installed        # 只显示已安装
vx list --available        # 只显示可用

# 更新工具
vx update                  # 完整命令
vx up                      # 别名

# 查找工具位置
vx which node              # 标准命令

# 查看可用版本
vx versions node           # 标准命令
```

### 项目管理命令

```bash
# 初始化项目
vx init                    # 基本初始化
vx init --template node    # 使用模板
vx init --interactive      # 交互式初始化

# 同步项目工具
vx sync                    # 同步安装
vx sync --check            # 检查状态
vx sync --dry-run          # 预览操作
```

### 配置管理命令

```bash
# 配置管理
vx config                  # 完整命令
vx cfg                     # 别名
vx config show             # 显示配置
vx config set key value    # 设置配置
vx config get key          # 获取配置
```

### 清理命令

```bash
# 系统清理
vx clean                   # 标准命令
vx clean --cache           # 只清理缓存
vx clean --orphaned        # 只清理孤立版本
vx clean --all             # 清理所有
vx clean --dry-run         # 预览清理
```

### Shell集成命令

```bash
# 统一的Shell命令
vx shell init              # 生成初始化脚本
vx shell completions bash  # 生成补全脚本
```

## 🚀 开始使用

所有标准命令和别名都已经可用：

```bash
# 使用标准命令
vx i node@18.17.0
vx uninstall node
vx which node
vx versions node
vx clean --cache
vx shell init
```

## 🎯 最佳实践

### 日常使用

```bash
# 使用别名提高效率
vx i node@18.17.0          # 而不是 vx install
vx ls                      # 而不是 vx list
vx cfg show                # 而不是 vx config show

# 使用标准命令
vx uninstall node          # 标准卸载命令
vx which node              # 标准位置查询
vx clean                   # 标准清理命令
```

### 脚本编写

```bash
# 在脚本中使用完整命令名以提高可读性
vx install node@18.17.0
vx uninstall node
vx config set auto_install true
vx clean --dry-run
```

### Shell集成

```bash
# 设置Shell集成
eval "$(vx shell init)"

# 或者保存到文件
vx shell init > ~/.vx-init.sh
echo 'source ~/.vx-init.sh' >> ~/.bashrc

# 设置补全
vx shell completions bash > /etc/bash_completion.d/vx
```

## 📊 命令使用统计

基于用户反馈和使用模式，以下是推荐的命令优先级：

### 高频命令 (建议使用别名)

- `vx i` (install)
- `vx ls` (list)
- `vx rm` (uninstall)
- `vx up` (update)

### 中频命令 (标准命令)

- `vx which` (位置查询)
- `vx versions` (版本查询)
- `vx clean` (系统清理)

### 低频命令 (保持完整名称)

- `vx init`
- `vx sync`
- `vx config`
- `vx shell`

## 🔮 未来计划

### 即将推出

- 更多智能别名
- 上下文感知的命令建议
- 自动命令纠错

### 长期规划

- 插件系统的命令扩展
- 自定义别名配置
- 命令使用分析和优化建议

## 📞 反馈和支持

如果您对命令改进有任何建议或遇到问题：

1. 查看 [troubleshooting.md](./troubleshooting.md)
2. 提交 GitHub Issue
3. 参与社区讨论

我们致力于提供最佳的CLI体验，您的反馈对我们非常重要！
