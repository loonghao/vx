# 直接执行

使用 vx 最简单的方式是直接执行 — 只需在任何命令前加上 `vx`。

## 基本用法

```bash
# 语言运行时
vx node --version
vx python --version
vx go version
vx cargo --version

# 包管理器
vx npm install
vx uvx ruff check .
vx pnpm dev

# DevOps 工具
vx terraform plan
vx kubectl get pods
vx dagu server

# 构建工具
vx just build
vx cmake --build build
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

## 运行语言运行时

### Node.js

```bash
# 运行 Node.js 脚本
vx node app.js
vx node --eval "console.log('Hello from vx!')"

# 交互式 REPL
vx node
```

### Python

```bash
# 运行 Python 脚本
vx python main.py
vx python -c "import sys; print(sys.version)"

# 直接运行模块
vx python -m http.server 8000
vx python -m json.tool data.json
```

### Go

```bash
# 构建和运行
vx go build -o myapp ./cmd/server
vx go run main.go
vx go test ./...

# 安装 Go 工具
vx go install golang.org/x/tools/gopls@latest
```

### Rust / Cargo

```bash
# 构建项目
vx cargo build --release
vx cargo test
vx cargo run -- --port 8080

# 创建新项目
vx cargo new my-cli
vx cargo init .

# 通过 cargo 安装工具
vx cargo install ripgrep
```

## 运行包管理器

### npm / npx

```bash
# 项目设置
vx npm init -y
vx npm install express typescript
vx npm run dev

# 使用 npx 运行一次性命令
vx npx create-react-app my-app
vx npx create-next-app@latest my-next-app
vx npx eslint --fix .
vx npx prettier --write .
vx npx tsx script.ts
```

### pnpm

```bash
# 项目设置
vx pnpm init
vx pnpm add express
vx pnpm install
vx pnpm dev

# 工作区管理
vx pnpm -r build          # 构建所有包
vx pnpm --filter api dev  # 运行指定包的 dev
```

### yarn

```bash
vx yarn init
vx yarn add react react-dom
vx yarn dev
```

### bun

```bash
vx bun init
vx bun add express
vx bun run dev
vx bunx create-next-app my-app
```

### uv / uvx（Python）

```bash
# 项目生命周期
vx uv init my-project
vx uv add requests flask pytest
vx uv sync
vx uv run python main.py
vx uv run pytest

# 虚拟环境管理
vx uv venv
vx uv pip install -r requirements.txt

# 无需安装即可运行 CLI 工具（uvx）
vx uvx ruff check .              # 代码检查
vx uvx ruff format .             # 代码格式化
vx uvx black .                   # 代码格式化
vx uvx mypy src/                 # 类型检查
vx uvx pytest                    # 运行测试
vx uvx jupyter notebook          # 启动 Jupyter
vx uvx cookiecutter gh:user/repo # 项目脚手架
vx uvx pre-commit run --all-files
```

## 运行 DevOps 工具

### Terraform

```bash
vx terraform init
vx terraform plan
vx terraform apply -auto-approve
vx terraform destroy
```

### kubectl & Helm

```bash
vx kubectl get pods -A
vx kubectl apply -f deployment.yaml
vx helm install my-release ./chart
vx helm upgrade my-release ./chart
```

### Dagu（工作流引擎）

```bash
# 启动 Web UI 仪表板
vx dagu server

# 运行工作流
vx dagu start my-workflow
vx dagu status my-workflow

# Dagu + vx：在 DAG 定义中使用 vx 管理的工具
# my-workflow.yaml:
#   steps:
#     - name: lint
#       command: vx uvx ruff check .
#     - name: test
#       command: vx uv run pytest
#     - name: build
#       command: vx cargo build --release
```

### GitHub CLI

```bash
vx gh repo clone owner/repo
vx gh pr create --fill
vx gh issue list
vx gh release create v1.0.0
```

## 运行构建工具

### Just（现代 Make）

```bash
# 运行任务
vx just build
vx just test
vx just --list

