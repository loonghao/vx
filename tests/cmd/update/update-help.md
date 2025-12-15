# Test: vx update --help

Verify that `vx update --help` displays usage information.

```console
$ vx update --help
Update tools to latest versions

Usage: vx[EXE] update [OPTIONS] [TOOL]

Arguments:
  [TOOL]  Tool name (optional, updates all if not specified)

Options:
      --apply            Apply updates automatically
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
  -h, --help             Print help

```
