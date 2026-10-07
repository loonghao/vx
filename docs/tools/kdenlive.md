# Kdenlive

[Kdenlive](https://kdenlive.org) is available through the `kdenlive` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `kdenlive` | Official standalone SFX archive, x64 | AppImage, x64 | Homebrew cask / installed app, x64 / ARM64 |

```bash
vx kdenlive --version
```

Automated installation checks verify the GUI executable file without starting the application. Running the application and connecting its DCC-MCP adapter still require validation in a real application environment.

Versioned Windows/Linux downloads support the stable `.04`, `.08` and `.12` release series from 24.04 onward. Beta and release candidate tags are excluded. macOS uses the Homebrew cask or an existing application; its version is not pinned by vx.

Windows uses the official standalone archive and includes melt, FFprobe and FFmpeg. The Provider sets `DCC_MCP_KDENLIVE_EXECUTABLE`, plus `DCC_MCP_KDENLIVE_MELT`, `DCC_MCP_KDENLIVE_FFPROBE` and `DCC_MCP_KDENLIVE_FFMPEG` on Windows, in the runtime environment. On Linux/macOS, configure renderer and probe paths separately when connecting the adapter. Linux AppImages require compatible host libraries and AppImage/FUSE support; the Qt version check also needs a display or an offscreen Qt platform.

The application remains subject to its upstream license. Installing it does not install or connect a DCC-MCP adapter. Resolve its executable with `vx where kdenlive` and follow the [DCC-MCP integration guide](../guide/dcc-mcp.md).
