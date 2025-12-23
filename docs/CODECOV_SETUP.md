# Codecov Setup Guide

## Overview

Codecov integration is **optional** for the vx project. The CI will run successfully without it, but setting it up provides better coverage reporting and badges.

## Current Status

- ✅ CI runs without Codecov token (coverage step will be skipped)
- ✅ Local coverage generation works: `cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info`
- ⚠️ Codecov upload requires token for public repositories

## Setting Up Codecov (Optional)

### 1. Create Codecov Account

1. Go to [codecov.io](https://codecov.io)
2. Sign in with your GitHub account
3. Add the `loonghao/vx` repository

### 2. Get Repository Token

1. Navigate to your repository on Codecov
2. Go to Settings → General
3. Copy the "Repository Upload Token"

### 3. Add Token to GitHub Secrets

1. Go to GitHub repository: `https://github.com/loonghao/vx`
2. Navigate to Settings → Secrets and variables → Actions
3. Click "New repository secret"
4. Name: `CODECOV_TOKEN`
5. Value: Paste the token from Codecov
6. Click "Add secret"

### 4. Verify Setup

After adding the token, the next CI run will automatically upload coverage reports to Codecov.

## Benefits of Codecov Integration

- 📊 **Coverage Reports**: Detailed line-by-line coverage analysis
- 📈 **Trend Tracking**: Coverage changes over time
- 🔍 **PR Comments**: Automatic coverage reports on pull requests
- 🏆 **Badges**: Professional coverage badges for README

## Alternative: Local Coverage

If you prefer not to use Codecov, you can still generate coverage reports locally:

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate LCOV report
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info

# Generate HTML report (opens in browser)
cargo llvm-cov --all-features --workspace --html --open
```

## Troubleshooting

### CI Fails with Codecov Error

- **Solution**: This is expected without token. The CI is configured to continue on Codecov errors.
- **Status**: ✅ CI will still pass, only the coverage upload step will be skipped.

### Coverage Badge Shows "unknown"

- **Cause**: No coverage data uploaded to Codecov
- **Solution**: Add `CODECOV_TOKEN` secret or use local coverage reports

### Token Not Working

1. Verify the token is correct in Codecov dashboard
2. Ensure the secret name is exactly `CODECOV_TOKEN`
3. Check that the repository is properly connected to Codecov

## Current Configuration

The CI is configured to:

- ✅ Generate coverage reports locally
- ✅ Continue on Codecov upload errors
- ✅ Not fail CI if Codecov is unavailable
- ✅ Work with or without the token

This ensures the project remains accessible to contributors regardless of Codecov setup.
