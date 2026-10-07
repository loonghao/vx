---
name: vx-troubleshooting
description: "Troubleshooting guide for vx issues. Use when encountering installation failures, version conflicts, PATH issues, or other vx problems."
---

# VX Troubleshooting Guide

> **Quick triage**: Start with `vx doctor` for diagnostics. Use `vx --debug <command>` for detailed logs. Use `vx cache clean` to clear corrupted state. Check exit codes (2=tool not found, 3=install failed, 4=version not found, 5=network error).

## Common Issues

### Installation Failures

#### Tool Download Failed

**Symptoms**: `Failed to download <tool>: network error` or `Connection refused`

**Solutions**:

```bash
# Enable CDN acceleration (China users)
vx config set cdn_acceleration true

# Use mirror
vx install node --mirror https://npmmirror.com/mirrors/node

# Retry with verbose output
vx install node --verbose --debug

# Check cache and retry
vx cache clean
vx install node
```

#### Checksum Mismatch

**Symptoms**: `Checksum mismatch: expected X, got Y`

**Solutions**:

```bash
# Clear corrupted download
vx cache clean

# Reinstall with fresh download
vx install node@22 --force
```

#### Permission Denied

**Symptoms**: `Permission denied` or `Access is denied`

**Solutions**:

```bash
# Check VX_HOME permissions
ls -la ~/.vx

# Fix permissions (Unix)
chmod -R u+rw ~/.vx

# Run with elevated permissions if needed
sudo vx install node  # Not recommended, use user installation
```

### Version Issues

#### Version Not Found

**Symptoms**: `Version X not found for <tool>`

**Solutions**:

```bash
# List available versions
vx versions node

# Use latest stable
vx install node@latest

# Use LTS version
vx install node@lts

# Check for typos
vx versions node | grep "20"
```

#### Version Conflict

**Symptoms**: Multiple versions installed, wrong version active

**Solutions**:

```bash
# List installed versions
vx list node --installed

# Switch to specific version
vx switch node@20

# Check which version is active
vx which node

# Remove conflicting versions
vx uninstall node@18
```

### PATH Issues

#### Plain Command Missing or Resolving to the Wrong Executable

First distinguish `vx node --version` from plain `node --version`: installing
a runtime does not by itself prove the shell can resolve a managed plain command.
Check `vx --version` and `vx --help`. Only when `shim` is listed, inspect
`vx shim --help` for `add/list/remove/sync/path`, then use `vx shim path` and
`vx shim list --json`. Release v0.9.34 does not include this interface.

Check resolution in the failing shell: PowerShell `Get-Command node -All` and
`where.exe node`; cmd.exe `where node`; Git Bash/POSIX `type -a node` and
`command -v node`. Substitute the actual exposed name. PowerShell aliases and
functions can precede PATH; Windows shims need both `.cmd` and extensionless
variants. `vx shim path` reports default targets, so also inspect each entry's
recorded directories when `--dir` was used.

Do not prepend global PATH or add `--force` as a generic repair. Prefer a unique
`--as` name, or explicit `vx <target>` execution. Before `add` or upgrade `sync`,
inspect every destination file and its ownership; an `ok`/`complete` listing
only confirms files exist. Before `remove`, preserve any locally replaced files:
unmarked replacements survive, but the registry entry is removed.

