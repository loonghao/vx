# 增强的脚本系统

vx 的增强脚本系统提供强大的参数传递、基于 DAG 的依赖执行和灵活的工作流自动化 —— 使其成为内置在项目配置中的完整任务运行器。

## 概述

增强的脚本系统解决了开发自动化中的常见痛点：

- **基于 DAG 的工作流执行**：脚本可以声明对其他脚本的依赖，形成有向无环图（DAG），通过拓扑排序自动解析执行顺序
- **循环依赖检测**：vx 在执行时检测并报告循环依赖
- **参数冲突**：不再有 `-p`、`--lib`、`--fix` 标志的问题
- **复杂工具集成**：完美适配 cargo、eslint、docker 和其他有许多选项的工具
- **脚本文档**：每个脚本的内置帮助系统
- **灵活工作流**：支持简单和复杂的参数模式

## 基于 DAG 的工作流执行

脚本系统最强大的功能是**基于依赖的执行**。你可以声明一个脚本依赖其他脚本，vx 将使用拓扑排序以正确的顺序执行它们。

### 工作原理

```
           ┌─────────┐
           │  deploy  │
           └────┬────┘
                │ depends
          ┌─────┴─────┐
          │           │
     ┌────▼───┐  ┌───▼────┐
     │  build  │  │  test  │
     └────┬───┘  └───┬────┘
          │           │ depends
          │     ┌─────┴─────┐
          │     │           │
          │  ┌──▼──┐  ┌───▼─────┐
          │  │ lint │  │typecheck│
          │  └─────┘  └─────────┘
          │
     ┌────▼───────┐
     │  generate   │
     └────────────┘
```

当你运行 `vx run deploy` 时，vx 会：

1. **构建依赖图** — 收集所有传递性依赖
2. **检测环路** — 如果存在循环依赖则报错（如 `A → B → A`）
3. **拓扑排序** — 确定正确的执行顺序
4. **按序执行** — 按依赖顺序逐个运行每个脚本，每个脚本只运行一次
5. **快速失败** — 如果任何依赖失败，整个链条立即停止

### 基本依赖示例

```toml
[scripts]
lint = "eslint . && prettier --check ."
typecheck = "tsc --noEmit"
test = "vitest run"
build = "npm run build"

[scripts.ci]
command = "echo '✅ 所有检查通过！'"
description = "运行所有 CI 检查"
depends = ["lint", "typecheck", "test", "build"]
```

```bash
vx run ci
# 执行顺序: lint → typecheck → test → build → ci
# （依赖关系通过拓扑排序解析）
```

### 多级依赖

依赖可以嵌套 — vx 解析完整的传递性依赖图：

```toml
[scripts]
generate = "protoc --go_out=. *.proto"
lint = "golangci-lint run"

[scripts.build]
command = "go build -o app ./cmd/server"
description = "构建服务器"
depends = ["generate"]

[scripts.test]
command = "go test ./..."
description = "运行测试"
depends = ["lint", "generate"]

[scripts.deploy]
command = "kubectl apply -f k8s/"
description = "部署到 Kubernetes"
depends = ["build", "test"]
```

```bash
vx run deploy
# 解析顺序: generate → lint → build → test → deploy
# 注意: generate 只运行一次，即使 build 和 test 都依赖它
```

### 每个脚本只运行一次

DAG 执行器跟踪已访问节点 — 依赖图中的每个脚本**最多执行一次**，即使多个脚本依赖它也是如此。

### 循环依赖检测

vx 检测循环依赖并报告清晰的错误：

```toml
[scripts.a]
command = "echo a"
depends = ["b"]

[scripts.b]
command = "echo b"
depends = ["a"]    # 循环！
```

```bash
vx run a
# 错误: Circular dependency detected: a -> b -> a
```

### 带环境变量的依赖

依赖链中的每个脚本可以拥有自己的环境变量和工作目录：

```toml
[env]
NODE_ENV = "development"

[scripts.migrate]
command = "prisma migrate deploy"
env = { DATABASE_URL = "postgres://localhost/myapp" }
cwd = "backend"

[scripts.seed]
command = "python seed.py"
cwd = "backend"
depends = ["migrate"]

[scripts.dev]
command = "npm run dev"
description = "在数据库准备好后启动开发服务器"
depends = ["seed"]
```

## 实际工作流模式

### 全栈 CI 管道

