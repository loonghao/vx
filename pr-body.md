## Summary

Fix the installation script download failures caused by incorrect release asset naming and URL construction.

## Problems Fixed

1. **Duplicate 'vx-' prefix in asset names**: Release assets were named `vx-vx-v0.5.7-x86_64-pc-windows-msvc.zip` instead of `vx-x86_64-pc-windows-msvc.zip`

2. **Incorrect tag format in download URLs**: Installer scripts used `v0.5.7` format but actual tags are `vx-v0.5.7`

3. **CDN fallbacks don't work**: jsDelivr/Fastly CDN don't support GitHub Release assets, only repository files

## Changes

| File | Change |
|------|--------|
| `.github/workflows/release.yml` | Simplify asset naming to `vx-{target}.{ext}` (without version in filename) |
| `install.ps1` | Use full tag name format, remove CDN fallbacks, support multiple version input formats |
| `install.sh` | Same changes as install.ps1 |

## After This Fix

- Asset name: `vx-x86_64-pc-windows-msvc.zip`
- Download URL: `https://github.com/loonghao/vx/releases/download/vx-v0.5.8/vx-x86_64-pc-windows-msvc.zip`

## Testing

After merging and releasing, test with:

```powershell
irm https://raw.githubusercontent.com/loonghao/vx/main/install.ps1 | iex
```

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/vx/main/install.sh | bash
```
