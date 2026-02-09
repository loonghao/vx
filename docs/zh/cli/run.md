# run 命令

运行 `vx.toml` 中定义的脚本。

## 语法

```bash
vx run <SCRIPT> [ARGS...]
vx run <SCRIPT> -H
vx run --list
vx run --help
```

## 描述

执行在 `vx.toml` 的 `[scripts]` 节中定义的脚本。增强的 run 命令支持：

- **灵活参数传递**：直接传递参数而不产生冲突
- **脚本特定帮助**：使用 `-H` 获取单个脚本的帮助
- **脚本列表**：使用 `--list` 查看所有可用脚本
- **高级参数处理**：支持 `-p`、`--lib` 等工具特定标志
- **DAG 依赖执行**：脚本可声明依赖，按拓扑排序执行
- 项目环境变量（来自 `[env]` 和 `.env` 文件）
- 变量插值支持（`{{var}}` 语法）
- Python venv 自动激活（如已配置）
- 工具路径自动配置

## 参数

| 参数 | 描述 |
|------|------|
| `SCRIPT` | 要运行的脚本名称（使用 `--list` 或 `--help` 时可选） |
| `ARGS` | 传递给脚本的额外参数（支持 `-p`、`--lib` 等连字符标志） |

## 选项

| 选项 | 描述 |
|------|------|
| `-h`, `--help` | 显示 run 命令帮助 |
| `-l`, `--list` | 列出所有可用脚本 |
| `-H`, `--script-help` | 显示脚本特定帮助（需提供脚本名） |

## 增强参数处理

run 命令支持直接向脚本传递复杂参数而不产生冲突：

```bash
# 直接传递工具特定标志
vx run test-pkgs -p vx-runtime --lib
vx run lint --fix --verbose
vx run build --release --target x86_64-pc-windows-msvc

# 使用 -- 分隔符进行显式参数分离（可选）
vx run test-pkgs -- -p vx-runtime --lib
```

## DAG 依赖执行

脚本可以通过 `depends` 字段声明对其他脚本的依赖。vx 使用**拓扑排序**确定正确的执行顺序，并具有以下特性：

- **循环检测**：检测并报告循环依赖（如 `A → B → A`）
- **去重执行**：每个脚本最多执行一次，即使被多个脚本依赖
- **快速失败**：任何依赖失败，整个执行链立即停止
- **环境隔离**：每个依赖脚本可以有自己的 `env` 和 `cwd`

### 示例

```toml
[scripts]
lint = "eslint ."
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
```

### 多级依赖

```toml
[scripts]
generate = "protoc --go_out=. *.proto"

[scripts.build]
command = "go build -o app"
depends = ["generate"]

[scripts.test]
command = "go test ./..."
depends = ["generate"]

[scripts.deploy]
command = "kubectl apply -f k8s/"
depends = ["build", "test"]
```

```bash
vx run deploy
# 解析顺序: generate → build → test → deploy
# generate 只运行一次
```

## 变量插值

脚本支持 `{{var}}` 语法的变量插值：

```toml
[scripts]
build = "cargo build -p {{project.name}}"
tag = "git tag v{{arg1}}"
info = "echo '构建于 {{os.name}} ({{os.arch}})'"
test-pkgs = "cargo test {{args}}"  # 使用 {{args}} 接收所有参数
```

### 内置变量

| 变量 | 描述 |
|------|------|
| `{{vx.version}}` | vx 版本 |
| `{{vx.home}}` | vx 主目录 (~/.vx) |
| `{{vx.runtimes}}` | 运行时目录 |
| `{{project.root}}` | 项目根目录 |
| `{{project.name}}` | 项目名称（目录名） |
| `{{os.name}}` | 操作系统 (linux, macos, windows) |
| `{{os.arch}}` | CPU 架构 (x86_64, aarch64) |
| `{{home}}` | 用户主目录 |
| `{{timestamp}}` | 当前 Unix 时间戳 |

### 参数变量

| 变量 | 描述 |
|------|------|
| `{{arg1}}`, `{{arg2}}`, ... | 位置参数 |
| `{{@}}` | 所有参数（字符串形式） |
| `{{#}}` | 参数数量 |
| `{{args}}` | **推荐**：所有参数（支持 `-p`、`--lib` 等复杂标志） |

### 环境变量

| 变量 | 描述 |
|------|------|
| `{{env.VAR}}` | 环境变量 VAR |

### 命令插值

使用反引号获取命令输出：

```toml
[scripts]
info = "echo '提交: `git rev-parse --short HEAD`'"
```

## 环境变量

### .env 文件支持

脚本自动加载以下环境变量文件：

1. `.env` — 基础环境文件
2. `.env.local` — 本地覆盖（更高优先级）

### 优先级顺序

