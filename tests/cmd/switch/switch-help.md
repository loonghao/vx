# Test: vx switch --help

Verify that `vx switch --help` displays usage information.

```console
$ vx switch --help
Switch to a different version of a tool

Usage: vx[EXE] switch [OPTIONS] <TOOL_VERSION>

Arguments:
  <TOOL_VERSION>  Tool and version (e.g., go@1.21.6, node@18.0.0)

Options:
      --global           Make this the global default
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help

```
