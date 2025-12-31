# RFC 0009: 统一控制台输出系统 (vx-console)

| 属性 | 值 |
|------|-----|
| RFC 编号 | 0009 |
| 标题 | 统一控制台输出系统 |
| 状态 | 草案 |
| 作者 | VX Team |
| 创建日期 | 2025-12-31 |
| 最后更新 | 2025-12-31 |

## 概述

本 RFC 提议创建 `vx-console` crate，统一管理跨平台的控制台输出、日志、进度条和耗时任务交互。

## 主流 Rust CLI 应用方案调研

在设计 `vx-console` 之前，我们调研了主流 Rust CLI 应用的控制台输出方案：

### 1. Cargo (rust-lang/cargo)

**架构**: `Shell` 结构体封装 stdout/stderr

**核心设计**:
```rust
// cargo/src/cargo/core/shell.rs
pub struct Shell {
    output: ShellOut,           // Stream | Write
    verbosity: Verbosity,       // Verbose | Normal | Quiet
    needs_clear: bool,          // 进度条清理标记
    hostname: Option<String>,
}

pub enum ShellOut {
    Stream {
        stdout: AutoStream<std::io::Stdout>,
        stderr: AutoStream<std::io::Stderr>,
        stderr_tty: bool,
        color_choice: ColorChoice,
        hyperlinks: bool,
    },
    Write(Box<dyn Write>),  // 用于测试
}
```

**关键特性**:
- 使用 `anstream` 库处理跨平台 ANSI 颜色
- `ColorChoice` 枚举: `Always` / `Never` / `CargoAuto`
- `Verbosity` 枚举控制输出级别
- 支持超链接（可点击的文件路径）
- `from_write()` 方法支持测试时注入内存缓冲区
- 使用 `annotate_snippets` 渲染错误报告

**依赖库**:
- `anstream` - 跨平台流处理
- `anstyle` - 样式定义
- `annotate_snippets` - 错误报告渲染

### 2. uv (astral-sh/uv)

**架构**: 独立的 `uv-console` crate

**核心功能**:
```rust
// 确认提示 - 实时响应按键
pub fn confirm(prompt: &str, default: bool) -> Result<bool>;

// 密码输入 - 隐藏输入
pub fn password(prompt: &str) -> Result<String>;

// 通用文本输入 - 支持光标移动、词级跳转
pub fn input(prompt: &str) -> Result<String>;

// 字节格式化
pub fn human_readable_bytes(bytes: u64) -> (f64, &'static str);
```

**关键特性**:
- 使用 `console` crate 进行终端控制
- 实时按键响应（无需按 Enter）
- 支持 Ctrl+C 退出
- 完整的编辑功能（光标移动、词级跳转）
- 跨平台兼容（包括 Windows 特殊处理）

### 3. ripgrep (BurntSushi/ripgrep)

**架构**: 独立的 `grep-printer` crate

**打印器类型**:
```rust
// 人类可读格式
pub struct Standard { ... }

// JSON Lines 格式（机器可读）
pub struct JSON { ... }

// 聚合摘要
pub struct Summary { ... }
```

**关键特性**:
- 模块化设计：color、hyperlink、path、standard、summary
- 支持搜索和替换功能
- 多行结果处理
- JSON 输出用于程序化处理
- 统计摘要报告

### 4. rustup (rust-lang/rustup)

**架构**: 内置终端处理模块

**关键特性**:
- 进度条显示下载进度
- 多组件并行安装进度
- 自动检测终端能力

### 方案对比

| 特性 | Cargo | uv | ripgrep |
|------|-------|-----|---------|
| 颜色库 | anstream/anstyle | console | termcolor |
| 进度条 | 自定义 | indicatif | 无 |
| 输出模式 | Verbose/Normal/Quiet | 类似 | Standard/JSON/Summary |
| 测试支持 | Write trait 注入 | 有 | Sink trait |
| 交互式输入 | 无 | 有 | 无 |
| 超链接 | 有 | 无 | 有 |

### 设计启示

基于以上调研，`vx-console` 应采用：

