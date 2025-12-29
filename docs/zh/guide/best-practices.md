# 最佳实践

本指南介绍配置 vx 项目的推荐模式和最佳实践。

## 项目设置

### 始终指定 min_version

确保配置与预期的 vx 版本兼容：

```toml
min_version = "0.6.0"

[project]
name = "my-project"
```

### 生产环境使用精确版本

为了可重现性，在生产项目中使用精确版本：

```toml
# 开发环境 - 灵活
[tools]
node = "20"        # 最新 20.x
uv = "latest"      # 最新稳定版

# 生产环境 - 固定
[tools]
node = "20.10.0"   # 精确版本
uv = "0.5.14"      # 精确版本
```

### 记录项目信息

添加元数据以提高可发现性：

```toml
[project]
name = "my-fullstack-app"
description = "AI 驱动的全栈应用"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"
```

## 工具配置

### 复杂工具使用详细配置

当工具需要安装后步骤或操作系统限制时：

```toml
[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin"]  # Windows 上跳过

[tools.rust]
version = "stable"
postinstall = "rustup component add clippy rustfmt"
```

### 按生态系统分组工具

按生态系统组织工具：

```toml
# 前端
[tools]
node = "20"
bun = "latest"

# 后端
[python]
version = "3.12"
package_manager = "uv"

# 基础设施
[tools]
terraform = "1.6"
kubectl = "1.28"
```

## 脚本

### 使用描述性名称

```toml
[scripts]
# 好 - 意图清晰
dev = "npm run dev"
test:unit = "pytest tests/unit"
test:integration = "pytest tests/integration"
build:prod = "npm run build -- --mode production"

# 避免 - 不清晰
run = "npm start"
t = "pytest"
```

### 为复杂脚本添加描述

```toml
[scripts.deploy]
command = "npm run build && aws s3 sync dist/ s3://bucket"
description = "构建并部署到生产 S3 存储桶"

[scripts.db:reset]
command = "prisma migrate reset --force"
description = "重置数据库（警告：会销毁所有数据）"
```

### 使用依赖处理多步骤任务

```toml
[scripts]
lint = "eslint . && prettier --check ."
typecheck = "tsc --noEmit"
test = "vitest run"

[scripts.ci]
command = "echo '所有检查通过！'"
description = "运行所有 CI 检查"
depends = ["lint", "typecheck", "test"]
```

### 避免硬编码路径

```toml
# 不好 - 硬编码路径
[scripts]
build = "cd /home/user/project && npm run build"

# 好 - 相对路径
[scripts]
build = "npm run build"

[scripts.backend]
command = "python main.py"
cwd = "backend"
```

## 环境变量

### 记录必需变量

```toml
[env.required]
DATABASE_URL = "PostgreSQL 连接字符串 (postgres://user:pass@host:5432/db)"
API_KEY = "外部 API 密钥，来自 https://api.example.com/keys"
JWT_SECRET = "JWT 签名密钥（至少 32 个字符）"
```

### 敏感数据使用 Secrets

```toml
[env.secrets]
provider = "auto"  # 自动检测 1password、vault 等
items = ["DATABASE_URL", "API_KEY", "JWT_SECRET"]
```

### 提供合理的默认值

```toml
[env]
NODE_ENV = "development"
LOG_LEVEL = "info"
PORT = "3000"

[env.optional]
DEBUG = "启用调试模式 (true/false)"
CACHE_TTL = "缓存 TTL 秒数（默认：3600）"
```

## 钩子

### 使用 pre_setup 进行验证

```toml
[hooks]
pre_setup = "node -e \"if(process.version.slice(1,3)<'18'){process.exit(1)}\""
```

### 使用 post_setup 进行初始化

```toml
[hooks]
post_setup = [
  "npm install",
  "vx run db:migrate",
  "vx run seed",
  "echo '✓ 设置完成！'"
]
```

### 使用 pre_commit 作为质量门

```toml
[hooks]
pre_commit = "vx run lint && vx run typecheck && vx run test:unit"
```

### 保持钩子快速

```toml
# 好 - 快速检查
[hooks]
pre_commit = "vx run lint:staged"  # 只检查更改的文件

# 避免 - 完整检查太慢
[hooks]
pre_commit = "vx run lint && vx run test:all"  # 提交时太慢
```

## 服务

### 定义健康检查

