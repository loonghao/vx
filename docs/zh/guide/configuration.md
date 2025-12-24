# 配置

vx 使用简单的基于 TOML 的配置系统，分为两个级别：

1. **项目配置** (`.vx.toml`) - 每个项目的设置
2. **全局配置** (`~/.config/vx/config.toml`) - 用户级默认设置

## 项目配置 (.vx.toml)

在项目根目录创建 `.vx.toml` 文件：

```toml
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

[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
```

## 配置节说明

### [project]

项目元数据：

```toml
[project]
name = "my-project"
description = "项目描述"
version = "1.0.0"
```

### [tools]

要使用的工具版本：

```toml
[tools]
node = "20"          # 主版本号
uv = "latest"        # 最新稳定版
go = "1.21.5"        # 精确版本
rust = "stable"      # 频道
```

### [python]

Python 环境配置：

```toml
[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt", "requirements-dev.txt"]
packages = ["pytest", "black", "ruff"]
git = ["https://github.com/user/repo.git"]
dev = ["pytest", "mypy"]
```

### [env]

环境变量：

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"

[env.required]
API_KEY = "必需变量的描述"

[env.optional]
CACHE_DIR = "可选的缓存目录"
```

### [scripts]

可运行的脚本：

```toml
[scripts]
# 简单命令
dev = "npm run dev"
test = "pytest"

# 带选项的复杂脚本
[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { PORT = "8080" }
```

### [settings]

行为设置：

```toml
[settings]
auto_install = true       # 自动安装缺失的工具
parallel_install = true   # 并行安装工具
cache_duration = "7d"     # 缓存持续时间
```

## 全局配置

位于 `~/.config/vx/config.toml`：

```toml
[defaults]
auto_install = true
parallel_install = true
cache_duration = "7d"

[tools]
# 工具的默认版本
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
3. 项目配置 (`.vx.toml`)
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

# 指定工具
vx init --tools node,uv,go
```

## 下一步

- [.vx.toml 参考](/zh/config/vx-toml) - 完整配置参考
- [环境变量](/zh/config/env-vars) - 所有环境变量
- [项目环境](/zh/guide/project-environments) - 使用项目环境
