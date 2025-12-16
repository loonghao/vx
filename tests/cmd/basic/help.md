# Test: vx --help

Verify that `vx --help` displays usage information.

```console
$ vx --help
Universal version executor for development tools
[..]
Usage: vx[..][COMMAND]
[..]
Commands:
  version      Show version information
  list         List supported tools
  install      Install a specific tool version
[..]
  self-update  Update vx itself to the latest version
  uninstall    Uninstall tool versions (preferred over remove)
  which        Show which tool version is being used (preferred over where)
  versions     Show available versions for a tool (preferred over fetch)
  switch       Switch to a different version of a tool
  config       Configuration management
  search       Search available tools
  sync         Sync project tools from .vx.toml
  init         Initialize vx configuration for current project
  clean        Clean up system (preferred over cleanup)
  stats        Show package statistics and disk usage
  plugin       Plugin management commands
  shell        Shell integration commands
  venv         Virtual environment management
  global       Global tool management
  env          Environment management
  help         Print this message or the help of the given subcommand(s)

Arguments:
  [ARGS]...  Tool and arguments to execute

Options:
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help
  -V, --version          Print version

```