```toml
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_PASSWORD = "dev" }
healthcheck = "pg_isready -U postgres"

[services.redis]
image = "redis:7-alpine"
ports = ["6379:6379"]
healthcheck = "redis-cli ping"
```

### 使用 depends_on 控制顺序

```toml
[services.api]
command = "npm run dev"
depends_on = ["database", "redis"]
ports = ["3000:3000"]

[services.worker]
command = "npm run worker"
depends_on = ["database", "redis"]
```

### 分离开发和测试服务

```toml
# 开发数据库
[services.database]
image = "postgres:16"
ports = ["5432:5432"]
env = { POSTGRES_DB = "myapp_dev" }

# 测试数据库（不同端口）
[services.database-test]
image = "postgres:16"
ports = ["5433:5432"]
env = { POSTGRES_DB = "myapp_test" }
```

## 依赖管理

### 启用审计

```toml
[dependencies]
audit = true
lockfile = true
```

### 配置包管理器

```toml
[dependencies.node]
package_manager = "pnpm"  # 更快，节省磁盘空间
registry = "https://registry.npmmirror.com"  # 中国镜像

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
```

### 设置许可证约束

```toml
[dependencies.constraints]
"*" = { licenses = ["MIT", "Apache-2.0", "BSD-3-Clause", "ISC"] }
```

## 团队协作

### 将 vx.toml 提交到 Git

```bash
# .gitignore - 不要忽略 vx.toml
# vx.toml  # 不要添加这行！

# 忽略本地覆盖
.vx.local.toml
```

### 使用一致的设置

```toml
[settings]
auto_install = true
parallel_install = true
shell = "auto"  # 让 vx 自动检测 shell
```

### 在 README 中记录设置

```markdown
## 开发设置

1. 安装 vx: `curl -fsSL https://get.vx.dev | bash`
2. 设置项目: `vx setup`
3. 开始开发: `vx run dev`
```

## 性能

### 启用并行安装

```toml
[settings]
parallel_install = true
```

### 使用缓存

```toml
[settings]
cache_duration = "7d"  # 缓存版本查询
```

### 最小化钩子执行时间

```toml
[hooks]
# 快速 - 只检查更改的内容
pre_commit = "lint-staged"

# 慢 - 检查所有内容
# pre_commit = "npm run lint && npm run test"
```

## 安全

### 永远不要提交密钥

```toml
# 好 - 引用环境变量
[env.required]
API_KEY = "API 密钥（在环境中设置）"

# 不好 - 硬编码密钥
[env]
API_KEY = "sk-1234567890abcdef"  # 永远不要这样做
```

### 使用密钥提供者

```toml
[env.secrets]
provider = "1password"  # 或 "vault"、"aws-secrets"
items = ["DATABASE_URL", "API_KEY"]
```

### 启用安全审计

```toml
[dependencies]
audit = true

[dependencies.constraints]
"lodash" = ">=4.17.21"  # 已知安全修复
```

## 调试

### 使用详细模式

```bash
vx setup --verbose
vx run dev --verbose
```

### 验证配置

```bash
vx config validate
vx config show
```

### 不做更改的测试

```bash
vx setup --dry-run
```

## 常见模式

### Monorepo 设置

```toml
[project]
name = "monorepo"

[settings.experimental]
monorepo = true
workspaces = true

[scripts]
dev:api = { command = "npm run dev", cwd = "packages/api" }
dev:web = { command = "npm run dev", cwd = "packages/web" }
dev:all = "concurrently 'vx run dev:api' 'vx run dev:web'"
```

### 全栈应用

```toml
[tools]
node = "20"

[python]
version = "3.12"
venv = ".venv"

[services.database]
image = "postgres:16"
ports = ["5432:5432"]

[services.backend]
command = "python -m uvicorn main:app --reload"
depends_on = ["database"]
ports = ["8000:8000"]
cwd = "backend"

[services.frontend]
command = "npm run dev"
ports = ["3000:3000"]
cwd = "frontend"

[scripts]
dev = "vx services up"
```

### CI/CD 集成

```toml
[scripts]
ci:lint = "npm run lint"
ci:test = "npm run test -- --coverage"
ci:build = "npm run build"

[scripts.ci]
command = "echo 'CI 通过'"
depends = ["ci:lint", "ci:test", "ci:build"]
```

## 另请参阅

- [配置参考](/zh/config/vx-toml) - 完整字段参考
- [迁移指南](/zh/guide/migration) - 从旧版本升级
- [CLI 参考](/zh/cli/overview) - 所有可用命令
