# RFC 0007: 增强 Hooks 系统

> **状态**: Draft
> **作者**: vx team
> **创建日期**: 2025-12-30
> **目标版本**: v0.7.0

## 摘要

本 RFC 提出一个全面增强的 Hooks 系统设计，包括：

1. **平台相关的 Hooks** - 支持根据操作系统执行不同的 hook 逻辑
2. **Scripts 生命周期 Hooks** - 为 scripts 添加 `pre_run`/`post_run` 支持
3. **虚拟环境执行策略** - 类似 nox/tox 的隔离执行环境
4. **Extension 依赖隔离** - 自动管理扩展的依赖环境
5. **Hook 执行环境配置** - 完整的执行上下文控制

## 动机

### 当前状态分析

目前 vx 的 hooks 系统存在以下限制：

| 功能 | 当前状态 | 问题 |
|------|----------|------|
| 平台相关 hooks | ❌ 不支持 | 无法为 Windows/Linux/macOS 指定不同脚本 |
| Scripts pre/post hooks | ❌ 不支持 | 无法在脚本执行前后自动运行任务 |
| 虚拟环境执行 | ❌ 不支持 | nox/tox 等工具需要手动管理环境 |
| Extension 依赖隔离 | ⚠️ 部分支持 | 依赖需要用户手动安装，无自动隔离 |
| 执行环境配置 | ⚠️ 基础支持 | 缺少超时、重试、条件执行等高级功能 |

### 行业最佳实践对比

| 工具 | 平台 hooks | 生命周期 hooks | 虚拟环境 | 依赖隔离 |
|------|-----------|---------------|---------|---------|
| **npm scripts** | ❌ | ✅ pre/post | ❌ | ❌ |
| **just** | ✅ `[windows]` | ❌ | ❌ | ❌ |
| **mise** | ✅ `[os]` | ✅ hooks | ❌ | ❌ |
| **nox** | ❌ | N/A | ✅ | ✅ |
| **tox** | ❌ | N/A | ✅ | ✅ |
| **cargo-make** | ✅ | ✅ | ❌ | ❌ |
| **Makefile** | ⚠️ 条件 | ❌ | ❌ | ❌ |

### 需求场景

1. **跨平台开发** - 团队成员使用不同操作系统，需要平台特定的 setup 脚本
2. **CI/CD 集成** - 不同 CI 环境需要不同的 hooks 配置
3. **测试隔离** - 类似 nox/tox，需要在隔离环境中运行测试
4. **扩展安全** - 扩展的依赖不应污染全局或项目环境

## 设计方案

### 1. 平台相关的 Hooks

#### 1.1 基础语法

```toml
# 简单形式 - 所有平台使用相同命令
[hooks]
post_setup = "echo 'Setup complete'"

# 平台特定形式
[hooks.post_setup]
windows = "scripts/setup.ps1"
linux = "scripts/setup.sh"
darwin = "scripts/setup.sh"
unix = "scripts/setup.sh"  # linux + darwin 的别名
default = "echo 'No platform-specific setup'"
```

#### 1.2 完整配置形式

```toml
[hooks.post_install]
# 平台特定配置
[hooks.post_install.windows]
run = "scripts/post-install.ps1"
shell = "pwsh"
env = { INSTALL_MODE = "windows" }

[hooks.post_install.unix]
run = "scripts/post-install.sh"
shell = "bash"
env = { INSTALL_MODE = "unix" }

# 通用配置（所有平台共享）
[hooks.post_install._common]
timeout = "5m"
continue_on_error = false
```

#### 1.3 平台标识符

| 标识符 | 匹配平台 | 说明 |
|--------|---------|------|
| `windows` | Windows | Windows 系统 |
| `linux` | Linux | Linux 系统 |
| `darwin` | macOS | macOS 系统 |
| `unix` | Linux + macOS | Unix-like 系统别名 |
| `default` | 任意 | 无匹配时的默认值 |

#### 1.4 平台匹配优先级

