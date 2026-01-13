# RFC 0020: CLI 结构简化与 Provider 测试拆分

> **状态**: Implemented (Phase 1 Complete)
> **作者**: vx team
> **创建日期**: 2026-01-12
> **目标版本**: v0.8.0

## 摘要

本 RFC 提议对 vx CLI 进行两项重构：

1. **Provider 测试逻辑拆分**：将 `vx-cli/src/commands/test.rs` 中的测试逻辑移至 `provider.toml` 声明式配置，利用已有的 `[runtimes.health]` 和 `[runtimes.detection]` 配置
2. **CLI 结构简化**：将 `cli.rs` 中的命令定义拆分到各自的命令模块，减少代码行数

## 动机

### 当前状态分析

#### 问题 1: `test.rs` 测试逻辑过于集中 (725 行)

当前 `vx-cli/src/commands/test.rs` 包含所有测试逻辑：

```rust
// 当前结构 - 所有逻辑都在一个文件中
struct RuntimeTester { ... }      // 测试单个 runtime
struct ProviderTester { ... }     // 测试所有 providers
struct LocalProviderTester { ... } // 测试本地 provider
struct ExtensionTester { ... }    // 测试远程扩展
```

**问题**：
- 测试逻辑硬编码，无法针对不同 provider 定制
- 添加新 provider 时需要修改 `test.rs`
- 无法利用 `provider.toml` 中已有的配置

#### 问题 2: `cli.rs` 代码量过大 (1400+ 行)

虽然命令处理已拆分到 `commands/` 目录，但 `cli.rs` 仍包含：

| 部分 | 行数 | 内容 |
|------|------|------|
| `Commands` 枚举 | ~600 | 每个命令的参数定义 |
| 子命令枚举 | ~300 | `ServicesCommand`, `CacheCommand` 等 |
| `CommandHandler::execute()` | ~500 | 巨大的 match 分发 |

### 需求分析

1. **Provider 测试应该声明式**：每个 provider 在 `provider.toml` 中声明自己的测试规则
2. **命令定义应该模块化**：每个命令模块定义自己的 Args 和子命令
3. **保持向后兼容**：现有的 `vx test` 命令行为不变

## 设计方案

### 方案 1: Provider 测试声明式配置

#### 1.1 扩展 `provider.toml` 添加 `[runtimes.test]` 配置

利用已有的 `[runtimes.health]` 和 `[runtimes.detection]` 配置，添加专门的测试配置：

```toml
# provider.toml
[[runtimes]]
name = "node"
executable = "node"

# 已有的健康检查配置 - 可复用于测试
[runtimes.health]
check_command = "{executable} -e 'console.log(process.version)'"
expected_pattern = "v\\d+\\.\\d+\\.\\d+"
exit_code = 0
timeout_ms = 5000

# 已有的检测配置 - 可复用于测试
[runtimes.detection]
command = "{executable} --version"
pattern = "v?(\\d+\\.\\d+\\.\\d+)"

# 新增：测试配置
[runtimes.test]
# 功能测试命令（默认使用 detection.command）
functional_commands = [
    { command = "{executable} --version", expect_success = true },
    { command = "{executable} -e 'console.log(1)'", expected_output = "1" },
]

# 安装验证命令
install_verification = [
    { command = "{executable} --version", expect_success = true },
]

# 测试超时（毫秒）
timeout_ms = 30000

# 跳过测试的条件
skip_on = ["ci-windows"]  # 在 Windows CI 上跳过

# 平台特定测试（可选）
[runtimes.test.platforms.windows]
functional_commands = [
    { command = "{executable} -e \"console.log(process.platform)\"", expected_output = "win32" },
]

[runtimes.test.platforms.unix]
functional_commands = [
    { command = "{executable} -e \"console.log(process.platform)\"", expected_output = "(linux|darwin)" },
]

# 内联测试脚本（可选，用于复杂测试逻辑）
[runtimes.test.inline_scripts]
# Windows 批处理脚本
windows = """
@echo off
{executable} --version
if %ERRORLEVEL% NEQ 0 exit /b 1
{executable} -e "console.log('test passed')"
"""

# Unix shell 脚本
unix = """
#!/bin/sh
set -e
{executable} --version
{executable} -e "console.log('test passed')"
"""
```

