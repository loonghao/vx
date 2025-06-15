# Release Process

This document describes the automated release process for vx.

## Overview

The release process is split into two main workflows:

1. **release-plz**: Handles version bumping, changelog generation, and GitHub releases
2. **publish-crates**: Handles publishing to crates.io (manual trigger)

## Automated Release Flow

### 1. Code Changes
- Make changes following conventional commits format:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation
  - `chore:` for maintenance

### 2. release-plz Workflow
**Trigger**: Push to `main` branch

**What it does**:
- ✅ Analyzes commits since last release
- ✅ Determines next version using semantic versioning
- ✅ Updates `Cargo.toml` version
- ✅ Generates/updates `CHANGELOG.md`
- ✅ Creates Git tag (e.g., `v0.1.5`)
- ✅ Creates GitHub release with binaries
- ❌ Does NOT publish to crates.io (manual step)

### 3. Package Manager Publishing
**Trigger**: After GitHub release is created

**What it does**:
- ✅ Publishes to WinGet
- ✅ Publishes to Chocolatey
- ✅ Publishes to Homebrew
- ✅ Publishes to Scoop

## Manual Steps

### Publishing to crates.io

After a GitHub release is created, manually publish to crates.io:

1. **Go to Actions tab** in GitHub
2. **Select "Publish to crates.io" workflow**
3. **Click "Run workflow"**
4. **Enter the version** (e.g., `v0.1.5`)
5. **Set dry_run to `true`** for testing
6. **Run and verify** the dry run succeeds
7. **Re-run with dry_run to `false`** to actually publish

### First-time Setup

For the first release, you need to:

1. **Set up GitHub Secrets**:
   ```
   CARGO_REGISTRY_TOKEN     # From crates.io
   CHOCOLATEY_API_KEY       # From chocolatey.org
   HOMEBREW_TAP_GITHUB_TOKEN # GitHub token for homebrew tap
   SCOOP_BUCKET_TOKEN       # GitHub token for scoop bucket
   WINGET_TOKEN            # Microsoft Partner Center token
   ```

2. **Manually publish first version**:
   ```bash
   # Login to crates.io
   cargo login <your-token>
   
   # Publish first version
   cargo publish
   ```

## Troubleshooting

### release-plz Fails with "package not found"

**Problem**:
```
ERROR failed to update packages
Caused by:
    0: failed to determine next versions
    1: failed to retrieve difference of package vx
    2: package `vx-*` not found in the registry, but the git tag v0.1.36 exists.
```

**Root Cause**: release-plz tries to check all workspace packages on crates.io, but workspace members haven't been published yet.

**Solution**:
1. **Workspace Configuration**: Set `release = false` in `[workspace]` to disable processing of all packages by default
2. **Main Package Only**: Set `release = true` only for the main `vx` package in `[[package]]` section
3. **Separate Publishing**: Use our custom workflow to publish workspace members to crates.io

**Current Configuration**:
```toml
[workspace]
release = false  # Disable all packages by default

[[package]]
name = "vx"
release = true   # Only process main package
publish = false  # But don't auto-publish to crates.io
```

### Package Manager Publishing Fails

Check the secrets are correctly configured and have the right permissions.

### Version Mismatch

Ensure the version in `Cargo.toml` matches the Git tag created by release-plz.

## Configuration Files

- `release-plz.toml` - release-plz configuration
- `.github/workflows/release-plz.yml` - GitHub release automation
- `.github/workflows/publish-crates.yml` - crates.io publishing
- `.github/workflows/package-managers.yml` - Package manager publishing

## Best Practices

1. **Use conventional commits** for automatic version detection
2. **Test dry runs** before actual publishing
3. **Monitor workflows** for failures
4. **Keep secrets updated** and secure
5. **Document breaking changes** in commit messages
