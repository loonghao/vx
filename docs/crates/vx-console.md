# vx-console

Unified console output system for vx.

## Overview

`vx-console` provides a consistent API for console output, progress bars, and terminal interactions across all vx components. It is inspired by Cargo's Shell implementation and uv's console module.

## Features

- **Shell**: Cargo-style output abstraction with verbosity control
- **Progress**: Unified progress bars and spinners using indicatif
- **Terminal**: Cross-platform terminal detection and adaptation
- **Interact**: Interactive input (confirm, password, select)
- **Theme**: Customizable output themes
- **Test**: Testing utilities for capturing output

## Quick Start

### Basic Output

```rust
use vx_console::{Shell, Verbosity, ColorChoice};

// Create a shell
let mut shell = Shell::new();

// Basic output
shell.status("Compiling", "vx v0.1.0")?;
shell.info("Installing node...")?;
shell.success("Installed node@20.10.0")?;
shell.warn("Version 18 is deprecated")?;
shell.error("Failed to download")?;
shell.hint("Try: vx install node@20")?;
```

### Global Console

```rust
use vx_console::Console;

// Get global console instance
let console = Console::global();
let guard = console.read().unwrap();

guard.info("Hello, world!");
guard.success("Done!");
```

### Progress Bars

```rust
use vx_console::Console;

let console = Console::new();

// Spinner
let spinner = console.spinner("Downloading...");
// ... do work ...
spinner.finish_success("Downloaded");

// Download progress
let progress = console.download_progress("Downloading", total_size);
progress.set_position(downloaded);
progress.finish_with_message("Downloaded 45.2 MB");
```

### Interactive Input

```rust
use vx_console::{confirm, password, select};

// Confirmation
let confirmed = confirm("Proceed with installation?", true)?;

// Password input
let token = password("Enter API token:")?;

// Selection
let choice = select("Choose package manager:", &["npm", "yarn", "pnpm"])?;
```

## API Reference

### Shell

The `Shell` struct is the core abstraction for console output.

```rust
pub struct Shell {
    // ...
}

impl Shell {
    // Create a new shell
    pub fn new() -> Self;

    // Create with builder
    pub fn builder() -> ShellBuilder;

    // Output methods
    pub fn status(&self, status: impl Display, message: impl Display) -> Result<()>;
    pub fn info(&self, message: impl Display) -> Result<()>;
    pub fn success(&self, message: impl Display) -> Result<()>;
    pub fn warn(&self, message: impl Display) -> Result<()>;
    pub fn error(&self, message: impl Display) -> Result<()>;
    pub fn hint(&self, message: impl Display) -> Result<()>;
    pub fn debug(&self, message: impl Display) -> Result<()>;

    // Verbosity control
    pub fn set_verbosity(&mut self, verbosity: Verbosity);
    pub fn verbosity(&self) -> Verbosity;
}
```

### Verbosity

```rust
pub enum Verbosity {
    Quiet,   // Only show errors
    Normal,  // Default output level
    Verbose, // Show debug information
}
```

### ColorChoice

```rust
pub enum ColorChoice {
    Always, // Always use colors
    Never,  // Never use colors
    Auto,   // Auto-detect based on terminal
}
```

### Theme

Customize output appearance:

```rust
use vx_console::{Theme, Style, Color};

let theme = Theme::builder()
    .success(Style::new().fg(Color::Green).bold())
    .error(Style::new().fg(Color::Red).bold())
    .spinner_chars(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    .progress_chars("━━╺")
    .build();
```

Built-in themes:
- `Theme::default()` - Standard colorful theme
- `Theme::minimal()` - No colors, ASCII characters
- `Theme::colorful()` - Bright colors
- `Theme::github()` - GitHub Actions compatible

### Progress Manager

Manage multiple progress bars:

```rust
use vx_console::ProgressManager;

let pm = ProgressManager::new();

// Add spinner
let spinner = pm.add_spinner("Loading...");
spinner.finish_success("Done");

// Add download progress
let download = pm.add_download(total_size, "Downloading");
download.set_position(downloaded);
download.finish_with_message("Complete");

// Suspend for other output
pm.suspend(|| {
    println!("Important message");
});
```

### Terminal Detection

```rust
use vx_console::Term;

let term = Term::detect();

if term.supports_color() {
    // Use colored output
}

if term.supports_unicode() {
    // Use Unicode characters
}

if term.is_interactive() {
    // Show animations
}

if term.is_ci() {
    // Simplify output for CI
}
```

### CI Environment

```rust
use vx_console::CiEnvironment;

if let Some(ci) = CiEnvironment::detect() {
    match ci {
        CiEnvironment::GitHubActions => {
            // Use GitHub Actions annotations
        }
        CiEnvironment::GitLabCi => {
            // Use GitLab CI format
        }
        _ => {}
    }
}
```

### JSON Output

For programmatic consumption:

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

### Testing

```rust
use vx_console::TestOutput;

#[test]
fn test_output() {
    let output = TestOutput::new();

    // Write to output
    output.write("✓ Success");
    output.write("✗ Error");

    // Verify
    assert!(output.has_success("Success"));
    assert!(output.has_error("Error"));
    assert!(output.contains("Success"));
}
```

## Cross-Platform Support

`vx-console` automatically handles:

- **Windows**: Enables ANSI support on Windows 10+, falls back to basic output on older systems
- **macOS/Linux**: Full ANSI color and Unicode support
- **CI Environments**: Detects GitHub Actions, GitLab CI, Jenkins, etc. and adjusts output accordingly
- **NO_COLOR**: Respects the NO_COLOR environment variable

## Dependencies

- `anstream` / `anstyle` - Cross-platform ANSI stream handling (same as Cargo)
- `indicatif` - Progress bars and spinners
- `console` - Terminal detection and interaction (same as uv)
- `terminal_size` - Terminal size detection

## Migration from vx-cli/ui.rs

If you're migrating from the old `UI` struct:

| Old API | New API |
|---------|---------|
| `UI::info(msg)` | `shell.info(msg)?` |
| `UI::success(msg)` | `shell.success(msg)?` |
| `UI::warn(msg)` | `shell.warn(msg)?` |
| `UI::error(msg)` | `shell.error(msg)?` |
| `UI::set_verbose(true)` | `shell.set_verbosity(Verbosity::Verbose)` |
| `ProgressSpinner::new(msg)` | `pm.add_spinner(msg)` |
| `DownloadProgress::new(size, msg)` | `pm.add_download(size, msg)` |