```
1. 精确匹配 (windows/linux/darwin)
2. 别名匹配 (unix)
3. 默认值 (default)
4. 无操作 (如果都没有定义)
```

### 2. Scripts 生命周期 Hooks

#### 2.1 内联 pre/post hooks

```toml
[scripts.test]
run = "pytest"
pre_run = "echo 'Running tests...'"
post_run = "echo 'Tests completed'"

# 多命令形式
[scripts.build]
run = "cargo build --release"
pre_run = ["cargo fmt --check", "cargo clippy"]
post_run = ["strip target/release/app", "echo 'Build done'"]
```

#### 2.2 平台特定的 script hooks

```toml
[scripts.deploy]
run = "deploy.sh"

[scripts.deploy.pre_run]
windows = "scripts/pre-deploy.ps1"
unix = "scripts/pre-deploy.sh"

[scripts.deploy.post_run]
windows = "scripts/post-deploy.ps1"
unix = "scripts/post-deploy.sh"
```

#### 2.3 引用其他 scripts 作为 hooks

```toml
[scripts]
lint = "eslint ."
typecheck = "tsc --noEmit"
test = "jest"

[scripts.ci]
run = "npm run build"
pre_run = ["@lint", "@typecheck"]  # @ 前缀引用其他 script
post_run = ["@test"]
```

#### 2.4 条件执行

```toml
[scripts.release]
run = "semantic-release"
pre_run = { run = "npm test", if = "env.CI == 'true'" }
post_run = { run = "notify-slack", if = "env.SLACK_WEBHOOK != ''" }
```

### 3. 虚拟环境执行策略

#### 3.1 设计理念

借鉴 nox/tox 的设计，提供三种执行模式：

| 模式 | 说明 | 适用场景 |
|------|------|---------|
| `inherit` | 继承当前环境 | 默认行为，快速执行 |
| `isolated` | 创建隔离虚拟环境 | 测试、CI、依赖冲突场景 |
| `managed` | 使用工具自身管理的环境 | nox/tox 等自带环境管理的工具 |

#### 3.2 基础配置

```toml
[scripts.test]
run = "pytest"
venv = ".venv"  # 使用指定虚拟环境

[scripts.test-isolated]
run = "pytest"
venv = {
    mode = "isolated",      # 创建隔离环境
    path = ".vx/venvs/test",
    python = "3.12",
    requirements = ["pytest", "pytest-cov"]
}

[scripts.nox-test]
run = "nox -s tests"
venv = { mode = "managed" }  # nox 自己管理环境
```

#### 3.3 完整虚拟环境配置

```toml
[scripts.integration-test]
run = "pytest tests/integration"

[scripts.integration-test.venv]
mode = "isolated"
path = ".vx/venvs/integration"
python = "3.11"  # 指定 Python 版本

# 依赖安装
requirements = [
    "pytest>=7.0",
    "pytest-asyncio",
    "httpx",
]
requirements_files = ["requirements-test.txt"]

# 环境配置
env = { TESTING = "true" }
inherit_env = true  # 是否继承父环境变量

# 生命周期
recreate = false    # 每次执行是否重建环境
cache = true        # 是否缓存环境
```

#### 3.4 会话式执行（类似 nox sessions）

```toml
# 定义可复用的虚拟环境模板
[venvs.test-base]
python = "3.11"
requirements = ["pytest", "pytest-cov"]

[venvs.lint]
python = "3.11"
requirements = ["ruff", "mypy"]

# 在 scripts 中引用
[scripts.test]
run = "pytest"
venv = "@test-base"  # 引用预定义环境

[scripts.lint]
run = "ruff check . && mypy ."
venv = "@lint"
```

#### 3.5 多 Python 版本测试（类似 tox）

```toml
[scripts.test-matrix]
run = "pytest"
matrix = [
    { python = "3.10", venv = ".vx/venvs/py310" },
    { python = "3.11", venv = ".vx/venvs/py311" },
    { python = "3.12", venv = ".vx/venvs/py312" },
]
parallel = true  # 并行执行
```

### 4. Extension 依赖隔离

#### 4.1 扩展依赖声明