```toml
[scripts]
# 独立检查任务
lint:frontend = "cd frontend && npm run lint"
lint:backend = "cd backend && uvx ruff check ."
typecheck = "cd frontend && tsc --noEmit"
test:unit = "cd backend && uv run pytest tests/unit"
test:integration = "cd backend && uv run pytest tests/integration"
build:frontend = "cd frontend && npm run build"
build:backend = "cd backend && cargo build --release"

# 使用 DAG 依赖的组合任务
[scripts.lint]
command = "echo '✅ 所有代码检查通过'"
depends = ["lint:frontend", "lint:backend"]

[scripts.test]
command = "echo '✅ 所有测试通过'"
depends = ["test:unit", "test:integration"]

[scripts.build]
command = "echo '✅ 所有构建完成'"
depends = ["build:frontend", "build:backend"]

[scripts.ci]
command = "echo '🎉 CI 管道通过！'"
description = "运行完整的 CI 管道"
depends = ["lint", "typecheck", "test", "build"]
```

```bash
vx run ci
# 运行: lint:frontend → lint:backend → lint → typecheck
#     → test:unit → test:integration → test
#     → build:frontend → build:backend → build → ci
```

### 发布工作流

```toml
[scripts]
changelog = "git-cliff -o CHANGELOG.md"
version-bump = "npm version {{arg1}}"

[scripts.build-release]
command = "cargo build --release"
depends = ["changelog"]

[scripts.package]
command = "tar czf dist/app.tar.gz -C target/release app"
depends = ["build-release"]

[scripts.publish]
command = "gh release create v{{arg1}} dist/app.tar.gz"
description = "创建新发布"
depends = ["version-bump", "package"]
```

```bash
vx run publish 1.2.0
# 运行: changelog → build-release → package → version-bump → publish
```

### 跨语言构建管道

```toml
[scripts]
proto-gen = "protoc --go_out=. --python_out=. api/*.proto"

[scripts.build:go]
command = "go build -o bin/server ./cmd/server"
depends = ["proto-gen"]

[scripts.build:python]
command = "uv run python -m build"
depends = ["proto-gen"]

[scripts.build:frontend]
command = "npm run build"
cwd = "frontend"

[scripts.build]
command = "echo '✅ 所有服务构建完成'"
description = "构建所有内容"
depends = ["build:go", "build:python", "build:frontend"]

[scripts.docker]
command = "docker compose build"
description = "构建 Docker 镜像"
depends = ["build"]
```

### 数据库迁移管道

```toml
[scripts]
db:backup = "pg_dump $DATABASE_URL > backup.sql"

[scripts.db:migrate]
command = "prisma migrate deploy"
description = "运行数据库迁移"
depends = ["db:backup"]

[scripts.db:seed]
command = "python manage.py seed"
depends = ["db:migrate"]

[scripts.db:reset]
command = "prisma migrate reset --force"
description = "重置并重新填充数据库"
depends = ["db:backup"]
```

## 高级参数传递

::: v-pre
### `{{args}}` 占位符
:::

直接向脚本传递复杂参数而不会产生冲突：

```bash
# 带包选择的 Cargo 测试
vx run test-pkgs -p vx-runtime --lib

# 带多个选项的 ESLint
vx run lint --fix --ext .js,.ts src/

# 带平台选择的 Docker 构建
vx run docker-build --platform linux/amd64 -t myapp .
```

### 脚本定义

::: v-pre
使用 `{{args}}` 获得最大灵活性：
:::

```toml
[scripts]
# 现代方法：灵活的参数处理
test-pkgs = "cargo test {{args}}"
lint = "eslint {{args}}"
build = "docker build {{args}}"

# 传统方法：仍然有效但有限制
test-simple = "cargo test"
```

### 脚本特定帮助

获取单个脚本的详细帮助：

```bash
# 显示特定脚本的帮助
vx run test-pkgs -H
vx run deploy --script-help

# 列出所有可用脚本
vx run --list
```

## 迁移指南

### 从简单脚本迁移

**之前：**
```toml
[scripts]
test = "cargo test"
lint = "eslint src/"
```

**之后：**
```toml
[scripts]
test = "cargo test {{args}}"
lint = "eslint {{args}}"
```

**好处：**
- `vx run test -p my-package --lib` 现在可以工作
- `vx run lint --fix --ext .js,.ts src/` 现在可以工作