#### 1.2 在 `vx-manifest` 中添加 `TestConfig` 类型

```rust
// crates/vx-manifest/src/provider.rs

/// 测试配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TestConfig {
    /// 功能测试命令
    #[serde(default)]
    pub functional_commands: Vec<TestCommand>,
    
    /// 安装验证命令
    #[serde(default)]
    pub install_verification: Vec<TestCommand>,
    
    /// 测试超时（毫秒）
    #[serde(default = "default_test_timeout")]
    pub timeout_ms: u64,
    
    /// 跳过测试的条件
    #[serde(default)]
    pub skip_on: Vec<String>,
    
    /// 平台特定配置
    #[serde(default)]
    pub platforms: Option<TestPlatformConfig>,
    
    /// 内联测试脚本
    #[serde(default)]
    pub inline_scripts: Option<InlineScripts>,
}

/// 平台特定测试配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TestPlatformConfig {
    /// Windows 特定测试
    #[serde(default)]
    pub windows: Option<PlatformTestConfig>,
    
    /// Unix (Linux/macOS) 特定测试
    #[serde(default)]
    pub unix: Option<PlatformTestConfig>,
}

/// 单个平台的测试配置
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct PlatformTestConfig {
    /// 功能测试命令
    #[serde(default)]
    pub functional_commands: Vec<TestCommand>,
}

/// 内联测试脚本
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct InlineScripts {
    /// Windows 批处理脚本
    #[serde(default)]
    pub windows: Option<String>,
    
    /// Unix shell 脚本
    #[serde(default)]
    pub unix: Option<String>,
}

/// 测试命令定义
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestCommand {
    /// 命令模板（支持 {executable}, {version} 等变量）
    pub command: String,
    
    /// 期望成功
    #[serde(default = "default_true")]
    pub expect_success: bool,
    
    /// 期望输出模式（正则）
    #[serde(default)]
    pub expected_output: Option<String>,
    
    /// 期望退出码
    #[serde(default)]
    pub expected_exit_code: Option<i32>,
}

fn default_test_timeout() -> u64 {
    30000
}
```

#### 1.3 在 `vx-runtime` 中添加 `RuntimeTester` trait

```rust
// crates/vx-runtime/src/testing.rs

use crate::{Runtime, RuntimeContext};
use anyhow::Result;

/// 测试结果
#[derive(Debug, Clone)]
pub struct RuntimeTestResult {
    pub runtime_name: String,
    pub platform_supported: bool,
    pub installed: bool,
    pub system_available: bool,
    pub functional_tests: Vec<TestCaseResult>,
    pub overall_passed: bool,
}

#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Runtime 测试器
pub struct RuntimeTester<'a> {
    runtime: &'a dyn Runtime,
    ctx: &'a RuntimeContext,
}

impl<'a> RuntimeTester<'a> {
    pub fn new(runtime: &'a dyn Runtime, ctx: &'a RuntimeContext) -> Self {
        Self { runtime, ctx }
    }
    
    /// 运行所有测试
    pub async fn run_all(&self) -> Result<RuntimeTestResult> {
        let mut result = RuntimeTestResult {
            runtime_name: self.runtime.name().to_string(),
            platform_supported: self.test_platform_support(),
            installed: false,
            system_available: false,
            functional_tests: vec![],
            overall_passed: false,
        };
        
        if !result.platform_supported {
            return Ok(result);
        }
        
        // 检查安装状态
        result.installed = self.check_installed().await?;
        result.system_available = self.check_system_available();
        
        // 运行功能测试（从 provider.toml 配置读取）
        result.functional_tests = self.run_functional_tests().await?;
        
        result.overall_passed = result.platform_supported 
            && (result.installed || result.system_available)
            && result.functional_tests.iter().all(|t| t.passed);
        
        Ok(result)
    }
    
    fn test_platform_support(&self) -> bool {
        self.runtime.is_platform_supported(&crate::Platform::current())
    }
    
    async fn check_installed(&self) -> Result<bool> {
        // 使用 PathManager 检查
        let path_manager = vx_paths::PathManager::new()?;
        let versions = path_manager.list_store_versions(self.runtime.name())?;
        Ok(!versions.is_empty())
    }
    
    fn check_system_available(&self) -> bool {
        which::which(self.runtime.name()).is_ok()
    }
    
    async fn run_functional_tests(&self) -> Result<Vec<TestCaseResult>> {
        // 从 manifest 获取测试配置
        // 如果没有配置，使用默认的 --version 测试
        let test_config = self.get_test_config();
        
        let mut results = vec![];
        
        for cmd in test_config.functional_commands {
            let result = self.run_test_command(&cmd).await;
            results.push(result);
        }
        
        // 如果没有配置任何测试，使用默认测试
        if results.is_empty() {
            let default_result = self.run_default_test().await;
            results.push(default_result);
        }
        
        Ok(results)
    }
    
    fn get_test_config(&self) -> TestConfig {
        // 从 manifest registry 获取测试配置
        // 如果没有配置，返回默认配置
        TestConfig::default()
    }
    
    async fn run_test_command(&self, cmd: &TestCommand) -> TestCaseResult {
        // 执行测试命令并验证结果
        todo!()
    }
    
    async fn run_default_test(&self) -> TestCaseResult {
        // 默认测试：执行 --version
        todo!()
    }
}
```

