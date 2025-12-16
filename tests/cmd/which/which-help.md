# Test: vx which --help

Verify that `vx which --help` shows help message.

```console
$ vx which --help
Show which tool version is being used (preferred over where)

Usage: vx[EXE] which [OPTIONS] <TOOL>

Arguments:
  <TOOL>  Tool name

Options:
      --all              Show all installed versions
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help

```
