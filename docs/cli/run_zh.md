# run

运行 `vx.toml` 中定义的脚本。

## 语法

```bash
vx run <SCRIPT> [ARGS...]
vx run <SCRIPT> -H
vx run --list
vx run --help
```

## 描述

执行 `vx.toml` 的 `[scripts]` 部分中定义的脚本。增强的 run 命令现在支持：

- **灵活的参数传递**：直接传递参数而不会产生冲突
- **脚本特定帮助**：使用 `-H` 获取单个脚本的帮助
- **脚本列表**：使用 `--list` 查看所有可用脚本
- **高级参数处理**：支持 `-p`、`--lib` 和其他工具特定标志
- 项目环境变量（来自 `[env]` 和 `.env` 文件）
- 变量插值支持（`\{\{var\}\}` 语法）
- Python 虚拟环境激活（如果配置）
- 工具路径配置

## 参数

| 参数 | 描述 |
|------|------|
| `SCRIPT` | 要运行的脚本名称（使用 `--list` 或 `--help` 时可选） |
| `ARGS` | 传递给脚本的附加参数（支持连字符标志如 `-p`、`--lib`） |

## 选项

| 选项 | 描述 |
|------|------|
| `-h`, `--help` | 显示 run 命令的帮助 |
| `-l`, `--list` | 列出所有可用脚本 |
| `-H`, `--script-help` | 显示脚本特定帮助（当提供脚本名称时） |

## 增强的参数处理

run 命令现在支持直接向脚本传递复杂参数而不会产生冲突：

```bash
# 直接传递工具特定标志
vx run test-pkgs -p vx-runtime --lib
vx run lint --fix --verbose
vx run build --release --target x86_64-pc-windows-msvc

# 使用 -- 分隔符进行显式参数分离（可选）
vx run test-pkgs -- -p vx-runtime --lib
```

## 变量插值

脚本支持使用 `\{\{var\}\}` 语法进行变量插值：

```toml
[scripts]
build = "cargo build -p \{\{project.name\}\}"
tag = "git tag v\{\{arg1\}\}"
info = "echo 'Building on \{\{os.name\}\} (\{\{os.arch\}\})'"
test-pkgs = "cargo test \{\{args\}\}"  # 使用 {{args}} 接收所有参数
```

### 内置变量

| 变量 | 描述 |
|------|------|
| `\{\{vx.version\}\}` | vx 版本 |
| `\{\{vx.home\}\}` | vx 主目录 (~/.vx) |
| `\{\{vx.runtimes\}\}` | 运行时目录 |
| `\{\{project.root\}\}` | 项目根目录 |
| `\{\{project.name\}\}` | 项目名称（目录名） |
| `\{\{os.name\}\}` | 操作系统 (linux, macos, windows) |
| `\{\{os.arch\}\}` | CPU 架构 (x86_64, aarch64) |
| `\{\{home\}\}` | 用户主目录 |
| `\{\{timestamp\}\}` | 当前 Unix 时间戳 |

### 参数变量

| 变量 | 描述 |
|------|------|
| `\{\{arg1\}\}`, `\{\{arg2\}\}`, ... | 位置参数 |
| `\{\{@\}\}` | 所有参数作为字符串 |
| `\{\{#\}\}` | 参数数量 |
| `\{\{args\}\}` | **推荐**：所有参数（支持复杂标志如 `-p`、`--lib`） |

### 环境变量

| 变量 | 描述 |
|------|------|
| `\{\{env.VAR\}\}` | 环境变量 VAR |

### 命令插值

使用反引号获取命令输出：

```toml
[scripts]
info = "echo 'Commit: `git rev-parse --short HEAD`'"
```

## 环境变量

### .env 文件支持

脚本自动从以下位置加载环境变量：

1. `.env` - 基础环境文件
2. `.env.local` - 本地覆盖（更高优先级）

### 优先级顺序

1. 脚本特定的 `env` 属性
2. vx.toml 中的全局 `[env]` 部分
3. `.env.local` 文件
4. `.env` 文件
5. 系统环境变量

### 配置

```toml
[env]
NODE_ENV = "development"
API_URL = "http://localhost:3000"

[scripts.dev]
run = "npm run dev"
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
deploy = "kubectl apply -f k8s/\{\{arg1\}\}.yaml"

# 现代方法：使用 {{args}} 处理复杂参数
test-pkgs = "cargo test \{\{args\}\}"
lint = "eslint \{\{args\}\}"
format = "prettier --write \{\{args\}\}"
```

