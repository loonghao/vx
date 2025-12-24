# setup 命令

安装项目所有工具和依赖。

## 语法

```bash
vx setup [options]
```

## 选项

| 选项 | 描述 |
|------|------|
| `--dry-run` | 仅显示将执行的操作 |
| `--force, -f` | 强制重新安装所有工具 |
| `--no-parallel` | 顺序安装（禁用并行） |
| `--verbose, -v` | 显示详细输出 |

## 示例

```bash
# 安装项目工具
vx setup

# 试运行
vx setup --dry-run

# 强制重新安装
vx setup --force

# 详细输出
vx setup --verbose
```

## 功能

`vx setup` 会：

1. 读取 `.vx.toml` 配置
2. 安装所有必需的工具
3. 创建 Python 虚拟环境（如果配置）
4. 安装 Python 依赖
5. 验证必需的环境变量

## 参见

- [项目环境](/zh/guide/project-environments) - 项目环境配置