1. **Cargo 风格的 Shell 架构** - 封装 stdout/stderr，支持测试注入
2. **anstream/anstyle** - Cargo 使用的现代跨平台颜色库，替代 colored
3. **uv 风格的交互式输入** - 支持确认、密码输入等
4. **ripgrep 风格的多输出格式** - Standard/JSON/Summary
5. **统一的 Verbosity 控制** - Verbose/Normal/Quiet

## 动机

### 当前问题

1. **代码重复**: 多个 crate 中存在相似的输出逻辑
   - `vx-cli/src/ui.rs` (730+ 行)
   - `vx-installer/src/progress.rs` (270 行)
   - `vx-runtime/src/impls.rs` 直接使用 indicatif

2. **风格不一致**: 不同模块的输出风格不统一
   - 有的用 `✓`，有的用 `✔`
   - 颜色方案不一致
   - spinner 动画风格各异

3. **跨平台问题**: Windows 终端对 Unicode 和 ANSI 颜色的支持不一致
   - Windows Terminal vs CMD vs PowerShell
   - CI/CD 环境（无 TTY）

4. **分散的 `println!`**: 40+ 个文件中有直接输出，难以统一控制

5. **日志与用户输出混淆**: `tracing` 日志和用户输出混在一起

### 目标

- **统一 API**: 一个 crate 处理所有输出需求
- **跨平台兼容**: 自动适配 Windows/macOS/Linux 和不同终端
- **可测试性**: 支持捕获输出进行测试
- **可配置性**: 支持 quiet/verbose/json 等输出模式
- **进度管理**: 统一的进度条和 spinner 管理

## 设计

### Crate 结构

基于 Cargo 的 Shell 架构和 uv 的交互式设计：

```
crates/vx-console/
├── src/
│   ├── lib.rs           # 公开 API
│   ├── shell.rs         # Shell 结构体（参考 Cargo）
│   ├── output.rs        # ShellOut 输出抽象
│   ├── style.rs         # 样式和主题（使用 anstyle）
│   ├── progress.rs      # 进度条和 spinner（使用 indicatif）
│   ├── term.rs          # 终端检测和适配
│   ├── interact.rs      # 交互式输入（参考 uv）
│   ├── format.rs        # 输出格式化（Standard/JSON）
│   └── test.rs          # 测试支持
└── Cargo.toml
```

### 核心 API

#### 1. Shell 结构体（参考 Cargo）

```rust
use vx_console::{Shell, Verbosity, ColorChoice};

// 创建 Shell（参考 Cargo 设计）
let mut shell = Shell::new();

// 或使用 builder
let shell = Shell::builder()
    .verbosity(Verbosity::Normal)  // Verbose | Normal | Quiet
    .color_choice(ColorChoice::Auto)  // Always | Never | Auto
    .build();

// 状态消息（Cargo 风格）
shell.status("Compiling", "vx v0.1.0")?;      // 绿色 "Compiling"
shell.status_with_color("Downloading", "node@20", Color::Cyan)?;

// 基本输出
shell.info("Installing node...")?;
shell.success("Installed node@20.10.0")?;
shell.warn("Version 18 is deprecated")?;
shell.error("Failed to download")?;
shell.hint("Try: vx install node@20")?;

// 条件输出
shell.verbose(|s| s.info("Cache hit"))?;  // 仅在 verbose 模式显示

// 测试支持（参考 Cargo）
let mut output = Vec::new();
let shell = Shell::from_write(Box::new(&mut output));
```

#### 2. 输出管理器 (Console) - 全局单例

```rust
use vx_console::{Console, OutputMode};

// 全局单例（便捷 API）
let console = Console::global();

// 基本输出
console.info("Installing node...");
console.success("Installed node@20.10.0");
console.warn("Version 18 is deprecated");
console.error("Failed to download");
console.hint("Try: vx install node@20");

// 设置模式
console.set_verbosity(Verbosity::Verbose);
console.set_color_choice(ColorChoice::Never);
```

#### 2. 进度条和 Spinner

```rust
use vx_console::{Console, SpinnerStyle};

let console = Console::global();

// 简单 spinner
let spinner = console.spinner("Downloading node...");
// ... 执行操作
spinner.success("Downloaded node");

// 带进度的下载
let progress = console.download_progress("Downloading node", total_size);
progress.set_position(downloaded);
progress.finish("Downloaded 45.2 MB");

// 多任务进度
let multi = console.multi_progress("Installing tools", 3);
multi.start_task("node@20");
multi.complete_task(true);
multi.start_task("npm@10");
multi.complete_task(true);
multi.finish("Installed 3 tools");

// 自动选择样式
// - 在 TTY 中显示动画 spinner
// - 在非 TTY 中显示静态消息
// - 在 CI 中显示简化输出
```