### 从 Shell 脚本链迁移

**之前（Makefile / Shell 脚本）：**
```bash
# 需要手动链接命令和跟踪依赖
lint:
	eslint .
typecheck:
	tsc --noEmit
test: lint typecheck
	vitest run
deploy: test
	npm run build && kubectl apply -f k8s/
```

**之后（vx.toml + DAG）：**
```toml
[scripts]
lint = "eslint ."
typecheck = "tsc --noEmit"

[scripts.test]
command = "vitest run"
depends = ["lint", "typecheck"]

[scripts.deploy]
command = "npm run build && kubectl apply -f k8s/"
depends = ["test"]
```

**好处：**
- 循环依赖检测
- 每个依赖只运行一次
- 内置脚本帮助和列表
- 跨平台（无需 Makefile/bash 依赖）

## 最佳实践

::: v-pre
### 1. 使用 `{{args}}` 进行工具集成
:::

对于有许多命令行选项的工具：

```toml
[scripts]
# ✅ 灵活 - 支持任何 cargo test 参数
test = "cargo test {{args}}"

# ✅ 灵活 - 支持任何 eslint 参数
lint = "eslint {{args}}"

# ❌ 僵化 - 只适用于特定用例
test-lib = "cargo test --lib"
```

### 2. 使用依赖代替命令链

不要用 `&&` 链接命令，使用 `depends`：

```toml
# ❌ 脆弱 - 无去重，无环检测
ci = "eslint . && tsc --noEmit && vitest run && npm run build"

# ✅ 健壮 - 基于 DAG 的执行，具备所有优势
[scripts]
lint = "eslint ."
typecheck = "tsc --noEmit"
test = "vitest run"
build = "npm run build"

[scripts.ci]
command = "echo '所有检查通过！'"
depends = ["lint", "typecheck", "test", "build"]
```

### 3. 为复杂脚本添加描述

```toml
[scripts.deploy]
command = "kubectl apply -f k8s/"
description = "部署到生产环境 Kubernetes 集群"
depends = ["build", "test"]
env = { KUBECONFIG = "~/.kube/production" }
```

### 4. 与环境变量结合

```toml
[env]
RUST_LOG = "debug"
CARGO_TERM_COLOR = "always"

[scripts]
test = "cargo test {{args}}"
test-quiet = "RUST_LOG=error cargo test {{args}}"
```

### 5. 在 Monorepo 中使用 cwd

```toml
[scripts.build:api]
command = "cargo build --release"
cwd = "services/api"

[scripts.build:web]
command = "npm run build"
cwd = "apps/web"

[scripts.build]
command = "echo '所有服务构建完成'"
depends = ["build:api", "build:web"]
```

## 高级用法

### 多工具工作流

```toml
[scripts]
# 按顺序格式化和检查
check = "cargo fmt && cargo clippy {{args}}"

# 使用参数构建和测试
ci = "cargo build {{args}} && cargo test {{args}}"

# 使用多个工具的复杂部署
deploy = "docker build -t myapp {{args}} . && kubectl apply -f k8s/"
```

### 条件参数

```toml
[scripts]
# 使用环境变量进行条件行为
test = "cargo test {{args}} ${EXTRA_TEST_ARGS:-}"
build = "cargo build {{args}} ${BUILD_PROFILE:+--profile $BUILD_PROFILE}"
```

### 与任务运行器集成

vx 脚本通过子进程 PATH 继承与 Dagu、Just、Make 等外部任务运行器无缝配合：

```toml
[scripts]
# 在 DAG 工作流中使用 vx 管理的工具
workflow = "dagu start pipeline.yaml"

# justfile 配方可以直接访问 vx 工具
just-ci = "just ci"
```

## 故障排除

### 循环依赖错误

**问题**：`Circular dependency detected: A -> B -> A`

**解决方案**：检查 `depends` 列表并打破循环：

```toml
# ❌ 循环
[scripts.a]
command = "echo a"
depends = ["b"]

[scripts.b]
command = "echo b"
depends = ["a"]

# ✅ 修复 - 提取共享依赖
[scripts]
shared = "echo shared"

[scripts.a]
command = "echo a"
depends = ["shared"]

[scripts.b]
command = "echo b"
depends = ["shared"]
```

### 依赖脚本未找到

**问题**：`Dependency script 'build' not found in vx.toml`

