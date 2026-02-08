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

#### Workflow Trigger Logic

The release workflow uses a sophisticated trigger mechanism to handle different scenarios:

| Scenario | Release-Please Job | Build Job | Notes |
|----------|-------------------|-----------|-------|
| Regular push (feat, fix, etc.) | Runs | Triggered if release created | Normal development flow |
| Release PR merge (`chore: release vX.Y.Z`) | **Skipped** | **Triggered** | Extracts version from commit message |
| Dependabot PR (`chore(deps): bump...`) | Skipped | Not triggered | Prevents duplicate builds |
| Manual workflow dispatch | Skipped | Triggered | Emergency/manual releases |

**Key Logic**:
```yaml
# Release-please job skips release commits to prevent recursion
if: |
  github.event_name == 'push' &&
  github.ref == 'refs/heads/main' &&
  !contains(github.event.head_commit.message, 'chore: release') &&
  github.event.head_commit.author.name != 'github-actions[bot]'

# Build job triggers on:
# 1. Release created by release-please
# 2. Release PR merge (detected via commit message)
# 3. Manual workflow dispatch
if: |
  always() &&
  (
    (needs.release-please.result == 'success' && needs.release-please.outputs.release_created == 'true') ||
    github.event_name == 'workflow_dispatch' ||
    (github.event_name == 'push' && contains(github.event.head_commit.message, 'chore: release'))
  )
```

This ensures that when a release PR is merged (e.g., "chore: release v0.6.24"), the build job still runs even though release-please is skipped.

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

## Testing Release Workflow Logic

The project includes tests to validate the release workflow trigger logic:

### Run Workflow Tests

```bash
cargo test --test release_workflow_tests
```

This validates:
- Version extraction from commit messages
- Version normalization
- Release commit detection
- Workflow trigger conditions for different scenarios

### Test Version Extraction

The project also includes test scripts to validate version extraction logic:

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

### Release Workflow Not Triggering

**Problem**: After merging a release PR (e.g., "chore: release v0.6.24"), the build job doesn't run.

**Cause**: The original workflow logic required `release-please` job to succeed and create a release. However, when a release PR is merged, the `release-please` job is intentionally skipped to prevent recursive PR creation. This caused the `get-tag` job (and subsequent build jobs) to be skipped as well.

**Solution**: The workflow now includes additional conditions to detect release PR merges:

```yaml
# Build job now triggers on release PR merges
if: |
  always() &&
  (
    (needs.release-please.result == 'success' && needs.release-please.outputs.release_created == 'true') ||
    github.event_name == 'workflow_dispatch' ||
    (github.event_name == 'push' && contains(github.event.head_commit.message, 'chore: release'))  # <-- Added
  )
```

When a commit message contains "chore: release", the workflow extracts the version from the message and proceeds with the build.

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

### Release Commits Triggering Unnecessary CI Runs

**Problem**: When Release Please merges a release PR (e.g., "chore: release v0.7.6"), the resulting commit triggers CI, CodeQL, and Benchmark workflows unnecessarily, wasting significant CI resources (15+ minutes for CI, 12+ minutes for CodeQL).

**Cause**: The CI, CodeQL, and Benchmark workflows had no filtering mechanism to exclude release commits on push to main. Since release commits modify `Cargo.toml` and `Cargo.lock` (version bumps), even path-filtered workflows like Benchmark were triggered.

**Solution**: Added `if` conditions at the job level to skip release commits:

```yaml
# Skip for release commits (applied to CI, CodeQL, and Benchmark)
if: >-
  github.event_name != 'push' ||
  !startsWith(github.event.head_commit.message, 'chore: release')
```

This condition:
- Allows the job to run normally for PRs, scheduled runs, and manual dispatches
- Only skips when the event is a push AND the commit message starts with `chore: release`
- When the first job in a workflow is skipped, all downstream dependent jobs are automatically skipped too

**Affected workflows**:
- `.github/workflows/ci.yml` - `detect-changes` job (gates all downstream CI jobs)
- `.github/workflows/codeql.yml` - `analyze` job
- `.github/workflows/benchmark.yml` - `benchmark` job

**Not affected** (intentionally):
- `.github/workflows/release-please.yml` - Must still run on release commits to detect `releases_created` and trigger the Release workflow

## Best Practices

1. **Always use semantic versioning**: `MAJOR.MINOR.PATCH`
2. **Test version extraction**: Run `scripts/test-winget-version.sh` before releasing
3. **Verify release assets**: Ensure all platform binaries are uploaded
4. **Monitor package publishing**: Check workflow status for each package manager
5. **Update documentation**: Keep version references up to date in docs

## Related Files

- `.github/workflows/release.yml` - Main release workflow
- `.github/workflows/release-please.yml` - Release Please workflow (creates release PRs and tags)
- `.github/workflows/package-managers.yml` - Package publishing workflow
- `.github/workflows/ci.yml` - CI workflow (skips release commits)
- `.github/workflows/codeql.yml` - CodeQL analysis (skips release commits)
- `.github/workflows/benchmark.yml` - Performance benchmarks (skips release commits)
- `scripts/test-winget-version.sh` - Version normalization tests
- `distribution.toml` - Distribution channel configuration
