# Frequently Asked Questions

## General

### What is vx?

vx is a universal development tool manager that provides a zero learning curve experience. Simply prefix any command you already know with `vx`, and tools are auto-installed and executed.

### How is vx different from asdf / mise / proto?

| Feature | vx | asdf | mise | proto |
|---------|-----|------|------|-------|
| Zero learning curve | ✅ | ❌ | ❌ | ❌ |
| Auto-install on use | ✅ | ❌ | ✅ | ✅ |
| 48+ built-in tools | ✅ | Plugins | ✅ | Limited |
| Declarative config | ✅ | ✅ | ✅ | ✅ |
| Native Windows | ✅ | ❌ | ✅ | ✅ |
| Script system | ✅ | ❌ | ✅ | ❌ |
| Extension system | ✅ | ❌ | ❌ | ❌ |
| Global package isolation | ✅ | ❌ | ❌ | ❌ |
| Written in | Rust | Shell | Rust | Rust |

### Where does vx store data?

By default in `~/.vx/`:

```
~/.vx/
├── store/        # Installed tool versions
├── cache/        # Download cache
├── bin/          # Global shims
├── envs/         # Virtual environments
├── providers/    # Custom providers
└── config/       # Configuration
```

Override with the `VX_HOME` environment variable.

### Does vx require admin/root permissions?

No. vx installs in the user directory and all operations stay within user permissions.

## Installation

### Which platforms are supported?

- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

### How do I uninstall vx?

```bash
# Remove binary and data
rm -rf ~/.vx
# Remove vx-related lines from your shell config
```

### How do I update vx?

```bash
vx self-update
```

## Tool Management

### How do I install a specific version?

```bash
vx install node@22.11.0     # Exact version
vx install node@22          # Latest 22.x
vx install node@lts         # Latest LTS
vx install "node@^22"       # Semver range
```

### Can I install multiple tools at once?

```bash
vx install node@22 python@3.12 go@1.23 uv@latest
```

### Are tools auto-installed on first use?

Yes! Run `vx node --version` and if Node.js isn't installed, vx automatically downloads and installs the latest stable version.

### How do I see installed tools?

```bash
vx list --installed
```

### How do I clean up old versions?

```bash
vx cache prune              # Clean expired cache
vx uninstall node@18        # Remove specific version
```

## Project Configuration

### What's the difference between vx.toml and vx.lock?

- **vx.toml** — Declarative tool requirements (version ranges), human-readable
- **vx.lock** — Exact pinned versions for reproducible environments

### How do I ensure my team uses the same tools?

1. Add `vx.toml` to your project
2. Run `vx lock` to generate the lock file
3. Commit both files to version control
4. Team members run `vx setup` to get started

## Shell Integration

### Which shells are supported?

Bash, Zsh, Fish, and PowerShell.

### What does shell integration provide?

- Auto-switch tool versions when entering project directories
- Tab completion
- Automatic PATH configuration

## CI/CD

### How do I use vx in GitHub Actions?

```yaml
- uses: loonghao/vx@main
  with:
    tools: node@22 python@3.12
```

Or use `vx setup` to install all tools from `vx.toml`.

### Are there Docker images?

Yes. `vx:latest` and `vx:tools-latest` (with common tools pre-installed).

## Performance

### What's the startup overhead?

vx is written in Rust. Startup time is typically a few milliseconds.

### Downloads are slow. How can I speed them up?

Enable CDN acceleration:

```bash
export VX_CDN_ENABLED=true
vx install node@22
```

## Extensibility

### How do I add support for a tool vx doesn't have?

Use [Manifest-Driven Providers](/guide/manifest-driven-providers) to create a custom provider:

```toml
# ~/.vx/providers/mytool/provider.toml
[provider]
name = "mytool"

[[runtimes]]
name = "mytool"
executable = "mytool"

[runtimes.version_source]
type = "github_releases"
owner = "org"
repo = "mytool"
```

### How do I create an extension?

See the [Extension Development Guide](/advanced/extension-development).

## More Help

- [Troubleshooting](/appendix/troubleshooting)
- [GitHub Issues](https://github.com/loonghao/vx/issues)
- [Contributing](/advanced/contributing)
