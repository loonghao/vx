# RFC 0001: vx.toml v2 配置增强方案

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2025-12-26
> **目标版本**: v0.6.0

## 摘要

本 RFC 提出对 `vx.toml` 配置格式进行全面增强，以适应 2025 年 AI 时代全栈开发的需求。增强内容包括：AI 代码生成集成、智能依赖管理、自动化测试流水线、团队协作规则、远程开发环境同步、性能监控、安全扫描、文档自动生成、容器化部署和版本控制策略。

## 动机

### 当前状态分析

现有 `vx.toml` 支持的配置项：

```toml
[project]        # 项目元数据
[tools]          # 工具版本
[python]         # Python 环境
[env]            # 环境变量
[scripts]        # 脚本定义
[settings]       # 行为设置
```

### 行业趋势对比

| 工具 | 特点 | vx 可借鉴 |
|------|------|----------|
| **mise** | 600+ 运行时、任务系统、hooks | 任务依赖、hooks 机制 |
| **devbox** | Nix 隔离、process-compose | 服务编排、隔离环境 |
| **devenv** | 声明式配置、服务定义 | 服务声明、预配置模板 |
| **Cursor/Copilot** | AI 规则文件 | AI 集成配置 |

### 2025 年开发需求

1. **AI 辅助开发** - 需要配置 AI 代码生成规则和上下文
2. **多云部署** - 需要统一的容器化和部署配置
3. **安全合规** - 需要内置安全扫描和依赖审计
4. **团队协作** - 需要共享的开发规范和工具配置
5. **远程开发** - 需要支持 Codespaces/GitPod 等远程环境

## 设计方案

### 完整配置结构预览