```toml
# vx-extension.toml
[extension]
name = "code-analyzer"
version = "1.0.0"

[runtime]
requires = "python >= 3.10"

# 依赖声明
[dependencies]
packages = ["ast-grep", "tree-sitter", "pyyaml>=6.0"]
requirements_file = "requirements.txt"

# 隔离配置
[dependencies.isolation]
enabled = true                          # 启用依赖隔离
mode = "venv"                           # venv | uv | conda
path = ".vx/extensions/code-analyzer/venv"
cache = true                            # 缓存环境
```

#### 4.2 隔离模式

| 模式 | 说明 | 优点 | 缺点 |
|------|------|------|------|
| `venv` | Python venv | 标准、兼容性好 | 创建较慢 |
| `uv` | uv 虚拟环境 | 快速、现代 | 需要 uv |
| `conda` | Conda 环境 | 支持非 Python 依赖 | 需要 conda |
| `none` | 不隔离 | 最快 | 可能冲突 |

#### 4.3 自动依赖安装流程

```
vx x code-analyzer analyze src/

1. 检查扩展配置
2. 检测依赖隔离配置
3. 如果启用隔离：
   a. 检查虚拟环境是否存在
   b. 如果不存在或过期，创建/更新
   c. 安装依赖
4. 在隔离环境中执行扩展
```

#### 4.4 扩展依赖锁定

```toml
# vx-extension.lock (自动生成)
[metadata]
generated_at = "2025-12-30T10:00:00Z"
python_version = "3.11.0"
platform = "linux-x86_64"

[packages]
ast-grep = { version = "0.12.0", hash = "sha256:..." }
tree-sitter = { version = "0.20.4", hash = "sha256:..." }
pyyaml = { version = "6.0.1", hash = "sha256:..." }
```

### 5. Hook 执行环境配置

#### 5.1 完整执行配置

```toml
[hooks.post_setup]
# 基础配置
run = "scripts/setup.py"
shell = "python"  # 或 bash, pwsh, sh, cmd

# 工作目录
cwd = "scripts"

# 环境变量
env = {
    DEBUG = "true",
    CONFIG_PATH = "{{project.root}}/config"
}
inherit_env = true  # 继承父进程环境变量

# 执行控制
timeout = "10m"           # 超时时间
retries = 3               # 重试次数
retry_delay = "5s"        # 重试间隔
continue_on_error = false # 失败是否继续

# 条件执行
if = "env.CI != 'true'"   # 条件表达式
unless = "file.exists('.skip-setup')"  # 反向条件

# 输出控制
quiet = false             # 静默模式
capture_output = false    # 捕获输出到变量
```

#### 5.2 Shell 配置

```toml
[hooks.build]
run = "build.ps1"

[hooks.build.shell]
type = "pwsh"             # bash, sh, pwsh, cmd, python, node
args = ["-NoProfile"]     # shell 参数
encoding = "utf-8"        # 输出编码
```

#### 5.3 条件表达式语法

```toml
# 环境变量条件
if = "env.CI == 'true'"
if = "env.NODE_ENV != 'production'"

# 文件存在条件
if = "file.exists('package.json')"
if = "file.missing('.env')"

# 平台条件
if = "os.name == 'windows'"
if = "os.arch == 'x86_64'"

# 组合条件
if = "env.CI == 'true' && os.name == 'linux'"
if = "env.SKIP_TESTS != 'true' || env.FORCE_TESTS == 'true'"
```

#### 5.4 输出捕获和链式执行

```toml
[hooks.version_check]
run = "git describe --tags"
capture_output = "GIT_VERSION"  # 捕获到环境变量

[hooks.build]
run = "cargo build --release"
env = { VERSION = "{{env.GIT_VERSION}}" }  # 使用捕获的输出
depends_on = ["version_check"]  # 依赖其他 hook
```

### 6. 全局 Hooks 配置

#### 6.1 完整 hooks 配置示例

