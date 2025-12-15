# Test: vx env --help

Verify that `vx env --help` displays usage information.

```console
$ vx env --help
Environment management

Usage: vx[EXE] env [OPTIONS] <COMMAND>

Commands:
  create  Create a new environment
  use     Activate an environment
  list    List all environments
  delete  Delete an environment
  show    Show current environment details
  add     Add a runtime to an environment
  remove  Remove a runtime from an environment
  help    Print this message or the help of the given subcommand(s)

Options:
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
  -h, --help             Print help

```
