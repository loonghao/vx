# RFC 0009: Unified Console Output System (vx-console)

| Property | Value |
|----------|-------|
| RFC Number | 0009 |
| Title | Unified Console Output System |
| Status | Draft |
| Author | VX Team |
| Created | 2025-12-31 |
| Last Updated | 2025-12-31 |

## Summary

This RFC proposes creating `vx-console` crate to unify cross-platform console output, logging, progress bars, and long-running task interactions.

## Survey of Mainstream Rust CLI Applications

Before designing `vx-console`, we surveyed console output approaches in mainstream Rust CLI applications:

### 1. Cargo (rust-lang/cargo)

**Architecture**: `Shell` struct wrapping stdout/stderr

**Core Design**:
```rust
// cargo/src/cargo/core/shell.rs
pub struct Shell {
    output: ShellOut,           // Stream | Write
    verbosity: Verbosity,       // Verbose | Normal | Quiet
    needs_clear: bool,          // Progress bar cleanup flag
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
    Write(Box<dyn Write>),  // For testing
}
```

**Key Features**:
- Uses `anstream` library for cross-platform ANSI colors
- `ColorChoice` enum: `Always` / `Never` / `CargoAuto`
- `Verbosity` enum controls output level
- Supports hyperlinks (clickable file paths)
- `from_write()` method supports test injection with memory buffer
- Uses `annotate_snippets` for error report rendering

**Dependencies**:
- `anstream` - Cross-platform stream handling
- `anstyle` - Style definitions
- `annotate_snippets` - Error report rendering

### 2. uv (astral-sh/uv)

**Architecture**: Separate `uv-console` crate

**Core Functions**:
```rust
// Confirmation prompt - real-time key response
pub fn confirm(prompt: &str, default: bool) -> Result<bool>;

// Password input - hidden input
pub fn password(prompt: &str) -> Result<String>;

// General text input - supports cursor movement, word-level jumping
pub fn input(prompt: &str) -> Result<String>;

// Byte formatting
pub fn human_readable_bytes(bytes: u64) -> (f64, &'static str);
```

**Key Features**:
- Uses `console` crate for terminal control
- Real-time key response (no Enter needed)
- Supports Ctrl+C exit
- Full editing capabilities (cursor movement, word-level jumping)
- Cross-platform compatible (including Windows special handling)

### 3. ripgrep (BurntSushi/ripgrep)

**Architecture**: Separate `grep-printer` crate

**Printer Types**:
```rust
// Human-readable format
pub struct Standard { ... }

// JSON Lines format (machine-readable)
pub struct JSON { ... }

// Aggregate summary
pub struct Summary { ... }
```

**Key Features**:
- Modular design: color, hyperlink, path, standard, summary
- Supports search and replace
- Multi-line result handling
- JSON output for programmatic processing
- Statistics summary report

### 4. rustup (rust-lang/rustup)

**Architecture**: Built-in terminal handling module

**Key Features**:
- Progress bar for download progress
- Multi-component parallel installation progress
- Auto-detect terminal capabilities

### Comparison

| Feature | Cargo | uv | ripgrep |
|---------|-------|-----|---------|
| Color Library | anstream/anstyle | console | termcolor |
| Progress Bar | Custom | indicatif | None |
| Output Modes | Verbose/Normal/Quiet | Similar | Standard/JSON/Summary |
| Test Support | Write trait injection | Yes | Sink trait |
| Interactive Input | No | Yes | No |
| Hyperlinks | Yes | No | Yes |

### Design Insights

Based on this survey, `vx-console` should adopt:

1. **Cargo-style Shell architecture** - Wrap stdout/stderr, support test injection
2. **anstream/anstyle** - Modern cross-platform color library used by Cargo, replacing colored
3. **uv-style interactive input** - Support confirmation, password input, etc.
4. **ripgrep-style multiple output formats** - Standard/JSON/Summary
5. **Unified Verbosity control** - Verbose/Normal/Quiet

## Motivation

### Current Problems

1. **Code Duplication**: Similar output logic exists in multiple crates
   - `vx-cli/src/ui.rs` (730+ lines)
   - `vx-installer/src/progress.rs` (270 lines)
   - `vx-runtime/src/impls.rs` directly uses indicatif

2. **Inconsistent Styling**: Different modules have different output styles
   - Some use `✓`, others use `✔`
   - Inconsistent color schemes
   - Various spinner animation styles

