# Test: vx versions --help

Verify that `vx versions --help` displays usage information.

```console
$ vx versions --help
Show available versions for a tool (preferred over fetch)

Usage: vx[EXE] versions [OPTIONS] <TOOL>

Arguments:
  <TOOL>  Tool name

Options:
      --latest <LATEST>  Show only latest versions (limit results)
      --prerelease       Include pre-release versions
      --detailed         Show detailed version information
  -i, --interactive      Interactive mode for version selection
      --use-system-path  Use system PATH to find tools instead of vx-managed versions
  -v, --verbose          Enable verbose output with detailed logging
      --debug            Enable debug output (equivalent to RUST_LOG=debug)
  -h, --help             Print help

```
