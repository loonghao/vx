# Provider Overrides

vx allows you to customize runtime dependency constraints without modifying the built-in provider configurations. This is useful when:

- Your company has specific Node.js version requirements
- You need to use a different version range than the default
- You want to add optional dependencies for certain tools

## How It Works

vx loads provider configurations in this order (later overrides earlier):

1. **Built-in manifests** - Default configurations bundled with vx
2. **User-level overrides** - `~/.vx/providers/*.override.toml`
3. **Project-level overrides** - `<project>/.vx/providers/*.override.toml`

## Creating Override Files

Override files use a simple TOML format. The filename determines which provider to override:

```
~/.vx/providers/
├── yarn.override.toml      # Overrides for yarn
├── pnpm.override.toml      # Overrides for pnpm
└── node.override.toml      # Overrides for node
```

## Override File Format

### Basic Structure

```toml
# ~/.vx/providers/yarn.override.toml

# Override constraints for the main runtime (yarn)
[[constraints]]
when = "^1"  # When yarn version matches ^1 (1.x)
requires = [
    { runtime = "node", version = ">=14, <21" }
]
```

### Version Constraint Syntax

The `when` field uses standard semver syntax to match the runtime version:

| Syntax | Meaning | Example |
|--------|---------|---------|
| `^1.2.3` | Compatible version | `>=1.2.3, <2.0.0` |
| `~1.2.3` | Patch version | `>=1.2.3, <1.3.0` |
| `>=1.2.3` | Greater than or equal | |
| `<2.0.0` | Less than | |
| `>=1, <3` | Range | |
| `1.2.*` | Wildcard | Matches 1.2.x |
| `*` | Any version | |

### Dependency Definition

Each dependency in `requires` can have these fields:

```toml
[[constraints]]
when = "^1"
requires = [
    {
        runtime = "node",           # Required: runtime name
        version = ">=14, <21",      # Required: version constraint
        recommended = "20",         # Optional: recommended version
        reason = "Company policy",  # Optional: explanation
        optional = false            # Optional: if true, not required
    }
]
```

## Examples

### Example 1: Company Node.js Policy

Your company requires Node.js 18+ for all projects:

```toml
# ~/.vx/providers/yarn.override.toml

[[constraints]]
when = "*"  # All yarn versions
requires = [
    { runtime = "node", version = ">=18", reason = "Company security policy" }
]
```

### Example 2: Project-Specific Requirements

A legacy project needs older Node.js:

```toml
# <project>/.vx/providers/yarn.override.toml

[[constraints]]
when = "^1"
requires = [
    { runtime = "node", version = ">=12, <17", reason = "Legacy compatibility" }
]
```

### Example 3: Adding Optional Dependencies

Add git as an optional dependency:

```toml
# ~/.vx/providers/yarn.override.toml

[[constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=16" },
    { runtime = "git", version = ">=2.0", optional = true }
]
```

### Example 4: Multiple Version Ranges

Different constraints for different versions:

```toml
# ~/.vx/providers/pnpm.override.toml

[[constraints]]
when = "^7"
requires = [
    { runtime = "node", version = ">=14, <19" }
]

[[constraints]]
when = "^8"
requires = [
    { runtime = "node", version = ">=16, <21" }
]

[[constraints]]
when = ">=9"
requires = [
    { runtime = "node", version = ">=18" }
]
```

### Example 5: Runtime-Specific Overrides

Override constraints for a specific runtime within a provider:

```toml
# ~/.vx/providers/node.override.toml

# Override for npm (bundled with node)
[[runtimes]]
name = "npm"

[[runtimes.constraints]]
when = "*"
requires = [
    { runtime = "node", version = ">=18" }
]
```

## Override Behavior

### Replacement Rules

- Override constraints **replace** existing constraints with the same `when` pattern
- New `when` patterns are **appended** to existing constraints
- Empty override files are ignored

### Priority

When multiple overrides exist:

1. Project-level overrides take precedence over user-level
2. Later-loaded overrides replace earlier ones with the same `when` pattern

## Verifying Overrides

Check which constraints are active:

```bash
# Show effective constraints for a runtime
vx info yarn --constraints

# Debug mode shows override sources
VX_DEBUG=1 vx yarn --version
```

## Best Practices

1. **Document your overrides** - Add comments explaining why constraints are changed
2. **Use project-level for specific needs** - Keep user-level for general policies
3. **Test after changes** - Run `vx yarn --version` to verify the override works
4. **Version control project overrides** - Include `.vx/providers/` in your repository

## Troubleshooting

### Override Not Applied

1. Check the filename matches the provider name (e.g., `yarn.override.toml` for yarn)
2. Verify the file is in the correct directory (`~/.vx/providers/` or `<project>/.vx/providers/`)
3. Check for TOML syntax errors

### Constraint Conflicts

If you see unexpected behavior:

```bash
# Enable debug logging
VX_DEBUG=1 vx yarn install

# Check loaded manifests
vx debug providers
```

## See Also

- [Configuration Guide](/guide/configuration) - General configuration
- [Version Management](/guide/version-management) - How vx manages versions
- [Provider Development](/advanced/plugin-development) - Creating custom providers
