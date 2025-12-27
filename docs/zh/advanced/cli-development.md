# CLI 命令开发指南

本指南说明如何为 vx 添加新的 CLI 命令。CLI 使用 **Command Trait** 模式实现清晰、可维护的命令路由。

## 架构概览

vx CLI 采用模块化架构：

```
┌─────────────────────────────────────────────────────────────┐
│                        vx-cli                                │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                     cli.rs                            │   │
│  │  - Cli 结构体 (clap Parser)                          │   │
│  │  - Commands 枚举 (所有子命令)                         │   │
│  │  - Commands 的 CommandHandler 实现                   │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                    lib.rs                             │   │
│  │  - main() 入口点                                     │   │
│  │  - CommandContext 创建                               │   │
│  │  - command.execute(&ctx)                             │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │               commands/*.rs                           │   │
│  │  - 各命令的具体实现                                   │   │
│  │  - handle() 函数                                     │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## 核心组件

### CommandHandler Trait

```rust
// commands/handler.rs

/// 命令执行的统一上下文
pub struct CommandContext {
    pub registry: Arc<ProviderRegistry>,
    pub runtime_context: Arc<RuntimeContext>,
    pub use_system_path: bool,
    pub verbose: bool,
    pub debug: bool,
}

/// 命令处理器 trait
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// 执行命令
    async fn execute(&self, ctx: &CommandContext) -> Result<()>;
    
    /// 获取命令名称（用于日志）
    fn name(&self) -> &'static str {
        "unknown"
    }
}
```

### Commands 枚举

所有命令在 `cli.rs` 中定义为 `Commands` 枚举的变体：

```rust
#[derive(Subcommand, Clone)]
pub enum Commands {
    /// 显示版本信息
    Version,
    
    /// 安装指定工具版本
    #[command(alias = "i")]
    Install {
        tool: String,
        version: Option<String>,
        #[arg(short, long)]
        force: bool,
    },
    
    // ... 更多命令
}
```

## 添加新命令

### 步骤 1: 在 cli.rs 中定义命令

在 `Commands` 枚举中添加新变体：

```rust
// 在 cli.rs 中

#[derive(Subcommand, Clone)]
pub enum Commands {
    // ... 现有命令 ...
    
    /// 我的新命令描述
    #[command(alias = "my")]  // 可选：短别名
    MyCommand {
        /// 必需参数
        name: String,
        
        /// 带默认值的可选参数
        #[arg(long, default_value = "default")]
        option: String,
        
        /// 布尔标志
        #[arg(short, long)]
        verbose: bool,
    },
}
```

### 步骤 2: 添加命令名称

更新 `CommandHandler` 实现中的 `name()` 方法：

```rust
// 在 cli.rs 中，impl CommandHandler for Commands 内部

fn name(&self) -> &'static str {
    match self {
        // ... 现有匹配 ...
        Commands::MyCommand { .. } => "my-command",
    }
}
```

### 步骤 3: 添加执行分支

在 `execute()` 方法中添加执行逻辑：

```rust
// 在 cli.rs 中，impl CommandHandler for Commands 内部

async fn execute(&self, ctx: &CommandContext) -> Result<()> {
    match self {
        // ... 现有匹配 ...
        
        Commands::MyCommand {
            name,
            option,
            verbose,
        } => {
            commands::my_command::handle(
                ctx.registry(),
                ctx.runtime_context(),
                name,
                option,
                *verbose,
            )
            .await
        }
    }
}
```

### 步骤 4: 创建命令模块

创建 `commands/my_command.rs`：

```rust
//! 我的命令实现

use anyhow::Result;
use crate::ui::UI;
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// 处理 my-command 命令
pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    name: &str,
    option: &str,
    verbose: bool,
) -> Result<()> {
    if verbose {
        UI::info(&format!("运行 my-command，name={}，option={}", name, option));
    }
    
    // 你的实现代码
    UI::success(&format!("成功处理: {}", name));
    
    Ok(())
}
```

### 步骤 5: 注册模块

在 `commands/mod.rs` 中添加模块：

```rust
// 在 commands/mod.rs 中

pub mod my_command;  // 添加这一行
```

## 命令模式

### 简单命令（无参数）

```rust
// cli.rs
Commands::Stats,

// execute()
Commands::Stats => commands::stats::handle(ctx.registry()).await,

