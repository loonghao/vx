# RFC 0003: 动态脚本和参数系统

> **状态**: Implemented
> **作者**: vx team
> **创建日期**: 2024-12-29
> **目标版本**: v0.6.0

## 摘要

设计一个动态脚本系统，支持参数传递、变量插值、环境变量自动填充和配置继承，使 vx 的 `run` 命令和扩展系统更加灵活强大。

## 动机

### 当前限制

1. **参数传递受限**：`vx run test arg1 arg2` 只是简单拼接，无法定义位置参数、可选参数、默认值
2. **无变量插值**：脚本中无法使用 `{{variable}}` 语法引用变量
3. **环境变量手动管理**：需要手动设置环境变量，无法自动从配置文件加载
4. **无配置继承**：无法从远程或本地模板继承配置

### 行业最佳实践

| 工具 | 参数系统 | 变量插值 | 环境变量 | 配置继承 |
|------|----------|----------|----------|----------|
| **just** | 位置参数、默认值、可变参数 | `{{var}}` 语法 | `env()` 函数、`.env` 文件 | `import` 语句 |
| **mise** | usage spec 声明式参数 | Tera 模板 | 内置变量、`.env` 支持 | 无 |
| **cargo-make** | `${@}` 获取参数 | `${var}` 语法 | `[env]` 块、env_files | `extend` 属性 |
| **npm scripts** | `-- args` 传递 | 无 | `cross-env` | 无 |

## 设计方案

### 1. 参数系统

#### 1.1 声明式参数定义 (vx.toml)

```toml
[scripts.deploy]
run = "deploy.sh {{environment}} {{region}}"
description = "Deploy to cloud"

[scripts.deploy.args]
environment = { required = true, choices = ["dev", "staging", "prod"], help = "Target environment" }
region = { default = "us-east-1", help = "Cloud region" }
verbose = { type = "flag", short = "v", help = "Enable verbose output" }
services = { type = "array", help = "Services to deploy" }
```

#### 1.2 参数类型

| 类型 | 说明 | 示例 |
|------|------|------|
| `string` | 字符串参数（默认） | `name = { help = "Your name" }` |
| `flag` | 布尔标志 | `verbose = { type = "flag", short = "v" }` |
| `array` | 多值参数 | `files = { type = "array" }` |
| `number` | 数字参数 | `port = { type = "number", default = 8080 }` |

#### 1.3 参数属性

```toml
[scripts.test.args]
target = {
    required = true,           # 必需参数
    default = "all",           # 默认值
    choices = ["unit", "e2e"], # 可选值
    env = "TEST_TARGET",       # 环境变量映射
    short = "t",               # 短标志
    help = "Test target",      # 帮助文本
    pattern = "\\w+",          # 正则验证
}
```

#### 1.4 使用示例

```bash
# 位置参数
vx run deploy prod us-west-2

# 命名参数
vx run deploy --environment prod --region us-west-2

# 标志
vx run deploy prod -v --dry-run

# 数组参数
vx run deploy prod --services api --services web

# 查看帮助
vx run deploy --help
```

### 2. 变量插值

#### 2.1 插值语法

```toml
[vars]
project = "my-app"
version = "1.0.0"
build_dir = "dist/{{project}}"

[scripts]
build = "cargo build --release -p {{project}}"
tag = "git tag v{{version}}"
clean = "rm -rf {{build_dir}}"
```

#### 2.2 内置变量

| 变量 | 说明 |
|------|------|
| `{{vx.version}}` | vx 版本 |
| `{{vx.home}}` | vx 主目录 |
| `{{vx.runtimes}}` | 运行时目录 |
| `{{project.root}}` | 项目根目录 |
| `{{project.name}}` | 项目名称 |
| `{{os.name}}` | 操作系统名称 |
| `{{os.arch}}` | CPU 架构 |
| `{{env.VAR}}` | 环境变量 |
| `{{date}}` | 当前日期 (YYYY-MM-DD) |
| `{{timestamp}}` | Unix 时间戳 |

#### 2.3 命令插值

```toml
[vars]
git_hash = { cmd = "git rev-parse --short HEAD" }
branch = { cmd = "git branch --show-current" }

[scripts]
info = "echo 'Building {{project}} ({{git_hash}}) on {{branch}}'"
```

#### 2.4 条件表达式

```toml
[vars]
compiler = { if = "os.name == 'windows'", then = "cl", else = "gcc" }

[scripts]
compile = "{{compiler}} -o app main.c"
```

### 3. 环境变量系统

#### 3.1 环境变量定义

```toml
[env]
NODE_ENV = "development"
API_URL = "http://localhost:3000"
DEBUG = { value = "true", if = "env.CI != 'true'" }

# 从命令获取
DATABASE_URL = { cmd = "vault read -field=url secret/db" }

# 条件设置
LOG_LEVEL = { value = "debug", if = "env.DEBUG == 'true'" }
```

