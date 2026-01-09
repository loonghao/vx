# RFC 0018: Extended Provider Manifest Schema

> **状态**: Partially Implemented ✅
> **作者**: vx team
> **创建日期**: 2026-01-09
> **目标版本**: v0.10.0 ~ v1.0.0
> **依赖**: RFC 0012 (Provider Manifest), RFC 0017 (Declarative RuntimeMap)
> **实现日期**: 2026-01-09

## 摘要

扩展 `provider.toml` schema，使其成为完整的声明式配置系统，遵循 Unix Philosophy 的设计原则。新增环境变量、版本检测、健康检查、Shell 集成、镜像配置等高级特性。

## 动机

### Unix Philosophy 原则

vx 的设计应遵循 Unix Philosophy：

| 原则 | 应用 |
|------|------|
| **Do one thing well** | 每个 runtime 专注单一职责 |
| **Composability** | 工具可以组合使用 |
| **Text streams** | 支持标准输入输出 |
| **Configuration over code** | 配置文件驱动行为 |
| **Separation of mechanism and policy** | 机制与策略分离 |

### 当前 provider.toml 的局限

```toml
# 当前只支持基本配置
[[runtimes]]
name = "node"
executable = "node"
bundled_with = "node"  # ✅
aliases = ["nodejs"]   # ✅

# ❌ 缺失的关键能力
# - 环境变量配置
# - 版本检测命令
# - 健康检查
# - Shell 补全
# - 镜像源
# - 自定义命令
```

## 详细设计

### Schema 总览

```
provider.toml
├── [provider]                 # Provider 元数据
│   ├── name, description...
│   └── [provider.config]      # 🆕 Provider 级别配置
│
└── [[runtimes]]               # Runtime 定义
    ├── name, executable...    # 基本字段（已有）
    ├── [runtimes.env]         # 🆕 环境变量
    ├── [runtimes.detection]   # 🆕 版本检测
    ├── [runtimes.health]      # 🆕 健康检查
    ├── [runtimes.shim]        # 🆕 Shim 策略
    ├── [runtimes.shell]       # 🆕 Shell 集成
    ├── [runtimes.commands]    # 🆕 自定义命令
    ├── [runtimes.cache]       # 🆕 缓存策略
    ├── [runtimes.mirrors]     # 🆕 镜像配置
    ├── [runtimes.toolchain]   # 🆕 工具链组合
    ├── [runtimes.output]      # 🆕 输出格式
    └── [runtimes.pipes]       # 🆕 管道支持
```

### 1. 环境变量配置 (`[runtimes.env]`)

支持静态、动态和条件环境变量：

```toml
[[runtimes]]
name = "node"

[runtimes.env]
# 静态环境变量
NODE_ENV = "production"

# 动态环境变量（使用模板）
PATH = "{install_dir}/bin:{PATH}"
NODE_PATH = "{install_dir}/lib/node_modules"

# 条件环境变量（版本相关）
[runtimes.env.when.">=18"]
NODE_OPTIONS = "--experimental-vm-modules"

[runtimes.env.when."<16"]
NODE_OPTIONS = "--experimental-modules"
```

**模板变量**：
- `{install_dir}` - 安装目录
- `{version}` - 当前版本
- `{executable}` - 可执行文件路径
- `{PATH}` - 原始 PATH
- `{env:VAR}` - 引用其他环境变量

### 2. 版本检测 (`[runtimes.detection]`)

声明如何检测已安装的版本：

```toml
[[runtimes]]
name = "node"

[runtimes.detection]
# 版本检测命令
command = "{executable} --version"

# 版本解析正则（捕获组 1 为版本号）
pattern = "v?(\\d+\\.\\d+\\.\\d+)"

# 系统路径检测（查找已存在的安装）
system_paths = [
    "/usr/bin/node",
    "/usr/local/bin/node",
    "{env:NVM_DIR}/versions/node/*/bin/node",
    "C:\\Program Files\\nodejs\\node.exe"
]

# 环境变量提示（可能指示已安装）
env_hints = ["NODE_HOME", "NVM_DIR", "VOLTA_HOME"]

# 注册表路径（Windows）
registry_paths = [
    "HKLM\\SOFTWARE\\Node.js"
]
```