3. **Cross-Platform Issues**: Windows terminal support for Unicode and ANSI colors is inconsistent
   - Windows Terminal vs CMD vs PowerShell
   - CI/CD environments (no TTY)

4. **Scattered `println!`**: 40+ files have direct output, hard to control uniformly

5. **Logging vs User Output Confusion**: `tracing` logs mixed with user-facing output

### Goals

- **Unified API**: One crate handles all output needs
- **Cross-Platform Compatibility**: Auto-adapt to Windows/macOS/Linux and different terminals
- **Testability**: Support capturing output for testing
- **Configurability**: Support quiet/verbose/json output modes
- **Progress Management**: Unified progress bar and spinner management

## Design

### Crate Structure

Based on Cargo's Shell architecture and uv's interactive design:

```
crates/vx-console/
├── src/
│   ├── lib.rs           # Public API
│   ├── shell.rs         # Shell struct (reference Cargo)
│   ├── output.rs        # ShellOut output abstraction
│   ├── style.rs         # Styles and themes (using anstyle)
│   ├── progress.rs      # Progress bars and spinners (using indicatif)
│   ├── term.rs          # Terminal detection and adaptation
│   ├── interact.rs      # Interactive input (reference uv)
│   ├── format.rs        # Output formatting (Standard/JSON)
│   └── test.rs          # Test support
└── Cargo.toml
```

### Core API

#### 1. Shell Struct (Reference Cargo)

```rust
use vx_console::{Shell, Verbosity, ColorChoice};

// Create Shell (reference Cargo design)
let mut shell = Shell::new();

// Or use builder
let shell = Shell::builder()
    .verbosity(Verbosity::Normal)  // Verbose | Normal | Quiet
    .color_choice(ColorChoice::Auto)  // Always | Never | Auto
    .build();

// Status messages (Cargo style)
shell.status("Compiling", "vx v0.1.0")?;      // Green "Compiling"
shell.status_with_color("Downloading", "node@20", Color::Cyan)?;

// Basic output
shell.info("Installing node...")?;
shell.success("Installed node@20.10.0")?;
shell.warn("Version 18 is deprecated")?;
shell.error("Failed to download")?;
shell.hint("Try: vx install node@20")?;

// Conditional output
shell.verbose(|s| s.info("Cache hit"))?;  // Only shown in verbose mode

// Test support (reference Cargo)
let mut output = Vec::new();
let shell = Shell::from_write(Box::new(&mut output));
```

#### 2. Output Manager (Console) - Global Singleton

```rust
use vx_console::{Console, OutputMode};

// Global singleton (convenient API)
let console = Console::global();

// Basic output
console.info("Installing node...");
console.success("Installed node@20.10.0");
console.warn("Version 18 is deprecated");
console.error("Failed to download");
console.hint("Try: vx install node@20");

// Set mode
console.set_verbosity(Verbosity::Verbose);
console.set_color_choice(ColorChoice::Never);
```

#### 2. Progress Bars and Spinners

```rust
use vx_console::{Console, SpinnerStyle};

let console = Console::global();

// Simple spinner
let spinner = console.spinner("Downloading node...");
// ... perform operation
spinner.success("Downloaded node");

// Download with progress
let progress = console.download_progress("Downloading node", total_size);
progress.set_position(downloaded);
progress.finish("Downloaded 45.2 MB");

// Multi-task progress
let multi = console.multi_progress("Installing tools", 3);
multi.start_task("node@20");
multi.complete_task(true);
multi.start_task("npm@10");
multi.complete_task(true);
multi.finish("Installed 3 tools");

// Auto-select style
// - Show animated spinner in TTY
// - Show static message in non-TTY
// - Show simplified output in CI
```

#### 3. Long-Running Tasks

```rust
use vx_console::{Console, Task};

let console = Console::global();

// Task with timing
let result = console.task("Building project", || {
    // ... perform operation
    Ok(())
})?;
// Output: ✓ Building project (2.3s)

// Async task
let result = console.task_async("Fetching versions", async {
    // ... async operation
    Ok(versions)
}).await?;

// Task with steps
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

#### 4. Terminal Adaptation

```rust
use vx_console::{Term, TermCapabilities};

let term = Term::detect();

// Check capabilities
if term.supports_color() {
    // Use colored output
}

if term.supports_unicode() {
    // Use Unicode characters
}

if term.is_interactive() {
    // Show animations
}

// Get terminal size
let (width, height) = term.size();

// Clear screen
term.clear_screen();

