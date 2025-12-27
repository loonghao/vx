# CLI Command Development Guide

This guide explains how to add new CLI commands to vx. The CLI uses a **Command Trait** pattern for clean, maintainable command routing.

## Architecture Overview

vx CLI follows a modular architecture:

```
┌─────────────────────────────────────────────────────────────┐
│                        vx-cli                                │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                     cli.rs                            │   │
│  │  - Cli struct (clap Parser)                          │   │
│  │  - Commands enum (all subcommands)                   │   │
│  │  - CommandHandler impl for Commands                  │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │                    lib.rs                             │   │
│  │  - main() entry point                                │   │
│  │  - CommandContext creation                           │   │
│  │  - command.execute(&ctx)                             │   │
│  └──────────────────────────────────────────────────────┘   │
│                           │                                  │
│                           ▼                                  │
│  ┌──────────────────────────────────────────────────────┐   │
│  │               commands/*.rs                           │   │
│  │  - Individual command implementations                │   │
│  │  - handle() functions                                │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### CommandHandler Trait

```rust
// commands/handler.rs

/// Unified context for command execution
pub struct CommandContext {
    pub registry: Arc<ProviderRegistry>,
    pub runtime_context: Arc<RuntimeContext>,
    pub use_system_path: bool,
    pub verbose: bool,
    pub debug: bool,
}

/// Trait for command handlers
#[async_trait]
pub trait CommandHandler: Send + Sync {
    /// Execute the command
    async fn execute(&self, ctx: &CommandContext) -> Result<()>;

    /// Get the command name (for logging)
    fn name(&self) -> &'static str {
        "unknown"
    }
}
```

### Commands Enum

All commands are defined in `cli.rs` as variants of the `Commands` enum:

```rust
#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Show version information
    Version,

    /// Install a specific tool version
    #[command(alias = "i")]
    Install {
        tool: String,
        version: Option<String>,
        #[arg(short, long)]
        force: bool,
    },

    // ... more commands
}
```

## Adding a New Command

### Step 1: Define the Command in cli.rs

Add a new variant to the `Commands` enum:

```rust
// In cli.rs

#[derive(Subcommand, Clone)]
pub enum Commands {
    // ... existing commands ...

    /// My new command description
    #[command(alias = "my")]  // Optional: short alias
    MyCommand {
        /// Required argument
        name: String,

        /// Optional argument with default
        #[arg(long, default_value = "default")]
        option: String,

        /// Boolean flag
        #[arg(short, long)]
        verbose: bool,
    },
}
```

### Step 2: Add Command Name in CommandHandler

Update the `name()` method in the `CommandHandler` impl:

```rust
// In cli.rs, inside impl CommandHandler for Commands

fn name(&self) -> &'static str {
    match self {
        // ... existing matches ...
        Commands::MyCommand { .. } => "my-command",
    }
}
```

### Step 3: Add Execute Branch

Add the execution logic in the `execute()` method:

```rust
// In cli.rs, inside impl CommandHandler for Commands