### 3. 健康检查 (`[runtimes.health]`)

验证安装是否正确工作：

```toml
[[runtimes]]
name = "node"

[runtimes.health]
# 简单命令检查
check_command = "{executable} -e 'console.log(process.version)'"
expected_pattern = "v\\d+\\.\\d+\\.\\d+"
timeout_ms = 5000

# 或使用退出码
exit_code = 0

# 可选：完整验证脚本
verify_script = "scripts/verify-node.sh"

# 检查时机
check_on = ["install", "activate", "run"]  # 默认只在 install
```

### 4. Shim 策略 (`[runtimes.shim]`)

控制 shim 的生成和行为：

```toml
[[runtimes]]
name = "node"

[runtimes.shim]
# Shim 类型
# - wrapper: 包装脚本，注入环境变量
# - symlink: 符号链接（最轻量）
# - passthrough: 直接传递，不做任何处理
type = "wrapper"

# 是否注入环境变量
inject_env = true

# 是否拦截子命令（vx node npm → vx npm）
intercept_subcommands = true

# 自定义包装脚本模板
template = "templates/node-wrapper.sh"

# 传递所有参数
pass_all_args = true

# 支持的 shell（用于生成不同格式的 shim）
shells = ["bash", "zsh", "fish", "powershell"]
```

### 5. Shell 集成 (`[runtimes.shell]`)

Shell 提示符和补全脚本：

```toml
[[runtimes]]
name = "node"

[runtimes.shell]
# 激活时的提示符格式
prompt_format = "(node-{version})"

# 激活/反激活脚本模板
activate_template = "templates/activate.sh"
deactivate_template = "templates/deactivate.sh"

# 自动补全脚本
[runtimes.shell.completions]
bash = "completions/node.bash"
zsh = "completions/_node"
fish = "completions/node.fish"
powershell = "completions/node.ps1"

# 别名定义（激活时设置）
[runtimes.shell.aliases]
n = "node"
nr = "npm run"
```

### 6. 自定义命令 (`[[runtimes.commands]]`)

Provider 提供的额外命令：

```toml
[[runtimes]]
name = "node"

# 内置命令
[[runtimes.commands]]
name = "repl"
description = "Start interactive REPL"
command = "{executable}"
category = "development"

[[runtimes.commands]]
name = "eval"
description = "Evaluate JavaScript expression"
command = "{executable} -e"
pass_args = true  # 将用户参数附加到命令后

[[runtimes.commands]]
name = "doctor"
description = "Diagnose Node.js installation"
script = "scripts/doctor.sh"  # 使用脚本而非命令
category = "maintenance"

[[runtimes.commands]]
name = "benchmark"
description = "Run performance benchmark"
command = "{executable} --expose-gc scripts/bench.js"
hidden = true  # 不在帮助中显示

# 使用方式：vx node doctor / vx node repl
```

### 7. 扩展 Hooks (`[runtimes.hooks]`)

完整的生命周期 hooks：

```toml
[[runtimes]]
name = "node"

[runtimes.hooks]
# 安装生命周期
pre_install = ["scripts/check-prereqs.sh"]
post_install = ["scripts/setup-npm-global.sh", "scripts/verify.sh"]
pre_uninstall = ["scripts/cleanup-cache.sh"]
post_uninstall = ["scripts/remove-shims.sh"]

# 激活生命周期
pre_activate = ["scripts/save-current-env.sh"]
post_activate = ["scripts/load-nvm-compat.sh"]
pre_deactivate = []
post_deactivate = ["scripts/restore-env.sh"]

# 执行生命周期
pre_run = ["scripts/check-version-compat.sh"]
post_run = []

# 错误处理 hooks
on_install_error = ["scripts/rollback.sh"]
on_version_not_found = ["scripts/suggest-alternatives.sh"]
on_health_check_fail = ["scripts/attempt-repair.sh"]

# Hook 行为配置
[runtimes.hooks.config]
fail_on_error = true        # hook 失败时是否终止
timeout_ms = 30000          # 单个 hook 超时
parallel = false            # 是否并行执行
```

