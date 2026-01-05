# Release Process

This document describes the release and package publishing process for vx.

## Overview

The vx project uses an automated release pipeline with GitHub Actions that handles:
- Release creation with release-please
- Binary building for multiple platforms
- Publishing to various package managers (WinGet, Chocolatey, Homebrew, Scoop)

## Version Format

### Git Tags

vx uses the following version tag formats:

- **Release-Please Format**: `vx-v0.1.0` (current format used by release-please)
- **Standard Format**: `v0.1.0` (traditional semantic versioning)

### Version Normalization

Different package managers require different version formats:

| Package Manager | Expected Format | Example | Notes |
|----------------|----------------|---------|-------|
| WinGet | `0.1.0` | `0.1.0` | Without `v` prefix, normalized from `vx-v0.1.0` |
| Chocolatey | `0.1.0` | `0.1.0` | Without `v` prefix |
| Homebrew | `0.1.0` | `0.1.0` | Without `v` prefix |
| Scoop | `0.1.0` | `0.1.0` | Without `v` prefix |

The workflows automatically handle version normalization to ensure compatibility with each package manager.

## GitHub Actions Workflows

### Release Workflow (`.github/workflows/release.yml`)

This workflow runs on pushes to the main branch and handles:

1. **Release-Please**: Creates release PRs and tags
2. **Binary Building**: Builds binaries for all supported platforms
3. **Asset Upload**: Uploads binaries to the GitHub release

**Version Extraction**:
```bash
# Extract version number: vx-v0.1.0 -> 0.1.0, v0.1.0 -> 0.1.0
VERSION=$(echo "${TAG}" | sed -E 's/^(vx-)?v//')
```

### Package Managers Workflow (`.github/workflows/package-managers.yml`)

This workflow runs after the Release workflow completes and publishes to package managers.

**Version Normalization for WinGet**:
```bash
# Remove 'vx-' prefix and 'v' prefix (vx-v0.1.0 -> 0.1.0)
normalized_version="${version#vx-}"
normalized_version="${normalized_version#v}"
```

This ensures WinGet receives `0.1.0` instead of `vx-v0.1.0`, which resolves the issue where WinGet was showing version numbers like "x-v0.1.0".

#### Publishing Steps

1. **Check Release**: Verifies the Release workflow succeeded
2. **Get Version**: Retrieves the latest release version
3. **Normalize Version**: Removes `vx-` prefix for package managers
4. **Verify Release**: Confirms the GitHub release exists
5. **Publish**: Publishes to each package manager in parallel

#### Supported Package Managers

- **WinGet** (`publish-winget`): Uses `vedantmgoyal9/winget-releaser`
- **Chocolatey** (`publish-chocolatey`): Downloads binary and creates `.nupkg`
- **Homebrew** (`publish-homebrew`): Generates formula with checksums
- **Scoop** (`publish-scoop`): Creates JSON manifest

## Testing Version Extraction

The project includes test scripts to validate version extraction logic:

### Test Version Normalization

Run the test script to verify version normalization:

```bash
bash scripts/test-winget-version.sh
```

This tests the following transformations:

| Input | Expected Output | Description |
|-------|----------------|-------------|
| `vx-v0.1.0` | `0.1.0` | Remove `vx-` and `v` prefix |
| `vx-v1.0.0` | `1.0.0` | Remove `vx-` and `v` prefix |
| `v0.1.0` | `0.1.0` | Remove `v` prefix |
| `v1.0.0` | `1.0.0` | Remove `v` prefix |

## Manual Publishing

### Trigger Package Publishing Manually

If you need to manually trigger package publishing:

1. Go to **Actions** → **Package Managers**
2. Click **Run workflow**
3. Enter the version tag (e.g., `vx-v0.1.0` or `v0.1.0`)
4. Check **Force run** if needed

### Publishing to Specific Package Managers

Each package manager can be published independently by running the respective job.

## Troubleshooting

### WinGet Version Issues

**Problem**: WinGet shows version like "x-v0.1.0" instead of "0.1.0"

**Cause**: The `release-tag` parameter was receiving the full tag name `vx-v0.1.0` without proper normalization to remove both the `vx-` and `v` prefixes.

**Solution**: The workflow now includes a normalization step:

```yaml
- name: Normalize version for WinGet
  id: normalize
  run: |
    version="${{ steps.version.outputs.version }}"
    # Remove 'vx-' prefix if present (vx-v0.1.0 -> v0.1.0)
    normalized_version="${version#vx-}"
    # Remove 'v' prefix for WinGet (v0.1.0 -> 0.1.0)
    normalized_version="${normalized_version#v}"
    echo "normalized_version=$normalized_version" >> $GITHUB_OUTPUT
```

### Verifying Release Assets

To verify release assets are available:

```bash
# Check release exists
curl -s https://api.github.com/repos/loonghao/vx/releases/tags/vx-v0.1.0

# List assets
curl -s https://api.github.com/repos/loonghao/vx/releases/tags/vx-v0.1.0 | \
  jq -r '.assets[] | "\(.name) (\(.size) bytes)"'
```

## Best Practices

1. **Always use semantic versioning**: `MAJOR.MINOR.PATCH`
2. **Test version extraction**: Run `scripts/test-winget-version.sh` before releasing
3. **Verify release assets**: Ensure all platform binaries are uploaded
4. **Monitor package publishing**: Check workflow status for each package manager
5. **Update documentation**: Keep version references up to date in docs

## Related Files

- `.github/workflows/release.yml` - Main release workflow
- `.github/workflows/package-managers.yml` - Package publishing workflow
- `scripts/test-winget-version.sh` - Version normalization tests
- `distribution.toml` - Distribution channel configuration
