# Test: vx global --help

Verify that `vx global --help` displays usage information.

```console
$ vx global --help
Global tool management

Usage: vx[EXE] global [OPTIONS] <COMMAND>

Commands:
  list        List all globally installed tools
  info        Show information about a specific global tool
  remove      Remove a global tool (only if not referenced by any venv)
  dependents  Show which virtual environments depend on a tool
  cleanup     Clean up unused global tools
  help        Print this message or the help of the given subcommand(s)

Options:
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
  -h, --help             Print help

```