### 8. 缓存策略 (`[runtimes.cache]`)

版本和下载缓存管理：

```toml
[[runtimes]]
name = "node"

[runtimes.cache]
# 版本列表缓存
versions_ttl = 3600              # 1 小时
versions_stale_while_revalidate = 86400  # 过期后仍可使用 1 天

# 下载包缓存
cache_downloads = true
downloads_retention_days = 30
max_cache_size_mb = 2048         # 最大缓存大小

# 共享缓存（跨项目）
shared_cache = true

# 缓存位置
# 默认使用 $VX_CACHE_DIR/{provider}/
custom_cache_dir = ""
```

### 9. 镜像配置 (`[[runtimes.mirrors]]`)

支持国内镜像和自定义源：

```toml
[[runtimes]]
name = "node"

# 镜像列表
[[runtimes.mirrors]]
name = "taobao"
region = "cn"
url = "https://npmmirror.com/mirrors/node"
priority = 100
enabled = true

[[runtimes.mirrors]]
name = "ustc"
region = "cn"
url = "https://mirrors.ustc.edu.cn/node"
priority = 90

[[runtimes.mirrors]]
name = "tsinghua"
region = "cn"
url = "https://mirrors.tuna.tsinghua.edu.cn/nodejs-release"
priority = 80

# 镜像策略
[runtimes.mirrors.strategy]
auto_detect = true          # 根据地理位置/网络自动选择
fallback = true             # 主镜像失败后尝试备用
parallel_probe = true       # 并行探测延迟
probe_timeout_ms = 3000     # 探测超时
```

### 10. 工具链组合 (`[runtimes.toolchain]`)

声明工具之间的关系：

```toml
[[runtimes]]
name = "node"

[runtimes.toolchain]
# 推荐的配套工具
recommended = [
    { runtime = "npm", version = "bundled", reason = "Default package manager" },
    { runtime = "corepack", version = "bundled", reason = "Package manager manager" }
]

# 可选工具
optional = [
    { runtime = "yarn", reason = "Alternative: Fast, reliable dependency management" },
    { runtime = "pnpm", reason = "Alternative: Fast, disk space efficient" },
    { runtime = "bun", reason = "Alternative: All-in-one JavaScript runtime" }
]

# 冲突检测
conflicts = [
    { runtime = "nvm", reason = "vx manages Node.js versions directly" },
    { runtime = "fnm", reason = "vx manages Node.js versions directly" },
    { runtime = "volta", reason = "vx manages Node.js versions directly" }
]

# 互补工具（自动建议）
complementary = [
    { runtime = "typescript", when = "project has tsconfig.json" },
    { runtime = "eslint", when = "project has .eslintrc" }
]
```

### 11. 输出格式 (`[runtimes.output]`)

遵循 Unix 文本流理念：

```toml
[[runtimes]]
name = "node"

[runtimes.output]
# 版本列表格式
list_format = "{version:>12} {lts:>10} {installed:>10} {date}"

# 当前版本格式
status_format = "{name} {version} ({path})"

# 支持的输出格式
formats = ["text", "json", "csv", "table"]

# 默认格式
default_format = "text"

# 机器可读标志
[runtimes.output.machine_flags]
list = "--json"
info = "--json"
status = "--json"

# 颜色配置
[runtimes.output.colors]
lts = "green"
current = "cyan"
installed = "blue"
outdated = "yellow"
error = "red"
```

### 12. 管道支持 (`[runtimes.pipes]`)

