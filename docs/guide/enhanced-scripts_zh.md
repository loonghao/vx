# 增强的脚本系统

vx 的增强脚本系统提供强大的参数传递能力，非常适合复杂的开发工作流和工具集成。

## 概述

增强的脚本系统解决了开发自动化中的常见痛点：

- **参数冲突**：不再有 `-p`、`--lib`、`--fix` 标志的问题
- **复杂工具集成**：完美适配 cargo、eslint、docker 和其他有许多选项的工具
- **脚本文档**：每个脚本的内置帮助系统
- **灵活工作流**：支持简单和复杂的参数模式

## 主要特性

### 1. 高级参数传递

直接向脚本传递复杂参数而不会产生冲突：

```bash
# 带包选择的 Cargo 测试
vx run test-pkgs -p vx-runtime --lib

# 带多个选项的 ESLint
vx run lint --fix --ext .js,.ts src/

# 带平台选择的 Docker 构建
vx run docker-build --platform linux/amd64 -t myapp .
```

### 2. 脚本特定帮助

获取单个脚本的详细帮助：

```bash
# 显示特定脚本的帮助
vx run test-pkgs -H
vx run deploy --script-help

# 列出所有可用脚本
vx run --list
```

### 3. 灵活的脚本定义

使用 `{{args}}` 获得最大灵活性：

```toml
[scripts]
# 现代方法：灵活的参数处理
test-pkgs = "cargo test {{args}}"
lint = "eslint {{args}}"
build = "docker build {{args}}"

# 传统方法：仍然有效但有限制
test-simple = "cargo test"
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

### 从复杂的变通方法迁移

**之前：**
```toml
[scripts]
test-unit = "cargo test --lib"
test-integration = "cargo test --test integration"
test-package = "cargo test -p"  # 不完整，需要手动编辑
```

**之后：**
```toml
[scripts]
test = "cargo test {{args}}"
```

**用法：**
```bash
vx run test --lib                    # 单元测试
vx run test --test integration       # 集成测试
vx run test -p my-package --lib      # 包特定测试
```

## 最佳实践

### 1. 使用 `{{args}}` 进行工具集成

对于有许多命令行选项的工具：

```toml
[scripts]
# ✅ 灵活 - 支持任何 cargo test 参数
test = "cargo test {{args}}"

# ✅ 灵活 - 支持任何 eslint 参数
lint = "eslint {{args}}"

# ✅ 灵活 - 支持任何 docker build 参数
build = "docker build {{args}}"

# ❌ 僵化 - 只适用于特定用例
test-lib = "cargo test --lib"
```

### 2. 提供脚本文档

添加注释来解释脚本用法：

```toml
[scripts]
# 使用灵活参数运行测试
# 示例：
#   vx run test -p my-package --lib
#   vx run test --test integration
test = "cargo test {{args}}"

# 使用灵活选项检查代码
# 示例：
#   vx run lint --fix
#   vx run lint --ext .js,.ts src/
lint = "eslint {{args}}"
```

### 3. 与环境变量结合

```toml
[env]
RUST_LOG = "debug"
CARGO_TERM_COLOR = "always"

[scripts]
test = "cargo test {{args}}"
test-quiet = "RUST_LOG=error cargo test {{args}}"
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

### 与外部工具集成

```toml
[scripts]
# 完美适配有许多选项的工具
prettier = "npx prettier {{args}}"
webpack = "npx webpack {{args}}"
terraform = "terraform {{args}}"
kubectl = "kubectl {{args}}"
```

## 故障排除

### 参数不工作

**问题**：参数没有传递给脚本。

**解决方案**：确保您的脚本使用 `{{args}}`：

```toml
# ❌ 不会接收参数
test = "cargo test"

# ✅ 会接收所有参数
test = "cargo test {{args}}"
```

### 复杂参数

**问题**：带引号或特殊字符的非常复杂的参数。

**解决方案**：使用 `--` 分隔符：

```bash
# 对于复杂情况，使用 -- 分隔符
vx run build -- --build-arg "VERSION=1.0.0" --target production
```

### 脚本帮助未显示

**问题**：`vx run script --help` 显示全局帮助而不是脚本帮助。

**解决方案**：使用 `-H` 代替：

```bash
# ✅ 显示脚本特定帮助
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
```

用法：
```bash
vx run test -p my-crate --lib
vx run clippy -- -D warnings
vx run doc --open --no-deps
```

### JavaScript/TypeScript 开发

```toml
[scripts]
lint = "eslint {{args}}"
format = "prettier {{args}}"
test = "jest {{args}}"
build = "webpack {{args}}"
```

用法：
```bash
vx run lint --fix --ext .js,.ts src/
vx run format --write "src/**/*.{js,ts}"
vx run test --watch --coverage
vx run build --mode production
```

### Docker 开发

```toml
[scripts]
build = "docker build {{args}}"
run = "docker run {{args}}"
compose = "docker-compose {{args}}"
```

用法：
```bash
vx run build -t myapp:latest --platform linux/amd64 .
vx run run -it --rm -p 3000:3000 myapp:latest
vx run compose up -d --scale web=3
```

## 另请参阅

- [run 命令参考](../cli/run_zh.md) - 完整命令文档
- [vx.toml 配置](../config/vx-toml.md) - 配置文件参考
- [变量插值](../config/vx-toml.md#variable-interpolation) - 高级变量用法