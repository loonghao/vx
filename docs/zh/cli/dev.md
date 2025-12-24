# dev 命令

进入项目开发环境。

## 语法

```bash
vx dev [options]
```

## 选项

| 选项 | 描述 |
|------|------|
| `-c, --command <cmd>` | 运行命令后退出 |
| `--shell <shell>` | 指定使用的 shell |

## 示例

```bash
# 进入开发环境
vx dev

# 运行单个命令
vx dev -c "npm run build"

# 指定 shell
vx dev --shell zsh
```

## 功能

进入开发环境时，vx 会：

1. 激活项目环境
2. 设置 PATH 包含项目工具
3. 激活 Python 虚拟环境（如果配置）
4. 设置环境变量
5. 启动新的 shell

## 参见

- [run](./run) - 运行脚本
- [项目环境](/zh/guide/project-environments) - 项目环境配置
