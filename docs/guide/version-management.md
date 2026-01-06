# Version Management

vx provides powerful version management capabilities, allowing you to specify exact versions of tools and automatically handle dependency constraints between tools.

## Version Syntax

Use the `@` symbol to specify a version when running any tool:

```bash
# Specific major version
vx node@20 --version

# Exact version
vx node@20.10.0 --version

# Latest version
vx node@latest --version

# Python with version
vx python@3.10 --version
vx python@3.12.8 script.py
```

### Supported Version Formats

| Format | Example | Description |
|--------|---------|-------------|
| `major` | `node@20` | Latest version of major release |
| `major.minor` | `python@3.10` | Latest patch of specific minor |
| `major.minor.patch` | `node@20.10.0` | Exact version |
| `latest` | `uv@latest` | Most recent stable version |

### Examples by Tool

```bash
# Node.js ecosystem
vx node@20 --version
vx npm@10 install
vx npx@20 create-react-app my-app

# Python ecosystem
vx python@3.10 --version
vx python@3.12.8 script.py
vx uv@0.4 pip install requests

# Go
vx go@1.21 version
vx go@1.22.5 build

# Yarn with specific version
vx yarn@1.22.22 install
```

## Dependency Version Constraints

Some tools depend on other runtimes. vx automatically manages these dependencies and ensures version compatibility.

### How It Works

When you run a tool that has dependencies, vx will:

1. **Check dependency versions** - Verify installed dependencies meet version requirements
2. **Detect incompatibilities** - Identify if current versions are outside the allowed range
3. **Auto-install compatible versions** - Install a compatible dependency version if needed
4. **Configure environment** - Set up PATH to use the compatible version

### Example: Yarn and Node.js

Yarn 1.x requires Node.js 12-22. If you have Node.js 23+ installed, vx will automatically use a compatible version:

```bash
# You have Node.js 23 installed, but yarn needs Node.js ≤22
vx yarn@1.22.22 install

# vx detects the incompatibility and:
# 1. Finds or installs Node.js 20 (recommended version)
# 2. Runs yarn with the compatible Node.js
# 3. Your command succeeds!
```

### Dependency Constraints by Tool

| Tool | Dependency | Min Version | Max Version | Recommended |
|------|------------|-------------|-------------|-------------|
| yarn | node | 12.0.0 | 22.99.99 | 20 |
| npm | node | 14.0.0 | - | - |
| npx | node | 14.0.0 | - | - |
| pnpm | node | 16.0.0 | - | - |

::: tip Why Version Constraints?
Some tools have compatibility issues with newer runtime versions. For example:
- Yarn 1.x has native module compilation issues with Node.js 23+
- Some npm packages require specific Node.js versions
- Python packages may need specific Python versions

vx handles these constraints automatically so you don't have to worry about compatibility.
:::

## Practical Examples

### Web Development with Specific Versions

```bash
# Create a React app with Node.js 20
vx node@20 npx create-react-app my-app
cd my-app

# Use yarn with automatic Node.js version management
vx yarn@1.22.22 install
vx yarn@1.22.22 start
```

### Python Development

```bash
# Run Python 3.10 specifically
vx python@3.10 --version

# Run a script with Python 3.12
vx python@3.12 script.py

# Use Python 3.11 for compatibility
vx python@3.11 -m pytest
```

### Multi-Version Testing

```bash
# Test your code with different Node.js versions
vx node@18 npm test
vx node@20 npm test
vx node@22 npm test

# Test with different Python versions
vx python@3.10 -m pytest
vx python@3.11 -m pytest
vx python@3.12 -m pytest
```

### CI/CD Pipeline

```yaml
# GitHub Actions example
jobs:
  test:
    strategy:
      matrix:
        node: [18, 20, 22]
        python: ['3.10', '3.11', '3.12']
    steps:
      - uses: actions/checkout@v4
      - name: Setup vx
        uses: loonghao/vx@v1
      - name: Test Node.js
        run: vx node@${{ matrix.node }} npm test
      - name: Test Python
        run: vx python@${{ matrix.python }} -m pytest
```

## Troubleshooting

### Version Not Found

If a specific version isn't available:

```bash
# List available versions
vx versions node
vx versions python

# Install a specific version first
vx install node@20.10.0
```

### Dependency Conflicts

If you see dependency warnings:

```bash
# Check what versions are installed
vx list --status

# The warning shows what vx is doing:
# "Dependency node version 23.0.0 is incompatible with yarn (requires: max=22.99.99)"
# "Installing compatible version: node@20"
```

### Force System Version

To bypass vx's version management:

```bash
# Use system-installed tool
vx --use-system-path node --version
```

## Best Practices

1. **Pin versions in projects** - Use `vx.toml` to ensure team consistency:

   ```toml
   [tools]
   node = "20"
   python = "3.11"
   yarn = "1.22.22"
   ```

2. **Use recommended versions** - When tools have dependencies, use the recommended versions for best compatibility

3. **Test across versions** - Use version syntax to test compatibility before upgrading

4. **Let vx manage dependencies** - Don't manually override dependency versions unless necessary

## Next Steps

- [Project Environments](/guide/project-environments) - Configure project-specific tool versions
- [Configuration](/guide/configuration) - Learn about `vx.toml` configuration
- [CLI Reference](/cli/overview) - Complete command reference
