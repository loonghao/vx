# 🔑 Setting up crates.io Token for vx Project

This guide helps you set up the correct crates.io API token for automated publishing.

## 🚨 Common Issues

### Version Synchronization Problems

If you're seeing errors like:
```
ERROR: failed to update packages
Caused by:
  package `vx` has a different version (0.2.3) with respect to the registry package (0.2.2),
  but the git tag v0.2.3 exists. Consider running `cargo publish` manually to publish the
  new version of this package.
```

**Root Cause**: Local package versions are ahead of registry versions, but git tags exist.
**Solution**: This is resolved by using `release_always = false` in release-plz.toml and proper workflow separation.

### Token Format Issues

If you're seeing this error:
```
error: failed to publish to registry at https://crates.io
Caused by:
  the remote server responded with an error (status 401 Unauthorized):
  The given API token does not match the format used by crates.io.
```

**Root Cause**: Your token is either:
1. Using the old format (pre-2020)
2. Incorrectly formatted
3. Expired or revoked

## ✅ Solution: Generate a New Token

### Step 1: Visit crates.io
1. Go to [https://crates.io/me](https://crates.io/me)
2. Log in with your GitHub account

### Step 2: Generate New Token
1. Scroll down to "API Tokens" section
2. Click "New Token"
3. Give it a descriptive name: `vx-project-ci`
4. **Important**: Select appropriate scopes:
   - ✅ `publish-new` - Allow publishing new crates
   - ✅ `publish-update` - Allow updating existing crates
   - ❌ Don't select unnecessary scopes for security

### Step 3: Verify Token Format
Your new token should:
- ✅ Start with `crates-io-`
- ✅ Be approximately 40+ characters long
- ✅ Look like: `crates-io-abcd1234efgh5678ijkl9012mnop3456qrst7890`

### Step 4: Update GitHub Secrets
1. Go to your GitHub repository
2. Navigate to Settings → Secrets and variables → Actions
3. Update or create `CARGO_REGISTRY_TOKEN` secret
4. Paste your new token (the entire string including `crates-io-` prefix)

## 🔍 Verification

### Test Locally (Optional)
```bash
# Set your token temporarily
export CARGO_REGISTRY_TOKEN="your-new-token-here"

# Test with dry-run (safe)
cargo publish --dry-run --package vx

# If successful, the token format is correct
```

### Test in CI
1. Create a test PR to trigger release-plz-pr job
2. Check that release-plz can access the registry
3. Merge the PR to trigger actual release

## 🛡️ Security Best Practices

### Token Permissions
- ✅ Use minimal required scopes
- ✅ Use repository-specific tokens when possible
- ❌ Don't use tokens with unnecessary permissions

### Token Management
- 🔄 Rotate tokens periodically (every 6-12 months)
- 🗑️ Delete unused tokens immediately
- 📝 Use descriptive names for tokens

### GitHub Secrets
- ✅ Use repository secrets for private repos
- ✅ Use environment secrets for additional protection
- ❌ Never commit tokens to code

## 🚨 Troubleshooting

### Still Getting 401 Errors?
1. **Double-check token format**: Must start with `crates-io-`
2. **Verify token permissions**: Ensure `publish-new` and `publish-update` are enabled
3. **Check crate ownership**: Ensure your account owns or has permissions for all packages
4. **Test with single package**: Try publishing one package manually first

### Package Ownership Issues
```bash
# Check who owns a package
cargo search vx-core

# Add yourself as owner (if you have permissions)
cargo owner --add your-username vx-core
```

### Network/Registry Issues
```bash
# Check if crates.io is accessible
curl -I https://crates.io/api/v1/crates

# Verify your token works with API
curl -H "Authorization: Bearer your-token" https://crates.io/api/v1/me
```

## 📞 Getting Help

If you're still having issues:

1. **Check crates.io status**: [https://status.crates.io/](https://status.crates.io/)
2. **Review Rust blog**: [Security advisory from 2020](https://blog.rust-lang.org/2020/07/14/crates-io-security-advisory.html)
3. **Ask for help**: Create an issue in this repository with:
   - The exact error message
   - Whether you've generated a new token
   - The first few characters of your token (e.g., "crates-io-abcd...")

## 🔗 Useful Links

- [crates.io Account Management](https://crates.io/me)
- [Cargo Publishing Guide](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Secrets Documentation](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
- [Rust Security Advisory (2020)](https://blog.rust-lang.org/2020/07/14/crates-io-security-advisory.html)
