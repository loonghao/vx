# OpenSCAD

[OpenSCAD](https://openscad.org) is available through the `openscad` Provider.

| Runtime | Windows | Linux | macOS |
| --- | --- | --- | --- |
| `openscad` | Portable ZIP, x64 / x86 | AppImage, x64 | Homebrew cask / installed app |

```bash
vx openscad --version
vx openscad -o part.stl part.scad
```

Windows uses the `openscad.com` console entry point from the portable archive. Linux AppImages require compatible system libraries and AppImage/FUSE support. macOS uses the Homebrew cask or an existing application installation.

OpenSCAD retains its upstream license. Resolve the executable with `vx where openscad`, then follow the DCC-MCP adapter's installation contract. Installing OpenSCAD does not install or connect its adapter.