async fn execute(&self, ctx: &CommandContext) -> Result<()> {
    match self {
        // ... existing matches ...

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

### Step 4: Create Command Module

Create `commands/my_command.rs`:

```rust
//! My command implementation

use anyhow::Result;
use crate::ui::UI;
use vx_runtime::{ProviderRegistry, RuntimeContext};

/// Handle the my-command command
pub async fn handle(
    registry: &ProviderRegistry,
    context: &RuntimeContext,
    name: &str,
    option: &str,
    verbose: bool,
) -> Result<()> {
    if verbose {
        UI::info(&format!("Running my-command with name={}, option={}", name, option));
    }

    // Your implementation here
    UI::success(&format!("Successfully processed: {}", name));

    Ok(())
}
```

### Step 5: Register the Module

Add the module to `commands/mod.rs`:

```rust
// In commands/mod.rs

pub mod my_command;  // Add this line
```

## Command Patterns

### Simple Command (No Arguments)

```rust
// cli.rs
Commands::Stats,

// execute()
Commands::Stats => commands::stats::handle(ctx.registry()).await,

// commands/stats.rs
pub async fn handle(registry: &ProviderRegistry) -> Result<()> {
    // Implementation
    Ok(())
}
```

### Command with Subcommands

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

### Command with Registry Access

```rust
pub async fn handle(
    registry: &ProviderRegistry,
    tool_name: &str,
) -> Result<()> {
    // Get runtime from registry
    let runtime = registry.get_runtime(tool_name)
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", tool_name))?;

    // Use the runtime
    let versions = runtime.fetch_versions(&ctx).await?;

    Ok(())
}
```

### Command with Progress Indicator

```rust
use crate::ui::{ProgressSpinner, UI};

pub async fn handle(name: &str) -> Result<()> {
    let spinner = ProgressSpinner::new(&format!("Processing {}...", name));

    // Do work...
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    spinner.finish_with_message(&format!("✓ Completed {}", name));

    Ok(())
}
```

## UI Helpers

The `ui` module provides consistent output formatting:

```rust
use crate::ui::UI;

// Information messages
UI::info("Processing...");

// Success messages
UI::success("Operation completed!");

// Warning messages
UI::warning("This might take a while");

// Error messages
UI::error("Something went wrong");

// Hints/tips
UI::hint("Use --force to override");

// Details
UI::detail(&format!("Installed to: {}", path.display()));

// Tool not found (with suggestions)
UI::tool_not_found("nod", &["node", "npm", "npx"]);
```

## Testing Commands

Create tests in `crates/vx-cli/tests/`:

```rust
// tests/my_command_tests.rs

use rstest::rstest;
use vx_cli::commands::my_command;

#[rstest]
#[tokio::test]
async fn test_my_command_basic() {
    // Test implementation
}

#[rstest]
#[case("input1", "expected1")]
#[case("input2", "expected2")]
#[tokio::test]
async fn test_my_command_parametrized(
    #[case] input: &str,
    #[case] expected: &str,
) {
    // Parametrized test
}
```

## Best Practices

### 1. Keep Commands Focused

Each command should do one thing well:

```rust
// Good: focused commands
Commands::Install { tool, version, force },
Commands::Uninstall { tool, version, force },

// Avoid: overloaded commands
Commands::Manage { action, tool, version, force, ... },
```

### 2. Provide Helpful Aliases

```rust
#[command(alias = "i")]
Install { ... },

#[command(alias = "rm", alias = "remove")]
Uninstall { ... },

#[command(alias = "ls")]
List { ... },
```

### 3. Use Consistent Argument Names

```rust
// Consistent naming across commands
--force, -f     // Force operation
--verbose, -v   // Verbose output
--dry-run       // Preview without executing
--all, -a       // Apply to all items
```

### 4. Validate Early

```rust
pub async fn handle(tool: &str, version: Option<&str>) -> Result<()> {
    // Validate inputs early
    if tool.is_empty() {
        return Err(anyhow::anyhow!("Tool name cannot be empty"));
    }

    // Continue with valid inputs
    Ok(())
}
```

### 5. Provide Context in Errors

```rust
let runtime = registry.get_runtime(tool_name)
    .ok_or_else(|| {
        let available = registry.runtime_names();
        anyhow::anyhow!(
            "Tool '{}' not found. Available tools: {}",
            tool_name,
            available.join(", ")
        )
    })?;
```

## File Structure

```
crates/vx-cli/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Entry point, VxCli struct
│   ├── cli.rs              # Cli, Commands, CommandHandler impl
│   ├── registry.rs         # Provider registry setup
│   ├── ui.rs               # UI helpers
│   └── commands/
│       ├── mod.rs          # Module exports
│       ├── handler.rs      # CommandHandler trait, CommandContext
│       ├── install.rs      # install command
│       ├── list.rs         # list command
│       ├── version.rs      # version command
│       └── ...             # Other commands
└── tests/
    ├── cli_parsing_tests.rs
    └── ...
```

## See Also

- [Provider Development Guide](./plugin-development) - Add new tool support
- [Extension Development Guide](./extension-development) - Add scripted extensions
- [Architecture Overview](./architecture) - System architecture
- [Contributing Guide](./contributing) - How to contribute