```toml
# vx.toml

# ============================================
# 全局 Hooks
# ============================================
[hooks]
# 项目进入/离开
enter = "vx sync --check"
leave = { run = "echo 'Leaving project'", quiet = true }

# Setup 生命周期
pre_setup = "echo 'Starting setup...'"
post_setup = ["vx run db:migrate", "vx run seed"]

# Install 生命周期（runtime 安装）
pre_install = { run = "echo 'Installing...'", if = "env.VERBOSE == 'true'" }
post_install = "scripts/post-install.sh"

# Git hooks
pre_commit = "vx run lint && vx run test:unit"
commit_msg = "scripts/validate-commit-msg.sh"

# ============================================
# 平台特定 Hooks
# ============================================
[hooks.post_setup.windows]
run = "scripts/setup-windows.ps1"
shell = "pwsh"

[hooks.post_setup.unix]
run = "scripts/setup-unix.sh"
shell = "bash"

# ============================================
# Scripts with Hooks
# ============================================
[scripts.test]
run = "pytest"
description = "Run tests"
pre_run = ["@lint", "@typecheck"]
post_run = "coverage report"

[scripts.test.pre_run.windows]
run = "scripts/pre-test.ps1"

[scripts.test.pre_run.unix]
run = "scripts/pre-test.sh"

[scripts.deploy]
run = "deploy.sh {{environment}}"
pre_run = { run = "vx run test", if = "env.SKIP_TESTS != 'true'" }
post_run = { run = "notify-team", if = "env.NOTIFY == 'true'" }

[scripts.deploy.args]
environment = { required = true, choices = ["dev", "staging", "prod"] }

# ============================================
# 虚拟环境配置
# ============================================
[venvs.test]
python = "3.11"
requirements = ["pytest", "pytest-cov", "pytest-asyncio"]

[venvs.lint]
python = "3.11"
requirements = ["ruff", "mypy", "black"]

[scripts.test-isolated]
run = "pytest"
venv = "@test"

[scripts.lint]
run = "ruff check . && mypy ."
venv = "@lint"

# ============================================
# 多版本测试
# ============================================
[scripts.test-all]
run = "pytest"
matrix = [
    { python = "3.10" },
    { python = "3.11" },
    { python = "3.12" },
]
parallel = true
```

## 实现架构

### Crate 结构

```
crates/vx-hooks/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── types.rs           # Hook 类型定义
│   ├── platform.rs        # 平台检测和匹配
│   ├── executor.rs        # Hook 执行器
│   ├── condition.rs       # 条件表达式解析
│   ├── venv/              # 虚拟环境管理
│   │   ├── mod.rs
│   │   ├── manager.rs     # 环境管理器
│   │   ├── python.rs      # Python venv
│   │   ├── uv.rs          # uv 环境
│   │   └── conda.rs       # Conda 环境
│   └── scripts/           # Script hooks
│       ├── mod.rs
│       └── lifecycle.rs   # 生命周期管理
└── tests/
    ├── platform_tests.rs
    ├── executor_tests.rs
    └── venv_tests.rs
```

### 核心 Trait 设计

```rust
/// Hook 执行上下文
pub struct HookContext {
    pub working_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub platform: Platform,
    pub variables: HashMap<String, String>,
}

/// Hook 执行结果
pub struct HookResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub duration: Duration,
}

/// Hook 执行器 trait
#[async_trait]
pub trait HookExecutor {
    async fn execute(&self, hook: &Hook, ctx: &HookContext) -> Result<HookResult>;
    async fn should_run(&self, hook: &Hook, ctx: &HookContext) -> bool;
}

/// 虚拟环境管理器 trait
#[async_trait]
pub trait VenvManager {
    async fn create(&self, config: &VenvConfig) -> Result<PathBuf>;
    async fn exists(&self, path: &Path) -> bool;
    async fn install_deps(&self, path: &Path, deps: &[String]) -> Result<()>;
    async fn activate_env(&self, path: &Path) -> Result<HashMap<String, String>>;
}

/// 平台匹配器
pub struct PlatformMatcher {
    current: Platform,
}

impl PlatformMatcher {
    pub fn matches(&self, target: &str) -> bool;
    pub fn select<T>(&self, options: &PlatformOptions<T>) -> Option<&T>;
}
```

