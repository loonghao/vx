# Test: vx uninstall --help

Verify that `vx uninstall --help` displays usage information.

```console
$ vx uninstall --help
Uninstall tool versions (preferred over remove)

Usage: vx[EXE] uninstall [OPTIONS] <TOOL> [VERSION]

Arguments:
  <TOOL>     Tool name
  [VERSION]  Version to uninstall (optional, removes all if not specified)

Options:
      --force            Force removal without confirmation
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help

```
