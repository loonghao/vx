# vx cleanup - 清理操作

清理孤立的包、缓存和未使用的文件。

## 语法

```bash
vx cleanup [OPTIONS]
```

## 描述

`vx cleanup` 命令用于清理 vx 系统中的孤立文件、过期缓存和未使用的工具版本，释放磁盘空间。

## 选项

- `--dry-run` - 预览清理操作，不实际执行
- `--cache-only` - 仅清理缓存文件
- `--orphaned-only` - 仅清理孤立的工具版本
- `--force` - 强制清理，跳过确认
- `--older-than <DAYS>` - 清理超过指定天数的文件
- `-v, --verbose` - 显示详细输出

## 清理类型

### 缓存清理

```bash
# 清理所有缓存
vx cleanup --cache-only

# 清理超过7天的缓存
vx cleanup --cache-only --older-than 7
```

### 孤立文件清理

```bash
# 清理孤立的工具版本
vx cleanup --orphaned-only

# 预览孤立文件
vx cleanup --orphaned-only --dry-run
```

### 完整清理

```bash
# 清理所有类型的文件
vx cleanup

# 预览完整清理操作
vx cleanup --dry-run
```

## 示例

### 基本清理

```bash
# 交互式清理
vx cleanup

# 强制清理，无需确认
vx cleanup --force
```

### 预览操作

```bash
# 预览将要清理的内容
vx cleanup --dry-run

# 详细预览
vx cleanup --dry-run --verbose
```

### 选择性清理

```bash
# 仅清理缓存
vx cleanup --cache-only

# 仅清理孤立文件
vx cleanup --orphaned-only

# 清理超过30天的文件
vx cleanup --older-than 30
```

## 清理内容

### 缓存文件

- 工具版本信息缓存
- 下载临时文件
- 安装包缓存
- 元数据缓存

### 孤立文件

- 未被任何虚拟环境引用的工具版本
- 损坏的安装文件
- 不完整的下载文件
- 过期的符号链接

### 临时文件

- 安装过程中的临时文件
- 解压缓存
- 日志文件（超过保留期限）

## 工作流程

1. **扫描文件**: 扫描 vx 目录结构
2. **分析依赖**: 检查文件的使用情况
3. **标记清理**: 标记可以安全清理的文件
4. **显示预览**: 显示将要清理的内容
5. **确认操作**: 等待用户确认（除非使用 `--force`）
6. **执行清理**: 删除标记的文件
7. **报告结果**: 显示清理统计信息

## 输出示例

```bash
$ vx cleanup --dry-run
扫描清理目标...

将要清理的内容:

缓存文件:
  ~/.vx/cache/versions/ (15.2 MB)
  ~/.vx/cache/downloads/ (128.5 MB)

孤立的工具版本:
  ~/.vx/tools/node/16.14.0/ (45.3 MB) - 未被引用
  ~/.vx/tools/python/3.9.0/ (67.8 MB) - 未被引用

临时文件:
  ~/.vx/tmp/ (2.1 MB)

总计可释放空间: 258.9 MB

运行 'vx cleanup' 执行清理操作
```

### 实际清理输出

```bash
$ vx cleanup
扫描清理目标...

将要清理 258.9 MB 的文件
确认继续? [y/N]: y

正在清理...
✓ 清理缓存文件 (143.7 MB)
✓ 清理孤立工具版本 (113.1 MB)
✓ 清理临时文件 (2.1 MB)

清理完成！释放了 258.9 MB 磁盘空间
```

## 安全措施

### 依赖检查

- 检查虚拟环境依赖
- 验证工具版本引用
- 保护活跃的安装

### 确认机制

- 默认需要用户确认
- 显示详细的清理列表
- 提供预览模式

### 备份保护

- 不清理最近使用的版本
- 保留全局默认版本
- 保护项目配置引用的版本

## 自动清理

### 定期清理

```bash
# 设置自动清理
vx config set auto_cleanup.enabled true
vx config set auto_cleanup.interval "7d"
vx config set auto_cleanup.max_cache_age "30d"
```

### 清理策略

```toml
# ~/.vx/config/global.toml
[auto_cleanup]
enabled = true
interval = "7d"           # 每7天运行一次
max_cache_age = "30d"     # 清理30天以上的缓存
keep_recent_versions = 3  # 保留最近3个版本
```

## 注意事项

1. **不可恢复**: 清理操作不可恢复，请谨慎操作
2. **网络重下载**: 清理缓存后可能需要重新下载
3. **虚拟环境影响**: 清理工具版本可能影响虚拟环境
4. **磁盘空间**: 定期清理有助于释放磁盘空间

## 相关命令

- [`vx stats`](./stats.md) - 查看磁盘使用统计
- [`vx global cleanup`](./global.md) - 清理全局工具
- [`vx remove`](./remove.md) - 移除特定工具版本