**解决方案**：确保 `depends` 中引用的所有脚本都已定义：

```toml
[scripts]
build = "cargo build"   # 必须存在！

[scripts.deploy]
command = "kubectl apply -f k8s/"
depends = ["build"]     # 引用上面的 "build"
```

### 参数不工作

**问题**：参数没有传递给脚本。

::: v-pre
**解决方案**：确保您的脚本使用 `{{args}}`：
:::

```toml
# ❌ 不会接收参数
test = "cargo test"

# ✅ 会接收所有参数
test = "cargo test {{args}}"
```

### 脚本帮助未显示

**问题**：`vx run script --help` 显示全局帮助而不是脚本帮助。

**解决方案**：使用 `-H` 代替：

```bash
# ✅ 显示脚本特定帮助（包括依赖信息）
vx run script -H

# ❌ 显示全局 vx 帮助
vx run script --help
```

## 示例

### Rust 开发

```toml
[scripts]
test = "cargo test {{args}}"
test-all = "cargo test --workspace {{args}}"
bench = "cargo bench {{args}}"
clippy = "cargo clippy {{args}}"
doc = "cargo doc {{args}}"
fmt = "cargo fmt"

[scripts.check]
command = "echo '✅ 所有检查通过'"
description = "运行所有质量检查"
depends = ["fmt", "clippy", "test-all"]
```

用法：
```bash
vx run test -p my-crate --lib
vx run clippy -- -D warnings
vx run doc --open --no-deps
vx run check   # 运行 fmt → clippy → test-all → check
```

### JavaScript/TypeScript 开发

```toml
[scripts]
lint = "eslint {{args}}"
format = "prettier {{args}}"
typecheck = "tsc --noEmit"
test = "vitest run {{args}}"
build = "vite build"

[scripts.ci]
command = "echo '✅ CI 通过'"
depends = ["lint", "typecheck", "test", "build"]
```

用法：
```bash
vx run lint --fix --ext .js,.ts src/
vx run test --watch --coverage
vx run ci   # 完整管道
```

### Python 开发

```toml
[scripts]
lint = "uvx ruff check . {{args}}"
format = "uvx ruff format . {{args}}"
typecheck = "uvx mypy src/"
test = "uv run pytest {{args}}"

[scripts.ci]
command = "echo '✅ 所有检查通过'"
depends = ["lint", "typecheck", "test"]

[scripts.publish]
command = "uv build && uvx twine upload dist/*"
description = "构建并发布到 PyPI"
depends = ["ci"]
```

用法：
```bash
vx run lint --fix
vx run test -x --tb=short
vx run publish   # 运行: lint → typecheck → test → ci → publish
```

### Docker 开发

```toml
[scripts]
build = "docker build {{args}}"
run = "docker run {{args}}"
compose = "docker-compose {{args}}"

[scripts.up]
command = "docker compose up -d"
description = "启动所有服务"

[scripts.down]
command = "docker compose down"
description = "停止所有服务"
```

用法：
```bash
vx run build -t myapp:latest --platform linux/amd64 .
vx run compose up -d --scale web=3
```

## 脚本配置参考

### 简单脚本

```toml
[scripts]
dev = "npm run dev"
```

### 详细脚本

```toml
[scripts.deploy]
command = "kubectl apply -f k8s/"      # 必需：要执行的命令
description = "部署到生产环境"            # 可选：显示在 --list 和 -H 中
args = ["--prune"]                     # 可选：默认参数
cwd = "infrastructure"                 # 可选：工作目录
env = { KUBECONFIG = "~/.kube/prod" }  # 可选：环境变量
depends = ["build", "test"]            # 可选：依赖脚本（DAG）
```

| 字段 | 类型 | 描述 |
|------|------|------|
| `command` | string | 要执行的命令 |
| `description` | string | 人类可读的描述 |
| `args` | string[] | 默认参数 |
| `cwd` | string | 工作目录（相对于项目根目录） |
| `env` | table | 脚本特定的环境变量 |
| `depends` | string[] | 先运行的脚本（DAG 依赖） |

## 另请参阅

- [run 命令参考](../cli/run.md) - 完整命令文档
- [vx.toml 配置](../config/vx-toml.md) - 配置文件参考
- [变量插值](../config/vx-toml.md#variable-interpolation) - 高级变量用法
- [最佳实践](./best-practices.md) - 更多工作流模式
