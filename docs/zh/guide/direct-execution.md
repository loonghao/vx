# 直接执行

使用 vx 最简单的方式是直接执行 - 只需在任何命令前加上 `vx`。

## 基本用法

```bash
# 运行任何工具
vx node --version
vx python --version
vx go version
vx cargo --version
```

如果工具未安装，vx 会自动安装它。

## 指定版本

使用 `@` 指定版本：

```bash
# 特定主版本
vx node@18 --version

# 精确版本
vx node@18.19.0 --version

# 最新版
vx node@latest --version
```

## 运行包管理器

### npm/npx

```bash
# 运行 npm 命令
vx npm install
vx npm run build

# 运行 npx
vx npx create-react-app my-app
vx npx eslint .
```

### Python/UV

```bash
# 运行 Python
vx python script.py
vx python -m pytest

# 运行 uv
vx uv pip install requests
vx uv venv .venv

# 运行 uvx（uv tool run）
vx uvx ruff check .
vx uvx black .
vx uvx mypy src/
```

### Go

```bash
# 运行 Go 命令
vx go build
vx go test ./...
vx go run main.go

# 安装 Go 工具
vx go install golang.org/x/tools/gopls@latest
```

### Rust/Cargo

```bash
# 运行 Cargo
vx cargo build --release
vx cargo test
vx cargo run

# 运行 rustc
vx rustc --version
```

## 传递参数

工具名称后的所有参数都会传递给工具：

```bash
# 这些是等效的
vx node script.js --port 3000
node script.js --port 3000  # （如果 node 在 PATH 中）

# 复杂参数也可以
vx npm run build -- --mode production
vx go build -ldflags "-s -w" -o app
```

## 环境变量

在命令前设置环境变量：

```bash
# Unix
NODE_ENV=production vx node server.js

# 或使用 env
env NODE_ENV=production vx node server.js
```

## 工作目录

vx 在当前目录运行命令：

```bash
cd my-project
vx npm install  # 在 my-project/ 中运行
```

## 使用系统工具

如果你想使用系统安装的工具而不是 vx 管理的：

```bash
vx --use-system-path node --version
```

## 详细输出

用于调试，使用详细模式：

```bash
vx --verbose node --version
```

这会显示：

- 版本解析
- 安装步骤
- 执行详情

## 示例

### 创建 React 应用

```bash
vx npx create-react-app my-app
cd my-app
vx npm start
```

### Python 数据科学

```bash
vx uvx jupyter notebook
vx python -c "import pandas; print(pandas.__version__)"
```

### Go Web 服务器

```bash
vx go mod init myserver
vx go get github.com/gin-gonic/gin
vx go run main.go
```

### Rust CLI 工具

```bash
vx cargo new my-cli
cd my-cli
vx cargo build --release
```

## 提示

::: tip 首次运行
首次运行会较慢，因为需要下载和安装工具。后续运行使用缓存版本，速度会快很多。
:::

::: tip 版本固定
在团队项目中始终指定版本以确保可重现性。
:::

## 下一步

- [项目环境](/zh/guide/project-environments) - 设置项目特定配置
- [CLI 参考](/zh/cli/overview) - 完整命令参考
