# Quick Provider Migration Guide

Fast-track guide for updating providers to RFC 0019 layout configuration.

## 5-Minute Checklist

- [ ] Identify download type (binary, archive, or git clone)
- [ ] Choose appropriate template
- [ ] Add layout configuration to provider.toml (use `snake_case` for values!)
- [ ] Test installation
- [ ] Update migration status

## Step-by-Step

### 1. Check Current Download

```bash
# Find the provider
cd crates/vx-providers/{name}

# Check provider.toml
cat provider.toml | grep -A 5 "versions"
```

Questions:
- What's the `source`? (github-releases, npm, pypi, etc.)
- Is it downloading a single file or an archive?

### 2. Inspect Download Structure

**Option A: Check GitHub Releases**
```bash
# Visit: https://github.com/{owner}/{repo}/releases/latest
# Download the asset for your platform
# Extract and check structure
```

**Option B: Use existing installation**
```bash
vx install {name}@latest
ls ~/.vx/store/{name}/latest/
```

### 3. Choose Template

| Download Format | Template |
|-----------------|----------|
| Single `.exe` or binary | Binary |
| Archive with `bin/` directory | Standard Archive |
| Archive, executable in root | Root Directory Archive |
| Archive with `{os}-{arch}/` | Platform Directory Archive |
| Git repository clone | Git Clone (`git_clone`) |

### 4. Add Configuration

**Binary Example:**
```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "{name}.exe"
target_name = "{name}.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "{name}"
target_name = "{name}"
target_dir = "bin"
target_permissions = "755"
```

**Archive Example:**
```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{name}-{version}"
executable_paths = [
    "bin/{name}.exe",
    "bin/{name}"
]
```

### 5. Insert Location

Place layout configuration **after** `[runtimes.versions]` and **before** `[runtimes.platforms]`:

```toml
[runtimes.versions]
source = "github-releases"
# ... version config ...

# ⬇️ INSERT HERE ⬇️
[runtimes.layout]
download_type = "archive"
# ... layout config ...

[runtimes.platforms.windows]
executable_extensions = [".exe"]
```

### 6. Verify

```bash
# Check syntax
cargo check -p vx-provider-{name}

# Test installation
cargo build --release
./target/release/vx install {name}@latest

# Verify
./target/release/vx which {name}
./target/release/vx {name} --version
```

## Common Patterns

### Pattern: GitHub Release with Version in Archive Name

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{name}-{version}"  # Strips "tool-1.0.0/"
executable_paths = ["bin/{name}.exe", "bin/{name}"]
```

### Pattern: Platform-Specific Archives

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = "{os}-{arch}"  # Strips "linux-amd64/" or "darwin-arm64/"
executable_paths = ["{name}.exe", "{name}"]
```

### Pattern: Single Binary Rename

```toml
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "{name}-{version}-win64.exe"
target_name = "{name}.exe"
target_dir = "bin"

[runtimes.layout.binary."linux-x86_64"]
source_name = "{name}-{version}-linux"
target_name = "{name}"
target_dir = "bin"
target_permissions = "755"
```

### Pattern: No Strip Needed

```toml
[runtimes.layout]
download_type = "archive"

[runtimes.layout.archive]
strip_prefix = ""  # No prefix to remove
executable_paths = ["{name}.exe", "{name}"]
```

## Troubleshooting

### Issue: "Executable not found"

**Cause**: Wrong `strip_prefix` or `executable_paths`

**Fix**:
1. Download the archive manually
2. Extract and check actual structure
3. Update configuration to match

**Example**:
```bash
# Download
wget https://github.com/owner/repo/releases/download/v1.0.0/tool.tar.gz

# Extract
tar xzf tool.tar.gz

# Check structure
ls -R
# Output: tool-1.0.0/bin/tool

# Configuration should be:
# strip_prefix = "tool-{version}"
# executable_paths = ["bin/tool"]
```

### Issue: "Permission denied" on Unix

