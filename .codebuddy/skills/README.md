# VX Project Skills

This directory contains specialized skills for working with the VX universal tool manager project.

## Available Skills

### 1. vx-provider-creator

**Purpose**: Create new runtime providers for VX

**Use when**:
- Adding support for a new tool/runtime (e.g., "add ripgrep support")
- Implementing a new provider from scratch
- Adding project analyzer integration for language-specific tools

**Key features**:
- Complete provider directory structure generation
- provider.toml manifest with RFC 0018 + RFC 0019 support
- Runtime trait implementation templates
- Project analyzer integration (optional)
- Test file generation
- Documentation templates

**Quick start**:
```
Use the vx-provider-creator skill to add support for {tool-name}
```

### 2. vx-provider-updater

**Purpose**: Update existing providers to RFC 0018 + RFC 0019 standards

**Use when**:
- Updating provider.toml to add layout configuration
- Migrating from custom post_extract hooks to declarative layout
- Fixing download/installation issues
- Batch updating multiple providers

**Key features**:
- 8 different update templates for common patterns
- Binary and archive layout configurations
- Quick migration guide (5-minute checklist)
- Troubleshooting guide
- Batch update support

**Quick start**:
```
Use the vx-provider-updater skill to update {provider-name} with RFC 0019 layout
```

### 3. project-analyze

**Purpose**: Analyze project structure and dependencies

**Use when**:
- Understanding project organization
- Finding specific implementations
- Exploring codebase structure

### 4. rfc-creator

**Purpose**: Create RFC (Request for Comments) documents for VX

**Use when**:
- Proposing new features or changes
- Documenting design decisions
- Standardizing approaches

## Skill Organization

Each skill directory contains:

```
skill-name/
├── SKILL.md              # Main skill documentation (frontmatter + guide)
├── references/           # Supporting reference materials
│   ├── templates.md      # Code/config templates
│   ├── examples.md       # Usage examples
│   └── ...              # Other references
└── README.md            # Skill-specific readme (optional)
```

## Common Workflows

### Creating a New Provider

1. Use `vx-provider-creator` skill
2. Follow the step-by-step workflow
3. Run tests and verification
4. Submit PR

### Updating Existing Providers

1. Use `vx-provider-updater` skill
2. Choose appropriate template based on tool type
3. Add layout configuration
4. Test and verify
5. Update migration status

### RFC 0019 Migration

The project is migrating all providers to RFC 0019 layout configuration:

**Current status**: ~33/41 providers updated (80%)

**Key documents**:
- `vx-provider-updater/references/rfc-0019-layout.md` - Complete RFC spec
- `vx-provider-updater/references/update-templates.md` - 8 update templates
- `vx-provider-updater/references/quick-migration-guide.md` - Fast-track guide
- `docs/provider-migration-status.md` - Migration progress tracker

## RFC 0019 Quick Reference

### Layout Types

| Type | Description | Examples |
|------|-------------|----------|
| `binary` | Single executable file | kubectl, ninja, yasm |
| `archive` | Compressed archive | node, go, terraform |

### Binary Layout

```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "tool.exe"
target_name = "tool.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"
```

### Archive Layout

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "tool-{version}"
executable_paths = [
    "bin/tool.exe",  # Windows
    "bin/tool"       # Unix
]
```

## Variables

Available in layout configuration:
- `{version}` - Runtime version (e.g., "20.0.0")
- `{os}` - Operating system (windows, linux, darwin)
- `{arch}` - Architecture (x86_64, aarch64, arm64, amd64)
- `{name}` - Runtime name

## Best Practices

1. **Always use skills for standard tasks** - They contain accumulated knowledge and best practices
2. **Check references** - Each skill has supporting documentation with examples
3. **Follow templates** - Use provided templates to maintain consistency
4. **Test thoroughly** - Run all verification steps before committing
5. **Update status** - Keep migration status documents current

## Getting Help

Each skill's SKILL.md contains:
- When to use the skill
- Step-by-step workflows
- Code templates
- Common patterns
- Troubleshooting guides
- Quick reference sections

## Contributing

When adding new skills:

1. Create skill directory: `.opencode/skills/skill-name/`
2. Add SKILL.md with frontmatter:
   ```markdown
   ---
   name: skill-name
   description: |
     Brief description of what the skill does
   ---
   ```
3. Add references directory with supporting materials
4. Update this README.md
5. Test the skill thoroughly

## Related Documentation

- `/docs/provider-migration-plan.md` - Overall migration plan
- `/docs/provider-migration-status.md` - Current progress
- `/docs/provider-update-summary.md` - Batch update summary
- `/docs/tools/` - Tool-specific documentation
