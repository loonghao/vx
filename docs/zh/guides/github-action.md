---
outline: deep
---

# 在 GitHub Actions 中使用 vx

vx 提供了官方的 GitHub Action，使您可以轻松地在 CI/CD 工作流程中使用 vx。这样可以确保本地开发和 CI 环境中使用一致的开发工具版本。

## 快速开始

在您的 GitHub Actions 工作流程中添加以下内容：

```yaml
- uses: loonghao/vx@main
  with:
    github-token: ${{secrets.GITHUB_TOKEN}}
```

> **注意**：您可以使用 `@main` 获取最新版本，或者固定到特定的发布标签（例如 `@vx-v0.6.4`）。查看 [releases](https://github.com/loonghao/vx/releases) 了解可用版本。

然后使用 vx 运行任何支持的工具：

```yaml
- run: vx node --version
- run: vx npm install
- run: vx uv pip install -r requirements.txt
- run: vx go build ./...
```

## 完整示例

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      # 设置 vx 并启用缓存
      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'node uv'  # 预安装这些工具
          cache: 'true'

      # 使用 vx 运行工具
      - name: 安装依赖
        run: vx npm ci

      - name: 运行测试
        run: vx npm test

      - name: 构建
        run: vx npm run build
```

## 输入参数

| 输入 | 描述 | 默认值 |
|------|------|--------|
| `version` | 要安装的 vx 版本（例如 "0.5.7", "latest"） | `latest` |
| `github-token` | 用于 API 请求的 GitHub 令牌（避免速率限制） | `github.token` |
| `tools` | 要预安装的工具列表，用空格分隔（例如 "node go uv"） | `''` |
| `cache` | 启用 vx 工具目录的缓存 | `true` |
| `cache-key-prefix` | 缓存键的自定义前缀 | `vx-tools` |

## 输出参数

| 输出 | 描述 |
|------|------|
| `version` | 安装的 vx 版本 |
| `cache-hit` | 是否命中缓存 |

## 使用 Docker 镜像

vx 提供官方 Docker 镜像，可以直接在您的 CI/CD 工作流程中使用。这些镜像同时托管在 Docker Hub 和 GitHub Container Registry。

### 可用镜像

| 镜像标签 | 描述 | 基础镜像 |
|----------|------|----------|
| `vx:latest` | 仅包含 vx 的最小镜像 | Ubuntu 24.04 (Noble) |
| `vx:tools-latest` | 预装常用工具（uv, ruff, node）的镜像 | Ubuntu 24.04 (Noble) |

::: info 为什么使用 Ubuntu 24.04？
我们使用 Ubuntu 24.04（glibc 2.39）因为：
1. vx 在 Ubuntu 24.04 runner 上编译，需要 glibc 2.39
2. 大多数开发工具提供 glibc 编译的二进制文件
3. Alpine（musl）会导致"找不到文件或目录"错误
4. 旧版 Debian/Ubuntu 的 glibc 版本过低
:::

### 在容器作业中使用工具镜像

`vx:tools-latest` 镜像预装了常用工具，非常适合需要快速启动的 CI/CD 工作流程：

**预装工具：**
- **uv** - 快速的 Python 包管理器
- **ruff**（通过 uvx）- Python 代码检查和格式化工具
- **Node.js** - JavaScript 运行时（LTS 版本）

```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    container:
      image: ghcr.io/loonghao/vx:tools-latest
    steps:
      - uses: actions/checkout@v6

      # 工具已经可用 - 无需安装！
      - name: 检查 Python 代码
        run: vx uvx ruff check .

      - name: 运行测试
        run: |
          vx uv sync
          vx uv run pytest

      - name: 构建前端
        run: |
          vx npm ci
          vx npm run build
```

### 拉取镜像

```bash
# 从 GitHub Container Registry（推荐）
docker pull ghcr.io/loonghao/vx:latest
docker pull ghcr.io/loonghao/vx:tools-latest

# 从 Docker Hub
docker pull longhal/vx:latest
docker pull longhal/vx:tools-latest
```

### 在 Docker Compose 中使用

```yaml
# docker-compose.yml
version: '3.8'

services:
  dev:
    image: ghcr.io/loonghao/vx:tools-latest
    working_dir: /app
    volumes:
      - .:/app
    command: bash -c "vx uv sync && vx uv run pytest"
```

### 构建自定义镜像

您可以扩展 vx 镜像并添加自己的工具：

```dockerfile
FROM ghcr.io/loonghao/vx:tools-latest

# 预安装其他工具
RUN vx go version

# 添加项目文件
COPY . /app
WORKDIR /app

# 运行应用程序
CMD ["vx", "uv", "run", "main.py"]
```

### 镜像标签

Docker Hub 和 GHCR 都提供以下标签：

- `latest` - 最新稳定的基础镜像
- `tools-latest` - 最新稳定的工具镜像
- `{version}` - 特定版本（例如 `0.6.5`）
- `tools-{version}` - 带工具的特定版本

::: tip 何时使用工具镜像
使用 `vx:tools-latest` 当：
- 您的工作流程需要 Python（uv/ruff）或 Node.js
- 您希望更快的 CI 启动时间
- 您运行多个都需要相同工具的作业

使用 `vx:latest` 当：
- 您只需要工具镜像中没有的特定工具
- 您希望尽可能小的镜像大小
- 您正在基于 vx 构建自定义镜像
:::

## 用例

### 自动依赖安装

vx 在运行命令前会自动检测并安装缺失的依赖。这在 CI 环境中特别有用，您不需要手动添加安装步骤。

| 工具 | 触发命令 | 自动运行 | 检测条件 |
|------|----------|----------|----------|
| **uv** | `vx uv run` | `uv sync` | 存在 `pyproject.toml`，缺少 `.venv` |
| **npm** | `vx npm run` | `npm install` | 存在 `package.json`，缺少 `node_modules` |
| **pnpm** | `vx pnpm run` | `pnpm install` | 存在 `package.json`，缺少 `node_modules` |
| **yarn** | `vx yarn run` | `yarn install` | 存在 `package.json`，缺少 `node_modules` |
| **bun** | `vx bun run` | `bun install` | 存在 `package.json`，缺少 `node_modules` |
| **go** | `vx go run` | `go mod download` | 存在 `go.mod`，缺少 `vendor` |

这意味着您的 CI 工作流程可以非常简单：

```yaml
- run: vx uv run pytest      # 自动先同步依赖
- run: vx npm run build      # 自动先安装 node_modules
- run: vx go run main.go     # 自动先下载模块
```

### Node.js 项目

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'node'

      - run: vx npm ci
      - run: vx npm test
      - run: vx npm run build
```

### 使用 UV 的 Python 项目

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'uv'

      # vx 会在 'uv run' 之前自动运行 'uv sync'（如果 .venv 不存在）
      - run: vx uv run pytest
      - run: vx uvx ruff check .
```

### Go 项目

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'go'

      - run: vx go build ./...
      - run: vx go test ./...
```

### 多语言项目

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}
          tools: 'node uv go'

      # 前端
      - run: vx npm ci
      - run: vx npm run build

      # 后端（Python）
      - run: vx uv sync
      - run: vx uv run pytest

      # 服务（Go）
      - run: vx go build ./cmd/...