Unix 管道和重定向支持：

```toml
[[runtimes]]
name = "node"

[runtimes.pipes]
# 标准输入处理
stdin = true
stdin_encoding = "utf-8"

# 标准输出处理
stdout = true
stdout_encoding = "utf-8"

# 错误输出
stderr = true
stderr_encoding = "utf-8"

# 与其他工具的组合示例
[runtimes.pipes.examples]
# 这些示例会在 --help 中显示
filter_json = "vx node -e 'JSON.parse(input)' | jq '.name'"
process_csv = "cat data.csv | vx node scripts/process.js"
```

### 13. 子命令映射 (`[runtimes.subcommands]`)

支持 `vx node npm` 形式的调用：

```toml
[[runtimes]]
name = "node"

# 子命令映射
[runtimes.subcommands]
npm = { runtime = "npm", pass_args = true }
npx = { runtime = "npx", pass_args = true }
corepack = { runtime = "corepack", pass_args = true }

# 自定义子命令
[runtimes.subcommands.serve]
command = "{executable} -e 'require(\"http\").createServer((q,s)=>s.end()).listen(3000)'"
description = "Start a simple HTTP server"
```

### 14. Provider 级别配置 (`[provider.config]`)

全局配置应用于所有 runtimes：

```toml
[provider]
name = "node"
version = "1.0.0"  # Provider manifest 版本

[provider.config]
# 下载配置
parallel_downloads = 4
download_timeout_ms = 300000
retry_attempts = 3
retry_delay_ms = 1000

# 安全配置
verify_signatures = true
verify_checksums = true
allowed_sources = ["nodejs.org", "npmmirror.com"]

# 清理配置
auto_cleanup = true
cleanup_interval_days = 7

# 日志配置
log_level = "info"
log_format = "text"
```

## 实施计划

### Phase 1: 核心功能 (v0.10.0) - ✅ 已实现

必须优先实现，直接影响用户体验：

| 特性 | 描述 | 复杂度 | 状态 |
|------|------|--------|------|
| `[runtimes.env]` | 环境变量配置 | 中 | ✅ 已实现 |
| `[runtimes.detection]` | 版本检测 | 中 | ✅ 已实现 |
| `[runtimes.health]` | 健康检查 | 低 | ✅ 已实现 |
| 扩展 Hooks | 更多生命周期 | 中 | ✅ 已实现 |
| `[runtimes.mirrors]` | 镜像配置 | 中 | ✅ 已实现 |
| `[runtimes.cache]` | 缓存配置 | 中 | ✅ 已实现 |
| `priority` | 安装优先级 | 低 | ✅ 已实现 |
| `auto_installable` | 自动安装标志 | 低 | ✅ 已实现 |

### Phase 2: 用户体验 (v0.11.0) - ✅ 已实现

提升日常使用体验：

| 特性 | 描述 | 复杂度 | 状态 |
|------|------|--------|------|
| `[runtimes.shell]` | Shell 集成和补全 | 高 | ✅ 已实现 |
| `[runtimes.commands]` | 自定义命令 | 中 | ✅ 已实现 |
| `[runtimes.output]` | 输出格式化 | 低 | ✅ 已实现 |

### Phase 3: 高级特性 (v0.12.0)

企业级和高级用户需求：

| 特性 | 描述 | 复杂度 |
|------|------|--------|
| `[runtimes.shim]` | Shim 策略 | 高 |
| `[runtimes.subcommands]` | 子命令支持 | 中 |
| `[runtimes.toolchain]` | 工具链组合 | 中 |
| `[runtimes.cache]` | 缓存策略 | 中 |

### Phase 4: 生态完善 (v1.0.0)

完整的 Unix Philosophy 支持：

| 特性 | 描述 | 复杂度 |
|------|------|--------|
| `[runtimes.pipes]` | 管道支持 | 中 |
| `[provider.config]` | Provider 级别配置 | 低 |
| Provider 版本管理 | manifest 版本控制 | 低 |