# Just + vx 子进程 PATH：工具无需 vx 前缀即可使用
# justfile:
#   lint:
#       uvx ruff check .     # 可用！vx 工具在子进程 PATH 中
#       npm run lint
```

### CMake & Ninja

```bash
vx cmake -B build -G Ninja
vx cmake --build build --config Release
vx ninja -C build
```

### Task (go-task)

```bash
vx task build
vx task test
vx task --list
```

## 运行数据和媒体工具

```bash
# JSON 处理
vx jq '.name' package.json
vx jq -r '.dependencies | keys[]' package.json

# 音视频处理
vx ffmpeg -i input.mp4 -c:v libx264 output.mp4
vx ffprobe -show_format video.mp4

# 图像处理
vx magick input.png -resize 50% output.png
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
vx cargo build --release --target x86_64-unknown-linux-musl
```

## 环境变量

在命令前设置环境变量：

```bash
# Unix
NODE_ENV=production vx node server.js
RUST_LOG=debug vx cargo run

# 或使用 env
env DATABASE_URL=postgres://localhost/mydb vx uv run main.py
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

## 子进程 PATH 继承

通过 `vx` 运行工具时，该工具生成的任何子进程都可以自动访问 PATH 中所有 vx 管理的工具。这意味着构建工具、任务运行器和脚本可以直接使用 vx 管理的工具，无需 `vx` 前缀。

### 示例：justfile

```just
# justfile — 所有工具无需 vx 前缀即可使用！
lint:
    uvx ruff check .
    uvx mypy src/

test:
    uv run pytest

build:
    npm run build
    cargo build --release
```

运行：

```bash
vx just lint    # justfile 中的任务可以直接使用 vx 工具
vx just test
vx just build
```

### 示例：Dagu 工作流

```yaml
# workflow.yaml — DAG 步骤中 vx 工具可用
steps:
  - name: lint
    command: uvx ruff check .
  - name: test
    command: uv run pytest
    depends:
      - lint
  - name: build
    command: cargo build --release
    depends:
      - test
```

运行：

```bash
vx dagu start workflow
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

如果需要禁用子进程 PATH 继承（例如隔离），可以在项目的 `vx.toml` 中配置：

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

## 实际示例

### 全栈 Web 应用

```bash
# 前端
vx npx create-next-app@latest my-app
cd my-app
vx npm install
vx npm run dev

# 后端 API（Python）
vx uv init api && cd api
vx uv add fastapi uvicorn
vx uv run uvicorn main:app --reload
```

### Python 数据科学

```bash
vx uv init analysis && cd analysis
vx uv add pandas numpy matplotlib scikit-learn
vx uvx jupyter notebook
vx python -c "import pandas; print(pandas.__version__)"
```

### Go 微服务

```bash
mkdir my-service && cd my-service
vx go mod init github.com/user/my-service
vx go get github.com/gin-gonic/gin
vx go run main.go
vx go build -o server .
```

### Rust CLI 工具

```bash
vx cargo new my-cli
cd my-cli
vx cargo add clap --features derive
vx cargo build --release
```

### 使用 Dagu 的跨语言项目

```bash
# 定义使用多个工具的工作流
# build-pipeline.yaml:
#   steps:
#     - name: frontend
#       command: npm run build
#       dir: frontend/
#     - name: backend
#       command: cargo build --release
#       dir: backend/
#     - name: deploy
#       command: terraform apply -auto-approve
#       depends: [frontend, backend]

vx dagu start build-pipeline
vx dagu server   # 通过 Web UI 监控 http://localhost:8080
```

### DevOps 自动化

```bash
# 基础设施
vx terraform init && vx terraform plan
vx kubectl apply -f k8s/

# 本地运行 CI 工作流
vx just ci       # 在本地运行所有 CI 检查
```

## 提示

::: tip 首次运行
首次运行会较慢，因为需要下载和安装工具。后续运行使用缓存版本，速度会快很多。
:::

::: tip 版本固定
在团队项目中始终指定版本以确保可重现性。
:::

::: tip 子进程 PATH
通过 vx 使用 `just`、`dagu` 或 `make` 等任务运行器时，所有 vx 管理的工具在子进程中自动可用 — 无需在任务/步骤中添加 `vx` 前缀。
:::

## 下一步

- [项目环境](/zh/guide/project-environments) - 设置项目特定配置
- [实际使用案例](/zh/guides/use-cases) - 更多实际示例
- [CLI 参考](/zh/cli/overview) - 完整命令参考