#### 3. 耗时任务

```rust
use vx_console::{Console, Task};

let console = Console::global();

// 带耗时统计的任务
let result = console.task("Building project", || {
    // ... 执行操作
    Ok(())
})?;
// 输出: ✓ Building project (2.3s)

// 异步任务
let result = console.task_async("Fetching versions", async {
    // ... 异步操作
    Ok(versions)
}).await?;

// 带步骤的任务
console.steps("Installing node", |steps| {
    steps.step("Downloading")?;
    // ...
    steps.step("Extracting")?;
    // ...
    steps.step("Configuring")?;
    // ...
    Ok(())
})?;
```

#### 4. 终端适配

```rust
use vx_console::{Term, TermCapabilities};

let term = Term::detect();

// 检测能力
if term.supports_color() {
    // 使用彩色输出
}

if term.supports_unicode() {
    // 使用 Unicode 字符
}

if term.is_interactive() {
    // 显示动画
}

// 获取终端尺寸
let (width, height) = term.size();

// 清屏
term.clear_screen();

// 移动光标
term.move_cursor_up(2);
```

#### 5. 输出模式

```rust
use vx_console::{Console, OutputMode};

// 交互模式（默认）
// - 彩色输出
// - 动画 spinner
// - 进度条

// 安静模式
// - 只显示错误
// - 无动画
let console = Console::builder().mode(OutputMode::Quiet).build();

// 详细模式
// - 显示 debug 信息
// - 显示耗时
let console = Console::builder().mode(OutputMode::Verbose).build();

// JSON 模式（用于脚本集成）
// - 结构化输出
// - 无颜色
let console = Console::builder().mode(OutputMode::Json).build();
// 输出: {"level":"info","message":"Installing node","tool":"node","version":"20"}

// CI 模式（自动检测）
// - 无动画
// - 简化进度
// - GitHub Actions 注解
let console = Console::builder().mode(OutputMode::Ci).build();
// 输出: ::group::Installing node
//       ...
//       ::endgroup::
```

#### 6. 主题系统

```rust
use vx_console::{Theme, Style, Color};

// 默认主题
let theme = Theme::default();

// 自定义主题
let theme = Theme::builder()
    .success(Style::new().fg(Color::Green).bold())
    .error(Style::new().fg(Color::Red).bold())
    .warn(Style::new().fg(Color::Yellow))
    .info(Style::new().fg(Color::Blue))
    .hint(Style::new().fg(Color::Cyan).dimmed())
    .spinner_chars(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    .progress_chars("━━╺")
    .build();

// 内置主题
let theme = Theme::minimal();    // 无颜色，ASCII 字符
let theme = Theme::colorful();   // 丰富的颜色
let theme = Theme::github();     // GitHub Actions 风格
```

#### 7. 测试支持

```rust
use vx_console::{Console, TestOutput};

#[test]
fn test_output() {
    let output = TestOutput::new();
    let console = Console::builder()
        .output(output.clone())
        .build();

    console.info("Hello");
    console.success("Done");

    // 验证输出
    assert!(output.contains("Hello"));
    assert!(output.has_success("Done"));

    // 获取所有输出
    let lines = output.lines();
    assert_eq!(lines.len(), 2);
}
```

#### 8. 日志集成

```rust
use vx_console::{Console, LogBridge};

// 将 tracing 日志桥接到 console
let console = Console::global();
let _guard = console.bridge_tracing();

// 现在 tracing::info! 会通过 console 输出
tracing::info!("Starting download");
// 输出: ℹ Starting download

// 或者分离日志和用户输出
let console = Console::builder()
    .log_to_file("vx.log")  // 日志写入文件
    .build();

// tracing 写入文件，console 输出到终端
```

#### 9. 交互式输入（参考 uv）