## Rust 类型定义

### 新增类型

```rust
// vx-manifest/src/provider.rs

/// Environment variable configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EnvConfig {
    /// Static environment variables
    #[serde(flatten)]
    pub vars: HashMap<String, String>,

    /// Conditional environment variables (version-based)
    #[serde(default, rename = "when")]
    pub conditional: HashMap<String, HashMap<String, String>>,
}

/// Version detection configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DetectionConfig {
    /// Command to get version
    pub command: String,

    /// Regex pattern to extract version
    pub pattern: String,

    /// System paths to check
    #[serde(default)]
    pub system_paths: Vec<String>,

    /// Environment variable hints
    #[serde(default)]
    pub env_hints: Vec<String>,

    /// Windows registry paths
    #[serde(default)]
    pub registry_paths: Vec<String>,
}

/// Health check configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthConfig {
    /// Command to check health
    pub check_command: String,

    /// Expected output pattern
    #[serde(default)]
    pub expected_pattern: Option<String>,

    /// Expected exit code
    #[serde(default)]
    pub exit_code: Option<i32>,

    /// Timeout in milliseconds
    #[serde(default = "default_health_timeout")]
    pub timeout_ms: u64,

    /// Verification script path
    #[serde(default)]
    pub verify_script: Option<String>,

    /// When to check
    #[serde(default)]
    pub check_on: Vec<String>,
}

fn default_health_timeout() -> u64 { 5000 }

/// Mirror configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MirrorConfig {
    pub name: String,
    #[serde(default)]
    pub region: Option<String>,
    pub url: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

/// Custom command definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandDef {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub pass_args: bool,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub hidden: bool,
}
```

### 扩展 RuntimeDef

```rust
/// Extended Runtime definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RuntimeDef {
    // ... existing fields ...

    /// Environment configuration
    #[serde(default, rename = "env")]
    pub env_config: Option<EnvConfig>,

    /// Detection configuration
    #[serde(default)]
    pub detection: Option<DetectionConfig>,

    /// Health check configuration
    #[serde(default)]
    pub health: Option<HealthConfig>,

    /// Shim configuration
    #[serde(default)]
    pub shim: Option<ShimConfig>,

    /// Shell integration
    #[serde(default)]
    pub shell: Option<ShellConfig>,

    /// Custom commands
    #[serde(default)]
    pub commands: Vec<CommandDef>,

    /// Cache configuration
    #[serde(default)]
    pub cache: Option<CacheConfig>,

    /// Mirror configurations
    #[serde(default)]
    pub mirrors: Vec<MirrorConfig>,

    /// Toolchain configuration
    #[serde(default)]
    pub toolchain: Option<ToolchainConfig>,

    /// Output configuration
    #[serde(default)]
    pub output: Option<OutputConfig>,

    /// Subcommand mappings
    #[serde(default)]
    pub subcommands: HashMap<String, SubcommandDef>,
}
```

## 完整示例：Node.js Provider

