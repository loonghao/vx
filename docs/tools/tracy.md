# Tracy

[Tracy](https://github.com/wolfpld/tracy) is available through the `tracy` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `tracy` | Profiler ZIP, x64 | ZIP containing the profiler AppImage, x64 | Profiler ZIP, ARM64 |

```bash
vx tracy --help
```

Versioned packages start at 0.14.1. The profiler's `--help` command prints usage and exits before opening a window. Linux still requires compatible host libraries and AppImage/FUSE support. macOS x64 packages are not supported by this Provider.

The `tracy-capture`, `tracy-csvexport` and `tracy-update` Runtimes ship with the profiler. Automated installation checks verify their executable files; capturing or processing traces still requires validation with a real profiling target and trace files. Use `vx where tracy-capture` and `vx where tracy-csvexport` to resolve the executables needed by the adapter.

The application remains subject to its upstream license. Installing it does not install or connect a DCC-MCP adapter. Follow the [DCC-MCP integration guide](../guide/dcc-mcp.md) to configure the adapter.