```toml
# vx.toml v2 完整示例
min_version = "0.6.0"

[project]
name = "my-fullstack-app"
description = "AI-powered fullstack application"
version = "1.0.0"
license = "MIT"
repository = "https://github.com/org/repo"

# ============================================
# 1. 工具版本管理 (增强)
# ============================================
[tools]
node = "20"
uv = "latest"
go = "1.22"
rust = "stable"

[tools.node]
version = "20"
postinstall = "corepack enable"
os = ["linux", "darwin", "windows"]

[tools.python]
version = "3.12"
venv = ".venv"
package_manager = "uv"  # uv | pip | poetry

# ============================================
# 2. AI 代码生成集成 (新增)
# ============================================
[ai]
enabled = true
provider = "auto"  # auto | openai | anthropic | local

[ai.context]
# 项目上下文文件，AI 工具会自动读取
include = [
  "docs/architecture.md",
  "docs/api-spec.md",
  "vx.toml",
]
exclude = ["node_modules", "dist", ".git"]

[ai.rules]
# 代码生成规则
style_guide = "docs/style-guide.md"
naming_convention = "camelCase"
max_file_lines = 500
prefer_composition = true

[ai.prompts]
# 预定义提示模板
code_review = "Review this code for security, performance, and best practices"
refactor = "Refactor this code to improve readability and maintainability"
test = "Generate comprehensive unit tests for this code"

# ============================================
# 3. 智能依赖管理 (增强)
# ============================================
[dependencies]
lockfile = true
audit = true
auto_update = "minor"  # none | patch | minor | major

[dependencies.node]
package_manager = "pnpm"  # npm | yarn | pnpm | bun
registry = "https://registry.npmmirror.com"

[dependencies.python]
index_url = "https://pypi.tuna.tsinghua.edu.cn/simple"
extra_index_urls = []

[dependencies.constraints]
# 依赖约束规则
"lodash" = ">=4.17.21"  # 安全版本
"*" = { licenses = ["MIT", "Apache-2.0", "BSD-3-Clause"] }

# ============================================
# 4. 自动化测试流水线 (新增)
# ============================================
[test]
framework = "auto"  # auto | jest | pytest | go test
parallel = true
coverage_threshold = 80

[test.unit]
command = "npm test"
watch = "npm test -- --watch"
pattern = "**/*.test.{ts,js}"

[test.integration]
command = "npm run test:integration"
requires = ["database", "redis"]
timeout = "5m"

[test.e2e]
command = "playwright test"
browser = ["chromium", "firefox"]
base_url = "http://localhost:3000"

[test.hooks]
pre = ["lint", "typecheck"]
post = ["coverage-report"]

# ============================================
# 5. 团队协作规则 (新增)
# ============================================
[team]
# 团队配置继承
extends = "https://github.com/org/vx-presets/base.toml"

[team.code_owners]
"src/api/**" = ["@backend-team"]
"src/ui/**" = ["@frontend-team"]
"*.md" = ["@docs-team"]

[team.review]
required_approvals = 2
auto_assign = true
dismiss_stale = true

[team.conventions]
commit_format = "conventional"  # conventional | angular | custom
branch_pattern = "^(feature|fix|docs|refactor)/[a-z0-9-]+$"
pr_template = ".github/pull_request_template.md"

# ============================================
# 6. 远程开发环境同步 (新增)
# ============================================
[remote]
enabled = true

[remote.codespaces]
machine = "standardLinux32gb"
dotfiles = "https://github.com/user/dotfiles"

[remote.gitpod]
image = "gitpod/workspace-full"
tasks = [
  { init = "vx setup", command = "vx run dev" }
]

[remote.devcontainer]
image = "mcr.microsoft.com/devcontainers/base:ubuntu"
features = [
  "ghcr.io/devcontainers/features/node:1",
  "ghcr.io/devcontainers/features/python:1",
]
postCreateCommand = "vx setup"

# ============================================
# 7. 性能监控埋点 (新增)
# ============================================
[telemetry]
enabled = false  # 默认关闭，尊重隐私
anonymous = true

[telemetry.metrics]
# 收集的指标
build_time = true
test_duration = true
install_time = true

[telemetry.export]
# 导出配置
format = "otlp"  # otlp | prometheus | json
endpoint = "http://localhost:4317"

# ============================================
# 8. 安全扫描规则 (新增)
# ============================================
[security]
enabled = true
fail_on = "high"  # low | medium | high | critical

[security.scan]
# 扫描配置
dependencies = true
secrets = true
sast = true
container = true

[security.ignore]
# 忽略的漏洞 (需要说明原因)
"CVE-2023-XXXX" = "False positive, not applicable"

[security.policies]
# 安全策略
no_root = true
read_only_fs = false
allowed_hosts = ["*.npmjs.org", "*.pypi.org"]

# ============================================
# 9. 文档自动生成 (新增)
# ============================================
[docs]
enabled = true
output = "docs/generated"

[docs.api]
# API 文档
generator = "typedoc"  # typedoc | sphinx | godoc | rustdoc
source = "src"
output = "docs/api"

[docs.changelog]
# 变更日志
generator = "conventional-changelog"
preset = "angular"

[docs.readme]
# README 自动更新
badges = ["ci", "coverage", "version", "license"]
sections = ["installation", "usage", "api", "contributing"]

# ============================================
# 10. 容器化部署配置 (新增)
# ============================================
[container]
enabled = true
runtime = "docker"  # docker | podman | containerd

[container.build]
dockerfile = "Dockerfile"
context = "."
target = "production"
args = { NODE_ENV = "production" }

[container.registry]
url = "ghcr.io"
username = "${GITHUB_ACTOR}"
password = "${GITHUB_TOKEN}"

[container.compose]
# Docker Compose 配置
file = "docker-compose.yml"
profiles = ["dev", "test", "prod"]

# ============================================
# 11. 版本控制策略 (新增)
# ============================================
[versioning]
strategy = "semver"  # semver | calver | custom
auto_bump = true

[versioning.release]
# 发布配置
branches = ["main", "release/*"]
prerelease = ["alpha", "beta", "rc"]
tag_prefix = "v"

[versioning.changelog]
# 变更日志分类
types = [
  { type = "feat", section = "Features" },
  { type = "fix", section = "Bug Fixes" },
  { type = "perf", section = "Performance" },
  { type = "docs", section = "Documentation" },
]

# ============================================
# 服务编排 (新增)
# ============================================
[services]
# 本地开发服务

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

# ============================================
# Hooks (新增)
# ============================================
[hooks]
# 生命周期钩子

[hooks.pre_setup]
run = "echo 'Starting setup...'"

[hooks.post_setup]
run = ["vx run db:migrate", "vx run seed"]

[hooks.pre_commit]
run = "vx run lint && vx run test:unit"

[hooks.enter]
# 进入项目目录时执行
run = "vx sync --check"

# ============================================
# 环境变量 (增强)
# ============================================
[env]
NODE_ENV = "development"

[env.required]
DATABASE_URL = "PostgreSQL connection string"
API_KEY = "External API key"

[env.optional]
DEBUG = "Enable debug mode"

[env.secrets]
# 敏感变量，从安全存储加载
provider = "auto"  # auto | 1password | vault | aws-secrets
items = ["DATABASE_URL", "API_KEY"]

# ============================================
# 脚本 (增强)
# ============================================
[scripts]
dev = "npm run dev"
build = "npm run build"
test = "npm test"

[scripts.db]
migrate = "prisma migrate dev"
seed = "prisma db seed"
studio = "prisma studio"

[scripts.lint]
command = "eslint . && prettier --check ."
description = "Run all linters"
depends = []

[scripts.release]
command = "semantic-release"
description = "Create a new release"
env = { CI = "true" }

# ============================================
# 设置 (增强)
# ============================================
[settings]
auto_install = true
parallel_install = true
cache_duration = "7d"
shell = "auto"  # auto | bash | zsh | fish | pwsh
log_level = "info"

[settings.experimental]
# 实验性功能
monorepo = false
workspaces = false
```

