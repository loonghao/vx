# VX Publishing Strategy

## 🎯 **Current Problem**

The vx project uses a workspace architecture with multiple crates:

```
vx (main package)
├── vx-core ✅ (published)
├── vx-cli
├── vx-tool-node
├── vx-tool-go  
├── vx-tool-rust
├── vx-tool-uv
└── vx-pm-npm
```

**Issue**: If we only publish the main `vx` package, users can't install it because the dependencies aren't available on crates.io.

## 💡 **Solution: Two-Phase Publishing**

### **Phase 1: Initial Workspace Publishing**

**Goal**: Get all packages published to crates.io for the first time.

**Steps**:
1. ✅ **vx-core** - Already published
2. 🔄 **Publish remaining packages** using our automated script
3. 🔄 **Publish main vx package** last

**Command**:
```bash
# Use our automated publishing script
DRY_RUN=false scripts/publish-workspace.sh
```

**Publishing Order** (dependency-based):
```
1. vx-core ✅ (skip - already published)
2. vx-tool-go
3. vx-tool-rust  
4. vx-tool-uv
5. vx-pm-npm
6. vx-tool-node (depends on vx-pm-npm)
7. vx-cli (depends on all tools)
8. vx (depends on everything)
```

### **Phase 2: Automated Release Management**

**Goal**: Set up release-plz for future automated releases.

**Strategy**: 
- **release-plz**: Handles GitHub releases and version management
- **Manual workflow**: Handles crates.io publishing
- **Package managers**: Auto-publish to WinGet, Chocolatey, etc.

## 🚀 **Implementation Plan**

### **Step 1: Complete Initial Publishing**

```bash
# 1. Test the publishing script
DRY_RUN=true scripts/publish-workspace.sh

# 2. Publish all packages
DRY_RUN=false scripts/publish-workspace.sh
```

### **Step 2: Configure release-plz**

Once all packages are published, update `release-plz.toml`:

```toml
[workspace]
# Enable processing of all packages
release = true
# Disable automatic publishing (use manual workflow)
publish = false

[[package]]
name = "vx"
# Enable GitHub releases for main package only
git_release_enable = true
git_tag_enable = true
```

### **Step 3: Future Release Workflow**

1. **Developer**: Push conventional commit
2. **release-plz**: Creates release PR with version bumps
3. **Merge**: Release PR gets merged
4. **release-plz**: Creates GitHub release
5. **Manual**: Run "Publish to crates.io" workflow
6. **Automated**: Package managers get updated

## 🛡️ **Safety Measures**

### **Version Consistency**
- All packages use unified version (0.1.36)
- Workspace dependencies specify exact versions
- Publishing script validates versions

### **Dependency Order**
- Script publishes in correct dependency order
- Waits 30 seconds between publishes
- Skips already-published packages

### **Error Handling**
- Dry-run mode for testing
- Build and test before publishing
- Duplicate detection
- Version verification

## 📋 **Current Status**

- ✅ **vx-core**: Published to crates.io
- ✅ **Scripts**: Automated publishing script ready
- ✅ **CI**: GitHub Actions workflow ready
- 🔄 **Next**: Publish remaining packages
- ⏳ **Then**: Configure release-plz for automation

## 🎯 **Expected Outcome**

After completing both phases:

1. **✅ All packages available on crates.io**
2. **✅ Users can install**: `cargo install vx`
3. **✅ Automated releases** via release-plz
4. **✅ Manual control** over crates.io publishing
5. **✅ Package managers** auto-updated

## 🚨 **Important Notes**

- **Don't configure release-plz** until all packages are published
- **Use manual publishing** for initial setup
- **Test with dry-run** before actual publishing
- **Monitor crates.io** for successful uploads

## 🔄 **Next Actions**

1. **Run publishing script** to publish all remaining packages
2. **Verify installations** work: `cargo install vx`
3. **Configure release-plz** for future automation
4. **Test complete workflow** end-to-end
