# Plugin Commands

Manage vx plugins (internally called "providers").

> **Note**: In the CLI we use "plugin" for user-friendliness. In the codebase, these are called "Providers" - modules that provide one or more runtimes. See [Core Concepts](/guide/concepts) for more details.

## Overview

```bash
vx plugin <command>
```

## Commands

### List Plugins

List all available plugins:

```bash
vx plugin list

# Show only enabled plugins
vx plugin list --enabled

# Filter by category
vx plugin list --category devops
```

### Plugin Info

Show detailed information about a plugin:

```bash
vx plugin info node
```

Output:
```
Plugin: node
  Provider: NodeProvider
  Runtimes: node, npm, npx
  Ecosystem: NodeJs
  Description: Node.js JavaScript runtime
```

### Plugin Statistics

Show plugin statistics:

```bash
vx plugin stats
```

Output:
```
Plugin Statistics:
  Total providers: 32
  Total runtimes: 38

  Providers:
    node (3 runtimes)
    go (1 runtimes)
    rust (3 runtimes)
    ...
```

### Enable/Disable Plugins

```bash
# Enable a plugin
vx plugin enable node

# Disable a plugin
vx plugin disable node
```

### Search Plugins

Search for plugins by name or description:

```bash
vx plugin search python
```

## Plugin Categories

| Category | Examples |
|----------|----------|
| **Language Runtimes** | node, go, rust, java, zig |
| **Package Managers** | uv, pnpm, yarn, bun |
| **Build Tools** | vite, just, task, cmake, ninja |
| **DevOps** | docker, terraform, kubectl, helm |
| **Cloud CLI** | awscli, azcli, gcloud |
| **Code Quality** | pre-commit |

## See Also

- [Core Concepts](/guide/concepts) - Understanding plugins and providers
- [Supported Tools](/tools/overview) - Complete list of supported tools
