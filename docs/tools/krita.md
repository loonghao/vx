# Krita

[Krita](https://krita.org) is available through the `krita` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `krita` | Portable ZIP, x64 | AppImage, x64 | Homebrew cask / installed app |

```bash
vx krita --version
```

Windows uses the `krita.com` console entry point from the portable archive. Linux AppImages require compatible system libraries and AppImage/FUSE support. macOS uses the Homebrew cask or an existing application installation.

Krita retains its upstream license. Resolve the executable with `vx where krita`, then follow the DCC-MCP adapter's installation contract. Installing Krita does not install or connect its adapter.
