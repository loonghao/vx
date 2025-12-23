# Release Workflows Guide

This project has multiple release workflows to ensure reliable releases. Choose the appropriate workflow based on your needs.

## Available Release Workflows

### 1. Main Release Workflow (Recommended)

**File**: `.github/workflows/release.yml`
**Trigger**: Automatic on push to main branch
**Features**:

- ✅ Uses release-please for automated versioning
- ✅ Integrates with GoReleaser for multi-platform builds
- ✅ Supports Homebrew and Scoop package managers
- ✅ Comprehensive cross-compilation

**How it works**:

1. Push commits with conventional commit messages to main
2. release-please creates a PR with version bump and changelog
3. Merge the PR to trigger the actual release
4. GoReleaser builds and publishes binaries

### 2. GoReleaser Release (Recommended for Manual Releases)

**File**: `.github/workflows/tag-release.yml`
**Trigger**: Manual tag creation or workflow dispatch
**Features**:

- ✅ **PGO-optimized builds** for maximum performance
- ✅ **Multi-platform releases** (Linux, Windows, macOS, FreeBSD)
- ✅ **Package manager integration** (Homebrew, Scoop, Chocolatey, AUR, Winget)
- ✅ **Docker images** with multi-arch support
- ✅ **Performance benchmarking** after release
- ✅ **No dependency on release-please**
- ✅ **Comprehensive distribution** via GoReleaser

**How to use**:

```bash
# Create and push tag
git tag v0.1.0
git push origin v0.1.0

# Or trigger manually via GitHub Actions UI
# Go to Actions → GoReleaser Release → Run workflow
```

### 3. Simple Release-Please (Fallback)

**File**: `.github/workflows/simple-release.yml`
**Trigger**: Manual workflow dispatch
**Features**:

- ✅ Simplified release-please configuration
- ✅ Manual trigger for testing
- ✅ Basic binary uploads

**How to use**:

1. Go to Actions tab in GitHub
2. Select "Simple Release" workflow
3. Click "Run workflow"

## Troubleshooting

### "Invalid previous_tag parameter" Error

This error occurs when release-please can't parse commit history. Solutions:

1. **Use the tag-based workflow** as a fallback
2. **Check commit message format** - ensure they follow [Conventional Commits](https://www.conventionalcommits.org/)
3. **Use the simple release workflow** for testing

### Release-Please Not Creating PRs

1. Ensure commits follow conventional commit format:
   - `feat: add new feature`
   - `fix: resolve bug`
   - `chore: update dependencies`

2. Check the release-please configuration in `release-please-config.json`

3. Verify the manifest file `.release-please-manifest.json` has correct version

## Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types**:

- `feat`: New features
- `fix`: Bug fixes
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Maintenance tasks

## Manual Release Process

If automated workflows fail, you can create releases manually:

1. **Update version** in `Cargo.toml`
2. **Create tag**: `git tag v0.1.0`
3. **Push tag**: `git push origin v0.1.0`
4. **Use GoReleaser workflow** (recommended) or create release manually in GitHub

### GoReleaser Benefits

The unified GoReleaser workflow provides:

- 🚀 **PGO Optimization**: Profile-Guided Optimization for 15-20% performance improvement
- 📦 **Multi-Platform**: Linux (x64, ARM64, musl), Windows (x64, ARM64), macOS (Intel, Apple Silicon), FreeBSD
- 🏪 **Package Managers**: Automatic publishing to Homebrew, Scoop, Chocolatey, AUR, Winget
- 🐳 **Docker**: Multi-arch container images published to GitHub Container Registry
- 📊 **Benchmarking**: Automatic performance testing after release
- 🔍 **Checksums**: SHA256 checksums for all artifacts

## Configuration Files

- `release-please-config.json`: Main release-please configuration
- `.release-please-manifest.json`: Version tracking
- `.goreleaser.yml`: GoReleaser configuration (if exists)

## Best Practices

1. **Always use conventional commits** for automated releases
2. **Test with simple-release workflow** before relying on main workflow
3. **Keep tag-based workflow** as a reliable fallback
4. **Monitor GitHub Actions** for any workflow failures
5. **Update documentation** when changing release processes
