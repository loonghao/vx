# self-update

Update vx itself to the latest version.

## Synopsis

```bash
vx self-update [OPTIONS] [VERSION]
```

## Description

`vx self-update` updates vx to the latest version or a specific version. By default, it uses the `stable` update channel.

## Arguments

| Argument | Description |
|----------|-------------|
| `[VERSION]` | Specific version to install (e.g., `v0.8.36`) |

## Options

| Option | Description |
|--------|-------------|
| `--check` | Only check for updates, don't install |
| `--channel <CHANNEL>` | Update channel: `stable`, `beta`, `dev` |
| `-f`, `--force` | Force update even if already up to date |
| `--token <TOKEN>` | GitHub token for authenticated API requests |
| `--use-system-path` | Use system PATH to find tools |
| `--inherit-env` | Inherit environment variables |

## Update Channels

| Channel | Description |
|---------|-------------|
| `stable` | Stable releases only (default) |
| `beta` | Include beta/pre-release versions |
| `dev` | Nightly/dev builds (future feature) |

## Usage Examples

```bash
# Update to latest stable version
vx self-update

# Check for updates without installing
vx self-update --check

# Update to a specific version
vx self-update v0.8.36

# Use beta channel
vx self-update --channel beta

# Force reinstall current version
vx self-update --force
```

## Non-Blocking Notification

vx periodically checks for updates in the background and notifies you when a new version is available. This check is non-blocking and won't interfere with your workflow.

To disable update checks, set the `VX_NO_UPDATE_CHECK` environment variable:

```bash
export VX_NO_UPDATE_CHECK=1
```

## Related

- [`overview`](./overview) - CLI overview
- [`install`](./install) - Install tools
- [`version`](./commands#version) - Show vx version