```

### 跨平台 CI

```yaml
jobs:
  build:
    runs-on: ${{matrix.os}}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v6

      - uses: loonghao/vx@main
        with:
          github-token: ${{secrets.GITHUB_TOKEN}}

      # 相同的命令在所有平台上都能运行！
      - run: vx node --version
      - run: vx npm ci
      - run: vx npm test
```

## 缓存

该 action 会自动缓存 vx 工具目录（`~/.vx`）以加速后续运行。您可以自定义缓存行为：

```yaml
- uses: loonghao/vx@main
  with:
    cache: 'true'
    cache-key-prefix: 'my-project-vx'
```

禁用缓存：

```yaml
- uses: loonghao/vx@main
  with:
    cache: 'false'
```

## 故障排除

### 速率限制

如果遇到 GitHub API 速率限制，请确保提供 GitHub 令牌：

```yaml
- uses: loonghao/vx@main
  with:
    github-token: ${{secrets.GITHUB_TOKEN}}
```

### 工具安装失败

如果工具安装失败，请检查：

1. vx 是否支持该工具（`vx list` 查看所有支持的工具）
2. 到工具下载源的网络连接
3. 磁盘空间是否充足

### 缓存问题

如果遇到缓存相关问题，请尝试：

1. 使用不同的缓存键前缀
2. 临时禁用缓存
3. 在 GitHub Actions 设置中清除缓存

## 支持的工具

vx 支持许多流行的开发工具：

- **JavaScript/TypeScript**：Node.js、npm、npx、Bun、Deno、pnpm、Yarn、Vite
- **Python**：UV、uvx
- **Go**：Go
- **Rust**：Cargo、rustc、rustup
- **Java**：Java、javac
- **DevOps**：Terraform、kubectl、Helm
- **其他**：Just、Zig 等

运行 `vx list` 查看所有可用工具。

## 使用 vx 的优势

1. **统一工具管理**：一个 action 管理所有开发工具
2. **版本一致性**：CI 和本地开发使用相同的工具版本
3. **跨平台**：支持 Linux、macOS 和 Windows
4. **缓存**：自动缓存加速 CI 运行
5. **简单性**：无需配置多个 setup action
