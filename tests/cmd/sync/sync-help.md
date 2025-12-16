# Test: vx sync --help

Verify that `vx sync --help` displays usage information.

```console
$ vx sync --help
Sync project tools from .vx.toml

Usage: vx[EXE] sync [OPTIONS]

Options:
      --check            Only check, don't install
  -f, --force            Force reinstall all tools
      --dry-run          Preview operations without executing
  -v, --verbose          Show verbose output
      --no-parallel      Disable parallel installation
      --no-auto-install  Disable auto-install
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help

```