#### 3.2 .env 文件支持

```toml
[env]
dotenv = true                    # 加载 .env
dotenv_files = [".env.local"]    # 额外的 .env 文件
dotenv_required = false          # .env 文件是否必须存在
```

#### 3.3 环境变量优先级

1. 命令行 `--env KEY=VALUE`
2. 脚本级 `[scripts.xxx.env]`
3. 全局 `[env]`
4. `.env` 文件
5. 系统环境变量

#### 3.4 脚本级环境变量

```toml
[scripts.dev]
run = "npm run dev"
env = { NODE_ENV = "development", PORT = "3000" }

[scripts.prod]
run = "npm run start"
env = { NODE_ENV = "production" }
```

### 4. 配置继承

#### 4.1 本地继承

```toml
# vx.toml
extends = ["./configs/base.toml", "./configs/dev.toml"]

[scripts]
# 覆盖或添加脚本
```

#### 4.2 远程继承

```toml
# 从 GitHub 继承
extends = ["github:company/vx-configs/node.toml@v1.0"]

# 从 URL 继承
extends = ["https://example.com/configs/base.toml"]
```

#### 4.3 继承合并规则

- **scripts**: 深度合并，子配置覆盖父配置
- **env**: 深度合并，子配置优先
- **vars**: 深度合并，子配置优先
- **tools**: 数组合并，去重

```toml
# base.toml
[scripts]
build = "cargo build"
test = "cargo test"

# vx.toml
extends = ["./base.toml"]

[scripts]
build = "cargo build --release"  # 覆盖
lint = "cargo clippy"            # 新增
# test 继承自 base.toml
```

### 5. 扩展系统增强

#### 5.1 扩展参数定义

```toml
# vx-extension.toml
[extension]
name = "docker-compose"

[args]
profile = { default = "dev", choices = ["dev", "prod"], help = "Docker profile" }
detach = { type = "flag", short = "d", help = "Run in background" }
services = { type = "array", help = "Services to start" }

[entrypoint]
main = "main.py"

[commands.up]
script = "up.py"
args_inherit = true  # 继承全局参数
```

#### 5.2 扩展执行

```bash
# 传递参数
vx x docker-compose up --profile prod -d
vx x docker-compose up web api --detach

# 查看帮助
vx x docker-compose --help
vx x docker-compose up --help
```

### 6. CLI 增强

#### 6.1 全局选项

```bash
vx run <script> [args...]
  --env KEY=VALUE    # 设置环境变量
  --var KEY=VALUE    # 设置变量
  --dry-run          # 显示将执行的命令
  --verbose          # 详细输出
```

#### 6.2 帮助系统

```bash
$ vx run deploy --help

Usage: vx run deploy <environment> [options]

Deploy to cloud

Arguments:
  environment    Target environment (required)
                 Choices: dev, staging, prod

Options:
  --region       Cloud region [default: us-east-1]
  -v, --verbose  Enable verbose output
  --services     Services to deploy (can be repeated)
  --dry-run      Show what would be deployed
  -h, --help     Show this help
```

## 实现计划

### Phase 1: 参数系统 (v0.6.0) ✅

- [x] 参数解析器 (`vx-args` crate)
- [x] 声明式参数定义
- [x] 位置参数和命名参数
- [x] 参数验证和默认值
- [x] 自动帮助生成

### Phase 2: 变量插值 (v0.6.1) ✅

- [x] 变量解析器
- [x] 内置变量支持
- [x] 命令插值
- [x] 循环引用检测

### Phase 3: 环境变量 (v0.6.2) ✅

- [x] `[env]` 配置块
- [x] `.env` 文件支持
- [x] 环境变量优先级
- [x] 脚本级环境变量

### Phase 4: 配置继承 (v0.7.0) ✅

- [x] 本地文件继承
- [x] 远程配置继承 (GitHub, URL)
- [x] 合并规则实现
- [x] 缓存机制

### Phase 5: 扩展增强 (v0.7.1) ✅

- [x] 扩展参数系统
- [x] 参数继承
- [x] 帮助系统集成

## 向后兼容性

1. **现有脚本兼容**：无参数定义的脚本保持原有行为
2. **渐进式采用**：新特性可选启用
3. **配置版本**：通过 `config_version` 字段区分

```toml
config_version = "2"  # 启用新特性

[scripts]
# 新语法
```

## 参考资料

- [just - Command runner](https://github.com/casey/just)
- [mise - Tasks](https://mise.jdx.dev/tasks/)
- [cargo-make](https://github.com/sagiegurari/cargo-make)
- [npm scripts](https://docs.npmjs.com/cli/v10/using-npm/scripts)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2024-12-29 | Draft | 初始草案 |
| 2024-12-29 | Implemented | 完成所有阶段实现 |
