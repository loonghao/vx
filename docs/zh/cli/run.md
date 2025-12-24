# run 命令

运行 `.vx.toml` 中定义的脚本。

## 语法

```bash
vx run <script> [-- args...]
```

## 参数

| 参数 | 描述 |
|------|------|
| `script` | 要运行的脚本名称 |
| `args` | 传递给脚本的额外参数 |

## 选项

| 选项 | 描述 |
|------|------|
| `--list, -l` | 列出可用脚本 |
| `--verbose, -v` | 显示详细输出 |

## 示例

```bash
# 运行脚本
vx run dev
vx run test
vx run build

# 传递额外参数
vx run test -- --coverage
vx run build -- --mode production

# 列出可用脚本
vx run --list
```

## 配置示例

```toml
[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--port", "8080"]
env = { DEBUG = "true" }
```

## 参见

- [dev](./dev) - 进入开发环境
- [配置指南](/zh/guide/configuration) - 脚本配置
