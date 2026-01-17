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

使用 `@` 来指定版本：

```bash
# 指定主版本
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

# 运行 uvx (uv tool run)
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

工具名称后的所有参数都会被传递：

```bash
# 这些是等效的
vx node script.js --port 3000
node script.js --port 3000  # (如果 node 在 PATH 中)

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

vx 在当前目录中运行命令：

```bash
cd my-project
vx npm install  # 在 my-project/ 中运行
```

## 使用系统工具

如果您想使用系统安装的工具而不是 vx 管理的：

```bash
vx --use-system-path node --version
```

## 子进程 PATH 继承

当通过 `vx` 运行工具时，该工具生成的任何子进程都会自动访问所有 vx 管理的工具。这意味着构建工具、任务运行器和脚本可以直接使用 vx 管理的工具，无需 `vx` 前缀。

### 示例：justfile

```just
# justfile
lint:
    uvx ruff check .      # 可以工作！uvx 在子进程 PATH 中
    uvx mypy src/

test:
    uv run pytest         # 可以工作！uv 在子进程 PATH 中

build:
    npm run build         # 可以工作！npm 在子进程 PATH 中
```

运行：

```bash
vx just lint    # justfile 配方可以直接使用 vx 工具
vx just test
```

### 示例：Makefile

```makefile
# Makefile
lint:
	uvx ruff check .

test:
	npm test

build:
	go build -o app
```

运行：

```bash
vx make lint    # Make 目标可以直接使用 vx 工具
```

### 禁用 PATH 继承

如果需要禁用子进程 PATH 继承（例如，为了隔离），可以在项目的 `vx.toml` 中配置：

```toml
[settings]
inherit_vx_path = false
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
首次运行较慢，因为需要下载和安装工具。后续运行使用缓存版本，速度更快。
:::

::: tip 版本固定
在团队项目中始终指定版本以确保可重现性。
:::

## 下一步

- [项目环境](/guide/project-environments) - 设置项目特定的配置
- [CLI 参考](/cli/overview) - 完整的命令参考