### 执行流程

```
Script 执行流程:
┌─────────────────────────────────────────────────────────────┐
│  vx run test                                                │
├─────────────────────────────────────────────────────────────┤
│  1. 解析 script 配置                                         │
│  2. 检查虚拟环境配置                                          │
│     ├─ mode: inherit → 使用当前环境                          │
│     ├─ mode: isolated → 创建/激活隔离环境                     │
│     └─ mode: managed → 跳过环境管理                          │
│  3. 执行 pre_run hooks                                       │
│     ├─ 平台匹配                                              │
│     ├─ 条件检查                                              │
│     └─ 执行命令                                              │
│  4. 执行主命令                                               │
│  5. 执行 post_run hooks                                      │
│     ├─ 平台匹配                                              │
│     ├─ 条件检查                                              │
│     └─ 执行命令                                              │
└─────────────────────────────────────────────────────────────┘

Extension 执行流程:
┌─────────────────────────────────────────────────────────────┐
│  vx x code-analyzer analyze                                 │
├─────────────────────────────────────────────────────────────┤
│  1. 加载扩展配置                                             │
│  2. 检查依赖隔离配置                                          │
│     ├─ isolation.enabled = true                             │
│     └─ 确保虚拟环境存在且依赖已安装                           │
│  3. 执行 pre-run hook                                        │
│  4. 在隔离环境中执行扩展                                      │
│  5. 执行 post-run hook                                       │
└─────────────────────────────────────────────────────────────┘
```

## 向后兼容性

### 兼容策略

1. **现有 hooks 兼容** - 简单字符串形式的 hooks 保持原有行为
2. **渐进式采用** - 新特性（平台特定、虚拟环境）都是可选的
3. **默认行为不变** - 不指定平台时使用通用配置

### 迁移示例

```toml
# 旧配置（继续工作）
[hooks]
post_setup = "echo 'done'"

# 新配置（增强功能）
[hooks.post_setup]
windows = "setup.ps1"
unix = "setup.sh"
```

## 实现计划

### Phase 1: 平台相关 Hooks (v0.7.0)

- [ ] 平台检测和匹配逻辑
- [ ] 平台特定 hook 配置解析
- [ ] `[hooks.xxx.windows/linux/darwin/unix]` 支持
- [ ] 单元测试

### Phase 2: Scripts 生命周期 (v0.7.1)

- [ ] `pre_run` / `post_run` 配置解析
- [ ] Script 引用语法 (`@script_name`)
- [ ] 条件执行支持
- [ ] 平台特定 script hooks

### Phase 3: 虚拟环境执行 (v0.8.0)

- [ ] `vx-hooks::venv` 模块
- [ ] Python venv 管理器
- [ ] uv 环境管理器
- [ ] `[venvs]` 配置块
- [ ] Script 虚拟环境集成

### Phase 4: Extension 依赖隔离 (v0.8.1)

- [ ] Extension 依赖配置解析
- [ ] 自动虚拟环境创建
- [ ] 依赖安装和缓存
- [ ] 锁文件支持

### Phase 5: 高级执行配置 (v0.9.0)

- [ ] 超时和重试机制
- [ ] 输出捕获
- [ ] 条件表达式完整支持
- [ ] Hook 依赖链

### Phase 6: 多版本测试矩阵 (v0.9.1)

- [ ] Matrix 配置解析
- [ ] 并行执行支持
- [ ] 结果聚合和报告

## 参考资料

- [nox - Flexible test automation](https://nox.thea.codes/)
- [tox - virtualenv management](https://tox.wiki/)
- [just - Command runner](https://github.com/casey/just)
- [mise - Tasks and hooks](https://mise.jdx.dev/tasks/)
- [npm scripts lifecycle](https://docs.npmjs.com/cli/v10/using-npm/scripts)
- [cargo-make](https://github.com/sagiegurari/cargo-make)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-12-30 | Draft | 初始草案 |
