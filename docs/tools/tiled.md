# Tiled

[Tiled](https://www.mapeditor.org) is available through the `tiled` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `tiled` | MSI extracted into the vx store, x64 | AppImage, x64 | Universal app ZIP, x64 / ARM64 |

```bash
vx tiled --version
```

Automated installation checks verify the GUI executable file without starting the application. Running the application and connecting its DCC-MCP adapter still require validation in a real application environment.

This Provider supports versioned packages from 1.12.0 onward. Windows packages require Windows 10 or newer and are extracted into the vx store. macOS packages require macOS 13 or newer, except version 1.12.0, which supports macOS 11. Linux AppImages require compatible host libraries and AppImage/FUSE support.

The Provider sets `DCC_MCP_TILED_EXECUTABLE` in the runtime environment. The application remains subject to its upstream license. Installing it does not install or connect a DCC-MCP adapter. Resolve its executable with `vx where tiled` and follow the [DCC-MCP integration guide](../guide/dcc-mcp.md).
