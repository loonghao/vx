# Global Configuration

vx stores global configuration in `~/.config/vx/config.toml`.

## Location

| Platform | Path |
|----------|------|
| Linux | `~/.config/vx/config.toml` |
| macOS | `~/.config/vx/config.toml` |
| Windows | `%APPDATA%\vx\config.toml` |

## Configuration File

```toml
[defaults]
auto_install = true
parallel_install = true
cache_duration = "7d"

[tools]
node = "lts"
python = "3.11"
go = "latest"
rust = "stable"
```

## Sections

### [defaults]

Default behavior settings.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `auto_install` | bool | `true` | Auto-install missing tools |
| `parallel_install` | bool | `true` | Install tools in parallel |
| `cache_duration` | string | `"7d"` | How long to cache version lists |

```toml
[defaults]
auto_install = true
parallel_install = true
cache_duration = "7d"
```

### [tools]

Default tool versions when no project config exists.

```toml
[tools]
node = "lts"
python = "3.11"
go = "latest"
rust = "stable"
uv = "latest"
```

## Managing Configuration

### View Configuration

```bash
vx config show
```

### Set Values

```bash
vx config set defaults.auto_install false
vx config set defaults.cache_duration "14d"
vx config set tools.node "20"
```

### Get Values

```bash
vx config get defaults.auto_install
vx config get tools.node
```

### Reset to Defaults

```bash
# Reset everything
vx config reset

# Reset specific key
vx config reset defaults.auto_install
```

### Edit Directly

```bash
vx config edit
```

Opens the config file in your default editor.

## Configuration Precedence

Settings are resolved in this order (later overrides earlier):

1. Built-in defaults
2. Global config (`~/.config/vx/config.toml`)
3. Project config (`vx.toml`)
4. Environment variables
5. Command-line flags

## Example Configurations

### Minimal

```toml
[defaults]
auto_install = true
```

### Conservative

```toml
[defaults]
auto_install = false
parallel_install = false
cache_duration = "1d"
```

### Full

```toml
[defaults]
auto_install = true
parallel_install = true
cache_duration = "7d"

[tools]
node = "20"
python = "3.11"
go = "1.21"
rust = "stable"
uv = "latest"
deno = "latest"
```
