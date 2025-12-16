# Test: vx self-update --help

Verify that `vx self-update --help` displays usage information.

```console
$ vx self-update --help
Update vx itself to the latest version

Usage: vx[EXE] self-update [OPTIONS] [VERSION]

Arguments:
  [VERSION]  Specific version to install

Options:
      --check            Only check for updates, don't install
      --token <TOKEN>    GitHub token for authenticated API requests (avoids rate limits)
      --prerelease       Include pre-release versions
      --force            Force update even if already up to date
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help

```
