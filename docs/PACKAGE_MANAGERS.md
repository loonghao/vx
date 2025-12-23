# Package Manager Publishing Guide

This document explains how to set up automated publishing to various package managers for the vx project.

## 🎯 Supported Package Managers

| Platform | Package Manager | Status | Auto-Publish |
|----------|----------------|--------|--------------|
| Windows  | WinGet         | ✅ Ready | ✅ Yes |
| Windows  | Chocolatey     | 🚧 Setup Required | ✅ Yes |
| Windows  | Scoop          | 🚧 Setup Required | ⚠️ Manual |
| macOS    | Homebrew       | 🚧 Setup Required | ✅ Yes |
| Linux    | Snap           | 📋 Planned | ❌ No |
| Linux    | Flatpak        | 📋 Planned | ❌ No |

## 🔧 Setup Instructions

### 1. WinGet (Windows Package Manager)

**Prerequisites:**

- At least one version of vx must already exist in [winget-pkgs](https://github.com/microsoft/winget-pkgs)
- Fork [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) under your account

**Required Secrets:**

```bash
WINGET_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx  # GitHub PAT with public_repo scope
```

**Configuration:**

- Package identifier: `loonghao.vx`
- Automatically detects new releases and creates PRs to winget-pkgs

### 2. Chocolatey

**Prerequisites:**

- Create account at [chocolatey.org](https://chocolatey.org/)
- Create a Chocolatey package (.nupkg file)
- Get API key from your Chocolatey account

**Required Secrets:**

```bash
CHOCOLATEY_API_KEY=xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

**Setup Steps:**

1. Create `vx.nuspec` file in project root
2. Build .nupkg file during release process
3. Upload to Chocolatey via GitHub Actions

### 3. Homebrew (macOS)

**Prerequisites:**

- Create a Homebrew tap repository (e.g., `loonghao/homebrew-vx`)
- Generate GitHub PAT with repo permissions

**Required Secrets:**

```bash
HOMEBREW_TAP_GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx  # GitHub PAT
```

**Configuration:**

- Tap repository: `loonghao/homebrew-vx`
- Formula name: `vx`
- Supports multiple architectures (Intel/ARM)

### 4. Scoop (Windows)

**Prerequisites:**

- Create a Scoop bucket repository (e.g., `loonghao/scoop-vx`)
- Generate Scoop manifest JSON

**Status:** Requires manual setup

- Consider using [Scoop-GithubActions](https://github.com/Ash258/Scoop-GithubActions)
- Or implement custom bucket update script

## 🚀 Workflow Trigger

The package manager workflow is triggered:

1. **Automatically**: After a successful release workflow
2. **Manually**: Via workflow_dispatch with version input

```yaml
# Automatic trigger (recommended)
on:
  workflow_run:
    workflows: ["Release"]
    types: [completed]

# Manual trigger
on:
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to publish'
        required: true
```

## 📋 Required Repository Secrets

Add these secrets to your GitHub repository:

| Secret Name | Description | Required For | How to Get |
|-------------|-------------|--------------|------------|
| `WINGET_TOKEN` | GitHub PAT for WinGet | WinGet | [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens) |
| `CHOCOLATEY_API_KEY` | Chocolatey API key | Chocolatey | [Chocolatey Account > API Keys](https://chocolatey.org/account) |
| `HOMEBREW_TAP_GITHUB_TOKEN` | GitHub PAT for Homebrew tap | Homebrew | [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens) |
| `SCOOP_BUCKET_TOKEN` | GitHub PAT for Scoop bucket | Scoop | [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens) |
| `CARGO_REGISTRY_TOKEN` | Cargo registry token | release-plz | [crates.io Account Settings](https://crates.io/settings/tokens) |

### Setting up GitHub Personal Access Tokens

For `WINGET_TOKEN`, `HOMEBREW_TAP_GITHUB_TOKEN`, and `SCOOP_BUCKET_TOKEN`:

1. Go to [GitHub Settings > Developer settings > Personal access tokens](https://github.com/settings/tokens)
2. Click "Generate new token (classic)"
3. Set expiration and select scopes:
   - `public_repo` (for public repositories)
   - `repo` (for private repositories)
4. Copy the generated token and add it to repository secrets

## 🔍 Monitoring and Troubleshooting

### Check Workflow Status

1. Go to Actions tab in GitHub
2. Look for "Package Managers" workflow
3. Check individual job status

### Common Issues

**WinGet:**

- Ensure package already exists in winget-pkgs
- Check PAT permissions (public_repo scope)
- Verify fork exists under correct account

**Chocolatey:**

- Verify API key is correct
- Ensure .nupkg file is properly built
- Check package naming conventions

**Homebrew:**

- Verify tap repository exists
- Check PAT has repo permissions for tap
- Ensure binary naming matches expectations

## 📚 Additional Resources

- [WinGet Package Manager](https://docs.microsoft.com/en-us/windows/package-manager/)
- [Chocolatey Documentation](https://docs.chocolatey.org/)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Scoop Buckets](https://github.com/ScoopInstaller/Scoop/wiki/Buckets)

## 🤝 Contributing

To add support for additional package managers:

1. Research the package manager's API and requirements
2. Find or create a GitHub Action for automation
3. Add configuration to `.github/workflows/package-managers.yml`
4. Update this documentation
5. Test with a non-production release

## 📞 Support

If you encounter issues with package manager publishing:

1. Check the workflow logs in GitHub Actions
2. Verify all required secrets are set
3. Ensure prerequisites are met for each package manager
4. Open an issue with detailed error information