#### 1.4 简化 `vx-cli/src/commands/test.rs`

```rust
// 重构后的 test.rs - 大幅简化

use crate::commands::{CommandContext, CommandHandler};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use vx_runtime::testing::{RuntimeTester, RuntimeTestResult};

#[derive(Args, Clone)]
pub struct TestCommand {
    /// Runtime name to test
    pub runtime: Option<String>,
    
    /// Test all registered runtimes
    #[arg(long)]
    pub all: bool,
    
    // ... 其他参数保持不变
}

#[async_trait]
impl CommandHandler for TestCommand {
    async fn execute(&self, ctx: &CommandContext) -> Result<()> {
        if self.all {
            self.test_all(ctx).await
        } else if let Some(ref runtime) = self.runtime {
            self.test_single(ctx, runtime).await
        } else {
            anyhow::bail!("Please specify a runtime or use --all");
        }
    }
}

impl TestCommand {
    async fn test_single(&self, ctx: &CommandContext, name: &str) -> Result<()> {
        let runtime = ctx.registry()
            .get_runtime(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown runtime: {}", name))?;
        
        // 使用 RuntimeTester 运行测试
        let tester = RuntimeTester::new(&*runtime, ctx.runtime_context());
        let result = tester.run_all().await?;
        
        self.output_result(&result);
        self.exit_with_result(&result);
    }
    
    async fn test_all(&self, ctx: &CommandContext) -> Result<()> {
        let mut results = vec![];
        
        for provider in ctx.registry().providers() {
            for runtime in provider.runtimes() {
                let tester = RuntimeTester::new(&*runtime, ctx.runtime_context());
                let result = tester.run_all().await?;
                results.push(result);
            }
        }
        
        self.output_summary(&results);
        self.exit_with_summary(&results);
    }
    
    // ... 输出方法
}
```

### 方案 2: CLI 结构模块化

#### 2.1 将命令参数移到各自模块

**当前结构**：
```
cli.rs (1400+ 行)
├── Commands 枚举定义
├── 子命令枚举定义
└── CommandHandler::execute() 分发

commands/
├── install.rs (只有 handle 函数)
├── list.rs
└── ...
```

**目标结构**：
```
cli.rs (~200 行)
├── Cli 结构体
├── Commands 枚举 (只有变体名)
└── 简单的分发逻辑

commands/
├── mod.rs
├── install/
│   ├── mod.rs
│   ├── args.rs      # InstallArgs 定义
│   └── handler.rs   # handle 函数
├── list/
│   ├── mod.rs
│   ├── args.rs
│   └── handler.rs
├── services/
│   ├── mod.rs
│   ├── args.rs      # ServicesCommand 子命令定义
│   └── handler.rs
└── ...
```

#### 2.2 使用宏简化命令注册

