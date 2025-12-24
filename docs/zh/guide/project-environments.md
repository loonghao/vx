# 项目环境

对于团队项目，vx 通过 `.vx.toml` 文件支持项目特定的工具配置。

## 创建项目配置

### 交互模式

```bash
vx init -i
```

这会引导你设置：

- 项目元数据
- 工具版本
- 脚本
- 环境变量

### 使用模板

```bash
# 列出可用模板
vx init --list-templates

# 使用模板
vx init --template nodejs
vx init --template python
vx init --template fullstack
```

### 手动创建

创建 `.vx.toml` 文件：

```toml
[project]
name = "my-project"
description = "一个示例项目"

[tools]
node = "20"
uv = "latest"

[scripts]
dev = "npm run dev"
test = "npm test"
```

## 设置环境

创建 `.vx.toml` 后，运行：

```bash
vx setup
```

这将：

1. 安装所有必需的工具
2. 设置 Python 虚拟环境（如果配置了）
3. 安装依赖
4. 验证环境变量

### 设置选项

```bash
# 试运行 - 显示将要执行的操作
vx setup --dry-run

# 强制重新安装所有工具
vx setup --force

# 详细输出
vx setup --verbose

# 顺序安装（无并行）
vx setup --no-parallel
```

## 运行脚本

在 `.vx.toml` 中定义脚本：

```toml
[scripts]
dev = "npm run dev"
test = "pytest"
build = "go build -o app"
lint = "npm run lint && uvx ruff check ."

[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--host", "0.0.0.0", "--port", "8080"]
env = { DEBUG = "true" }
cwd = "src"
```

运行脚本：

```bash
vx run dev
vx run test
vx run start
```

传递额外参数：

```bash
vx run test -- -v --coverage
```

## 开发环境

进入包含所有工具的开发 shell：

```bash
vx dev
```

这将：

1. 激活项目环境
2. 设置包含项目工具的 PATH
3. 激活 Python venv（如果配置了）
4. 设置环境变量
5. 启动新的 shell

### 在开发环境中运行命令

```bash
# 运行单个命令
vx dev -c "npm run build"

# 指定 shell
vx dev --shell zsh
```

## Python 项目

对于 Python 项目，配置虚拟环境：

```toml
[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["requirements.txt"]
packages = ["pytest", "black", "ruff"]
git = [
    "https://github.com/user/repo.git",
]
dev = ["pytest", "mypy"]
```

运行 `vx setup` 时：

1. 使用 Python 3.11 创建 `.venv`
2. 从 `requirements.txt` 安装
3. 安装列出的包
4. 安装 git 依赖

## 环境变量

### 静态变量

```toml
[env]
NODE_ENV = "development"
DEBUG = "true"
```

### 必需变量

```toml
[env.required]
API_KEY = "服务的 API 密钥"
DATABASE_URL = "数据库连接字符串"
```

必需变量必须在运行脚本前设置。如果缺失，vx 会发出警告。

### 可选变量

```toml
[env.optional]
CACHE_DIR = "可选的缓存目录"
LOG_LEVEL = "日志级别（默认：info）"
```

## 管理工具

### 添加工具

```bash
vx add node
vx add node --version 18
```

### 删除工具

```bash
vx rm-tool node
```

### 更新工具

编辑 `.vx.toml` 并运行：

```bash
vx setup
```

## 与团队同步

克隆包含 `.vx.toml` 的项目时：

```bash
git clone https://github.com/team/project
cd project
vx setup  # 安装所有必需的工具
```

检查环境是否同步：

```bash
vx sync --check
```

## 最佳实践

::: tip 提交 .vx.toml
通过将 `.vx.toml` 提交到版本控制来与团队共享工具版本。
:::

::: tip 使用特定版本
为了可重现性，避免使用 "latest"。使用特定版本如 `node = "20.10"`。
:::

::: tip 记录必需变量
使用带描述的 `[env.required]` 帮助团队成员正确设置。
:::

## 示例：全栈项目

```toml
[project]
name = "fullstack-app"
description = "一个全栈 Web 应用"

[tools]
node = "20"
uv = "latest"
go = "1.21"

[python]
version = "3.11"
venv = ".venv"

[python.dependencies]
requirements = ["backend/requirements.txt"]

[env]
NODE_ENV = "development"

[env.required]
DATABASE_URL = "PostgreSQL 连接字符串"
JWT_SECRET = "JWT 令牌密钥"

[scripts]
# 前端
frontend = "cd frontend && npm run dev"
frontend-build = "cd frontend && npm run build"

# 后端
backend = "cd backend && python main.py"
migrate = "cd backend && python manage.py migrate"

# 全栈
dev = "concurrently 'vx run frontend' 'vx run backend'"
test = "vx run test-frontend && vx run test-backend"
test-frontend = "cd frontend && npm test"
test-backend = "cd backend && pytest"
```

## 下一步

- [环境管理](/zh/guide/environment-management) - 管理多个环境
- [.vx.toml 参考](/zh/config/vx-toml) - 完整配置参考
