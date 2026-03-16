# vx.toml 参考

`vx.toml` 项目配置文件完整参考。

## 概述

`vx.toml` 是 vx 的项目级配置文件。它声明项目需要的运行时、定义脚本、管理环境变量、配置开发服务 —— 全部在一个 TOML 文件中完成。

## 文件位置

将 `vx.toml` 放在项目根目录。vx 会从当前目录向上递归查找此文件。

## 最低版本要求

使用 `min_version` 指定解析此配置所需的最低 vx 版本：

```toml
min_version = "0.6.0"
```

如果已安装的 vx 版本低于 `min_version`，vx 会显示错误并建议升级。

## 完整示例

```toml
min_version = "0.6.0"

[project]
name = "my-fullstack-app"
description = "AI 驱动的全栈应用"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"

[tools]
node = "20"
uv = "latest"
go = "1.22"
rustup = "latest"
just = "latest"

[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]

[tools.pwsh]
version = "7.4.13"
os = ["windows"]

[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]

[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "你的 API 密钥"
DATABASE_URL = "数据库连接字符串"

[env.optional]
CACHE_DIR = "可选的缓存目录"

[env.secrets]
provider = "auto"
items = ["DATABASE_URL", "API_KEY"]

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--host", "0.0.0.0", "--port", "8080"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"

[hooks]
pre_setup = "echo '开始设置...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.app]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]
env_file = ".env.local"

[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
```

---

## 各配置节

### 顶级字段

| 字段 | 类型 | 必需 | 描述 |
|------|------|------|------|
| `min_version` | string | 否 | 所需最低 vx 版本（如 `"0.6.0"`） |

---

### `[project]`

项目元数据。所有字段均为可选。

| 字段 | 类型 | 描述 |
|------|------|------|
| `name` | string | 项目名称 |
| `description` | string | 项目描述 |
| `version` | string | 项目版本 |
| `license` | string | 许可证标识符（如 `MIT`、`Apache-2.0`） |
| `repository` | string | 仓库 URL |

```toml
[project]
name = "my-project"
description = "一个示例项目"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"
```

---

### `[tools]`

运行时版本配置。这是声明项目所需运行时的主要节。

> **术语说明**: vx 使用"运行时 (runtime)"来指代管理的可执行工具（node、go、uv 等）。配置节命名为 `[tools]` 以便用户理解，但内部管理的是运行时。

> **向后兼容**: `[runtimes]` 作为 `[tools]` 的别名被接受。如果两者同时存在，`[tools]` 优先。

#### 简单版本

```toml
[tools]
node = "20"          # 主版本号 — 解析为最新的 20.x.x
uv = "latest"        # 最新稳定版
go = "1.21.5"        # 精确版本
rustup = "latest"    # Rust 工具链管理器
just = "latest"      # Just 命令运行器
```

#### 详细配置

使用表格语法进行高级的运行时配置：