```rust
// crates/vx-cli/src/commands/mod.rs

/// 命令注册宏
macro_rules! register_commands {
    ($($name:ident => $module:ident),* $(,)?) => {
        $(
            pub mod $module;
            pub use $module::Args as $name;
        )*
        
        #[derive(Subcommand, Clone)]
        pub enum Commands {
            $(
                $name($module::Args),
            )*
        }
        
        impl Commands {
            pub async fn execute(&self, ctx: &CommandContext) -> Result<()> {
                match self {
                    $(
                        Commands::$name(args) => $module::handle(ctx, args).await,
                    )*
                }
            }
        }
    };
}

register_commands! {
    Install => install,
    List => list,
    Update => update,
    Uninstall => uninstall,
    Which => which,
    Versions => versions,
    Switch => switch,
    Config => config,
    Test => test,
    Sync => sync,
    Init => init,
    Clean => clean,
    Cache => cache,
    Stats => stats,
    Plugin => plugin,
    Shell => shell,
    Venv => venv,
    Global => global,
    Env => env,
    Dev => dev,
    Setup => setup,
    Add => add,
    Remove => remove,
    Run => run,
    Services => services,
    Hook => hook,
    Container => container,
    Ext => ext,
    X => x,
    Migrate => migrate,
    Lock => lock,
    Info => info,
}
```

#### 2.3 示例：重构后的 install 命令

```rust
// crates/vx-cli/src/commands/install/mod.rs
mod args;
mod handler;

pub use args::Args;
pub use handler::handle;

// crates/vx-cli/src/commands/install/args.rs
use clap::Args as ClapArgs;

#[derive(ClapArgs, Clone)]
#[command(alias = "i")]
pub struct Args {
    /// Tools to install (e.g., uv, node@22, go@1.22, rust)
    #[arg(required = true, num_args = 1..)]
    pub tools: Vec<String>,
    
    /// Force reinstallation even if already installed
    #[arg(short, long)]
    pub force: bool,
}

// crates/vx-cli/src/commands/install/handler.rs
use super::Args;
use crate::commands::CommandContext;
use anyhow::Result;

pub async fn handle(ctx: &CommandContext, args: &Args) -> Result<()> {
    // 实现逻辑
    todo!()
}
```

#### 2.4 简化后的 cli.rs

```rust
// crates/vx-cli/src/cli.rs (~200 行)

use crate::commands::{Commands, CommandContext, GlobalOptions};
use anyhow::Result;
use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum CacheModeArg {
    Normal,
    Refresh,
    Offline,
    NoCache,
}

#[derive(Parser)]
#[command(name = "vx")]
#[command(about = "Universal version executor for development tools")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Use system PATH to find tools
    #[arg(long, global = true)]
    pub use_system_path: bool,

    /// Cache mode
    #[arg(long, global = true, value_enum, default_value = "normal")]
    pub cache_mode: CacheModeArg,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable debug output
    #[arg(long, global = true)]
    pub debug: bool,

    /// Tool and arguments to execute
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

impl Cli {
    pub async fn run(&self, ctx: &CommandContext) -> Result<()> {
        match &self.command {
            Some(cmd) => cmd.execute(ctx).await,
            None if !self.args.is_empty() => {
                // 动态命令转发
                crate::commands::execute::handle_dynamic(ctx, &self.args).await
            }
            None => {
                // 显示帮助
                println!("Use --help for usage information");
                Ok(())
            }
        }
    }
}
```

## 向后兼容性

### 兼容策略

1. **命令行接口不变**：`vx test node` 等命令保持原有行为
2. **provider.toml 向后兼容**：`[runtimes.test]` 是可选配置
3. **默认测试行为**：如果没有配置测试规则，使用默认的 `--version` 测试

### 迁移路径

```bash
# 阶段 1: 添加测试配置到 provider.toml
# 这是可选的，不添加也能正常工作

# 阶段 2: 内部重构 cli.rs
# 对用户透明，命令行接口不变

# 阶段 3: 逐步迁移命令模块
# 每个命令独立迁移，不影响其他命令
```

## 实现计划

### Phase 1: Provider 测试配置 (v0.8.0) ✅ 已完成

