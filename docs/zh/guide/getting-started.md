# 快速上手

5 分钟内开始使用 vx。

## 前置条件

- **Windows 10+**、**macOS 10.15+** 或 **Linux**（glibc 2.17+）
- 首次下载工具需要网络连接

## 第一步：安装 vx

::: code-group
```bash [Linux / macOS]
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

```powershell [Windows (PowerShell)]
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

```bash [Cargo]
cargo install vx
```
:::

验证安装：

```bash
vx --version
```

## 第二步：使用你的第一个工具

在任何命令前加上 `vx`。工具会在首次使用时自动安装：

```bash
# Node.js
vx node --version        # 下载并安装 Node.js，然后输出版本

# Python
vx python --version      # 下载并安装 Python，然后输出版本

# Go
vx go version            # 下载并安装 Go，然后输出版本
```

::: tip 自动安装
首次运行 `vx <tool>` 时，vx 会自动下载并安装最新稳定版本。无需手动设置！
:::

## 第三步：安装指定版本

```bash
# 安装指定版本
vx install node@22
vx install python@3.12
vx install go@1.23

# 同时安装多个工具
vx install node@22 python@3.12 uv@latest

# 使用语义化版本范围
vx install "node@^22"    # 最新 22.x.x
vx install "python@~3.12" # 最新 3.12.x
```

## 第四步：设置项目

创建 `vx.toml` 来定义项目的工具链：

```bash
cd my-project
vx config init
```

编辑生成的 `vx.toml`：

```toml
[tools]
node = "22"
python = "3.12"
uv = "latest"

[scripts]
dev = "vx node server.js"
test = "vx uv run pytest"
lint = "vx uvx ruff check ."
build = "vx node scripts/build.js"
```

安装所有项目工具：

```bash
vx setup
```

## 第五步：运行项目脚本

```bash
# 运行已定义的脚本
vx run dev               # 启动开发服务器
vx run test              # 运行测试
vx run lint              # 运行代码检查

# 列出可用脚本
vx run --list

# 向脚本传递参数
vx run test -- -v --coverage
```

## 第六步：进入开发环境

```bash
# 进入一个包含所有项目工具的交互式 Shell
vx dev

# 或在项目环境中运行单个命令
vx dev -c "node --version && python --version"

# 导出 CI/CD 环境
vx dev --export --format github >> $GITHUB_PATH
```

## 第七步：设置 Shell 集成（可选）

启用自动版本切换和 Tab 补全：

::: code-group
```bash [Bash]
echo 'eval "$(vx shell init bash)"' >> ~/.bashrc
```

```bash [Zsh]
echo 'eval "$(vx shell init zsh)"' >> ~/.zshrc
```

```bash [Fish]
echo 'vx shell init fish | source' >> ~/.config/fish/config.fish
```

```powershell [PowerShell]
Add-Content $PROFILE 'Invoke-Expression (vx shell init powershell | Out-String)'
```
:::

## 常见工作流

### Node.js 项目

```bash
vx npx create-react-app my-app
cd my-app
vx npm install
vx npm start
```

### Python 项目

```bash
vx uv init my-project
cd my-project
vx uv add flask pytest
vx uv run flask run
```

### Go 项目

```bash
mkdir my-service && cd my-service
vx go mod init github.com/user/my-service
vx go run main.go
```

### 多语言项目

```toml
# vx.toml
[tools]
node = "22"
python = "3.12"
go = "1.23"
just = "latest"

[scripts]
frontend = "cd frontend && vx npm run dev"
backend = "cd backend && vx go run ."
api = "cd api && vx uv run flask run"
all = "just dev"
```

## 下一步

- [核心概念](/zh/guide/concepts) — 了解 Provider、运行时和版本
- [配置](/zh/guide/configuration) — 深入了解 `vx.toml`
- [CLI 参考](/zh/cli/overview) — 探索所有可用命令
- [支持的工具](/zh/tools/overview) — 查看 50+ 工具的完整列表