```toml
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]
install_env = { NODE_OPTIONS = "--max-old-space-size=4096" }

[tools.pwsh]
version = "7.4.13"
os = ["windows"]     # 仅在 Windows 上安装

[tools.msvc]
version = "14.42"
os = ["windows"]
components = ["spectre", "mfc", "atl"]
exclude_patterns = ["Microsoft.VisualStudio.Component.VC.Llvm*"]
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `version` | string | 版本说明符（见下表） |
| `postinstall` | string | 安装后执行的命令 |
| `os` | string[] | 限制在特定操作系统上安装（`"windows"`、`"darwin"`、`"linux"`） |
| `install_env` | table | 安装期间设置的环境变量 |
| `components` | string[] | 要安装的可选组件（如 MSVC: `spectre`、`mfc`、`atl`、`asan`、`cli`） |
| `exclude_patterns` | string[] | 安装时要排除的包 ID 模式 |

如果未指定 `os`，运行时会在所有平台上安装。指定后，vx 仅在列出的操作系统上安装。

#### 版本说明符

| 格式 | 示例 | 描述 |
|------|------|------|
| 主版本 | `"20"` | 最新 20.x.x |
| 次版本 | `"20.10"` | 最新 20.10.x |
| 精确版本 | `"20.10.0"` | 精确匹配 |
| 最新版 | `"latest"` | 最新稳定发布版 |
| LTS | `"lts"` | 最新 LTS 版本（运行时特定，如 Node.js） |
| 通道 | `"stable"` | 发布通道（如 Rust: `stable`、`nightly`、`beta`） |

> **Rust 说明**: 在 `[tools]` 中配置 `rustup`，而不是 `rust`。`rustup` 版本是工具链管理器本身的版本，不是 Rust 编译器版本。在脚本中使用 `vx cargo` / `vx rustc`。

---

### `[python]`

Python 特定的环境配置。此节为 Python 项目提供比基本的 `[tools].python` 版本指定更深入的集成。

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `version` | string | — | Python 版本 |
| `venv` | string | `".venv"` | 虚拟环境目录路径 |
| `package_manager` | string | `"uv"` | 包管理器（`uv`、`pip`、`poetry`） |

```toml
[python]
version = "3.12"
venv = ".venv"
package_manager = "uv"
```

#### `[python.dependencies]`

Python 项目依赖。

| 字段 | 类型 | 描述 |
|------|------|------|
| `requirements` | string[] | 要安装的 requirements 文件 |
| `packages` | string[] | 直接安装的包名 |
| `git` | string[] | 从 Git 仓库 URL 安装 |
| `dev` | string[] | 仅开发环境的依赖 |

```toml
[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["requests", "pandas"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "black", "mypy"]
```

---

### `[env]`

环境变量，支持必需/可选声明和密钥管理。

#### 静态变量

直接设置环境变量：

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"
PORT = "3000"
```

#### 必需变量

**必须**设置的变量。如果缺失，vx 会发出警告。值为人类可读的描述。

```toml
[env.required]
API_KEY = "服务的 API 密钥"
DATABASE_URL = "PostgreSQL 连接字符串"
```

#### 可选变量

可选变量及其描述：

```toml
[env.optional]
CACHE_DIR = "可选的缓存目录"
LOG_LEVEL = "日志级别（默认: info）"
```

#### 密钥

从安全存储提供者加载密钥：

```toml
[env.secrets]
provider = "auto"  # auto | 1password | vault | aws-secrets
items = ["DATABASE_URL", "API_KEY"]
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `provider` | string | 密钥提供者（`auto`、`1password`、`vault`、`aws-secrets`） |
| `items` | string[] | 要加载的密钥名称 |

---

### `[scripts]`

通过 `vx run <脚本名>` 调用的可运行脚本。支持简单命令和带 DAG 依赖的详细配置。

#### 简单脚本

```toml
[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
lint = "cargo clippy --workspace"
```

#### 参数化脚本

使用 `{{args}}` 转发额外参数：

```toml
[scripts]
test-pkgs = "cargo test {{args}}"      # vx run test-pkgs -- -p vx-cli
just = "just {{args}}"                  # vx run just -- build
```

#### 脚本中的包执行

直接使用生态系统包执行语法：

```toml
[scripts]
tox = "uvx:tox {{args}}"               # 通过 uvx 运行 tox (Python)
```

#### 详细脚本

```toml
[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--host", "0.0.0.0", "--port", "8080"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]

[scripts.ci]
command = "echo '所有检查通过'"
description = "运行 CI 管道"
depends = ["lint", "test", "build"]     # DAG: 按拓扑顺序执行
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `command` | string | 要执行的命令 |
| `description` | string | 人类可读的描述（通过 `vx run --list` 显示） |
| `args` | string[] | 追加到命令后的默认参数 |
| `cwd` | string | 工作目录（相对于项目根目录） |
| `env` | table | 脚本专用环境变量 |
| `depends` | string[] | 必须先运行的脚本（DAG 拓扑排序） |

---

### `[settings]`

项目内 vx 的行为设置。

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `auto_install` | bool | `true` | 首次使用时自动安装缺失的运行时 |
| `parallel_install` | bool | `true` | 并行安装多个运行时 |
| `cache_duration` | string | `"7d"` | 版本列表缓存时长（如 `"1h"`、`"7d"`、`"30d"`） |
| `shell` | string | `"auto"` | 默认 shell（`auto`、`bash`、`zsh`、`fish`、`pwsh`、`cmd`） |
| `log_level` | string | `"info"` | 日志级别（`trace`、`debug`、`info`、`warn`、`error`） |
| `isolation` | bool | `true` | 在 `vx dev` 中启用环境隔离 |
| `passenv` | string[] | — | 隔离模式下透传的环境变量（支持 glob 模式，如 `"SSH_*"`） |
| `setenv` | table | — | 显式设置的环境变量（覆盖 passenv） |

```toml
[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"
log_level = "info"
isolation = true
passenv = ["SSH_*", "GPG_*", "EDITOR"]
setenv = { TERM = "xterm-256color" }
```

#### 隔离模式

当 `isolation = true`（默认）时，`vx dev` 创建隔离环境，仅 vx 管理的运行时在 `PATH` 中。系统变量会被过滤，只有基本变量被透传：

- **Windows**: `SYSTEMROOT`、`TEMP`、`TMP`、`PATHEXT`、`COMSPEC`、`WINDIR`
- **Unix**: `HOME`、`USER`、`SHELL`、`LANG`、`LC_*`、`TERM`

使用 `passenv` 显式允许额外的变量（支持 glob 模式）。

#### 实验性功能

```toml
[settings.experimental]
monorepo = false       # Monorepo 工作区支持
workspaces = false     # 多工作区支持
```

---

### `[hooks]` <Badge type="tip" text="v0.6.0+" />

用于在特定时间点自动执行任务的生命周期钩子。

| 钩子 | 触发时机 |
|------|---------|
| `pre_setup` | `vx setup` 之前 |
| `post_setup` | `vx setup` 完成后 |
| `pre_commit` | Git commit 之前（需要配置 git hooks） |
| `enter` | 进入项目目录时 |

钩子可以是单条命令字符串或命令数组：

```toml
[hooks]
pre_setup = "echo '开始设置...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test:unit"
enter = "vx sync --check"
```

#### 自定义钩子

定义你自己的钩子，通过 `vx hook <名称>` 触发：

```toml
[hooks.custom]
deploy = "vx run build && vx run deploy"
release = "vx run test && vx run build && gh release create"
```

---

### `[setup]` <Badge type="tip" text="v0.6.0+" />

配置 `vx setup` 流水线，用于可复现的环境引导，包括 CI 集成。

```toml
[setup]
pipeline = ["pre_setup", "install_tools", "export_paths", "post_setup"]

[setup.hooks.install_tools]
enabled = true
parallel = true
force = false

[setup.hooks.export_paths]
enabled = true
ci_only = true
extra_paths = ["/usr/local/bin"]

[setup.ci]
enabled = true              # 自动检测 CI 环境
provider = "github"         # github | gitlab | azure | circleci | jenkins | generic
path_env_file = ""          # 自定义 PATH 导出文件（根据 CI 提供者自动检测）
env_file = ""               # 自定义环境导出文件
```

#### CI 自动检测

vx 通过环境变量自动检测 CI 环境：

| 提供者 | 检测变量 |
|--------|---------|
| GitHub Actions | `GITHUB_ACTIONS` |
| GitLab CI | `GITLAB_CI` |
| Azure Pipelines | `TF_BUILD` |
| CircleCI | `CIRCLECI` |
| Jenkins | `JENKINS_URL` |
| 通用 | `CI=true` |

#### 自定义 Setup 钩子

```toml
[setup.hooks.custom.setup_database]
command = "podman compose up -d postgres"
enabled = true
ci_only = false
continue_on_failure = false
working_dir = "infra"
env = { PGPASSWORD = "dev" }
```

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `command` | string/string[] | — | 要执行的命令 |
| `enabled` | bool | `true` | 此钩子是否激活 |
| `ci_only` | bool | `false` | 仅在 CI 环境中运行 |
| `local_only` | bool | `false` | 仅在本地运行（不在 CI 中） |
| `continue_on_failure` | bool | `false` | 此钩子失败时继续流水线 |
| `working_dir` | string | — | 此钩子的工作目录 |
| `env` | table | — | 此钩子的环境变量 |

---

### `[services]` <Badge type="tip" text="v0.6.0+" />

本地开发的服务定义。容器服务会使用配置里的运行时，默认是 Podman，并通过 `vx services` 命令管理。

```toml
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
volumes = ["./data:/var/lib/postgresql/data"]
healthcheck = "pg_isready"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]

[services.app]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]
env_file = ".env.local"
working_dir = "./frontend"
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `image` | string | 容器镜像（容器服务） |
| `command` | string | 要运行的命令（非容器服务） |
| `ports` | string[] | 端口映射（`"主机:容器"`） |
| `env` | table | 环境变量 |
| `env_file` | string | `.env` 文件路径 |
| `volumes` | string[] | 卷挂载（`"主机:容器"`） |
| `depends_on` | string[] | 必须先启动的服务 |
| `healthcheck` | string | 健康检查命令 |
| `working_dir` | string | 工作目录 |

> 每个服务必须有 `image`（容器）或 `command`（进程）之一，不能同时有两者。

---

### `[dependencies]` <Badge type="tip" text="v0.6.0+" />

按生态系统的智能依赖管理配置。

```toml
[dependencies]
lockfile = true
audit = true
auto_update = "minor"    # none | patch | minor | major
```

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `lockfile` | bool | — | 生成/使用锁文件以确保可复现性 |
| `audit` | bool | — | 对依赖运行安全审计 |
| `auto_update` | string | — | 自动更新策略 |

#### Node.js 依赖

```toml
[dependencies.node]
package_manager = "pnpm"                         # npm | yarn | pnpm | bun
registry = "https://registry.npmmirror.com"      # 自定义注册表
```

#### Python 依赖

```toml
[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
extra_index_urls = ["https://download.pytorch.org/whl/cu121"]
```

#### Go 依赖

```toml
[dependencies.go]
proxy = "https://goproxy.cn,direct"
private = "github.com/myorg/*"
vendor = false
mod_mode = "readonly"     # readonly | vendor | mod
```

#### C++ 依赖

```toml
[dependencies.cpp]
package_manager = "vcpkg"          # conan | vcpkg | cmake
vcpkg_triplet = "x64-windows"     # x64-windows | x64-linux | x64-osx
cmake_generator = "Ninja"
cmake_build_type = "Release"       # Debug | Release | RelWithDebInfo | MinSizeRel
std = "20"                          # C++ 标准: 11, 14, 17, 20, 23
compiler = "msvc"                   # gcc | clang | msvc
```

#### 依赖约束

```toml
[dependencies.constraints]
"lodash" = ">=4.17.21"                                   # 版本约束
"*" = { licenses = ["MIT", "Apache-2.0", "BSD-3-Clause"] } # 许可证策略
```

---

## 计划中的配置节

以下配置节已设计并有 Rust 结构体定义，但仍在开发中：

| 配置节 | 阶段 | 描述 |
|--------|------|------|
| `[ai]` | 阶段 2 | AI 代码生成集成 |
| `[docs]` | 阶段 2 | 文档自动生成 |
| `[team]` | 阶段 3 | 团队协作规则（代码所有者、审查、规范） |
| `[remote]` | 阶段 3 | 远程开发环境（Codespaces、Gitpod、DevContainer） |
| `[security]` | 阶段 4 | 安全扫描（审计、密钥检测、SAST） |
| `[test]` | 阶段 4 | 测试流水线配置（覆盖率、测试环境） |
| `[telemetry]` | 阶段 4 | 性能监控和追踪（OTLP） |
| `[container]` | 阶段 5 | 容器部署（Dockerfile 生成、注册表、多阶段构建） |
| `[versioning]` | 阶段 5 | 版本控制策略（语义版本号、日历版本号） |

参见 [RFC 0001: vx.toml v2 增强](../rfcs/0001-vx-toml-v2-enhancement.md) 了解完整路线图。

---

## 验证

vx 在加载时会验证 `vx.toml`。验证检查包括：

- TOML 语法正确性
- `min_version` 格式
- 运行时名称有效性（字母数字 + 连字符）
- 脚本名称有效性
- 版本说明符格式
- 服务定义（必须有 `image` 或 `command`）
- 端口映射格式（`"主机:容器"`）
- 循环脚本依赖检测

手动运行验证：

```bash
vx config validate
```

---

## 提示

1. **提交到 git** — 与团队共享 `vx.toml` 以确保一致的环境
2. **使用精确版本** — 为生产环境可复现性固定精确版本
3. **使用 `os` 过滤** — 限制平台特定的运行时（如 `pwsh` 仅在 Windows）
4. **定义脚本** — 通过 `vx run --list` 使常用任务可发现
5. **使用 `depends`** — 脚本依赖确保正确的执行顺序
6. **使用 `{{args}}`** — 将 CLI 参数转发到脚本以提高灵活性
7. **记录环境变量** — 使用 `[env.required]` 描述帮助新贡献者
8. **使用钩子** — 自动化 `vx setup` 和 git 工作流

---

## 另请参阅

- [配置指南](../guide/configuration) — 配置入门
- [vx.toml 语法指南](../guide/vx-toml-syntax) — 语法模式和最佳实践
- [全局配置](./global) — 用户级默认设置
- [命令语法规则](../guide/command-syntax-rules) — 规范命令形式