## 向后兼容性

### 兼容策略

1. **版本检测**: 通过 `min_version` 字段检测配置版本
2. **渐进增强**: 所有新字段都是可选的
3. **默认值**: 新字段都有合理的默认值
4. **警告提示**: 遇到未知字段时发出警告而非报错

### 迁移路径

```bash
# 检查配置兼容性
vx config check

# 自动迁移到 v2
vx config migrate --to v2

# 验证迁移结果
vx config validate
```

## 实现计划

### Phase 1: 核心增强 (v0.6.0)

- [ ] `[hooks]` - 生命周期钩子
- [ ] `[services]` - 服务编排
- [ ] `[dependencies]` 增强 - 智能依赖管理
- [ ] 配置验证和迁移工具

### Phase 2: AI 集成 (v0.7.0)

- [ ] `[ai]` - AI 代码生成配置
- [ ] `[docs]` - 文档自动生成
- [ ] AI 上下文导出命令

### Phase 3: 团队协作 (v0.8.0)

- [ ] `[team]` - 团队协作规则
- [ ] `[remote]` - 远程开发环境
- [ ] 配置继承和预设

### Phase 4: DevSecOps (v0.9.0)

- [ ] `[security]` - 安全扫描
- [ ] `[test]` - 测试流水线
- [ ] `[telemetry]` - 性能监控

### Phase 5: 部署集成 (v1.0.0)

- [ ] `[container]` - 容器化部署
- [ ] `[versioning]` - 版本控制策略
- [ ] CI/CD 集成增强

## 参考资料

- [mise 配置文档](https://mise.jdx.dev/configuration.html)
- [devbox 配置参考](https://www.jetpack.io/devbox/docs/configuration/)
- [Cursor Rules 规范](https://cursor.sh/docs/rules)
- [GitHub Copilot 配置](https://docs.github.com/en/copilot)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-12-26 | Draft | 初始草案 |
