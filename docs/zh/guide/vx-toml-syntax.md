# vx.toml 语法指南

本指南涵盖 `vx.toml` 配置文件的语法模式、常用配方和最佳实践。完整的字段参考请查看 [vx.toml 参考](/zh/config/vx-toml)。

## 快速开始

最小的 `vx.toml` 只需要一个 `[tools]` 节：

```toml
[tools]
node = "20"
```

这告诉 vx："这个项目需要 Node.js 20.x"。运行 `vx node --version` 时会在需要时自动安装 Node.js 20。

## 语法基础

### TOML 基础

`vx.toml` 使用 [TOML](https://toml.io/cn/) 格式。核心概念：

```toml
# 注释以 # 开始
key = "value"                        # 字符串
enabled = true                       # 布尔值
count = 42                           # 整数

[section]                            # 表（节）
field = "value"

[section.nested]                     # 嵌套表
field = "value"

inline = { key1 = "a", key2 = "b" } # 内联表

list = ["item1", "item2"]           # 数组
```

### 版本说明符

vx 支持多种版本说明符格式：

```toml
[tools]
# 部分版本号 — 解析为最新匹配版本
node = "20"              # 最新 20.x.x（如 20.18.1）
go = "1.22"              # 最新 1.22.x（如 1.22.7）

# 精确版本 — 精确锁定
python = "3.12.1"        # 精确匹配 3.12.1

# 特殊关键字
uv = "latest"            # 最新稳定发布版
node = "lts"             # 最新 LTS 发布版（运行时特定）

# 通道名（Rust 特定）
rustup = "stable"        # 稳定通道
rustup = "nightly"       # 每夜构建通道
rustup = "beta"          # Beta 通道
```

**解析顺序**：当调用 `vx <运行时>` 时：
1. CLI 显式版本（`vx node@22`）
2. `vx.lock`（如果存在）
3. `vx.toml` 的 `[tools]` 节
4. 最新兼容版本

### 简单与详细配置

每个 `[tools]` 条目支持两种形式：

```toml
# 简单：仅版本字符串
node = "20"

# 详细：带版本和选项的表
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]
install_env = { NODE_OPTIONS = "--max-old-space-size=4096" }
```

同一文件中可以为不同运行时混用两种形式。

### 脚本语法

脚本同样支持简单和详细形式：

```toml
[scripts]
# 简单：仅命令字符串
test = "cargo test --workspace"

# 详细：带命令和选项的表
[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]
```

---

## 配方

### Node.js 项目

```toml
min_version = "0.6.0"

[project]
name = "my-web-app"

[tools]
node = "20"

[env]
NODE_ENV = "development"

[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"
lint = "npm run lint"

[settings]
auto_install = true

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"
```

### Python 项目

```toml
[project]
name = "ml-pipeline"

[tools]
uv = "latest"

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt"]
dev = ["pytest", "ruff", "mypy"]

[scripts]
test = "pytest"
lint = "ruff check ."
format = "ruff format ."
typecheck = "mypy src/"

[env]
PYTHONPATH = "src"
```

### Go 项目

```toml
[project]
name = "api-server"

[tools]
go = "1.22"

[scripts]
build = "go build -o bin/server ./cmd/server"
test = "go test ./..."
lint = "golangci-lint run"
run = "go run ./cmd/server"

[dependencies.go]
proxy = "https://goproxy.cn,direct"
private = "github.com/myorg/*"
```

### Rust 项目

```toml
[project]
name = "my-cli-tool"

[tools]
rustup = "latest"
just = "latest"

[scripts]
build = "cargo build"
build-release = "cargo build --release"
test = "cargo test --workspace"
lint = "cargo clippy --workspace --all-targets -- -D warnings"
fmt = "cargo fmt --all"
fmt-check = "cargo fmt --all -- --check"
doc = "cargo doc --open"
```

### 全栈项目

```toml
min_version = "0.6.0"

[project]
name = "fullstack-app"
description = "React + Python API"

[tools]
node = "20"
uv = "latest"

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[env]
NODE_ENV = "development"
API_URL = "http://localhost:8000"

[env.required]
DATABASE_URL = "PostgreSQL 连接字符串"

[scripts]
dev = "npm run dev"
api = "uvicorn main:app --reload"
test = "pytest && npm test"
build = "npm run build"

[scripts.start]
command = "npm run start"
description = "启动生产服务器"
depends = ["build"]

[services.postgres]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"

[dependencies.node]
package_manager = "pnpm"

[hooks]
post_setup = ["vx run db:migrate"]
enter = "vx sync --check"
```

### CI/CD 集成

```toml
[tools]
node = "20"
uv = "latest"

[setup]
pipeline = ["install_tools", "export_paths", "post_setup"]

[setup.hooks.install_tools]
parallel = true

[setup.hooks.export_paths]
ci_only = true

[setup.ci]
enabled = true        # 自动检测 GitHub Actions、GitLab CI 等

[scripts]
ci-test = "npm test && pytest"
ci-build = "npm run build"
ci-lint = "npm run lint && ruff check ."
```

在 GitHub Actions 中使用：

```yaml
- name: 设置工具
  run: |
    curl -fsSL https://get.vx.dev | sh
    vx setup
```

### 平台特定运行时

```toml
[tools]
node = "20"
go = "1.22"

# PowerShell 仅在 Windows
[tools.pwsh]
version = "7.4.13"
os = ["windows"]

# MSVC 仅在 Windows，带组件
[tools.msvc]
version = "14.42"
os = ["windows"]
components = ["spectre", "mfc", "atl"]
```

### Monorepo 共享配置

```toml
[project]
name = "my-monorepo"

[tools]
node = "20"
uv = "latest"
go = "1.22"
just = "latest"

[scripts]
# 委托给 just 进行 monorepo 任务编排
just = "just {{args}}"
build-all = "just build-all"
test-all = "just test-all"
lint-all = "just lint-all"

[settings]
parallel_install = true

[settings.experimental]
monorepo = true
workspaces = true
```

---

## 脚本模式

### 参数转发

使用 `{{args}}` 传递 CLI 参数：

```toml
[scripts]
test = "cargo test {{args}}"
# 使用: vx run test -- --nocapture -p vx-cli
# 展开为: cargo test --nocapture -p vx-cli
```

### 包执行

在脚本中直接引用生态系统包：

```toml
[scripts]
tox = "uvx:tox {{args}}"              # Python: 通过 uvx 运行 tox
create-app = "npm:create-react-app"    # Node.js: 通过 npx 运行 create-react-app
```

### 脚本依赖 (DAG)

脚本可以声明依赖，按拓扑顺序先执行：

```toml
[scripts]
lint = "cargo clippy"
test = "cargo test"
build = "cargo build --release"

[scripts.ci]
command = "echo '所有检查通过！'"
description = "运行完整 CI 管道"
depends = ["lint", "test", "build"]    # 运行 lint → test → build → ci
```

### 每脚本环境变量

每个脚本可以有自己的环境变量：

```toml
[scripts.dev]
command = "npm run dev"
env = { NODE_ENV = "development", DEBUG = "*" }

[scripts.prod]
command = "npm run start"
env = { NODE_ENV = "production" }
```

---

## 环境变量模式

### 分层配置

```toml
# 所有脚本可用的静态值
[env]
APP_NAME = "my-app"
LOG_FORMAT = "json"

# 必需 — 缺失时 vx 会警告
[env.required]
DATABASE_URL = "PostgreSQL 连接字符串"
REDIS_URL = "Redis 连接字符串"

# 可选 — 记录但不强制
[env.optional]
SENTRY_DSN = "Sentry 错误追踪 DSN"
CACHE_TTL = "缓存 TTL（秒，默认: 3600）"
```

### 密钥管理

```toml
[env.secrets]
provider = "1password"
items = ["DATABASE_URL", "API_KEY", "STRIPE_SECRET"]
```

支持的提供者：
- `auto` — 自动检测可用的提供者
- `1password` — 1Password CLI
- `vault` — HashiCorp Vault
- `aws-secrets` — AWS Secrets Manager

---

## 钩子模式

### Setup 自动化

```toml
[hooks]
pre_setup = "echo '检查前置条件...'"
post_setup = [
    "vx run db:migrate",
    "vx run seed",
    "echo '设置完成！'"
]
```

### Git 工作流

```toml
[hooks]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"
```

### 自定义部署钩子

```toml
[hooks.custom]
deploy-staging = "vx run build && kubectl apply -f k8s/staging/"
deploy-prod = "vx run build && kubectl apply -f k8s/production/"
```

---

## 验证与故障排除

### 验证配置

```bash
vx config validate
```

常见验证错误：

| 错误 | 原因 | 修复 |
|------|------|------|
| TOML 语法无效 | TOML 格式错误 | 检查引号、括号、逗号 |
| 未知运行时 | `[tools]` 中有未识别的名称 | 运行 `vx list` 查看可用运行时 |
| 无效版本 | 版本字符串格式错误 | 使用 `"20"`、`"20.10.0"`、`"latest"` 等 |
| 循环依赖 | 脚本 `depends` 形成了环 | 移除 `depends` 中的循环 |
| 缺少 image/command | 服务既没有 image 也没有 command | 为每个服务添加 `image` 或 `command` |

### 检查项目状态

```bash
# 验证运行时是否与 vx.toml 匹配
vx sync --check

# 验证锁文件是否最新
vx lock --check
```

---

## 最佳实践

1. **生产环境固定版本** — 使用精确版本（`"20.18.1"`）以确保可复现性
2. **开发环境使用部分版本** — `"20"` 自动追踪最新的次要/补丁版本
3. **始终设置 `min_version`** — 防止团队成员使用不同 vx 版本时产生困惑
4. **使用 `[env.required]`** — 记录新团队成员需要的密钥/配置
5. **定义脚本** — 通过 `vx run --list` 使项目任务可发现
6. **CI 使用 `depends`** — 基于 DAG 的脚本依赖确保正确的执行顺序
7. **使用 `os` 平台过滤** — 避免在 CI 上安装不相关的运行时
8. **使用 `{{args}}` 转发** — 保持脚本灵活而无需重复定义
9. **将 `vx.toml` 提交到 git** — 与团队共享环境配置
10. **生成 `vx.lock`** — 使用 `vx lock` 实现完全可复现的构建

---

## 另请参阅

- [vx.toml 参考](/zh/config/vx-toml) — 完整的逐字段参考
- [配置指南](/zh/guide/configuration) — 入门概述
- [命令语法规则](/zh/guide/command-syntax-rules) — 规范 CLI 形式
