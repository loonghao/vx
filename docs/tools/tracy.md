# Tracy

[Tracy](https://github.com/wolfpld/tracy) is available through the `tracy` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `tracy` | Profiler ZIP, x64 | ZIP containing the profiler AppImage, x64 | Profiler ZIP, ARM64 |

```bash
vx tracy --help
vx tracy-csvexport --version
```

Versioned packages start at 0.14.1. The profiler parses `--help` before opening a window, but the system loader must first resolve its libraries. On Linux this includes EGL (`libEGL.so.1`) and compatible graphics libraries; the AppImage also needs AppImage/FUSE support. A graphical session is required to use the profiler GUI. macOS x64 packages are not supported by this Provider.

The `tracy-capture`, `tracy-csvexport` and `tracy-update` Runtimes ship with the profiler. Automated profiler validation checks its executable file, and installation completeness requires all three helper executables. These checks verify the package layout; they do not certify GUI startup. The separate `tracy-csvexport` validation runs its headless `--version` command and requires version output. Capturing or processing traces still requires validation with a real profiling target and trace files. Use `vx where tracy-capture` and `vx where tracy-csvexport` to resolve the executables needed by the adapter.

The application remains subject to its upstream license. Installing it does not install or connect a DCC-MCP adapter. Follow the [DCC-MCP integration guide](../guide/dcc-mcp.md) to configure the adapter.
