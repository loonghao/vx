# vx.toml 参考

`vx.toml` 是项目配置文件，定义项目的工具需求和脚本。

## 完整示例

```toml
[project]
name = "my-project"
description = "项目描述"
version = "1.0.0"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["pytest", "black"]

[env]
NODE_ENV = "development"

[env.required]
API_KEY = "API 密钥"

[scripts]
dev = "npm run dev"
test = "pytest"

[settings]
auto_install = true
```

## 配置节

### [project]

项目元数据。

| 字段 | 类型 | 描述 |
|------|------|------|
| `name` | string | 项目名称 |
| `description` | string | 项目描述 |
| `version` | string | 项目版本 |

### [tools]

工具版本配置。

```toml
[tools]
node = "20"          # 主版本
uv = "latest"        # 最新版
go = "1.21.5"        # 精确版本
rust = "stable"      # 频道
```

### [python]

Python 环境配置。

| 字段 | 类型 | 描述 |
|------|------|------|
| `version` | string | Python 版本 |
| `venv` | string | 虚拟环境路径 |

### [python.dependencies]

Python 依赖配置。

| 字段 | 类型 | 描述 |
|------|------|------|
| `requirements` | array | requirements 文件列表 |
| `packages` | array | 要安装的包 |
| `git` | array | Git 仓库依赖 |
| `dev` | array | 开发依赖 |

### [env]

环境变量。

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"
```

### [env.required]

必需的环境变量（带描述）。

```toml
[env.required]
API_KEY = "服务的 API 密钥"
```

### [env.optional]

可选的环境变量。

```toml
[env.optional]
CACHE_DIR = "缓存目录"
```

### [scripts]

可运行的脚本，支持简单命令和详细配置（含 DAG 依赖）。

```toml
[scripts]
dev = "npm run dev"
test = "pytest"

[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--port", "8080"]
env = { DEBUG = "true" }
cwd = "src"
depends = ["build"]

[scripts.ci]
command = "echo '所有检查通过'"
description = "运行 CI 管道"
depends = ["lint", "test", "build"]
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `command` | string | 要执行的命令 |
| `description` | string | 人类可读的描述 |
| `args` | string[] | 默认参数 |
| `cwd` | string | 工作目录（相对于项目根目录） |
| `env` | table | 脚本特定环境变量 |
| `depends` | string[] | 先运行的脚本（DAG 依赖，拓扑排序执行） |

### [settings]

行为设置。

| 字段 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `auto_install` | bool | true | 自动安装缺失工具 |
| `parallel_install` | bool | true | 并行安装 |
| `cache_duration` | string | "7d" | 缓存持续时间 |
