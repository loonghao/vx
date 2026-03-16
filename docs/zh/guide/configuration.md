# 配置

vx 使用简单的基于 TOML 的配置系统，分为两个级别：

1. **项目配置** (`vx.toml`) — 每个项目的设置
2. **全局配置** (`~/.config/vx/config.toml`) — 用户级默认设置

## 项目配置 (vx.toml)

在项目根目录创建 `vx.toml` 文件：

```toml
min_version = "0.6.0"

[project]
name = "my-project"
description = "一个示例项目"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[python]
version = "3.11"
venv = ".venv"
package_manager = "uv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["pytest", "black"]

[env]
NODE_ENV = "development"

[env.required]
API_KEY = "你的 API 密钥"

[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

[scripts.start]
command = "python main.py"
description = "启动应用程序"
args = ["--port", "8080"]
env = { DEBUG = "true" }
depends = ["build"]

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"

[hooks]
post_setup = "vx run db:migrate"
pre_commit = "vx run lint"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
```

## 配置节说明

### `[project]`

项目元数据：

```toml
[project]
name = "my-project"
description = "项目描述"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"
```

### `[tools]`

要使用的运行时版本。支持简单字符串或详细配置：

```toml
[tools]
node = "20"          # 主版本号 — 最新 20.x.x
uv = "latest"        # 最新稳定版
go = "1.21.5"        # 精确版本
rustup = "latest"    # Rust 工具链管理器

# 带平台过滤的详细配置
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]

# 仅 Windows 的运行时
[tools.pwsh]
version = "7.4.13"
os = ["windows"]
```

> **Rust 说明**: 在 `[tools]` 中配置 `rustup`，然后在工作流中使用 `vx cargo` / `vx rustc`。`rustup` 的版本号是工具链管理器本身的版本，不是 Rust 编译器版本。

### `[python]`

Python 环境配置：

```toml
[python]
version = "3.11"
venv = ".venv"
package_manager = "uv"  # uv | pip | poetry

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]
```

### `[env]`

环境变量，支持必需/可选声明：

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "必需变量的描述"
DATABASE_URL = "数据库连接字符串"

[env.optional]
CACHE_DIR = "可选的缓存目录"

[env.secrets]
provider = "auto"  # auto | 1password | vault | aws-secrets
items = ["DATABASE_URL", "API_KEY"]
```

### `[scripts]`

可运行的脚本，通过 `vx run <名称>` 调用：

```toml
[scripts]
# 简单命令
dev = "npm run dev"
test = "pytest"

# 参数化脚本 — {{args}} 转发 CLI 参数
test-pkgs = "cargo test {{args}}"     # vx run test-pkgs -- -p vx-cli

# 包执行语法
tox = "uvx:tox {{args}}"             # 通过 uvx 运行 tox

# 带选项的复杂脚本
[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { PORT = "8080" }
depends = ["build"]  # 先运行 build（DAG 顺序）
```

### `[settings]`

行为设置：

```toml
[settings]
auto_install = true       # 自动安装缺失的运行时
parallel_install = true   # 并行安装运行时
cache_duration = "7d"     # 版本列表缓存时长
shell = "auto"            # Shell (auto, bash, zsh, fish, pwsh)
log_level = "info"        # 日志级别
isolation = true          # 隔离 vx dev 环境
passenv = ["SSH_*"]       # 透传环境变量（glob 模式）

[settings.experimental]
monorepo = false
workspaces = false
```

### `[hooks]` <Badge type="tip" text="v0.6.0+" />

用于自动化的生命周期钩子：

```toml
[hooks]
pre_setup = "echo '开始设置...'"
post_setup = ["vx run db:migrate", "vx run seed"]
pre_commit = "vx run lint && vx run test"
enter = "vx sync --check"

[hooks.custom]
deploy = "vx run build && vx run deploy"
```

可用钩子：

- `pre_setup` — `vx setup` 之前
- `post_setup` — `vx setup` 之后
- `pre_commit` — Git commit 之前
- `enter` — 进入项目目录时

### `[setup]` <Badge type="tip" text="v0.6.0+" />

Setup 流水线及 CI 集成：

```toml
[setup]
pipeline = ["pre_setup", "install_tools", "export_paths", "post_setup"]

[setup.ci]
enabled = true
provider = "github"   # 从环境自动检测
```

### `[services]` <Badge type="tip" text="v0.6.0+" />

本地开发服务（Podman 管理的容器 + 本地命令）：

```toml
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
```

### `[dependencies]` <Badge type="tip" text="v0.6.0+" />

智能依赖管理：

```toml
[dependencies]
lockfile = true
audit = true
auto_update = "minor"

[dependencies.node]
package_manager = "pnpm"
registry = "https://registry.npmmirror.com"

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"

[dependencies.go]
proxy = "https://goproxy.cn,direct"
```

## 全局配置

位于 `~/.config/vx/config.toml`（Windows: `%APPDATA%\vx\config.toml`）：

```toml
[defaults]
auto_install = true
parallel_install = true
cache_duration = "7d"

[tools]
# 运行时默认版本
node = "lts"
python = "3.11"
```

### 管理全局配置

```bash
# 显示当前配置
vx config show

# 设置值
vx config set defaults.auto_install true

# 获取值
vx config get defaults.auto_install

# 重置为默认值
vx config reset

# 编辑配置文件
vx config edit

# 验证配置
vx config validate
```

## 环境变量

vx 支持以下环境变量：

| 变量 | 描述 |
|------|------|
| `VX_HOME` | 覆盖 vx 数据目录 |
| `VX_ENV` | 当前环境名称 |
| `VX_AUTO_INSTALL` | 启用/禁用自动安装 |
| `VX_VERBOSE` | 启用详细输出 |
| `VX_DEBUG` | 启用调试输出 |

## 配置优先级

配置按以下顺序解析（后面的覆盖前面的）：

1. 内置默认值
2. 全局配置 (`~/.config/vx/config.toml`)
3. 项目配置 (`vx.toml`)
4. 环境变量
5. 命令行标志

## 初始化项目

使用 `vx init` 交互式创建配置：

```bash
# 交互模式
vx init -i

# 使用模板
vx init --template nodejs
vx init --template python
vx init --template fullstack

# 指定运行时
vx init --tools node,uv,go
```

## 从旧版本迁移

如果你有旧版的 `vx.toml`，可以迁移到新格式：

```bash
# 检查兼容性
vx config check

# 自动迁移到 v2 格式
vx config migrate --to v2

# 迁移后验证
vx config validate
```

参见 [迁移指南](/zh/guide/migration) 了解详细说明。

## 下一步

- [vx.toml 参考](/zh/config/vx-toml) — 完整配置参考
- [vx.toml 语法指南](/zh/guide/vx-toml-syntax) — 语法模式和最佳实践
- [全局配置](/zh/config/global) — 全局配置参考
- [命令语法规则](/zh/guide/command-syntax-rules) — 规范命令形式