// Move cursor
term.move_cursor_up(2);
```

#### 5. Output Modes

```rust
use vx_console::{Console, OutputMode};

// Interactive mode (default)
// - Colored output
// - Animated spinners
// - Progress bars

// Quiet mode
// - Only show errors
// - No animations
let console = Console::builder().mode(OutputMode::Quiet).build();

// Verbose mode
// - Show debug info
// - Show timing
let console = Console::builder().mode(OutputMode::Verbose).build();

// JSON mode (for script integration)
// - Structured output
// - No colors
let console = Console::builder().mode(OutputMode::Json).build();
// Output: {"level":"info","message":"Installing node","tool":"node","version":"20"}

// CI mode (auto-detected)
// - No animations
// - Simplified progress
// - GitHub Actions annotations
let console = Console::builder().mode(OutputMode::Ci).build();
// Output: ::group::Installing node
//         ...
//         ::endgroup::
```

#### 6. Theme System

```rust
use vx_console::{Theme, Style, Color};

// Default theme
let theme = Theme::default();

// Custom theme
let theme = Theme::builder()
    .success(Style::new().fg(Color::Green).bold())
    .error(Style::new().fg(Color::Red).bold())
    .warn(Style::new().fg(Color::Yellow))
    .info(Style::new().fg(Color::Blue))
    .hint(Style::new().fg(Color::Cyan).dimmed())
    .spinner_chars(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    .progress_chars("━━╺")
    .build();

// Built-in themes
let theme = Theme::minimal();    // No colors, ASCII chars
let theme = Theme::colorful();   // Rich colors
let theme = Theme::github();     // GitHub Actions style
```

#### 7. Test Support

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

    // Verify output
    assert!(output.contains("Hello"));
    assert!(output.has_success("Done"));

    // Get all output
    let lines = output.lines();
    assert_eq!(lines.len(), 2);
}
```

#### 8. Logging Integration

```rust
use vx_console::{Console, LogBridge};

// Bridge tracing logs to console
let console = Console::global();
let _guard = console.bridge_tracing();

// Now tracing::info! outputs through console
tracing::info!("Starting download");
// Output: ℹ Starting download

// Or separate logs and user output
let console = Console::builder()
    .log_to_file("vx.log")  // Logs write to file
    .build();

// tracing writes to file, console outputs to terminal
```

#### 9. Interactive Input (Reference uv)

```rust
use vx_console::{Console, interact};

let console = Console::global();

// Confirmation prompt - real-time key response (no Enter needed)
let confirmed = console.confirm("Proceed with installation?", true)?;
// Output: ? Proceed with installation? [Y/n]

// Confirmation with default
let confirmed = console.confirm_default("Override existing?", false)?;

// Password input - hidden input content
let password = console.password("Enter token:")?;

// General text input - supports cursor movement, word-level jumping
let name = console.input("Project name:")?;

// Selection list
let choice = console.select("Choose package manager:", &["npm", "yarn", "pnpm"])?;

// Multi-select
let choices = console.multi_select("Select tools:", &["node", "python", "go"])?;
```

### Cross-Platform Adaptation

#### Windows Support

```rust
// Auto-handle Windows terminal differences
impl Term {
    fn detect() -> Self {
        #[cfg(windows)]
        {
            // Detect Windows Terminal
            if std::env::var("WT_SESSION").is_ok() {
                return Self::windows_terminal();
            }

            // Detect ConEmu/Cmder
            if std::env::var("ConEmuANSI").is_ok() {
                return Self::conemu();
            }

            // Enable ANSI support (Windows 10+)
            if enable_virtual_terminal_processing() {
                return Self::windows_ansi();
            }

            // Fallback to basic mode
            Self::windows_basic()
        }

        #[cfg(unix)]
        {
            Self::unix()
        }
    }
}
```

#### CI Environment Detection

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

### Migration Plan

#### Phase 1: Create vx-console crate

1. Implement core API
2. Implement cross-platform terminal detection
3. Implement basic output methods
4. Add unit tests

#### Phase 2: Migrate vx-cli/ui.rs

1. Migrate `UI` struct to vx-console
2. Migrate `ProgressSpinner` etc. to vx-console
3. Update vx-cli to use vx-console
4. Delete vx-cli/ui.rs

#### Phase 3: Migrate vx-installer/progress.rs

1. Unify `ProgressReporter` trait
2. Update vx-installer to use vx-console
3. Delete duplicate code

#### Phase 4: Clean up scattered println!

1. Audit all `println!/eprintln!` usage
2. Replace with `Console::global()` calls
3. Add clippy lint to forbid direct println

#### Phase 5: Logging Integration

1. Implement tracing bridge
2. Unify logging and user output
3. Add log file support

### Dependencies

Based on the survey of mainstream Rust CLI applications, recommended dependencies:

```toml
[dependencies]
# Terminal handling (modern approach used by Cargo)
anstream = "0.6"           # Cross-platform ANSI stream handling (used by Cargo)
anstyle = "1.0"            # Style definitions (pairs with anstream)
console = "0.15"           # Terminal detection and interaction (used by uv)

# Progress bars
indicatif = "0.17"         # Progress bars and spinners

# Cross-platform
terminal_size = "0.3"      # Terminal size

# Logging
tracing = { workspace = true }
tracing-subscriber = { workspace = true, optional = true }

# Serialization (JSON mode)
serde = { workspace = true }
serde_json = { workspace = true }

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.52", features = ["Win32_System_Console"] }

[features]
default = ["progress"]
progress = []
log-bridge = ["tracing-subscriber"]
```

**Why anstream/anstyle instead of colored**:
- `anstream` is the library officially used by Cargo, validated at scale
- Automatically handles terminal capability detection and ANSI escape code adaptation
- `anstyle` provides zero-overhead style definitions
- Better cross-platform support (especially Windows)

### API Examples

#### Complete Example: vx sync

```rust
use vx_console::{Console, OutputMode};

pub async fn handle_sync(args: SyncArgs) -> Result<()> {
    let console = Console::global();

    // Set mode
    if args.quiet {
        console.set_mode(OutputMode::Quiet);
    } else if args.verbose {
        console.set_mode(OutputMode::Verbose);
    }

    // Start task
    let spinner = console.spinner("Reading vx.toml...");

    let config = read_config()?;
    spinner.success("Read vx.toml");

    // Multi-tool installation
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

#### Complete Example: Download Progress

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

## Implementation Priority

| Priority | Feature | Reason |
|----------|---------|--------|
| P0 | Basic output API | Core functionality |
| P0 | Cross-platform terminal detection | Windows compatibility |
| P1 | Progress bars and spinners | User experience |
| P1 | Output modes | CLI requirements |
| P2 | Theme system | Customizability |
| P2 | Test support | Code quality |
| P3 | Logging integration | Unified logging |
| P3 | JSON mode | Script integration |

## Alternatives

### 1. Continue with scattered implementations

**Pros**: No migration needed
**Cons**: Code duplication, inconsistent styling, hard to maintain

### 2. Directly use Cargo's Shell module

**Pros**: Validated at scale
**Cons**: Cargo's Shell is not a separate crate, would need to copy code

### 3. Use only indicatif + console

**Pros**: Mature library combination
**Cons**: Lacks unified API wrapper, needs composition at each usage site

### 4. Use dialoguer

**Pros**: Feature-rich interactive input
**Cons**: Mainly focused on interaction, doesn't handle general output and progress bars

### Recommended Approach

Adopt **Cargo-style Shell architecture** + **uv-style interactive input**:
- Core design references Cargo's `Shell` struct
- Use same dependencies as Cargo: `anstream`/`anstyle`
- Interactive input references uv's `console` module
- Progress bars use `indicatif`

## References

### Mainstream Rust CLI Application Source Code

- [Cargo Shell](https://github.com/rust-lang/cargo/blob/master/src/cargo/core/shell.rs) - Cargo's output system, main reference for this RFC
- [uv console](https://github.com/astral-sh/uv/tree/main/crates) - uv's console interaction implementation
- [ripgrep printer](https://github.com/BurntSushi/ripgrep/tree/master/crates/printer) - ripgrep's printer design

### Dependency Libraries

- [anstream](https://github.com/rust-cli/anstyle/tree/main/crates/anstream) - Cross-platform ANSI stream handling used by Cargo
- [anstyle](https://github.com/rust-cli/anstyle) - Zero-overhead style definitions
- [indicatif](https://github.com/console-rs/indicatif) - Rust progress bar library
- [console](https://github.com/console-rs/console) - Terminal handling library (used by uv)
- [dialoguer](https://github.com/console-rs/dialoguer) - Interactive prompt library

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-31 | v0.1.0 | Initial draft |
| 2025-12-31 | v0.2.0 | Added survey of mainstream Rust CLI applications (Cargo, uv, ripgrep) |
