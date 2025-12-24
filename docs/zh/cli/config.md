# config 命令

管理 vx 配置。

## 语法

```bash
vx config <subcommand> [options]
```

## 子命令

### show

显示当前配置。

```bash
vx config show
```

### get

获取配置值。

```bash
vx config get <key>
```

### set

设置配置值。

```bash
vx config set <key> <value>
```

### reset

重置为默认配置。

```bash
vx config reset
```

### edit

在编辑器中打开配置文件。

```bash
vx config edit
```

## 示例

```bash
# 显示所有配置
vx config show

# 获取特定值
vx config get defaults.auto_install

# 设置值
vx config set defaults.auto_install true

# 重置配置
vx config reset

# 编辑配置文件
vx config edit
```

## 配置键

| 键 | 描述 | 默认值 |
|---|------|--------|
| `defaults.auto_install` | 自动安装缺失的工具 | `true` |
| `defaults.parallel_install` | 并行安装工具 | `true` |
| `defaults.cache_duration` | 缓存持续时间 | `7d` |

## 参见

- [配置指南](/zh/guide/configuration) - 配置详解
- [环境变量](/zh/config/env-vars) - 环境变量参考
