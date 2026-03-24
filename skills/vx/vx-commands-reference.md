# VX Command Quick Reference

## Tool Execution

```bash
vx <tool> [args...]                     # Run any tool (auto-installs)
vx <tool>@<version> [args...]           # Run specific version
vx --with <tool> <target> [args...]     # Multi-runtime composition
```

## Installation & Management

```bash
vx install <tool>[@<version>]           # Install tool
vx uninstall <tool>[@<version>]         # Uninstall tool
vx list [--installed] [--json]          # List tools
vx versions <tool> [--json]             # Available versions
vx which <tool> [--json]               # Find tool path
vx switch <tool>@<version>              # Switch active version
vx search <query> [--json]              # Search tools
```

## Project Management

```bash
vx init [--template <name>]             # Initialize vx.toml
                                        # Templates: node, node-pnpm, node-yarn,
                                        # node-bun, python, python-pip, rust,
                                        # rust-python, go, electron, fullstack,
                                        # openclaw, minimal
vx add <tool>[@<version>]               # Add tool to vx.toml
vx remove <tool>                        # Remove from vx.toml
vx sync [--clean] [--check]             # Sync from vx.toml
vx setup                                # Full project setup
vx lock [--update]                      # Generate/update vx.lock
vx check [--fix] [--json]              # Verify constraints
vx run <script> [--list]               # Run project scripts
vx dev                                  # Enter dev environment
```

## AI Integration

```bash
vx ai setup [--global] [--force]        # Install AI agent skills
vx ai context [--json] [--minimal]      # Generate AI-friendly context
vx ai session init|status|cleanup       # Manage AI sessions
vx analyze [--json]                     # Analyze project structure
```

## Cache & Diagnostics

```bash
vx doctor [--fix]                       # Run diagnostics
vx info                                 # System info
vx cache dir|info|list|clean            # Cache management
vx config show|validate                 # Configuration
```

## Global Flags

```
--json                  JSON output
--format <text|json|toon>  Output format (toon saves 40-60% tokens)
--verbose               Verbose output
--debug                 Debug output
--trace                 Trace-level output
--use-system-path       Use system PATH
--cache-mode <mode>     Cache: normal, refresh, offline, no-cache
```

## Environment Variables

```bash
VX_OUTPUT=json          # Default output format
VX_OUTPUT=toon          # Token-optimized output (recommended for AI)
VX_HOME=~/.vx           # vx home directory
VX_CDN=true             # Enable CDN acceleration
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Tool not found |
| 3 | Installation failed |
| 4 | Version not found |
| 5 | Network error |
| 6 | Permission error |
| 7 | Configuration error |
