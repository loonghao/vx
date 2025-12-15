# Test: vx venv --help

Verify that `vx venv --help` displays usage information.

```console
$ vx venv --help
Virtual environment management

Usage: vx[EXE] venv [OPTIONS] <COMMAND>

Commands:
  create      Create a new virtual environment
  list        List all virtual environments
  activate    Activate a virtual environment
  deactivate  Deactivate the current virtual environment
  remove      Remove a virtual environment
  current     Show current virtual environment
  help        Print this message or the help of the given subcommand(s)

Options:
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
  -h, --help             Print help

```
