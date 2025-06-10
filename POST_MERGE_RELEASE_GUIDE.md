# 🚀 Post-Merge Release Guide

This guide explains how to create the first release after merging the CI/CD PR.

## ✅ Prerequisites

Before creating a release, ensure:

- [ ] PR #5 has been merged to main
- [ ] All CI checks passed
- [ ] You have push access to the repository

## 🎯 Quick Release Steps

### Step 1: Switch to Main Branch
```bash
git checkout main
git pull origin main
```

### Step 2: Create and Push Release Tag
```bash
# Create the first release tag
git tag v0.1.0

# Push the tag to trigger release workflow
git push origin v0.1.0
```

### Step 3: Monitor Release Process
1. Go to [GitHub Actions](https://github.com/loonghao/vx/actions)
2. Watch the "Release" workflow (takes ~10-15 minutes)
3. The workflow will:
   - Build binaries for all platforms
   - Create packages (deb, rpm, apk)
   - Build and publish Docker images
   - Create GitHub Release with assets
   - Update package managers

### Step 4: Verify Release Success
After the workflow completes:

1. **Check GitHub Release**: Visit [Releases page](https://github.com/loonghao/vx/releases)
2. **Test Docker Image**:
   ```bash
   docker run --rm ghcr.io/loonghao/vx:v0.1.0 --help
   ```
3. **Test Installation Scripts**:
   ```bash
   # Linux/macOS
   curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
   
   # Windows (PowerShell)
   irm https://raw.githubusercontent.com/loonghao/vx/main/install-release.ps1 | iex
   ```

## 🎉 What Gets Released

The automated release includes:

### 📦 Binary Packages
- **Linux**: x86_64/aarch64 (glibc/musl) - tar.gz, deb, rpm, apk
- **macOS**: x86_64/aarch64 - tar.gz
- **Windows**: x86_64/aarch64 - zip
- **FreeBSD**: x86_64 - tar.gz

### 🐳 Docker Images
- Multi-arch images (amd64/arm64)
- Published to GitHub Container Registry
- Tagged with version and 'latest'

### 📋 Package Managers
- **Homebrew**: Formula automatically updated
- **Scoop**: Manifest automatically updated

## 🔧 Troubleshooting

### Release Workflow Fails
1. Check workflow logs in GitHub Actions
2. Common issues:
   - Cross-compilation failures
   - Docker registry permissions
   - Package manager update failures

### Missing Assets
If some assets are missing from the release:
1. Re-run the failed workflow
2. Check platform-specific build logs
3. May need to update dependencies or toolchain

## 📈 Future Releases

For subsequent releases, simply:
1. Update version in `Cargo.toml`
2. Create and push a new tag
3. The same automated process will run

Example:
```bash
# Update Cargo.toml version to 0.1.1
git add Cargo.toml
git commit -m "chore: bump version to 0.1.1"
git tag v0.1.1
git push origin main
git push origin v0.1.1
```

## 🎊 Success!

Once the release is complete, you'll have:
- ✅ Multi-platform binaries available for download
- ✅ Docker images ready for use
- ✅ Package manager integration
- ✅ Automated release pipeline for future versions

The vx tool is now ready for global distribution! 🌍
