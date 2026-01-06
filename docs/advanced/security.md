# Security

vx includes several security features to help protect your development environment from supply chain attacks and untrusted code execution.

## Overview

Modern development workflows often involve:
- Remote configuration presets from URLs
- Project-level extensions with custom code
- Third-party tool installations

vx provides security warnings and verification mechanisms to help you stay aware of potential risks.

## Remote Preset Verification

When using remote presets in your `vx.toml`, vx can verify the integrity of downloaded content using SHA256 hashes.

### Basic Usage

```toml
# vx.toml
[preset]
url = "https://example.com/presets/nodejs-dev.toml"
sha256 = "a1b2c3d4e5f6..."  # Optional but recommended
```

### How It Works

1. **Without SHA256**: vx will display a warning when loading unverified remote presets
2. **With SHA256**: vx verifies the downloaded content matches the expected hash

### Security Warnings

When loading a remote preset without hash verification, you'll see:

```
⚠️  Warning: Remote preset 'https://example.com/preset.toml' has no SHA256 verification.
    Consider adding a sha256 hash for security.
```

### Generating SHA256 Hash

To generate a SHA256 hash for your preset:

::: code-group

```bash [Linux/macOS]
curl -fsSL https://example.com/preset.toml | sha256sum
```

```powershell [Windows]
(Invoke-WebRequest -Uri "https://example.com/preset.toml").Content | 
    Get-FileHash -Algorithm SHA256 | 
    Select-Object -ExpandProperty Hash
```

:::

### Best Practices

1. **Always use SHA256** for production environments
2. **Pin preset URLs** to specific versions or commits
3. **Review preset content** before adding to your project
4. **Use trusted sources** (official repositories, verified organizations)

## Extension Security

vx's extension system allows custom functionality, but project-level extensions can pose security risks.

### Extension Sources

Extensions can come from three sources:

| Source | Location | Trust Level |
|--------|----------|-------------|
| **Builtin** | Shipped with vx | ✅ Trusted |
| **User** | `~/.vx/extensions/` | ⚠️ User-installed |
| **Project** | `.vx/extensions/` | ⚠️ Potentially untrusted |

### Security Warnings

When vx discovers project-level extensions, it displays warnings:

```
⚠️  Warning: Extension 'custom-tool' is loaded from project directory.
    Source: .vx/extensions/custom-tool
    Project-level extensions can execute arbitrary code.
    Only use extensions from trusted sources.
```

### Extension Trust Model

```
┌─────────────────────────────────────────────────────────┐
│                    Extension Loading                     │
├─────────────────────────────────────────────────────────┤
│  1. Builtin Extensions (always loaded)                  │
│     └── Shipped with vx, fully trusted                  │
│                                                         │
│  2. User Extensions (~/.vx/extensions/)                 │
│     └── Installed by user, moderate trust               │
│                                                         │
│  3. Project Extensions (.vx/extensions/)                │
│     └── From repository, requires review                │
│     └── ⚠️ Warning displayed on load                    │
└─────────────────────────────────────────────────────────┘
```

### Reviewing Project Extensions

Before using a project with custom extensions:

1. **Check the `.vx/extensions/` directory** for unexpected files
2. **Review extension code** for suspicious behavior
3. **Verify extension source** matches expected origin
4. **Consider sandboxing** when running untrusted projects

### Disabling Project Extensions

To run vx without loading project-level extensions:

```bash
# Set environment variable
VX_DISABLE_PROJECT_EXTENSIONS=1 vx node --version
```

## Observability

vx includes structured logging for security-relevant operations.

### Tracing Spans

Key operations are instrumented with tracing spans:

```rust
// Example span structure
vx_execute {
    runtime: "node",
    version: "20.10.0",
    args_count: 3
}
```

### Enabling Debug Logging

```bash
# Enable debug output
VX_LOG=debug vx node --version

# Enable trace output (most verbose)
VX_LOG=trace vx node --version
```

### Cache Logging

Resolution cache operations are logged with structured fields:

```
DEBUG runtime="node" cache_hit=true "Resolution cache hit"
DEBUG runtime="go" cache_hit=false "Resolution cache miss"
```

## CI/CD Security

### GitHub Actions

When using vx in CI/CD pipelines:

```yaml
- name: Setup vx
  uses: loonghao/vx@v1

- name: Install tools with verification
  run: |
    # vx will warn about unverified presets
    vx setup
```

### Inline Test Detection

vx enforces a test convention where tests should be in separate `tests/` directories. The CI pipeline checks for inline tests:

```yaml
- name: Check for inline tests
  run: |
    # Warns about inline tests that should be migrated
    ./scripts/check-inline-tests.sh
```

## Security Checklist

### For Project Maintainers

- [ ] Use SHA256 verification for remote presets
- [ ] Review all project-level extensions
- [ ] Pin tool versions in `vx.toml`
- [ ] Document extension requirements

### For Contributors

- [ ] Don't add untrusted presets
- [ ] Review extension code before committing
- [ ] Follow test conventions (no inline tests)
- [ ] Use structured logging for security events

### For Users

- [ ] Review `vx.toml` before running `vx setup`
- [ ] Check `.vx/extensions/` in new projects
- [ ] Enable debug logging when investigating issues
- [ ] Report security concerns to maintainers

## Reporting Security Issues

If you discover a security vulnerability in vx:

1. **Do not** open a public GitHub issue
2. Email security concerns to the maintainers
3. Include detailed reproduction steps
4. Allow time for a fix before public disclosure

## Future Improvements

Planned security enhancements:

- **Extension signing**: Cryptographic verification of extension authors
- **Preset caching**: Local cache with integrity verification
- **Audit logging**: Comprehensive audit trail for security events
- **Sandbox mode**: Isolated execution environment for untrusted code
