# install 命令

安装工具版本。

## 语法

```bash
vx install <tool>[@version] [options]
```

## 参数

| 参数 | 描述 |
|------|------|
| `tool` | 要安装的工具名称 |
| `version` | 可选的版本说明符 |

## 选项

| 选项 | 描述 |
|------|------|
| `--force, -f` | 强制重新安装，即使已安装 |
| `--global, -g` | 设为全局默认版本 |
| `--verbose, -v` | 显示详细输出 |

## 示例

```bash
# 安装最新版本
vx install node
vx install python
vx install go

# 安装特定版本
vx install node@20
vx install node@20.10.0
vx install python@3.11

# 安装并设为全局默认
vx install node@20 --global

# 强制重新安装
vx install node --force
```

## 版本说明符

| 格式 | 描述 | 示例 |
|------|------|------|
| `latest` | 最新稳定版 | `node@latest` |
| `lts` | 最新 LTS 版本 | `node@lts` |
| `X` | 主版本 | `node@20` |
| `X.Y` | 次版本 | `node@20.10` |
| `X.Y.Z` | 精确版本 | `node@20.10.0` |

## 参见

- [list](./list) - 列出可用工具