```rust
use vx_console::{Console, interact};

let console = Console::global();

// 确认提示 - 实时响应按键（无需按 Enter）
let confirmed = console.confirm("Proceed with installation?", true)?;
// 输出: ? Proceed with installation? [Y/n]

// 带默认值的确认
let confirmed = console.confirm_default("Override existing?", false)?;

// 密码输入 - 隐藏输入内容
let password = console.password("Enter token:")?;

// 通用文本输入 - 支持光标移动、词级跳转
let name = console.input("Project name:")?;

// 选择列表
let choice = console.select("Choose package manager:", &["npm", "yarn", "pnpm"])?;

// 多选
let choices = console.multi_select("Select tools:", &["node", "python", "go"])?;
```

### 跨平台适配

#### Windows 支持

```rust
// 自动处理 Windows 终端差异
impl Term {
    fn detect() -> Self {
        #[cfg(windows)]
        {
            // 检测 Windows Terminal
            if std::env::var("WT_SESSION").is_ok() {
                return Self::windows_terminal();
            }

            // 检测 ConEmu/Cmder
            if std::env::var("ConEmuANSI").is_ok() {
                return Self::conemu();
            }

            // 启用 ANSI 支持（Windows 10+）
            if enable_virtual_terminal_processing() {
                return Self::windows_ansi();
            }

            // 回退到基本模式
            Self::windows_basic()
        }

        #[cfg(unix)]
        {
            Self::unix()
        }
    }
}
```

#### CI 环境检测

```rust
impl Term {
    fn detect_ci() -> Option<CiEnvironment> {
        if std::env::var("GITHUB_ACTIONS").is_ok() {
            return Some(CiEnvironment::GitHubActions);
        }
        if std::env::var("GITLAB_CI").is_ok() {
            return Some(CiEnvironment::GitLabCi);
        }
        if std::env::var("JENKINS_URL").is_ok() {
            return Some(CiEnvironment::Jenkins);
        }
        if std::env::var("CI").is_ok() {
            return Some(CiEnvironment::Generic);
        }
        None
    }
}
```

### 迁移计划

#### Phase 1: 创建 vx-console crate

1. 实现核心 API
2. 实现跨平台终端检测
3. 实现基本输出方法
4. 添加单元测试

#### Phase 2: 迁移 vx-cli/ui.rs

1. 将 `UI` 结构体迁移到 vx-console
2. 将 `ProgressSpinner` 等迁移到 vx-console
3. 更新 vx-cli 使用 vx-console
4. 删除 vx-cli/ui.rs

#### Phase 3: 迁移 vx-installer/progress.rs

1. 统一 `ProgressReporter` trait
2. 更新 vx-installer 使用 vx-console
3. 删除重复代码

#### Phase 4: 清理分散的 println!

1. 审计所有 `println!/eprintln!` 使用
2. 替换为 `Console::global()` 调用
3. 添加 clippy lint 禁止直接 println

#### Phase 5: 日志集成

1. 实现 tracing 桥接
2. 统一日志和用户输出
3. 添加日志文件支持

### 依赖

基于主流 Rust CLI 应用的调研，推荐使用以下依赖：

```toml
[dependencies]
# 终端处理（Cargo 使用的现代方案）
anstream = "0.6"           # 跨平台 ANSI 流处理（Cargo 使用）
anstyle = "1.0"            # 样式定义（与 anstream 配套）
console = "0.15"           # 终端检测和交互（uv 使用）

# 进度条
indicatif = "0.17"         # 进度条和 spinner

# 跨平台
terminal_size = "0.3"      # 终端尺寸

# 日志
tracing = { workspace = true }
tracing-subscriber = { workspace = true, optional = true }

# 序列化（JSON 模式）
serde = { workspace = true }
serde_json = { workspace = true }

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.52", features = ["Win32_System_Console"] }

[features]
default = ["progress"]
progress = []
log-bridge = ["tracing-subscriber"]
```

**为什么选择 anstream/anstyle 而不是 colored**:
- `anstream` 是 Cargo 官方使用的库，经过大规模验证
- 自动处理终端能力检测和 ANSI 转义码适配
- `anstyle` 提供零开销的样式定义
- 更好的跨平台支持（特别是 Windows）

### API 示例

#### 完整示例：vx sync

