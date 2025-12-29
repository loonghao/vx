# FAQ

Frequently asked questions about vx.

## General

### What is vx?

vx is a universal development tool manager that lets you run any development tool with automatic installation and version management. Just prefix your commands with `vx`.

### How is vx different from nvm, pyenv, etc.?

vx manages **all** development tools in one place, not just one language. Instead of learning separate tools for Node.js (nvm), Python (pyenv), Go (gvm), etc., you use one tool for everything.

### Is vx free?

Yes, vx is open source under the MIT license.

## Installation

### How do I install vx?

**Linux/macOS:**

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```

**Windows:**

```powershell
powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"
```

### How do I update vx?

```bash
vx self-update
```

### How do I uninstall vx?

Remove the vx binary and data directory:

- Linux/macOS: `~/.local/share/vx`
- Windows: `%LOCALAPPDATA%\vx`

## Usage

### Do I need to install tools before using them?

No! Tools are automatically installed on first use:

```bash
vx node --version  # Installs Node.js if needed
```

### How do I specify a tool version?

Use `@` syntax:

```bash
vx node@18 --version
vx go@1.21 build
```

Or configure in `vx.toml`:

```toml
[tools]
node = "20"
go = "1.21"
```

### Where are tools installed?

In the vx store directory:

- Linux/macOS: `~/.local/share/vx/store/`
- Windows: `%LOCALAPPDATA%\vx\store\`

### Can I use system-installed tools?

Yes, use `--use-system-path`:

```bash
vx --use-system-path node --version
```

## Configuration

### What is vx.toml?

A project configuration file that defines tool versions and scripts for your project. See [Configuration](/guide/configuration).

### Where is the global config?

- Linux/macOS: `~/.config/vx/config.toml`
- Windows: `%APPDATA%\vx\config.toml`

### How do I disable auto-install?

```bash
vx config set defaults.auto_install false
```

Or set environment variable:

```bash
export VX_AUTO_INSTALL=false
```

## Troubleshooting

### Tool not found

1. Check if the tool is supported: `vx list`
2. Try installing explicitly: `vx install <tool>`
3. Check verbose output: `vx --verbose <tool> --version`

### Version not available

1. Check available versions: `vx versions <tool>`
2. Try a different version specifier
3. Clear cache: `vx clean --cache`

### Slow first run

The first run downloads and installs the tool. Subsequent runs use the cached version and are fast.

### Permission denied

Ensure you have write access to the vx directories. On Unix, check:

```bash
ls -la ~/.local/share/vx
```

## Performance

### Is vx slow?

vx is written in Rust and is very fast. The first run of a tool may be slower due to download/installation, but subsequent runs add minimal overhead.

### How much disk space does vx use?

Depends on installed tools. Check with:

```bash
vx stats
```

Clean up with:

```bash
vx clean --all
```

## Compatibility

### What platforms are supported?

- Linux (x64, ARM64)
- macOS (x64, ARM64/Apple Silicon)
- Windows (x64)

### What shells are supported?

- Bash
- Zsh
- Fish
- PowerShell

### Can I use vx in CI/CD?

Yes! vx works great in CI/CD:

```yaml
steps:
  - run: curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
  - run: vx setup
  - run: vx run test
```