**Cause**: Missing `target_permissions`

**Fix**: Add `target_permissions = "755"` to all Unix binary layouts:

```toml
[runtimes.layout.binary."linux-x86_64"]
source_name = "tool"
target_name = "tool"
target_dir = "bin"
target_permissions = "755"  # ← Add this
```

### Issue: Wrong platform detected

**Cause**: Platform identifier mismatch

**Fix**: Use correct platform identifiers:
- Windows: `windows-x86_64`, `windows-aarch64`
- macOS: `macos-x86_64`, `macos-aarch64`
- Linux: `linux-x86_64`, `linux-aarch64`

## Batch Update Script

For updating multiple providers:

```bash
#!/bin/bash

PROVIDERS=(
    "kubectl:binary"
    "terraform:archive-root"
    "helm:archive-platform"
    "just:archive-root"
    "task:archive-root"
)

for entry in "${PROVIDERS[@]}"; do
    IFS=: read -r name type <<< "$entry"
    echo "Updating $name ($type)..."
    
    case $type in
        binary)
            # Add binary layout
            ;;
        archive-root)
            # Add archive layout with empty strip
            ;;
        archive-platform)
            # Add archive layout with platform strip
            ;;
    esac
done
```

## Validation Checklist

After updating, verify:

- [ ] `download_type` is `"binary"`, `"archive"`, or `"git_clone"`
- [ ] **Values use snake_case** (e.g., `git_clone` not `git-clone`)
- [ ] Binary: All platforms covered (windows, macos, linux)
- [ ] Binary: Unix platforms have `target_permissions = "755"`
- [ ] Archive: `strip_prefix` matches actual archive structure
- [ ] Archive: `executable_paths` includes both `.exe` and non-extension
- [ ] Paths use forward slashes `/` (not backslashes)
- [ ] Variables `{version}`, `{os}`, `{arch}` used correctly
- [ ] Test passes: `cargo check -p vx-provider-{name}`
- [ ] Installation works: `vx install {name}@latest`
- [ ] Execution works: `vx {name} --version`

## Update Migration Status

After successful update, mark as complete in `docs/provider-migration-status.md`:

```markdown
## ✅ Completed (X providers)

- **{name}** ✅ - {layout_type} layout ({description})
```

## Reference Quick Links

- Full templates: `references/update-templates.md`
- RFC 0019 spec: `references/rfc-0019-layout.md`
- Migration status: `docs/provider-migration-status.md`

## Example: Complete Update

Before:
```toml
[[runtimes]]
name = "kubectl"
description = "Kubernetes command-line tool"
executable = "kubectl"

[runtimes.versions]
source = "github-releases"
owner = "kubernetes"
repo = "kubernetes"
strip_v_prefix = true

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

After (with RFC 0019):
```toml
[[runtimes]]
name = "kubectl"
description = "Kubernetes command-line tool"
executable = "kubectl"

[runtimes.versions]
source = "github-releases"
owner = "kubernetes"
repo = "kubernetes"
strip_v_prefix = true

# RFC 0019: Executable Layout Configuration
[runtimes.layout]
download_type = "binary"

[runtimes.layout.binary."windows-x86_64"]
source_name = "kubectl.exe"
target_name = "kubectl.exe"
target_dir = "bin"

[runtimes.layout.binary."macos-x86_64"]
source_name = "kubectl"
target_name = "kubectl"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."macos-aarch64"]
source_name = "kubectl"
target_name = "kubectl"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-x86_64"]
source_name = "kubectl"
target_name = "kubectl"
target_dir = "bin"
target_permissions = "755"

[runtimes.layout.binary."linux-aarch64"]
source_name = "kubectl"
target_name = "kubectl"
target_dir = "bin"
target_permissions = "755"

[runtimes.platforms.windows]
executable_extensions = [".exe"]

[runtimes.platforms.unix]
executable_extensions = []
```

Result: ✅ kubectl now uses RFC 0019 layout configuration