```rust
use vx_console::{Console, OutputMode};

pub async fn handle_sync(args: SyncArgs) -> Result<()> {
    let console = Console::global();

    // 设置模式
    if args.quiet {
        console.set_mode(OutputMode::Quiet);
    } else if args.verbose {
        console.set_mode(OutputMode::Verbose);
    }

    // 开始任务
    let spinner = console.spinner("Reading vx.toml...");

    let config = read_config()?;
    spinner.success("Read vx.toml");

    // 多工具安装
    let tools: Vec<_> = config.tools.iter().collect();
    let multi = console.multi_progress("Installing tools", tools.len());

    for (name, version) in tools {
        multi.start_task(&format!("{}@{}", name, version));

        match install_tool(name, version).await {
            Ok(_) => {
                multi.complete_task(true);
                console.debug(&format!("Installed {} in {:?}", name, elapsed));
            }
            Err(e) => {
                multi.complete_task(false);
                console.error(&format!("Failed to install {}: {}", name, e));
            }
        }
    }

    multi.finish(&format!("Installed {} tools", tools.len()));

    Ok(())
}
```

#### 完整示例：下载进度

```rust
use vx_console::Console;

pub async fn download_with_progress(url: &str, dest: &Path) -> Result<()> {
    let console = Console::global();

    let response = client.get(url).send().await?;
    let total_size = response.content_length().unwrap_or(0);

    let progress = console.download_progress("Downloading", total_size);

    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        progress.set_position(downloaded);
    }

    progress.finish(&format!("Downloaded {:.1} MB", downloaded as f64 / 1_000_000.0));

    Ok(())
}
```

## 实现优先级

| 优先级 | 功能 | 原因 |
|--------|------|------|
| P0 | 基本输出 API | 核心功能 |
| P0 | 跨平台终端检测 | Windows 兼容性 |
| P1 | 进度条和 Spinner | 用户体验 |
| P1 | 输出模式 | CLI 需求 |
| P2 | 主题系统 | 可定制性 |
| P2 | 测试支持 | 代码质量 |
| P3 | 日志集成 | 统一日志 |
| P3 | JSON 模式 | 脚本集成 |

## 替代方案

### 1. 继续使用分散的实现

**优点**: 无需迁移
**缺点**: 代码重复，风格不一致，难以维护

### 2. 直接使用 Cargo 的 Shell 模块

**优点**: 经过大规模验证
**缺点**: Cargo 的 Shell 不是独立 crate，需要复制代码

### 3. 只使用 indicatif + console

**优点**: 成熟的库组合
**缺点**: 缺少统一的 API 封装，需要在每个使用处组合

### 4. 使用 dialoguer

**优点**: 功能丰富的交互式输入
**缺点**: 主要关注交互，不处理一般输出和进度条

### 推荐方案

采用 **Cargo 风格的 Shell 架构** + **uv 风格的交互式输入**：
- 核心设计参考 Cargo 的 `Shell` 结构
- 使用 Cargo 同款依赖 `anstream`/`anstyle`
- 交互式输入参考 uv 的 `console` 模块
- 进度条使用 `indicatif`

## 参考资料

### 主流 Rust CLI 应用源码

- [Cargo Shell](https://github.com/rust-lang/cargo/blob/master/src/cargo/core/shell.rs) - Cargo 的输出系统，本 RFC 的主要参考
- [uv console](https://github.com/astral-sh/uv/tree/main/crates) - uv 的控制台交互实现
- [ripgrep printer](https://github.com/BurntSushi/ripgrep/tree/master/crates/printer) - ripgrep 的打印器设计

### 依赖库

- [anstream](https://github.com/rust-cli/anstyle/tree/main/crates/anstream) - Cargo 使用的跨平台 ANSI 流处理
- [anstyle](https://github.com/rust-cli/anstyle) - 零开销样式定义
- [indicatif](https://github.com/console-rs/indicatif) - Rust 进度条库
- [console](https://github.com/console-rs/console) - 终端处理库（uv 使用）
- [dialoguer](https://github.com/console-rs/dialoguer) - 交互式提示库

## 更新记录

| 日期 | 版本 | 变更 |
|------|------|------|
| 2025-12-31 | v0.1.0 | 初始草案 |
| 2025-12-31 | v0.2.0 | 添加主流 Rust CLI 应用方案调研（Cargo, uv, ripgrep） |
