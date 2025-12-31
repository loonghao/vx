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

```
crates/vx-console/
├── src/
│   ├── lib.rs           # Public API
│   ├── output.rs        # Output manager
│   ├── style.rs         # Styles and themes
│   ├── progress.rs      # Progress bars and spinners
│   ├── term.rs          # Terminal detection and adaptation
│   ├── log.rs           # Logging integration
│   └── test.rs          # Test support
└── Cargo.toml
```

### Core API

#### 1. Output Manager (Console)

```rust
use vx_console::{Console, OutputMode, Theme};

// Global singleton
let console = Console::global();

// Or create instance
let console = Console::builder()
    .mode(OutputMode::Interactive)  // Interactive | Quiet | Verbose | Json
    .theme(Theme::default())
    .build();

// Basic output
console.info("Installing node...");
console.success("Installed node@20.10.0");
console.warn("Version 18 is deprecated");
console.error("Failed to download");
console.hint("Try: vx install node@20");

// Formatted output
console.info_fmt(format_args!("Installing {}@{}", tool, version));

// Conditional output
console.debug("Cache hit");  // Only shown in verbose mode
console.trace("HTTP GET ...");  // Only shown in trace mode
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

```toml
[dependencies]
# Terminal handling
console = "0.15"           # Terminal detection and styling
indicatif = "0.17"         # Progress bars
colored = "2"              # Colored output

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

### 2. Use only indicatif

**Pros**: Mature library
**Cons**: Doesn't handle logging, output modes, cross-platform adaptation

### 3. Use console + dialoguer

**Pros**: Feature-rich
**Cons**: Needs additional wrapping, API not unified enough

## References

- [indicatif](https://github.com/console-rs/indicatif) - Rust progress bar library
- [console](https://github.com/console-rs/console) - Terminal handling library
- [owo-colors](https://github.com/jam1garner/owo-colors) - Zero-overhead colors library
- [Cargo's output system](https://github.com/rust-lang/cargo/tree/master/src/cargo/core/shell.rs)

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2025-12-31 | v0.1.0 | Initial draft |