- [x] 在 `vx-manifest` 中添加 `TestConfig` 类型
  - 实现位置: `crates/vx-manifest/src/provider/test_config.rs`
  - 支持 `functional_commands`, `install_verification`, `platforms`, `inline_scripts`
- [x] 在 `vx-runtime` 中添加 `RuntimeTester`
  - 实现位置: `crates/vx-runtime/src/testing.rs`
  - 支持从 manifest 配置运行测试
- [x] 重构 `vx-cli/src/commands/test.rs` 使用新的测试器
  - 集成 `ManifestTester` 运行基于配置的测试
  - 添加 `CommandContext.get_runtime_manifest()` 方法获取测试配置
- [x] 为几个核心 provider 添加测试配置示例
  - `node/provider.toml` - 版本检查、eval 测试、平台检测
  - `go/provider.toml` - 版本检查、环境变量检查
  - `uv/provider.toml` - 版本检查、Python 版本检查
  - `python/provider.toml` - 版本检查、平台检测
  - `rust/provider.toml` - rustup 版本检查、工具链显示

### Phase 2: CLI 模块化 (v0.8.1) ✅ 已完成

已完成命令模块化重构：
- 每个复杂命令有独立的目录结构 (`commands/*/`)
- 使用 `args.rs` 定义命令参数
- 使用 `handler.rs` 实现命令逻辑
- `mod.rs` 导出 `Args` 和 `handle()` 函数

已重构的命令：
- [x] `install/` - 安装命令 (args.rs, handler.rs, mod.rs)
- [x] `list/` - 列表命令 (args.rs, handler.rs, mod.rs)
- [x] `test/` - 测试命令 (args.rs, handler.rs, mod.rs)

目录结构：
```
commands/
├── install/
│   ├── mod.rs      # 模块导出
│   ├── args.rs     # Args 结构体定义
│   └── handler.rs  # handle() 函数实现
├── list/
│   ├── mod.rs
│   ├── args.rs
│   └── handler.rs
├── test/
│   ├── mod.rs
│   ├── args.rs
│   └── handler.rs
└── ...             # 其他简单命令保持单文件
```

### Phase 3: 清理和文档 (v0.8.2) - 延期

- [ ] 进一步简化 `cli.rs` 中的 Commands 枚举
- [ ] 更新开发文档
- [ ] 添加命令开发指南

## 替代方案

### 替代方案 1: 保持现状

**优点**：
- 无需重构
- 代码已经可以工作

**缺点**：
- 测试逻辑无法定制
- cli.rs 难以维护
- 添加新命令需要修改多处

### 替代方案 2: 使用 Rust 代码定义测试

在每个 provider 的 Rust 代码中定义测试逻辑，而不是 provider.toml。

**优点**：
- 更灵活
- 可以使用 Rust 的全部能力

**缺点**：
- 需要编译才能修改测试
- 与声明式 provider.toml 理念不符
- 增加代码量

### 选择理由

选择方案 1 (provider.toml 配置) 因为：
1. 与现有的 `[runtimes.health]` 和 `[runtimes.detection]` 配置一致
2. 声明式配置更容易理解和维护
3. 可以在不重新编译的情况下修改测试规则

## 参考资料

### 相关 RFC
- RFC 0005: VX 测试框架 - 通用测试框架设计
- RFC 0018: Extended Provider Schema - provider.toml 扩展

### 相关代码
- `crates/vx-cli/src/cli.rs` - 当前 CLI 定义
- `crates/vx-cli/src/commands/test.rs` - 测试命令实现
- `crates/vx-cli/src/commands/handler.rs` - CommandContext 和 CommandHandler
- `crates/vx-manifest/src/provider/test_config.rs` - TestConfig 类型定义
- `crates/vx-runtime/src/testing.rs` - RuntimeTester 实现

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2026-01-12 | Draft | 初始草案 |
| 2026-01-12 | v0.2 | 添加跨平台测试支持、内联脚本、平台特定配置 |
| 2026-01-12 | v1.0 | Phase 1 完成：TestConfig、RuntimeTester、核心 provider 测试配置 |