1. 脚本级 `env` 属性
2. 全局 `[env]` 节（vx.toml）
3. `.env.local` 文件
4. `.env` 文件
5. 系统环境变量

### 配置

```toml
[env]
NODE_ENV = "development"
API_URL = "http://localhost:3000"

[scripts.dev]
command = "npm run dev"
env = { PORT = "3000" }  # 脚本特定
```

## 配置

在 `vx.toml` 中定义脚本：

```toml
[scripts]
# 简单命令
dev = "npm run dev"
test = "pytest"
build = "go build -o app"

# 使用变量插值
deploy = "kubectl apply -f k8s/{{arg1}}.yaml"

# 使用 {{args}} 处理复杂参数
test-pkgs = "cargo test {{args}}"
lint = "eslint {{args}}"
format = "prettier --write {{args}}"

# 带依赖的详细脚本
[scripts.ci]
command = "echo '所有检查通过'"
description = "运行 CI 管道"
depends = ["lint", "test", "build"]

[scripts.start]
command = "python main.py"
description = "启动服务器"
args = ["--host", "0.0.0.0"]
cwd = "src"
env = { DEBUG = "true" }
depends = ["build"]
```

## 示例

### 基本用法

```bash
# 运行脚本
vx run dev

# 列出所有可用脚本
vx run --list
```

### 增强参数传递

```bash
# 直接传递复杂参数
vx run test-pkgs -p vx-runtime --lib
vx run test-pkgs -p vx-provider-python -p vx-runtime

# 使用 -- 分隔符的传统方式（仍然支持）
vx run test-pkgs -- -p vx-runtime --lib

# 多个标志和选项
vx run lint --fix --ext .js,.ts src/
vx run build --release --target x86_64-pc-windows-msvc
```

### DAG 依赖执行

```bash
# 运行带有依赖的脚本（自动按拓扑排序执行）
vx run ci       # 先运行 lint, typecheck, test, build，最后运行 ci
vx run deploy   # 先运行 build 和 test，再运行 deploy
```

### 脚本特定帮助

```bash
# 获取脚本帮助（包括依赖信息）
vx run deploy -H
vx run ci --script-help

# 通用 run 命令帮助
vx run --help
```

### 变量插值示例

```bash
# 参数通过 {{arg1}}, {{arg2}} 等插值
vx run deploy production

# 使用 {{args}}（推荐用于复杂参数）
vx run test-pkgs -p vx-runtime --lib  # 作为 {{args}} 传递
```

### 列出可用脚本

```bash
vx run --list
```

输出：

```text
Available scripts:
  dev             npm run dev
  test            pytest
  build           go build -o app
  test-pkgs       cargo test {{args}}
  ci              echo '所有检查通过' (运行 CI 管道)
```

## 最佳实践

### 使用 `{{args}}` 实现灵活脚本

```toml
[scripts]
# ✅ 推荐：灵活参数处理
test-pkgs = "cargo test {{args}}"
lint = "eslint {{args}}"
build = "cargo build {{args}}"

# ❌ 旧风格：参数受限
test-old = "cargo test"
```

### 使用依赖代替命令链

```toml
# ❌ 脆弱 - 无去重，无环检测
ci = "eslint . && tsc --noEmit && vitest run"

# ✅ 健壮 - 基于 DAG 执行
[scripts]
lint = "eslint ."
typecheck = "tsc --noEmit"
test = "vitest run"

[scripts.ci]
command = "echo '通过'"
depends = ["lint", "typecheck", "test"]
```

### 为复杂脚本添加描述

```toml
[scripts.deploy]
command = "kubectl apply -f k8s/"
description = "部署到生产环境 Kubernetes 集群"
depends = ["build", "test"]
```

## 故障排除

### 循环依赖

```bash
# 错误: Circular dependency detected: a -> b -> a
# 解决: 检查 depends 列表，打破循环
```

### 参数未正确传递

1. **检查脚本是否使用 `{{args}}`**：
   ```toml
   # 添加 {{args}} 以接收所有参数
   test = "cargo test {{args}}"
   ```

2. **对复杂情况使用 `--` 分隔符**：
   ```bash
   vx run test -- -p vx-runtime --lib
   ```

3. **查看脚本帮助**：
   ```bash
   vx run test -H  # 显示参数处理方式
   ```

### 脚本未找到

```bash
# 列出可用脚本
vx run --list

# 检查 vx.toml 文件
cat vx.toml
```

## 参见

- [增强脚本系统](/zh/guide/enhanced-scripts) - 完整脚本功能和 DAG 工作流
- [dev](./dev) - 进入开发环境
- [setup](./setup) - 安装项目工具
- [配置指南](/zh/guide/configuration) - 脚本配置
- [vx.toml 参考](/zh/config/vx-toml) - 配置文件参考
