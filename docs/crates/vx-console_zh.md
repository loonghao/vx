# vx-console

vx 统一控制台输出系统。

## 概述

`vx-console` 为所有 vx 组件提供一致的控制台输出、进度条和终端交互 API。设计灵感来自 Cargo 的 Shell 实现和 uv 的 console 模块。

## 功能特性

- **Shell**: Cargo 风格的输出抽象，支持详细级别控制
- **Progress**: 基于 indicatif 的统一进度条和旋转动画
- **Terminal**: 跨平台终端检测和适配
- **Interact**: 交互式输入（确认、密码、选择）
- **Theme**: 可定制的输出主题
- **Test**: 用于捕获输出的测试工具

## 快速开始

### 基本输出

```rust
use vx_console::{Shell, Verbosity, ColorChoice};

// 创建 shell
let mut shell = Shell::new();

// 基本输出
shell.status("Compiling", "vx v0.1.0")?;
shell.info("Installing node...")?;
shell.success("Installed node@20.10.0")?;
shell.warn("Version 18 is deprecated")?;
shell.error("Failed to download")?;
shell.hint("Try: vx install node@20")?;
```

### 全局 Console

```rust
use vx_console::Console;

// 获取全局 console 实例
let console = Console::global();
let guard = console.read().unwrap();

guard.info("Hello, world!");
guard.success("Done!");
```

### 进度条

```rust
use vx_console::Console;

let console = Console::new();

// Spinner
let spinner = console.spinner("Downloading...");
// ... 执行操作 ...
spinner.finish_success("Downloaded");

// 下载进度
let progress = console.download_progress("Downloading", total_size);
progress.set_position(downloaded);
progress.finish_with_message("Downloaded 45.2 MB");
```

### 交互式输入

```rust
use vx_console::{confirm, password, select};

// 确认
let confirmed = confirm("Proceed with installation?", true)?;

// 密码输入
let token = password("Enter API token:")?;

// 选择
let choice = select("Choose package manager:", &["npm", "yarn", "pnpm"])?;
```

## API 参考

### Shell

`Shell` 结构体是控制台输出的核心抽象。

```rust
pub struct Shell {
    // ...
}

impl Shell {
    // 创建新 shell
    pub fn new() -> Self;

    // 使用 builder 创建
    pub fn builder() -> ShellBuilder;

    // 输出方法
    pub fn status(&self, status: impl Display, message: impl Display) -> Result<()>;
    pub fn info(&self, message: impl Display) -> Result<()>;
    pub fn success(&self, message: impl Display) -> Result<()>;
    pub fn warn(&self, message: impl Display) -> Result<()>;
    pub fn error(&self, message: impl Display) -> Result<()>;
    pub fn hint(&self, message: impl Display) -> Result<()>;
    pub fn debug(&self, message: impl Display) -> Result<()>;

    // 详细级别控制
    pub fn set_verbosity(&mut self, verbosity: Verbosity);
    pub fn verbosity(&self) -> Verbosity;
}
```

### Verbosity（详细级别）

```rust
pub enum Verbosity {
    Quiet,   // 只显示错误
    Normal,  // 默认输出级别
    Verbose, // 显示调试信息
}
```

### ColorChoice（颜色选择）

```rust
pub enum ColorChoice {
    Always, // 始终使用颜色
    Never,  // 从不使用颜色
    Auto,   // 根据终端自动检测
}
```

### Theme（主题）

自定义输出外观：

```rust
use vx_console::{Theme, Style, Color};

let theme = Theme::builder()
    .success(Style::new().fg(Color::Green).bold())
    .error(Style::new().fg(Color::Red).bold())
    .spinner_chars(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    .progress_chars("━━╺")
    .build();
```

内置主题：
- `Theme::default()` - 标准彩色主题
- `Theme::minimal()` - 无颜色，ASCII 字符
- `Theme::colorful()` - 明亮颜色
- `Theme::github()` - GitHub Actions 兼容

### ProgressManager（进度管理器）

管理多个进度条：

```rust
use vx_console::ProgressManager;

let pm = ProgressManager::new();

// 添加 spinner
let spinner = pm.add_spinner("Loading...");
spinner.finish_success("Done");

// 添加下载进度
let download = pm.add_download(total_size, "Downloading");
download.set_position(downloaded);
download.finish_with_message("Complete");

// 暂停以进行其他输出
pm.suspend(|| {
    println!("Important message");
});
```

### 终端检测

```rust
use vx_console::Term;

let term = Term::detect();

if term.supports_color() {
    // 使用彩色输出
}

if term.supports_unicode() {
    // 使用 Unicode 字符
}

if term.is_interactive() {
    // 显示动画
}

if term.is_ci() {
    // 为 CI 简化输出
}
```

### CI 环境

```rust
use vx_console::CiEnvironment;

if let Some(ci) = CiEnvironment::detect() {
    match ci {
        CiEnvironment::GitHubActions => {
            // 使用 GitHub Actions 注解
        }
        CiEnvironment::GitLabCi => {
            // 使用 GitLab CI 格式
        }
        _ => {}
    }
}
```

### JSON 输出

用于程序化消费：

```rust
use vx_console::JsonOutput;

let output = JsonOutput::info("Installing node")
    .with_context(serde_json::json!({
        "tool": "node",
        "version": "20.10.0"
    }));

println!("{}", output.to_json());
// {"level":"info","message":"Installing node","context":{"tool":"node","version":"20.10.0"}}
```

### 测试支持

```rust
use vx_console::TestOutput;

#[test]
fn test_output() {
    let output = TestOutput::new();

    // 写入输出
    output.write("✓ Success");
    output.write("✗ Error");

    // 验证
    assert!(output.has_success("Success"));
    assert!(output.has_error("Error"));
    assert!(output.contains("Success"));
}
```

## 跨平台支持

`vx-console` 自动处理：

- **Windows**: 在 Windows 10+ 上启用 ANSI 支持，在旧系统上回退到基本输出
- **macOS/Linux**: 完整的 ANSI 颜色和 Unicode 支持
- **CI 环境**: 检测 GitHub Actions、GitLab CI、Jenkins 等并相应调整输出
- **NO_COLOR**: 遵守 NO_COLOR 环境变量

## 依赖

- `anstream` / `anstyle` - 跨平台 ANSI 流处理（与 Cargo 相同）
- `indicatif` - 进度条和旋转动画
- `console` - 终端检测和交互（与 uv 相同）
- `terminal_size` - 终端尺寸检测

## 从 vx-cli/ui.rs 迁移

如果你正在从旧的 `UI` 结构体迁移：

| 旧 API | 新 API |
|--------|--------|
| `UI::info(msg)` | `shell.info(msg)?` |
| `UI::success(msg)` | `shell.success(msg)?` |
| `UI::warn(msg)` | `shell.warn(msg)?` |
| `UI::error(msg)` | `shell.error(msg)?` |
| `UI::set_verbose(true)` | `shell.set_verbosity(Verbosity::Verbose)` |
| `ProgressSpinner::new(msg)` | `pm.add_spinner(msg)` |
| `DownloadProgress::new(size, msg)` | `pm.add_download(size, msg)` |