// commands/stats.rs
pub async fn handle(registry: &ProviderRegistry) -> Result<()> {
    // 实现
    Ok(())
}
```

### 带子命令的命令

```rust
// cli.rs
#[derive(Subcommand, Clone)]
pub enum ConfigCommand {
    Show,
    Set { key: String, value: String },
    Get { key: String },
}

Commands::Config {
    #[command(subcommand)]
    command: Option<ConfigCommand>,
},

// execute()
Commands::Config { command } => match command {
    Some(ConfigCommand::Show) | None => commands::config::handle().await,
    Some(ConfigCommand::Set { key, value }) => {
        commands::config::handle_set(key, value).await
    }
    Some(ConfigCommand::Get { key }) => {
        commands::config::handle_get(key).await
    }
},
```

### 带进度指示器的命令

```rust
use crate::ui::{ProgressSpinner, UI};

pub async fn handle(name: &str) -> Result<()> {
    let spinner = ProgressSpinner::new(&format!("正在处理 {}...", name));
    
    // 执行工作...
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    
    spinner.finish_with_message(&format!("✓ 完成 {}", name));
    
    Ok(())
}
```

## UI 辅助函数

`ui` 模块提供一致的输出格式：

```rust
use crate::ui::UI;

// 信息消息
UI::info("正在处理...");

// 成功消息
UI::success("操作完成！");

// 警告消息
UI::warning("这可能需要一段时间");

// 错误消息
UI::error("出错了");

// 提示
UI::hint("使用 --force 覆盖");

// 详情
UI::detail(&format!("已安装到: {}", path.display()));

// 工具未找到（带建议）
UI::tool_not_found("nod", &["node", "npm", "npx"]);
```

## 测试命令

在 `crates/vx-cli/tests/` 中创建测试：

```rust
// tests/my_command_tests.rs

use rstest::rstest;
use vx_cli::commands::my_command;

#[rstest]
#[tokio::test]
async fn test_my_command_basic() {
    // 测试实现
}

#[rstest]
#[case("input1", "expected1")]
#[case("input2", "expected2")]
#[tokio::test]
async fn test_my_command_parametrized(
    #[case] input: &str,
    #[case] expected: &str,
) {
    // 参数化测试
}
```

## 最佳实践

### 1. 保持命令专注

每个命令应该只做一件事：

```rust
// 好：专注的命令
Commands::Install { tool, version, force },
Commands::Uninstall { tool, version, force },

// 避免：过度复杂的命令
Commands::Manage { action, tool, version, force, ... },
```

### 2. 提供有用的别名

```rust
#[command(alias = "i")]
Install { ... },

#[command(alias = "rm", alias = "remove")]
Uninstall { ... },

#[command(alias = "ls")]
List { ... },
```

### 3. 使用一致的参数命名

```rust
// 跨命令的一致命名
--force, -f     // 强制操作
--verbose, -v   // 详细输出
--dry-run       // 预览不执行
--all, -a       // 应用到所有项目
```

### 4. 尽早验证

```rust
pub async fn handle(tool: &str, version: Option<&str>) -> Result<()> {
    // 尽早验证输入
    if tool.is_empty() {
        return Err(anyhow::anyhow!("工具名称不能为空"));
    }
    
    // 继续处理有效输入
    Ok(())
}
```

### 5. 在错误中提供上下文

```rust
let runtime = registry.get_runtime(tool_name)
    .ok_or_else(|| {
        let available = registry.runtime_names();
        anyhow::anyhow!(
            "未找到工具 '{}'。可用工具: {}",
            tool_name,
            available.join(", ")
        )
    })?;
```

## 文件结构

```
crates/vx-cli/
├── Cargo.toml
├── src/
│   ├── lib.rs              # 入口点，VxCli 结构体
│   ├── cli.rs              # Cli, Commands, CommandHandler 实现
│   ├── registry.rs         # Provider 注册表设置
│   ├── ui.rs               # UI 辅助函数
│   └── commands/
│       ├── mod.rs          # 模块导出
│       ├── handler.rs      # CommandHandler trait, CommandContext
│       ├── install.rs      # install 命令
│       ├── list.rs         # list 命令
│       ├── version.rs      # version 命令
│       └── ...             # 其他命令
└── tests/
    ├── cli_parsing_tests.rs
    └── ...
```

## 参见

- [Provider 开发指南](./plugin-development) - 添加新工具支持
- [Extension 开发指南](./extension-development) - 添加脚本扩展
- [架构概览](./architecture) - 系统架构
- [贡献指南](./contributing) - 如何贡献
