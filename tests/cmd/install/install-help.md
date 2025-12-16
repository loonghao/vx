# Test: vx install --help

Verify that `vx install --help` displays usage information.

```console
$ vx install --help
Install a specific tool version

Usage: vx[EXE] install [OPTIONS] <TOOL> [VERSION]

Arguments:
  <TOOL>     Tool name (e.g., uv, node, go, rust)
  [VERSION]  Version to install (e.g., 1.0.0, latest, lts)

Options:
      --force            Force reinstallation even if already installed
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help

```