Follow the [managed command shim guide](https://github.com/loonghao/vx/blob/main/docs/guide/managed-command-shims.md)
for Codex, PATH precedence, shell-specific commands, `sync`, and safe removal.

### Runtime Issues

#### Tool Crashes on Startup

**Symptoms**: Tool exits immediately or crashes

**Solutions**:

```bash
# Check tool version
vx which node
vx node --version

# Reinstall the tool
vx install node --force

# Check for missing dependencies
vx doctor

# Try with debug output
vx node --verbose script.js
```

#### Dependency Missing

**Symptoms**: `error while loading shared libraries` or `DLL not found`

**Solutions**:

```bash
# Check dependencies (Linux)
ldd $(vx which node)

# Install system dependencies
# Ubuntu/Debian
sudo apt-get install build-essential libssl-dev

# macOS (via Homebrew)
brew install openssl

# Windows - usually bundled, check PATH
```

### Configuration Issues

#### vx.toml Not Loading

**Symptoms**: Settings in vx.toml ignored

**Solutions**:

```bash
# Verify file location
ls vx.toml

# Check syntax
vx check

# Validate configuration
vx config validate

# Show effective configuration
vx config show
```

#### Lock File Conflicts

**Symptoms**: `vx.lock is out of sync`

**Solutions**:

```bash
# Regenerate lock file
vx lock --update

# Or remove and regenerate
rm vx.lock
vx lock
```

## Diagnostic Commands

### System Information

```bash
# General diagnostics
vx doctor

# System info
vx info

# Environment check
vx check --json

# Show configuration
vx config show
```

### Debug Mode

```bash
# Enable debug logging
vx --debug node --version

# Enable trace logging
vx --trace node --version

# Verbose output
vx --verbose install node
```

### Noisy CI and Log Triage

When debugging GitHub Actions, do not start by reading the full log. Use
structured status first, then capped searches, then compact mode if the failure
still needs broad context:

```bash
# Job status without logs
vx gh run view <run-id> --json status,conclusion,jobs --jq '.jobs[] | {name,conclusion}'

# Focused failure search
vx gh run view <run-id> --log | vx rg -n -m 80 "error|failed|panic|Traceback|FAILED|warning"

# Broad fallback with compact filtering
vx --compact gh run view <run-id> --log
```

Interpret keyword searches carefully. Passing tests may contain names like
`returns_failure_envelope PASSED`, and build commands may include flags such as
`--warnings-as-errors`; confirm the job conclusion and surrounding lines before
treating a match as the root cause.

### Cache Inspection

```bash
# Show cache location
vx cache dir

# Show cache size
vx cache info

# List cached items
vx cache list

# Clean cache
vx cache clean
```

## Error Messages Reference

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Tool not found` | Unknown tool name | Check `vx list` for available tools |
| `Version not found` | Invalid version | Use `vx versions <tool>` to see available |
| `Network error` | Connection issues | Check network, enable CDN, use mirror |
| `Permission denied` | Insufficient permissions | Check directory permissions |
| `Checksum mismatch` | Corrupted download | Run `vx cache clean` and retry |
| `Out of disk space` | Disk full | Clean cache: `vx cache clean` |

### Exit Codes

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

## Recovery Procedures

### Complete Reset

```bash
# Backup configuration
cp -r ~/.vx ~/.vx.backup

# Remove everything
rm -rf ~/.vx

# Reinstall
vx install node go uv

# Restore configuration
cp ~/.vx.backup/vx.toml ~/.vx/
```

### Repair Installation

```bash
# Verify and repair
vx doctor --fix

# Reinstall all tools from vx.toml
vx sync --force
```

For managed command shims, use the capability and ownership checks in
[PATH Issues](#path-issues) before `vx shim sync`. It rewrites every registered
entry; `vx shim rebuild` is not a supported subcommand.

## Getting Help

### Collect Diagnostics

```bash
# Generate diagnostic report
vx doctor --output diagnostics.txt

# Include in bug report
cat diagnostics.txt
```

### Useful Information to Provide

1. vx version: `vx --version`
2. Operating system: `vx info | grep -i os`
3. Command that failed
4. Error message (use `--debug`)
5. Contents of `vx.toml` (if applicable)
6. `vx doctor` output

### Support Channels

- GitHub Issues: https://github.com/loonghao/vx/issues
- Documentation: https://github.com/loonghao/vx#readme

## Quick Triage for AI Agents

When a user reports a vx issue, follow this decision tree:

```
1. "command not found: vx"
   → vx is not installed. Run the install script.
   → Linux/macOS: curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
   → Windows: powershell -c "irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex"

2. "Failed to download" / "network error" (exit code 5)
   → Try: vx cache clean && vx install <tool> --verbose
   → If in China: vx config set cdn_acceleration true
   → Check if GITHUB_TOKEN is set for API rate limits

3. "version not found" (exit code 4)
   → Run: vx versions <tool> to list available versions
   → The user may have a typo in the version string
   → Try: vx install <tool>@latest

4. "permission denied" (exit code 6)
   → Check: ls -la ~/.vx (Unix) or icacls %USERPROFILE%\.vx (Windows)
   → Fix: chmod -R u+rw ~/.vx
   → Never use sudo with vx

5. Tool works but wrong version
   → Run: vx which <tool> to see which version is active
   → Check: vx.toml may specify a different version
   → Run: vx switch <tool>@<version>

6. vx.toml not being picked up (exit code 7)
   → Ensure file is in the project root (same dir as .git)
   → Run: vx check to validate syntax

7. CI failing with vx
   → Ensure the GitHub Action is used: loonghao/vx@main
   → Add github-token for rate limit avoidance
   → Use cache: 'true' for faster CI runs
   → Inspect logs in order: `--json --jq`, capped `vx rg`, then `vx --compact`

8. General error (exit code 1)
   → Run: vx doctor for full diagnostics
   → Run: vx --debug <command> for detailed logs
   → Check: vx cache clean to clear corrupted state
```


---

## Delivery Surface — External Systems Are Evidence

GitHub PRs, CI runs, and dashboards are **supporting evidence**, not the delivery
surface. The conclusion has to land where the work is tracked.

- **Record the outcome once, completely** — one issue comment carrying status,
  branch/commit/PR, what you validated, the blocker, and the next owner.
- **Verify terminal state, not intermediate state.** A green CI run or an open,
  review-ready PR is not "shipped". Confirm merged / released / deployed, then
  record *that*.
- **Keep public surfaces public-safe.** PR titles, bodies, and commit messages
  carry technical content only — no internal issue IDs, routing history,
  reviewer handoffs, local absolute paths, or internal hostnames.
- **Collect results in the foreground.** A queued or pending state is a handoff,
  not a completion; never end a turn "standing by" for background work.

Concretely: the delivery surface is the team's issue or task tracker — the
issue comment plus its metadata. A PR comment alone delivers nothing.
