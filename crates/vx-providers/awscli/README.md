# AWS CLI Provider

AWS CLI v2 provider for vx.

## Features

- ✅ Cross-platform support (Windows, macOS, Linux)
- ✅ Automatic silent installation
- ✅ Version management
- ✅ No admin/sudo privileges required (installs to user directory)

## Installation Process

This provider implements the complete AWS CLI installation workflow for each platform:

### Windows

1. **Download**: Downloads `.msi` installer from `awscli.amazonaws.com`
2. **Extract**: MSI file is kept as-is (no extraction needed)
3. **Install**: Runs `msiexec /i /qn /norestart TARGETDIR=<path>` for silent installation
4. **Verify**: Searches for `aws.exe` in multiple possible locations

### macOS & Linux

1. **Download**: Downloads `.pkg` (macOS) or `.zip` (Linux) from `awscli.amazonaws.com`
2. **Extract**: Extracts the archive to get `aws/` directory
3. **Install**: Runs `./aws/install --install-dir <path> --bin-dir <bin-path>`
4. **Verify**: Checks for `aws` binary in `bin/` or `aws-cli/`

## Runtime Lifecycle

The AWS CLI provider demonstrates the full Runtime lifecycle:

```rust
┌─────────────────────────────────────────────────────────┐
│                   Runtime Lifecycle                      │
└─────────────────────────────────────────────────────────┘

1. Parse (解析)
   ├─ fetch_versions()    → Get available versions
   └─ download_url()      → Generate download URL

2. Download (下载)
   └─ [Handled by vx-installer]

3. Extract (提取)
   ├─ [Handled by vx-installer]
   └─ post_extract()      → Post-extraction hook (optional)

4. Install (安装)
   └─ post_install()      → Platform-specific installation
                            - Windows: msiexec
                            - Linux/macOS: ./aws/install script

5. Verify (验证)
   └─ verify_installation() → Check executable exists and is valid
```

## Usage

```bash
# Install latest version
vx aws --version

# Install specific version
vx aws@2.32.25 --version

# Use AWS CLI
vx aws s3 ls
vx aws ec2 describe-instances
```

## Version Management

AWS CLI versions are hardcoded in the provider since AWS doesn't use GitHub releases for stable versions:

- `latest` - Always points to the latest stable version
- `2.32.25`, `2.32.0`, `2.31.0`, ... - Specific versions

To update the version list, edit `runtime.rs` and add new versions to the `fetch_versions()` method.

## Architecture Notes

### Why Custom `post_install()`?

Unlike most runtimes that are distributed as pre-built binaries, AWS CLI uses platform-specific installers:

- **Windows MSI**: Requires `msiexec` to extract and install
- **macOS PKG**: Contains an install script that must be run
- **Linux ZIP**: Contains an install script that must be run

The `post_install()` hook handles these platform-specific installation procedures.

### Why Custom `verify_installation()`?

AWS CLI installers don't always install to predictable locations:

- **Windows**: MSI may install to `Amazon/AWSCLIV2/` or other subdirectories
- **Linux/macOS**: Install script creates symlinks in `bin/` and installs to `aws-cli/`

The custom verification logic searches multiple possible locations to find the actual executable.

## References

- [AWS CLI Installation Guide](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html)
- [AWS CLI v2 Changelog](https://github.com/aws/aws-cli/blob/v2/CHANGELOG.rst)
