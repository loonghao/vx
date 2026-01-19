# Virtual Environment Isolation

vx provides automatic virtual environment isolation when a `vx.toml` file is present. This ensures that subprocesses use the exact tool versions specified in your project configuration, preventing global tool versions from affecting your project.

## The Problem

When running commands like `vx npm run build` or `vx just gallery-pack`, vx needs to set up the PATH environment variable for subprocesses. Without project-aware isolation, vx would use the **latest installed version** of each tool globally, which can cause:

1. **Version Mismatch**: Your project specifies `node = "20"` but `node 24` is used because it's installed globally
2. **Broken Dependencies**: npm packages compiled with newer Node.js may fail with your project's specified version
3. **Inconsistent Environments**: Different team members get different behavior based on their globally installed tools
4. **Hard-to-Debug Issues**: Errors like `Cannot find module` due to incompatible Node.js versions

## The Solution: Project Configuration Priority

When vx detects a `vx.toml` file in your project (or parent directories), it automatically prioritizes the tool versions specified in that configuration.

### How It Works

1. **Detection**: When executing any command, vx searches for `vx.toml` upward from the current directory
2. **Version Selection**: For each tool in PATH, vx uses this priority:
   - **First**: Version specified in `vx.toml` (if installed)
   - **Fallback**: Latest installed version (if specified version not installed)
   - **Warning**: If specified version isn't installed, vx warns and suggests installation

### Example

Given this `vx.toml`:

```toml
[tools]
node = "20"
go = "1.21"
uv = "latest"
```

And these installed versions:
- Node.js: 18.0.0, 20.0.0, 22.0.0, 24.0.0
- Go: 1.20.0, 1.21.0, 1.22.0
- uv: 0.4.0, 0.5.0

When you run `vx npm run build`:
- Node.js **20.0.0** is used (from `vx.toml`)
- Go **1.21.0** is used (from `vx.toml`)
- uv **0.5.0** is used (latest, as specified)

Without `vx.toml`:
- Node.js **24.0.0** would be used (latest)
- Go **1.22.0** would be used (latest)
- uv **0.5.0** would be used (latest)

## Version Matching

vx supports flexible version matching:

### Exact Version
```toml
[tools]
node = "20.10.0"  # Matches exactly 20.10.0
```

### Major Version
```toml
[tools]
node = "20"  # Matches latest 20.x.x (e.g., 20.10.0)
```

### Major.Minor Version
```toml
[tools]
node = "20.10"  # Matches latest 20.10.x
```

## Configuration

### Enable/Disable PATH Inheritance

By default, vx passes all managed tools to subprocesses. You can control this:

```toml
[settings]
inherit_vx_path = true  # Default: enabled
```

Or via CLI:

```bash
vx --no-inherit-vx-path npm run build
```

### Strict Mode

For maximum reproducibility, use exact versions:

```toml
[tools]
node = "20.10.0"
go = "1.21.5"
uv = "0.5.0"
```

## Common Use Cases

### Task Runners (Just, Make)

When using task runners like `just`:

```just
# justfile
build:
    npm run build  # Uses node version from vx.toml
    
test:
    uvx pytest     # Uses uv version from vx.toml
```

Run with `vx just build` - all tools use project-specified versions.

### CI/CD Pipelines

```yaml
# .github/workflows/ci.yml
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
      - uses: loonghao/vx-action@v1
      - run: vx setup
      - run: vx npm run build  # Uses versions from vx.toml
```

### Monorepos

Each subdirectory can have its own `vx.toml`:

```
monorepo/
├── vx.toml           # node = "20"
├── frontend/
│   └── vx.toml       # node = "22" (different version)
└── backend/
    └── vx.toml       # node = "18" (another version)
```

When you `cd frontend && vx npm run dev`, node 22 is used.

## Troubleshooting

### Version Not Found Warning

If you see:
```
Warning: Version 20 specified in vx.toml for node is not installed.
Using latest installed version instead.
Run 'vx install node@20' to install the specified version.
```

Run the suggested command to install the missing version:
```bash
vx install node@20
```

### Verifying Active Versions

Check which versions are being used:

```bash
vx node --version
vx npm --version
vx go version
```

### Debug Mode

For detailed information about version selection:

```bash
VX_LOG=debug vx npm run build
```

## Best Practices

1. **Always commit `vx.toml`** to version control
2. **Use specific versions** for production projects
3. **Run `vx setup`** after cloning to install all specified versions
4. **Use `vx lock`** to lock exact versions for reproducibility
5. **Document version requirements** in your README

## Related

- [Project Environments](/guide/project-environments) - Complete project setup guide
- [Version Management](/guide/version-management) - Managing tool versions
- [vx.toml Reference](/config/vx-toml) - Configuration file reference