```toml
# ============================================
# Node.js Provider Manifest
# vx provider manifest version: 1.0
# ============================================

[provider]
name = "node"
description = "JavaScript runtime built on Chrome's V8 engine"
homepage = "https://nodejs.org"
repository = "https://github.com/nodejs/node"
ecosystem = "nodejs"

[provider.config]
parallel_downloads = 4
verify_signatures = true

# ============================================
# Node.js Runtime
# ============================================

[[runtimes]]
name = "node"
description = "Node.js JavaScript runtime"
executable = "node"
aliases = ["nodejs"]
priority = 100
auto_installable = true

# Version source
[runtimes.versions]
source = "nodejs-org"
lts_pattern = "lts/*"
channels = ["lts", "current"]

# Environment variables
[runtimes.env]
PATH = "{install_dir}/bin:{PATH}"
NODE_PATH = "{install_dir}/lib/node_modules"

[runtimes.env.when.">=18"]
NODE_OPTIONS = "--experimental-vm-modules"

# Version detection
[runtimes.detection]
command = "{executable} --version"
pattern = "v?(\\d+\\.\\d+\\.\\d+)"
system_paths = ["/usr/bin/node", "/usr/local/bin/node"]
env_hints = ["NODE_HOME", "NVM_DIR"]

# Health check
[runtimes.health]
check_command = "{executable} -e 'console.log(process.version)'"
expected_pattern = "v\\d+\\.\\d+\\.\\d+"
timeout_ms = 5000

# Shim configuration
[runtimes.shim]
type = "wrapper"
inject_env = true

# Shell integration
[runtimes.shell]
prompt_format = "(node-{version})"

[runtimes.shell.completions]
bash = "completions/node.bash"
zsh = "completions/_node"

# Custom commands
[[runtimes.commands]]
name = "repl"
description = "Start interactive REPL"
command = "{executable}"

[[runtimes.commands]]
name = "doctor"
description = "Diagnose installation"
script = "scripts/doctor.sh"

# Hooks
[runtimes.hooks]
post_install = ["scripts/setup-npm-prefix.sh"]
on_install_error = ["scripts/rollback.sh"]

# Mirrors (crucial for Chinese users)
[[runtimes.mirrors]]
name = "taobao"
region = "cn"
url = "https://npmmirror.com/mirrors/node"
priority = 100

[[runtimes.mirrors]]
name = "ustc"
region = "cn"
url = "https://mirrors.ustc.edu.cn/node"
priority = 90

[runtimes.mirrors.strategy]
auto_detect = true
fallback = true

# Toolchain
[runtimes.toolchain]
recommended = [
    { runtime = "npm", version = "bundled" }
]
optional = [
    { runtime = "yarn" },
    { runtime = "pnpm" }
]
conflicts = [
    { runtime = "nvm", reason = "vx manages Node.js versions directly" }
]

# Subcommands
[runtimes.subcommands]
npm = { runtime = "npm" }
npx = { runtime = "npx" }

# Platforms
[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []

# Constraints
[[runtimes.constraints]]
when = "*"
recommends = [
    { runtime = "npm", version = "*", reason = "Default package manager" }
]
```

## 向后兼容性

1. **所有新字段都是可选的** - 现有 provider.toml 继续工作
2. **默认值保持现有行为** - 无新字段时行为不变
3. **渐进增强** - 可逐步添加新配置

## 替代方案

### 方案 A: 分离配置文件

使用多个文件：`provider.toml`, `env.toml`, `hooks.toml` 等。

**优点**: 更清晰的职责分离
**缺点**: 管理复杂，需要多文件同步

### 方案 B: YAML 格式

使用 YAML 替代 TOML。

**优点**: 更好的层级表达
**缺点**: 与现有生态不一致，TOML 已是 Rust 生态标准

**选择**: 保持单一 TOML 文件，通过良好的 section 组织实现清晰结构。

## 参考资料

- [RFC 0012: Provider Manifest](./0012-provider-manifest.md)
- [RFC 0017: Declarative RuntimeMap](./0017-declarative-runtime-map.md)
- [The Unix Philosophy](https://en.wikipedia.org/wiki/Unix_philosophy)
- [TOML Specification](https://toml.io/en/)

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-09 | Draft | 初始草案 |
| 2026-01-09 | Partially Implemented | Phase 1 核心功能已实现：EnvConfig, DetectionConfig, HealthConfig, HooksConfig, MirrorConfig, CacheConfig, priority, auto_installable |
| 2026-01-09 | RFC 0017 Integration | RuntimeMap 现在通过 from_manifests() 加载，deprecated 方法已标记，vx-cli 使用新的单一数据源方式 |
| 2026-01-09 | Phase 2 Complete | 用户体验功能已实现：CommandDef, OutputConfig, ShellConfig, ShellCompletionsConfig |

