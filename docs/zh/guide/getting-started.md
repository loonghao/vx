# 快速上手

本指南将帮助你在几分钟内开始使用 vx。

## 前提条件

- 一个终端（bash、zsh、PowerShell 等）
- 用于下载工具的网络连接

## 步骤 1：安装 vx

::: code-group

```bash [Linux/macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

:::

## 步骤 2：立即使用任何工具

只需在命令前加上 `vx`。工具在首次使用时自动安装：

```bash
# 运行 Node.js
vx node --version

# 运行 Python
vx python --version

# 运行 Go
vx go version

# 运行 npm/npx
vx npx create-react-app my-app

# 运行 uv/uvx
vx uvx ruff check .
```

**就是这样！** 无需配置，无需设置，无需学习新命令。

## 步骤 3：设置项目（可选）

对于团队项目，创建 `.vx.toml` 文件以确保每个人使用相同的工具版本：

```bash
# 初始化新的项目配置
vx init
```

或手动创建：

```toml
[project]
name = "my-project"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
test = "npm test"
```

然后运行：

```bash
# 安装所有项目工具
vx setup

# 运行脚本
vx run dev
```

## 常用命令

| 命令 | 描述 |
|------|------|
| `vx <tool> [args]` | 运行工具（如果需要则自动安装） |
| `vx install <tool>` | 安装特定工具 |
| `vx list` | 列出可用工具 |
| `vx setup` | 从 `.vx.toml` 安装所有项目工具 |
| `vx run <script>` | 运行 `.vx.toml` 中定义的脚本 |
| `vx dev` | 进入开发环境 |
| `vx --help` | 显示帮助 |

## 示例工作流

### Web 开发

```bash
# 创建 React 应用
vx npx create-react-app my-app
cd my-app

# 启动开发服务器
vx npm start
```

### Python 开发

```bash
# 运行 Python 脚本
vx python script.py

# 使用 uvx 运行工具
vx uvx ruff check .
vx uvx black .
```

### Go 开发

```bash
# 构建 Go 项目
vx go build -o myapp

# 运行测试
vx go test ./...
```

## 下一步

- [配置指南](/zh/guide/configuration) - 了解 `.vx.toml` 配置
- [CLI 参考](/zh/cli/overview) - 完整命令参考
- [Shell 集成](/zh/guide/shell-integration) - 设置 Shell 集成