## 示例

### 基本用法

```bash
# 运行简单脚本
vx run dev

# 列出所有可用脚本
vx run --list
```

### 增强的参数传递

```bash
# 直接传递复杂参数（新功能！）
vx run test-pkgs -p vx-runtime --lib
vx run test-pkgs -p vx-provider-python -p vx-runtime

# 使用 -- 分隔符的传统方法（仍然支持）
vx run test-pkgs -- -p vx-runtime --lib

# 多个标志和选项
vx run lint --fix --ext .js,.ts src/
vx run build --release --target x86_64-pc-windows-msvc
```

### 脚本特定帮助

```bash
# 获取特定脚本的帮助（新功能！）
vx run test-pkgs -H
vx run deploy --script-help

# 通用 run 命令帮助
vx run --help
```

### 变量插值示例

```bash
# 如果脚本使用 \{\{arg1\}\}、\{\{arg2\}\} 等，参数会被插值
vx run deploy production

# 使用 {{args}}（推荐用于复杂参数）
vx run test-pkgs -p vx-runtime --lib  # 作为 {{args}} 传递
```

### 脚本帮助输出

当您运行 `vx run deploy -H` 时，您会看到：

```text
Script: deploy
Command: kubectl apply -f k8s/\{\{arg1\}\}.yaml

Usage: vx run deploy [args...]

Arguments are passed directly to the script.

Variable Interpolation:
  \{\{arg1\}\}          第一个参数
  \{\{arg2\}\}          第二个参数
  \{\{@\}\}             所有参数
  \{\{#\}\}             参数数量
  \{\{args\}\}          所有参数（推荐）
  \{\{env.VAR\}\}       环境变量 VAR
  \{\{project.root\}\}  项目根目录
  \{\{project.name\}\}  项目名称
  \{\{os.name\}\}       操作系统
  \{\{vx.version\}\}    VX 版本
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
  lint            eslint {{args}}
```

## 最佳实践

### 使用 `\{\{args\}\}` 编写现代脚本

为了获得最大的灵活性，在脚本定义中使用 `\{\{args\}\}`：

```toml
[scripts]
# ✅ 推荐：灵活的参数处理
test-pkgs = "cargo test \{\{args\}\}"
lint = "eslint \{\{args\}\}"
build = "cargo build \{\{args\}\}"

# ❌ 旧式：仅限于简单参数
test-old = "cargo test"
```

### 复杂工具集成

非常适合需要特定标志的工具：

```toml
[scripts]
# 带包选择的 Cargo 测试
test-pkgs = "cargo test \{\{args\}\}"
# 用法：vx run test-pkgs -p vx-runtime --lib

# 带灵活选项的 ESLint
lint = "eslint \{\{args\}\}"
# 用法：vx run lint --fix --ext .js,.ts src/

# 带平台选择的 Docker 构建
docker-build = "docker build \{\{args\}\}"
# 用法：vx run docker-build --platform linux/amd64 -t myapp .
```

### 从旧式迁移

如果您有不使用 `\{\{args\}\}` 的现有脚本，它们仍然有效但有限制：

```toml
[scripts]
# 这有效但仅适用于简单参数
test = "cargo test"

# 这对复杂参数更好
test-new = "cargo test \{\{args\}\}"
```

## 故障排除

### 参数未正确传递

如果您的脚本没有按预期接收参数：

1. **检查您的脚本是否使用 `\{\{args\}\}`**：
   ```toml
   # 添加 {{args}} 以接收所有参数
   test = "cargo test \{\{args\}\}"
   ```

2. **对复杂情况使用 `--` 分隔符**：
   ```bash
   vx run test -- -p vx-runtime --lib
   ```

3. **检查脚本帮助**：
   ```bash
   vx run test -H  # 显示参数如何处理
   ```

### 脚本未找到

如果您收到"Script not found"错误：

```bash
# 列出可用脚本
vx run --list

# 检查您的 vx.toml 文件
cat vx.toml
```

### 使用环境变量

```bash
# 从 .env 文件
echo "API_KEY=secret123" > .env
vx run deploy  # API_KEY 可用

# 内联覆盖
API_KEY=newkey vx run deploy
```

## 另请参阅

- [dev](./dev) - 进入开发环境
- [setup](./setup) - 安装项目工具
- [ext](./ext) - 运行扩展
- [配置](/config/vx-toml) - vx.toml 参考